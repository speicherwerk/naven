# Naven

Naven (*n*o m*aven*) is a toml-based (non-complete) maven wrapper to avoid its
horrible boilerplate.

## Example

Below is the full `pom.toml` for a project with two dependencies and a plugin:

```toml
[project]
group = "org.example"
artifact = "my-app"
version = "0.1.0"
name = "My Awesome App"
url = "https://example.org/my-app"

[java]
version = "21"
source = "21"
target = "21"

[dependencies]
"com.google.guava:guava" = "33.7"
"org.junit.jupiter:junit-jupiter" = { version = "RELEASE", scope = "test" }

[plugins]
":maven-assembly-plugin" = {
    configuration.archive.manifest.mainClass = "org.example.my-app.Main",
    configuration.descriptorRefs.descriptorRef = "jar-with-dependencies",
    executions.execution = {
        id = "make-assembly",
        goals.goal = "single",
        phase = "package"
    }
}
```

While this `pom.toml` has 26 lines, the (minimal) `.pom.xml` that naven
generates has 52 lines.

### Minimal Example

The minimal `pom.toml` has 4 lines:

```toml
[project]
group = "org.example"
artifact = "my-app"
version = "0.1.1"
```

Whereas the minimal `pom.xml` has 6 lines, eyesoreing XML and some mundane stuff
to remember:

```xml
<project xmlns="http://maven.apache.org/POM/4.0.0">
  <modelVersion>4.0.0</modelVersion>
  <groupId>foo</groupId>
  <artifactId>bar</artifactId>
  <version>0.1</version>
</project>
```

## How it works

When invoking naven it first builds `.pom.xlm` from `pom.toml` (if the latter is
newer) and then invokes the maven wrapper or `mvn` with all arguments given to
naven.
