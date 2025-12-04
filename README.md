# Toml Example
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help generate toml example

## Introduction
This crate provides the `TomlExample` trait and an accompanying derive macro.

Deriving `TomlExample` on a struct will provide `to_example` function help generate toml example file base documentation
- support `#[serde(default)]`, `#[serde(default = "function_name")]` attributes on both structs and struct fields (`serde` feature, opt-in)
- support `#[serde(rename)]`, `#[serde(rename_all = "renaming rules")]`, the renaming rules can be `lowercase`, `UPPERCASE`,
`PascalCase`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`
- provide `#[toml_example(default)]`, `#[toml_example(default = 0)]`, `#[toml_example(default = "default_string")]` attributes on struct fields
- `#[toml_example(default)]` is also supported as an outer attribute for structs
- The order matter of attribute macro, if `#[serde(default = ..]` and `#[toml_example(default = ..)]` existing at the same time with different value

## Quick Example
```rust 
use toml_example::TomlExample;
use serde::Deserialize;

/// Config is to arrange something or change the controls on a computer or other device
/// so that it can be used in a particular way
#[derive(TomlExample, Deserialize)]
struct Config {
    /// Config.a should be a number
    a: usize,
    /// Config.b should be a string
    b: String,
    /// Optional Config.c is a number
    c: Option<usize>,
    /// Config.d is a list of number
    d: Vec<usize>,
    /// Config.e should be a number
    #[serde(default = "default_int")]
    e: usize,
    /// Config.f should be a string
    #[serde(default = "default_str")]
    f: String,
    /// Config.g should be a number
    #[toml_example(default =7)]
    g: usize,
    /// Config.f should be a string
    #[toml_example(default = "seven")]
    h: String,
}
fn default_int() -> usize {
    7
}
fn default_str() -> String {
    "seven".into()
}

Config::to_toml_example("example.toml");  // write example to a file
let example = Config::toml_example();
```

Toml example base on the docstring of each field
```toml
# Config is to arrange something or change the controls on a computer or other device
# so that it can be used in a particular way

# Config.a should be a number
a = 0

# Config.b should be a string
b = ""

# Optional Config.c is a number
# c = 0

# Config.d is a list of number
# d = [ 0, ]

# Config.e should be a number
e = 7

# Config.f should be a string
f = "seven"

# Config.g should be a number
g = 7

# Config.h should be a string
h = "seven"

```

The fields of a struct can inherit their defaults from the parent struct when the
`#[toml_example(default)]`, `#[serde(default)]` or `#[serde(default = "default_fn")]`
attribute is set as an outer attribute of the parent struct:

```rust
use serde::Serialize;
use toml_example::TomlExample;

#[derive(TomlExample, Serialize)]
#[serde(default)]
struct Config {
    /// Name of the theme to use
    theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: String::from("Dark"),
        }
    }
}

assert_eq!(Config::toml_example(),
r#"# Name of the theme to use
theme = "Dark"

"#);
```

## Nesting Struct
A nesting struct wrap with `Option<T>`, `Vec<T>`, `HashMap<String, T>`, `BTreeMap<String, T>` are handled.
Please add `#[toml_example(nesting)]`, or `#[toml_example(nesting = prefix)]` on the field.
`#[toml_example(nesting)]`

```rust
  /// Service with specific port
  #[derive(TomlExample)]
  struct Service {
      /// port should be a number
      #[toml_example(default = 80)]
      port: usize,
  }
  #[derive(TomlExample)]
  #[allow(dead_code)]
  struct Node {
      /// Services are running in the node
      #[toml_example(default = http, nesting)]
      services: HashMap<String, Service>,
  }
```
`Node::toml_example()` will be following string.
```toml
# Services are running in the node
# Service with specific port
[services.http]
# port should be a number
port = 80

```

## Flattening
Flattening means treating the fields of a nested struct as if they were defined directly in the wrapping struct.
```rust
#[derive(TomlExample)]
struct ItemWrapper {
    #[toml_example(flatten, nesting)]
    item: Item,
}
#[derive(TomlExample)]
struct Item {
    value: String,
}

assert_eq!(ItemWrapper::toml_example(), Item::toml_example());
```

This works with maps too!

```rust
#[derive(TomlExample, Deserialize)]
struct MainConfig {
    #[serde(flatten)]
    #[toml_example(nesting)]
    nested: HashMap<String, ConfigItem>,
}
#[derive(TomlExample, Deserialize)]
struct ConfigItem {
    #[toml_example(default = false)]
    enabled: bool,
}

let example = MainConfig::toml_example();
assert!(toml::from_str::<MainConfig>(&example).is_ok());
println!("{example}");
```
```toml
[example]
enabled = false
```

## Enum Field
You can also use fieldless enums, but you have to annotate them with `#[toml_example(enum)]` or
`#[toml_example(is_enum)]` if you mind the keyword highlight you likely get when writing "enum".
When annotating a field with `#[toml_example(default)]` it will use the [Debug](core::fmt::Debug) implementation.
However for non-TOML data types like enums, this does not work as the value needs to be treated as a string in TOML.
The `#[toml_example(enum)]` attribute just adds the needed quotes around the [Debug](core::fmt::Debug) implementation
and can be omitted if a custom [Debug](core::fmt::Debug) already includes those.

```rust
use toml_example::TomlExample;
#[derive(TomlExample)]
struct Config {
    /// Config.priority is an enum
    #[toml_example(enum, default)]
    priority: Priority,
}
#[derive(Debug, Default)]
enum Priority {
    #[default]
    Important,
    Trivial,
}
assert_eq!(Config::toml_example(),
r#"# Config.priority is an enum
priority = "Important"

"#)
```

## More
If you want an optional field become a required field in example,
place the `#[toml_example(require)]` on the field.
If you want to skip some field you can use `#[toml_example(skip)]`,
the `#[serde(skip)]`, `#[serde(skip_deserializing)]` also works.
```rust
use toml_example::TomlExample;
#[derive(TomlExample)]
struct Config {
    /// Config.a is an optional number
    #[toml_example(require)]
    a: Option<usize>,
    /// Config.b is an optional string
    #[toml_example(require)]
    b: Option<String>,
    #[toml_example(require, default = "third")]
    c: Option<String>,
    #[toml_example(skip)]
    d: usize,
}
```
```toml
# Config.a is an optional number
a = 0

# Config.b is an optional string
b = ""

c = "third"

```

## Why TOML is good for program configuration file
There are serveral common config file solution: INI, JSON, YAML, TOML.
- INI is legacy and no nesting support, and the comment char is not designed at first, so some parser will use `;` or `#`.
- JSON is not support document, it is efficient for data exchange, but there should be documentation in configure, so it not good for this using case.
- YAML is too greedy on covering derserialize issue, so it is too complex for a confiture and vulnerable, for example:
  -  `y` ambiguously stand for "y" or true
  - [CVE-2019-11253](https://github.com/kubernetes/kubernetes/issues/83253)
- TOML is good for now, and easy to write and read for humman, that is a real need for confiture file.

[crates-badge]: https://img.shields.io/crates/v/toml-example.svg
[crate-url]: https://crates.io/crates/toml-example
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/toml-example/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/toml-example/
