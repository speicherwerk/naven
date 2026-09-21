use super::*;

fn parse_pom(toml: &str) -> Result<String, String> {
    let table = toml.parse().map_err(|e| format!("{e}"))?;
    let project = Project::parse(table)?;
    let mut buf = BufWriter::new(Vec::new());
    project.write_pom(&mut buf)?;
    String::from_utf8(buf.into_inner().map_err(|e| format!("{e}"))?).map_err(|e| format!("{e}"))
}

fn test_compare(pom_toml: &str, pom_xml: &str) {
    match parse_pom(pom_toml) {
        Ok(xml) => assert_eq!(xml.as_str(), pom_xml),
        Err(e) => assert!(false, "{e}"),
    }
}

#[test]
fn minimal_pom() {
    const POM_TOML: &str = include_str!("minimal.toml");
    const POM_XML: &str = include_str!("minimal.xml");
    test_compare(POM_TOML, POM_XML);
}

#[test]
fn version_only_dependency() {
    const POM_TOML: &str = include_str!("version_only_dependency.toml");
    const POM_XML: &str = include_str!("version_only_dependency.xml");
    test_compare(POM_TOML, POM_XML);
}

#[test]
fn dependency_with_scope() {
    const POM_TOML: &str = include_str!("dependency_with_scope.toml");
    const POM_XML: &str = include_str!("dependency_with_scope.xml");
    test_compare(POM_TOML, POM_XML);
}

#[test]
fn dependency_with_arbitrary_data() {
    const POM_TOML: &str = include_str!("dependency_with_arbitrary_data.toml");
    const POM_XML: &str = include_str!("dependency_with_arbitrary_data.xml");
    test_compare(POM_TOML, POM_XML);
}

#[test]
fn mixed_dependencies() {
    const POM_TOML: &str = include_str!("mixed_dependencies.toml");
    const POM_XML: &str = include_str!("mixed_dependencies.xml");
    test_compare(POM_TOML, POM_XML);
}

#[test]
fn all_recognized_keys() {
    const POM_TOML: &str = include_str!("all.toml");
    const POM_XML: &str = include_str!("all.xml");
    test_compare(POM_TOML, POM_XML);
}
