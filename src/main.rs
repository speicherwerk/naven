use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    process::Command,
};

use naven::*;
use toml::Table;

#[cfg(target_os = "windows")]
const MVNW_PATH: &str = "mvnw.cmd";
#[cfg(not(target_os = "windows"))]
const MVNW_PATH: &str = "./mvnw";

fn main() -> Result<(), String> {
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
                .chain(std::env::args().skip(1)),
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
