#[cfg(test)]
mod test;

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process::Command,
};

use naven::*;
use toml::Table;

#[cfg(target_os = "windows")]
const MVNW_PATH: &str = "mvnw.cmd";
#[cfg(not(target_os = "windows"))]
const MVNW_PATH: &str = "./mvnw";

fn main() -> Result<(), String> {
    let verb = env::args().nth(1);
    if let Some(v) = verb
        && v == "init"
    {
        return run_init();
    }

    let mut pom_toml =
        File::open("pom.toml").map_err(|e| format!("Error opening pom.toml: {e}"))?;
    let pom_xml = OpenOptions::new()
        .read(true)
        .write(true)
        .open(".pom.xml")
        .ok();

    if is_pom_toml_newer(&pom_toml, pom_xml.as_ref()) {
        let mut pom_xml = match pom_xml {
            Some(f) => f,
            None => {
                File::create(".pom.xml").map_err(|e| format!("Error creating `.pom.xml`: {e}"))?
            }
        };

        rebuild_pom_xml(&mut pom_toml, &mut pom_xml)?;
    }

    let maven_command =
        if std::fs::exists(MVNW_PATH).map_err(|e| format!("Error looking for {MVNW_PATH}: {e}"))? {
            MVNW_PATH
        } else {
            "mvn"
        };

    Command::new(maven_command)
        .args(
            [String::from("--file"), String::from(".pom.xml")]
                .into_iter()
                .chain(env::args().skip(1)),
        )
        .spawn()
        .map_err(|e| format!("{e}"))?
        .wait()
        .map(|_| ())
        .map_err(|e| format!("{e}"))
}

fn is_pom_toml_newer(pom_toml: &File, pom_xml: Option<&File>) -> bool {
    let Ok(pom_toml_metadata) = pom_toml.metadata() else {
        return true;
    };
    let Ok(pom_toml_modified) = pom_toml_metadata.modified() else {
        return true;
    };
    let Some(Ok(pom_xml_metadata)) = pom_xml.map(File::metadata) else {
        return true;
    };
    let Ok(pom_xml_modified) = pom_xml_metadata.modified() else {
        return true;
    };
    pom_toml_modified > pom_xml_modified
}

fn rebuild_pom_xml(pom_toml: &mut File, pom_xml: &mut File) -> Result<(), String> {
    let mut src = String::new();
    pom_toml
        .read_to_string(&mut src)
        .map_err(|e| format!("unable to read {pom_toml:?}: {e}"))?;

    let table = src
        .parse::<Table>()
        .map_err(|e| format!("Error parsing pom.toml: {e}"))?;
    let project = Project::parse(table).map_err(|e| format!("Error parsing pom.toml: {e}"))?;

    write_pom_xml(pom_xml, project)
}

fn write_pom_xml(pom_xml: &mut File, project: Project) -> Result<(), String> {
    pom_xml
        .set_len(0)
        .map_err(|e| format!("Error writing to `.pom.xml`: {e}"))?;
    pom_xml
        .seek(SeekFrom::Start(0))
        .map_err(|e| format!("Error writing to `.pom.xml`: {e}"))?;
    let mut buf_writer = BufWriter::new(pom_xml);
    project
        .write_pom(&mut buf_writer)
        .map_err(|e| format!("Error writing pom.xml: {e}"))?;
    buf_writer
        .flush()
        .map_err(|e| format!("Error writing to `.pom.xml`: {e}"))
}

macro_rules! pom_toml_template {
    () => {
        r#"group = "{group}"
artifact = "{artifact}"
version = "{version}"

[properties]
release = "17"
encoding = "UTF-8"
"exec.mainClass" = "{group}.Main""#
    };
}

macro_rules! main_java_template {
    () => {
        r#"package {group};

public class Main {{
    public static void main(String[] args) {{
        System.out.println("\nHello World!\n");
    }}
}}"#
    };
}

fn run_init() -> Result<(), String> {
    let current_dir = env::current_dir().map_err(|e| format!("env error: {e}"))?;
    if fs::read_dir(&current_dir)
        .map_err(|e| format!("IO error: {e}"))?
        .next()
        .is_some()
    {
        return Err(String::from(
            "refusing to init naven project: directoy is non-empty",
        ));
    }
    let artifact = current_dir
        .file_name()
        .unwrap()
        .to_str()
        .ok_or_else(|| format!("current dir contains non-unicode symbols: {current_dir:?}"))?;
    let group = std_prompt("Enter the project's groupId")?;
    if group.is_empty() {
        return Err(String::from("group may not be empty"));
    }
    let version = String::from("0.1.0-SNAPSHOT");
    let pom_toml_str = format!(
        pom_toml_template!(),
        group = group,
        artifact = artifact,
        version = version
    );
    let gitignore_str = "target/";
    let main_java_str = format!(main_java_template!(), group = group);

    fs::write("pom.toml", pom_toml_str).map_err(|e| format!("error writing `pom.toml`: {e}"))?;
    fs::write(".gitignore", gitignore_str)
        .map_err(|e| format!("error writing `.gitignore`: {e}"))?;
    let package_path: PathBuf = ["src", "main", "java"]
        .into_iter()
        .chain(group.split('.'))
        .collect();
    fs::create_dir_all(&package_path)
        .map_err(|e| format!("error creating directory structure: {e}"))?;
    fs::create_dir(
        ["src", "main", "resources"]
            .into_iter()
            .collect::<PathBuf>(),
    )
    .map_err(|e| format!("error creating directory structure: {e}"))?;
    fs::create_dir_all(["src", "test", "java"].into_iter().collect::<PathBuf>())
        .map_err(|e| format!("error creating directory structure: {e}"))?;
    let mut main_java_path = package_path;
    main_java_path.push("Main.java");
    fs::write(&main_java_path, main_java_str)
        .map_err(|e| format!("error writing `{main_java_path:?}`: {e}"))?;

    println!("\nSuccessfully generated project `{group}:{artifact}:{version}`!");
    Ok(())
}

fn std_prompt(prompt: &str) -> Result<String, String> {
    print!("{prompt}: ");
    io::stdout().flush().map_err(|e| format!("IO error: {e}"))?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .map_err(|e| format!("IO error: {e}"))?;
    Ok(buf.trim().to_owned())
}
