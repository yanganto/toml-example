# Struct Patch
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

#[derive(TomlExample)]
struct Config {
    /// Config.a should be a number
    a: usize,
    /// Config.b should be a string
    b: String,
    /// Optional Config.c is a number
    c: usize,
}
let doc = Config::toml_example();

// doc is String toml example base on the docstring

// # Config.a should be a number
// a = 0
// # Config.b should be a string
// b = ""
// # Optional Config.c is a number
// c = 0

```

## Will do later
- use structure doc for example header
- handle Vec
- nesting structure
- use `#[serde(default = "default_resource")]` for example
- function to write example file, `to_toml_example(file_name)`

[crates-badge]: https://img.shields.io/crates/v/toml-example.svg
[crate-url]: https://crates.io/crates/toml-example
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/toml-example/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/toml-example/
