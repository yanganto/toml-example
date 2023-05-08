# Toml Example
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help generate toml example

## Introduction
This crate provides the `TomlExample` trait and an accompanying derive macro.

Deriving `TomlExample` on a struct will provide `to_example` function help generate toml example file base documentation

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
}
fn default_int() -> usize {
    7
}
fn default_str() -> String {
    "default".into()
}
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
f = "default"
```

## Will do later
- nesting structure
- function to write example file, `to_toml_example(file_name)`

[crates-badge]: https://img.shields.io/crates/v/toml-example.svg
[crate-url]: https://crates.io/crates/toml-example
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/toml-example/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/toml-example/
