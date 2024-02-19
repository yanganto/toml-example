# Toml Example
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help generate toml example

## Introduction
This crate provides the `TomlExample` trait and an accompanying derive macro.

Deriving `TomlExample` on a struct will provide `to_example` function help generate toml example file base documentation
- support `#[serde(default)]`, `#[serde(default = "function_name")]` attributes (`serde` feature, opt-in)
- support `#[serde(rename)]`, `#[serde(rename_all = "renaming rules")]`, the renaming rules can be `lowercase`, `UPPERCASE`,
`PascalCase`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`
- provide `#[toml_example(default)]`, `#[toml_example(default = 0)]`, `#[toml_example(default = "default_string")]` attributes
- The order matter of attribute macro, if `#[serde(default = ..]` and `#[toml_example(default = ..)]` existing at the same time with different value

## Quick Example
```rust 
use toml_example::TomlExample;

/// Config is to arrange something or change the controls on a computer or other device
/// so that it can be used in a particular way
#[derive(TomlExample)]
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

Toml example base on the doc string of each field
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
      #[toml_example(nesting)]
      #[toml_example(default = http)]
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
    #[toml_example(require)]
    #[toml_example(default = "third")]
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

[crates-badge]: https://img.shields.io/crates/v/toml-example.svg
[crate-url]: https://crates.io/crates/toml-example
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/toml-example/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/toml-example/
