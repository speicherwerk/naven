mod xmlwriter;

use std::io::Write;

use toml::{Table, Value};

use xmlwriter::*;

#[derive(Clone, Debug)]
pub struct Project {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub packaging: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub java: Option<Java>,
    pub repositories: Vec<Repository>,
    pub dependencies: Vec<Dependency>,
    pub plugins: Vec<Plugin>,
}

#[derive(Clone, Debug)]
pub struct Java {
    pub version: Option<String>,
    pub source: Option<String>,
    pub target: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Repository {
    pub id: String,
    pub url: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Dependency {
    pub group: String,
    pub artifact: String,
    pub spec: Table,
}

#[derive(Clone, Debug)]
pub struct Plugin {
    pub group: String,
    pub artifact: String,
    pub spec: Table,
}

impl Project {
    pub fn parse(mut table: Table) -> Result<Self, String> {
        let project_tbl = mandatory_table_from_table(&mut table, "project")
            .map_err(|()| String::from("missing `project` table"))?;

        let group = mandatory_string_from_table(project_tbl, "group")
            .map_err(|()| String::from("expected string `group` in `project` table"))?
            .to_owned();
        let artifact = mandatory_string_from_table(project_tbl, "artifact")
            .map_err(|()| String::from("expected string `artifact` in `project` table"))?
            .to_owned();
        let version = mandatory_string_from_table(project_tbl, "version")
            .map_err(|()| String::from("expected string `version` in `project` table"))?
            .to_owned();

        let packaging = optional_string_from_table(project_tbl, "packaging")
            .map_err(|()| String::from("expected `packaging` in `project` table to be a string"))?
            .map(ToOwned::to_owned);
        let name = optional_string_from_table(project_tbl, "name")
            .map_err(|()| String::from("expected `name` in `project` table to be a string"))?
            .map(ToOwned::to_owned);
        let url = optional_string_from_table(project_tbl, "url")
            .map_err(|()| String::from("expected `url` in `project` table to be a string"))?
            .map(ToOwned::to_owned);

        let java = optional_table_from_table(&mut table, "java")
            .map_err(|()| String::from("expected `java` to be a table"))?
            .map(|t| Java::parse(t))
            .transpose()?;

        let repositories = if let Some(repository_tbl) =
            optional_table_from_table(&mut table, "repositories")
                .map_err(|()| String::from("expected `repositories` to be a table"))?
        {
            let mut repositories = Vec::new();
            for (key, val) in repository_tbl.into_iter() {
                repositories.push(Repository::parse(key.to_owned(), val)?);
            }
            repositories
        } else {
            Vec::new()
        };

        let dependencies = if let Some(dependency_table) =
            optional_table_from_table(&mut table, "dependencies")
                .map_err(|()| String::from("expected `dependencies` to be a value"))?
        {
            parse_dependencies_like(dependency_table)?
                .into_iter()
                .map(|(group, artifact, spec)| Dependency {
                    group,
                    artifact,
                    spec,
                })
                .collect()
        } else {
            Vec::new()
        };

        let plugins = if let Some(plugin_table) =
            optional_table_from_table(&mut table, "plugins")
                .map_err(|()| String::from("expected `plugins` to be a value"))?
        {
            parse_dependencies_like(plugin_table)?
                .into_iter()
                .map(|(group, artifact, spec)| Plugin {
                    group,
                    artifact,
                    spec,
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(Self {
            group,
            artifact,
            version,
            packaging,
            name,
            url,
            java,
            repositories,
            dependencies,
            plugins,
        })
    }

    pub fn write_pom(&self, writer: &mut impl Write) -> Result<(), &'static str> {
        let mut xml_writer = XmlWriter::new(writer, 2);
        self.write(&mut xml_writer)
    }
}

/// Depth-searches the given table and returns all final-layer values together
/// with the dot-separated path and the name of the final-layer value.
fn find_values_and_concat_path(table: &Table) -> Vec<(String, String, &Value)> {
    fn recurse<'a>(
        table: &'a toml::Table,
        path: &mut Vec<String>,
        results: &mut Vec<(String, String, &'a Value)>,
    ) {
        for (name, value) in table {
            if let Value::Table(t) = value
                && t.len() == 1
            {
                path.push(name.clone());
                recurse(t, path, results);
                path.pop();
            } else {
                results.push((path.join("."), name.clone(), value));
            }
        }
    }

    let mut results = Vec::new();
    recurse(table, &mut Vec::new(), &mut results);
    results
}

/// Returns (groupId, artifactId, specification)
fn parse_dependencies_like(table: &Table) -> Result<Vec<(String, String, Table)>, String> {
    let mut result = Vec::new();
    let entries = find_values_and_concat_path(table);
    for (group, artifact, value) in entries.into_iter() {
        let spec = match value {
            Value::Table(t) => t.clone(),
            Value::String(s) => {
                let mut table = Table::new();
                table.insert(String::from("version"), Value::String(s.to_owned()));
                table
            }
            _ => {
                return Err(format!(
                    "Expected table or version string as specification of {group}.{artifact}"
                ));
            }
        };
        result.push((group, artifact, spec));
    }
    Ok(result)
}

fn optional_string_from_table<'a>(table: &'a Table, name: &str) -> Result<Option<&'a str>, ()> {
    if let Some(val) = table.get(name) {
        val.as_str().ok_or(()).map(Option::Some)
    } else {
        Ok(None)
    }
}

fn mandatory_string_from_table<'a>(table: &'a Table, name: &str) -> Result<&'a str, ()> {
    table.get(name).and_then(Value::as_str).ok_or(())
}

fn optional_table_from_table<'a>(
    table: &'a mut Table,
    name: &str,
) -> Result<Option<&'a mut Table>, ()> {
    if let Some(val) = table.get_mut(name) {
        val.as_table_mut().ok_or(()).map(Option::Some)
    } else {
        Ok(None)
    }
}

fn mandatory_table_from_table<'a>(table: &'a mut Table, name: &str) -> Result<&'a mut Table, ()> {
    table.get_mut(name).and_then(Value::as_table_mut).ok_or(())
}

impl Java {
    fn parse(table: &Table) -> Result<Self, String> {
        let version = optional_string_from_table(table, "version")
            .map_err(|()| String::from("expected `version` in `java` table to be a string"))?
            .map(ToOwned::to_owned);
        let source = optional_string_from_table(table, "source")
            .map_err(|()| String::from("expected `source` in `java` table to be a string"))?
            .map(ToOwned::to_owned);
        let target = optional_string_from_table(table, "target")
            .map_err(|()| String::from("expected `target` in `java` table to be a string"))?
            .map(ToOwned::to_owned);

        Ok(Self {
            version,
            source,
            target,
        })
    }
}

impl Repository {
    fn parse(id: String, value: &Value) -> Result<Self, String> {
        let url = value
            .as_str()
            .ok_or_else(|| {
                format!("expected value of repository `{id}` to be a string; found `{value}`")
            })?
            .to_owned();

        Ok(Repository {
            id,
            url,
            name: None,
        })
    }
}

impl XmlWrite for Project {
    fn write<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<(), &'static str> {
        writer.open_tag_with_attrib(
            String::from("project"),
            "xmlns",
            "http://maven.apache.org/POM/4.0.0",
        )?;
        writer.element("modelVersion", "4.0.0")?;

        writer.element("groupId", &self.group)?;
        writer.element("artifactId", &self.artifact)?;
        writer.element("version", &self.version)?;

        if let Some(name) = &self.name {
            writer.element("name", name)?;
        }
        if let Some(url) = &self.url {
            writer.element("url", url)?;
        }

        if let Some(java) = &self.java {
            java.write(writer)?;
        }

        if !self.repositories.is_empty() {
            writer.open_tag(String::from("repositories"))?;
            for repo in &self.repositories {
                repo.write(writer)?;
            }
            writer.close_tag()?;
        }

        if !self.dependencies.is_empty() {
            writer.open_tag(String::from("dependencies"))?;
            for dep in &self.dependencies {
                dep.write(writer)?;
            }
            writer.close_tag()?;
        }

        if !self.plugins.is_empty() {
            writer.open_tag(String::from("build"))?;
            writer.open_tag(String::from("plugins"))?;
            for plugin in &self.plugins {
                plugin.write(writer)?;
            }
            writer.close_tag()?;
            writer.close_tag()?;
        }

        writer.close_all()
    }
}

impl XmlWrite for Repository {
    fn write<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<(), &'static str> {
        writer.open_tag(String::from("repository"))?;
        writer.element("id", &self.id)?;
        writer.element("url", &self.url)?;
        if let Some(name) = &self.name {
            writer.element("scope", name)?;
        }
        writer.close_tag()
    }
}

impl XmlWrite for Java {
    fn write<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<(), &'static str> {
        writer.open_tag(String::from("properties"))?;
        if let Some(version) = &self.version {
            writer.element("java.version", version)?;
        }
        if let Some(source) = &self.source {
            writer.element("maven.compiler.source", source)?;
        }
        if let Some(target) = &self.target {
            writer.element("maven.compiler.target", target)?;
        }
        writer.close_tag()?;
        Ok(())
    }
}

impl XmlWrite for Dependency {
    fn write<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<(), &'static str> {
        writer.open_tag(String::from("dependency"))?;
        writer.element("groupId", &self.group)?;
        writer.element("artifactId", &self.artifact)?;
        writer.write_table(&self.spec)?;
        writer.close_tag()
    }
}

impl XmlWrite for Plugin {
    fn write<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<(), &'static str> {
        writer.open_tag(String::from("plugin"))?;
        // ommiting the groupId is valid for some plugins
        if !self.group.is_empty() {
            writer.element("groupId", &self.group)?;
        }
        writer.element("artifactId", &self.artifact)?;
        writer.write_table(&self.spec)?;
        writer.close_tag()
    }
}
