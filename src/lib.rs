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
    pub properties: Option<Properties>,
    pub repositories: Vec<Repository>,
    pub dependencies: Vec<Dependency>,
    pub plugins: Vec<Plugin>,
    pub rest: Table,
}

#[derive(Clone, Debug)]
pub struct Properties {
    pub release: Option<String>,
    pub encoding: Option<String>,
    pub rest: Table,
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
        let group = mandatory_string(&mut table, "group")
            .map_err(|()| String::from("expected string `group`"))?;
        let artifact = mandatory_string(&mut table, "artifact")
            .map_err(|()| String::from("expected string `artifact`"))?;
        let version = mandatory_string(&mut table, "version")
            .map_err(|()| String::from("expected string `version`"))?;

        let packaging = optional_string(&mut table, "packaging")
            .map_err(|()| String::from("expected `packaging` to be a string"))?;
        let name = optional_string(&mut table, "name")
            .map_err(|()| String::from("expected `name` to be a string"))?;
        let url = optional_string(&mut table, "url")
            .map_err(|()| String::from("expected `url` to be a string"))?;

        let properties = optional_table(&mut table, "properties")
            .map_err(|()| String::from("expected `properties` to be a table"))?
            .map(Properties::parse)
            .transpose()?;

        let repositories = if let Some(repository_tbl) = optional_table(&mut table, "repositories")
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
            optional_table(&mut table, "dependencies")
                .map_err(|()| String::from("expected `dependencies` to be a table"))?
        {
            parse_dependencies_like(dependency_table)
                .map_err(|e| format!("{e} for entries in `dependency` table"))?
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

        let plugins = if let Some(plugin_table) = optional_table(&mut table, "plugins")
            .map_err(|()| String::from("expected `plugins` to be a table"))?
        {
            parse_dependencies_like(plugin_table)
                .map_err(|e| format!("{e} for entries in `plugin` table"))?
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
            properties,
            repositories,
            dependencies,
            plugins,
            rest: table,
        })
    }

    pub fn write_pom(&self, writer: &mut impl Write) -> Result<(), &'static str> {
        let mut xml_writer = XmlWriter::new(writer, 2);
        self.write(&mut xml_writer)
    }
}

/// Returns (groupId, artifactId, specification)
fn parse_dependencies_like(table: Table) -> Result<Vec<(String, String, Table)>, String> {
    let mut result = Vec::new();
    for (name, value) in table {
        let (group, artifact) = name
            .split_once(':')
            .ok_or_else(|| format!("expected `group:artifact`, found `{name}`"))?;
        let spec = match value {
            Value::Table(t) => t,
            Value::String(s) => {
                let mut t = Table::new();
                t.insert(String::from("version"), Value::String(s.to_owned()));
                t
            }
            _ => {
                return Err(format!(
                    "Expected table or version string as specification of {group}.{artifact}"
                ));
            }
        };
        result.push((group.to_owned(), artifact.to_owned(), spec));
    }
    Ok(result)
}

fn optional_string(table: &mut Table, name: &str) -> Result<Option<String>, ()> {
    if let Some(val) = table.remove(name) {
        if let Value::String(s) = val {
            Ok(Some(s))
        } else {
            Err(())
        }
    } else {
        Ok(None)
    }
}

fn mandatory_string(table: &mut Table, name: &str) -> Result<String, ()> {
    match table.remove(name) {
        Some(Value::String(s)) => Ok(s),
        _ => Err(()),
    }
}

fn optional_table(table: &mut Table, name: &str) -> Result<Option<Table>, ()> {
    if let Some(val) = table.remove(name) {
        if let Value::Table(t) = val {
            Ok(Some(t))
        } else {
            Err(())
        }
    } else {
        Ok(None)
    }
}

impl Properties {
    fn parse(mut table: Table) -> Result<Self, String> {
        let release = optional_string(&mut table, "release").map_err(|()| {
            String::from("expected `release` in `properties` table to be a string")
        })?;
        let encoding = optional_string(&mut table, "encoding").map_err(|()| {
            String::from("expected `encoding` in `properties` table to be a string")
        })?;

        Ok(Self {
            release,
            encoding,
            rest: table,
        })
    }
}

impl Repository {
    fn parse(id: String, value: Value) -> Result<Self, String> {
        let Value::String(url) = value else {
            return Err(format!(
                "expected value of repository `{id}` to be a string; found `{value}`"
            ));
        };

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

        if let Some(properties) = &self.properties {
            properties.write(writer)?;
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

        writer.write_table(&self.rest)?;

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

impl XmlWrite for Properties {
    fn write<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<(), &'static str> {
        writer.open_tag(String::from("properties"))?;
        if let Some(release) = &self.release {
            writer.element("maven.compiler.release", release)?;
        }
        if let Some(encoding) = &self.encoding {
            writer.element("project.build.sourceEncoding", encoding)?;
        }
        writer.write_table(&self.rest)?;
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
