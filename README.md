# Naven

Naven (*n*o m*aven*) is a toml-based (non-complete) maven wrapper to avoid its
horrible boilerplate.

## Quick start

Install naven via `cargo`:

```sh
cargo install --git https://github.com/speicherwerk/naven
```

Create a project using `naven init`:

```sh
mkdir my-app && cd my-app
naven init <<< "org.example"    # naven init asks for groupId
naven compile exec:java
```

This is the `pom.toml` created by `naven init`:

```toml
group = "org.example"
artifact = "naven-test"
version = "0.1.0-SNAPSHOT"

[properties]
release = "17"
encoding = "UTF-8"
"exec.mainClass" = "org.example.Main"
```

Note that `"exec.mainClass"` has to be put in quotes because without quotes, it
would wrongly denote a nested map.

Specify a dependency like google's guava by adding the following to `pom.toml`:

```toml
[dependencies]
"com.google.guava:guava" = "33.7"
```

See below for more information.

## Example

Below is the full `pom.toml` for a project with two dependencies and a plugin:

```toml
group = "org.example"
artifact = "my-app"
version = "0.1.0"

[properties]
release = "21"

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

While this `pom.toml` has 18 lines of code, the (minimal) `.pom.xml` that naven
generates from it has 48 lines.

### Minimal Example

The minimal `pom.toml` has 4 lines:

```toml
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

When invoking naven it first builds `.pom.xml` from `pom.toml` (if the latter is
newer) and then invokes the maven wrapper or `mvn` with all arguments given to
naven.
