//! This crate provides the [`TomlExample`] trait and an accompanying derive macro.
//!
//! Deriving [`TomlExample`] on a struct will generate functions `toml_example()`, `to_toml_example(file_name)` for generating toml example content.
//!
//! The following code shows how `toml-example` can be used.
//! ```rust
//! use toml_example::TomlExample;
//!
//! /// Config is to arrange something or change the controls on a computer or other device
//! /// so that it can be used in a particular way
//! #[derive(TomlExample)]
//! struct Config {
//!     /// Config.a should be a number
//!     a: usize,
//!     /// Config.b should be a string
//!     b: String,
//!     /// Optional Config.c is a number
//!     c: Option<usize>,
//!     /// Config.d is a list of number
//!     d: Vec<usize>,
//!     #[toml_example(default =7)]
//!     e: usize,
//!     /// Config.f should be a string
//!     #[toml_example(default = "seven")]
//!     f: String,
//! }
//! assert_eq!( Config::toml_example(),
//! r#"# Config is to arrange something or change the controls on a computer or other device
//! ## so that it can be used in a particular way
//! ## Config.a should be a number
//! a = 0
//!
//! ## Config.b should be a string
//! b = ""
//!
//! ## Optional Config.c is a number
//! ## c = 0
//!
//! ## Config.d is a list of number
//! d = [ 0, ]
//!
//! e = 7
//!
//! ## Config.f should be a string
//! f = "seven"
//!
//! "#);
//! ```
//!
//! Also, toml-example will use `#[serde(default)]`, `#[serde(default = "default_fn")]` for the
//! example value.
//!
//! With nesting structure, `#[toml_example(nesting)]` should set on the field as following
//! example.
//!
//! ```rust
//! use std::collections::HashMap;
//! use toml_example::TomlExample;
//!
//! /// Service with specific port
//! #[derive(TomlExample)]
//! struct Service {
//!     /// port should be a number
//!     #[toml_example(default = 80)]
//!     port: usize,
//! }
//! #[derive(TomlExample)]
//! #[allow(dead_code)]
//! struct Node {
//!     /// Services are running in the node
//!     #[toml_example(default = http, nesting)]
//!     services: HashMap<String, Service>,
//! }
//!
//! assert_eq!(Node::toml_example(),
//! r#"# Services are running in the node
//! ## Service with specific port
//! [services.http]
//! ## port should be a number
//! port = 80
//!
//! "#);
//! ```
//!
//! Flattened items are supported as well.
//!
//! ```rust
//! use toml_example::TomlExample;
//!
//! #[derive(TomlExample)]
//! struct ItemWrapper {
//!     #[toml_example(flatten, nesting)]
//!     item: Item,
//! }
//! #[derive(TomlExample)]
//! struct Item {
//!     value: String,
//! }
//!
//! assert_eq!(ItemWrapper::toml_example(), Item::toml_example());
//! ```
//!
//! Flattening works with maps too!
//!
//! ```rust
//! use serde::Deserialize;
//! use toml_example::TomlExample;
//! # use std::collections::HashMap;
//!
//! #[derive(TomlExample, Deserialize)]
//! struct MainConfig {
//!     #[serde(flatten)]
//!     #[toml_example(nesting)]
//!     nested: HashMap<String, ConfigItem>,
//! }
//! #[derive(TomlExample, Deserialize)]
//! struct ConfigItem {
//!     #[toml_example(default = false)]
//!     enabled: bool,
//! }
//!
//! let example = MainConfig::toml_example();
//! assert!(toml::from_str::<MainConfig>(&example).is_ok());
//! assert_eq!(example, r#"[example]
//! enabled = false
//!
//! "#);
//! ```
//!
//! The fields of a struct can inherit their defaults from the parent struct when the
//! `#[toml_example(default)]`, `#[serde(default)]` or `#[serde(default = "default_fn")]`
//! attribute is set as an outer attribute of the parent struct:
//!
//! ```rust
//! use serde::Serialize;
//! use toml_example::TomlExample;
//!
//! #[derive(TomlExample, Serialize)]
//! #[serde(default)]
//! struct Config {
//!     /// Name of the theme to use
//!     theme: String,
//! }
//!
//! impl Default for Config {
//!     fn default() -> Self {
//!         Self {
//!             theme: String::from("Dark"),
//!         }
//!     }
//! }
//!
//! assert_eq!(Config::toml_example(),
//! r#"# Name of the theme to use
//! theme = "Dark"
//!
//! "#);
//! ```
//!
//! If you want an optional field become a required field in example,
//! place the `#[toml_example(require)]` on the field.
//! If you want to skip some field you can use `#[toml_example(skip)]`,
//! the `#[serde(skip)]`, `#[serde(skip_deserializing)]` also works.
//! ```rust
//! use toml_example::TomlExample;
//! #[derive(TomlExample)]
//! struct Config {
//!     /// Config.a is an optional number
//!     #[toml_example(require)]
//!     a: Option<usize>,
//!     /// Config.b is an optional string
//!     #[toml_example(require)]
//!     b: Option<String>,
//!     #[toml_example(require, default = "third")]
//!     c: Option<String>,
//!     #[toml_example(skip)]
//!     d: usize,
//! }
//! assert_eq!(Config::toml_example(),
//! r#"# Config.a is an optional number
//! a = 0
//!
//! ## Config.b is an optional string
//! b = ""
//!
//! c = "third"
//!
//! "#)
//! ```
//!
//! You can also use fieldless enums, but you have to annotate them with `#[toml_example(enum)]` or
//! `#[toml_example(is_enum)]` if you mind the keyword highlight you likely get when writing
//! "enum".<br>
//! When annotating a field with `#[toml_example(default)]` it will use the
//! [Debug](core::fmt::Debug) implementation.
//! However for non-TOML data types like enums, this does not work as the value needs to be treated
//! as a string in TOML. The `#[toml_example(enum)]` attribute just adds the needed quotes around
//! the [Debug](core::fmt::Debug) implementation and can be omitted if a custom
//! [Debug](core::fmt::Debug) already includes those.
//! ```rust
//! use toml_example::TomlExample;
//! #[derive(TomlExample)]
//! struct Config {
//!     /// Config.priority is an enum
//!     #[toml_example(default, enum)]
//!     priority: Priority,
//! }
//! #[derive(Debug, Default)]
//! enum Priority {
//!     #[default]
//!     Important,
//!     Trivial,
//! }
//! assert_eq!(Config::toml_example(),
//! r#"# Config.priority is an enum
//! priority = "Important"
//!
//! "#)
//! ```

#[doc(hidden)]
pub use toml_example_derive::TomlExample;
pub mod traits;
pub use traits::*;

#[cfg(test)]
mod tests {
    use crate as toml_example;
    use serde_derive::Deserialize;
    use std::collections::HashMap;
    use toml_example::TomlExample;

    #[test]
    fn basic() {
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a should be a number
            a: usize,
            /// Config.b should be a string
            b: String,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a should be a number
a = 0

# Config.b should be a string
b = ""

"#
        );
        assert_eq!(
            toml::from_str::<Config>(&Config::toml_example()).unwrap(),
            Config::default()
        );
        let mut tmp_file = std::env::temp_dir();
        tmp_file.push("config.toml");
        Config::to_toml_example(&tmp_file.as_path().to_str().unwrap()).unwrap();
        assert_eq!(
            std::fs::read_to_string(tmp_file).unwrap(),
            r#"# Config.a should be a number
a = 0

# Config.b should be a string
b = ""

"#
        );
    }

    #[test]
    fn option() {
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a is an optional number
            a: Option<usize>,
            /// Config.b is an optional string
            b: Option<String>,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a is an optional number
# a = 0

# Config.b is an optional string
# b = ""

"#
        );
        assert_eq!(
            toml::from_str::<Config>(&Config::toml_example()).unwrap(),
            Config::default()
        )
    }

    #[test]
    fn vec() {
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a is a list of number
            a: Vec<usize>,
            /// Config.b is a list of string
            b: Vec<String>,
            /// Config.c
            c: Vec<Option<usize>>,
            /// Config.d
            d: Option<Vec<usize>>,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a is a list of number
a = [ 0, ]

# Config.b is a list of string
b = [ "", ]

# Config.c
c = [ 0, ]

# Config.d
# d = [ 0, ]

"#
        );
        assert!(toml::from_str::<Config>(&Config::toml_example()).is_ok())
    }

    #[test]
    fn struct_doc() {
        /// Config is to arrange something or change the controls on a computer or other device
        /// so that it can be used in a particular way
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a should be a number
            /// the number should be greater or equal zero
            a: usize,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config is to arrange something or change the controls on a computer or other device
# so that it can be used in a particular way
# Config.a should be a number
# the number should be greater or equal zero
a = 0

"#
        );
        assert_eq!(
            toml::from_str::<Config>(&Config::toml_example()).unwrap(),
            Config::default()
        )
    }

    #[test]
    fn serde_default() {
        fn default_a() -> usize {
            7
        }
        fn default_b() -> String {
            "default".into()
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a should be a number
            #[serde(default = "default_a")]
            a: usize,
            /// Config.b should be a string
            #[serde(default = "default_b")]
            b: String,
            /// Config.c should be a number
            #[serde(default)]
            c: usize,
            /// Config.d should be a string
            #[serde(default)]
            d: String,
            #[serde(default)]
            e: Option<usize>,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a should be a number
a = 7

# Config.b should be a string
b = "default"

# Config.c should be a number
c = 0

# Config.d should be a string
d = ""

# e = 0

"#
        );
    }

    #[test]
    fn toml_example_default() {
        fn default_str() -> String {
            "seven".into()
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a should be a number
            #[toml_example(default = 7)]
            a: usize,
            /// Config.b should be a string
            #[toml_example(default = "default")]
            #[serde(default = "default_str")]
            b: String,
            #[serde(default = "default_str")]
            #[toml_example(default = "default")]
            c: String,
            #[toml_example(default = [ "default", ])]
            e: Vec<String>,
            #[toml_example(
                default = "super looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong string"
            )]
            f: String,
            #[toml_example(default = [ "super looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong string",
               "second",
               "third",
            ])]
            g: Vec<String>,
            /// Config.color should be a hex color code
            #[toml_example(default = "#FAFAFA")]
            color: String,
        }
        assert_eq!(
            Config::toml_example(),
            r##"# Config.a should be a number
a = 7

# Config.b should be a string
b = "seven"

c = "default"

e = ["default",]

f = "super looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong string"

g = ["super looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong string",
"second", "third",]

# Config.color should be a hex color code
color = "#FAFAFA"

"##
        );
    }

    #[test]
    fn struct_serde_default() {
        #[derive(TomlExample, Deserialize, PartialEq)]
        #[serde(default)]
        struct Foo {
            bar: String,
            #[serde(default)]
            x: usize,
        }
        impl Default for Foo {
            fn default() -> Self {
                Foo {
                    bar: String::from("hello world"),
                    x: 12,
                }
            }
        }
        assert_eq!(
            Foo::toml_example(),
            r##"bar = "hello world"

x = 0

"##
        );
    }

    #[test]
    fn struct_serde_default_fn() {
        #[derive(TomlExample, Deserialize, PartialEq)]
        #[serde(default = "default")]
        struct Foo {
            bar: String,
            #[toml_example(default = "field override")]
            baz: String,
        }
        fn default() -> Foo {
            Foo {
                bar: String::from("hello world"),
                baz: String::from("custom default"),
            }
        }
        assert_eq!(
            Foo::toml_example(),
            r##"bar = "hello world"

baz = "field override"

"##
        );
    }

    #[test]
    fn struct_toml_example_default() {
        #[derive(TomlExample, Deserialize, PartialEq)]
        #[toml_example(default)]
        struct Foo {
            #[serde(default = "paru")]
            yay: &'static str,
            aur_is_useful: bool,
        }
        impl Default for Foo {
            fn default() -> Self {
                Foo {
                    yay: "yay!",
                    aur_is_useful: true,
                }
            }
        }
        fn paru() -> &'static str {
            "no, paru!"
        }
        assert_eq!(
            Foo::toml_example(),
            r##"yay = "no, paru!"

aur_is_useful = true

"##
        );
    }

    #[test]
    fn no_nesting() {
        /// Inner is a config live in Outer
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Inner {
            /// Inner.a should be a number
            a: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Outer {
            /// Outer.inner is a complex struct
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex struct
inner = ""

"#
        );
    }

    #[test]
    fn nesting() {
        /// Inner is a config live in Outer
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Inner {
            /// Inner.a should be a number
            a: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Outer {
            /// Outer.inner is a complex struct
            #[toml_example(nesting)]
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex struct
# Inner is a config live in Outer
[inner]
# Inner.a should be a number
a = 0

"#
        );
        assert_eq!(
            toml::from_str::<Outer>(&Outer::toml_example()).unwrap(),
            Outer::default()
        );
    }

    #[test]
    fn nesting_by_section() {
        /// Inner is a config live in Outer
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Inner {
            /// Inner.a should be a number
            a: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Outer {
            /// Outer.inner is a complex struct
            #[toml_example(nesting = section)]
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex struct
# Inner is a config live in Outer
[inner]
# Inner.a should be a number
a = 0

"#
        );
        assert_eq!(
            toml::from_str::<Outer>(&Outer::toml_example()).unwrap(),
            Outer::default()
        );
    }

    #[test]
    fn nesting_by_prefix() {
        /// Inner is a config live in Outer
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Inner {
            /// Inner.a should be a number
            a: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Outer {
            /// Outer.inner is a complex struct
            #[toml_example(nesting = prefix)]
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex struct
# Inner is a config live in Outer
# Inner.a should be a number
inner.a = 0

"#
        );
        assert_eq!(
            toml::from_str::<Outer>(&Outer::toml_example()).unwrap(),
            Outer::default()
        );
    }

    #[test]
    fn nesting_vector() {
        /// Service with specific port
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            port: usize,
        }
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Node {
            /// Services are running in the node
            #[toml_example(nesting)]
            services: Vec<Service>,
        }
        assert_eq!(
            Node::toml_example(),
            r#"# Services are running in the node
# Service with specific port
[[services]]
# port should be a number
port = 0

"#
        );
        assert!(toml::from_str::<Node>(&Node::toml_example()).is_ok());
    }

    #[test]
    fn nesting_hashmap() {
        /// Service with specific port
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            port: usize,
        }
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Node {
            /// Services are running in the node
            #[toml_example(nesting)]
            services: HashMap<String, Service>,
        }
        assert_eq!(
            Node::toml_example(),
            r#"# Services are running in the node
# Service with specific port
[services.example]
# port should be a number
port = 0

"#
        );
        assert!(toml::from_str::<Node>(&Node::toml_example()).is_ok());
    }

    #[test]
    fn optional_nesting() {
        /// Inner is a config live in Outer
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Inner {
            /// Inner.a should be a number
            a: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Outer {
            /// Outer.inner is a complex struct
            #[toml_example(nesting)]
            inner: Option<Inner>,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex struct
# Inner is a config live in Outer
# [inner]
# Inner.a should be a number
# a = 0

"#
        );
        assert_eq!(
            toml::from_str::<Outer>(&Outer::toml_example()).unwrap(),
            Outer::default()
        );
    }

    #[test]
    fn optional_nesting_by_section() {
        /// Inner is a config live in Outer
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Inner {
            /// Inner.a should be a number
            a: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Outer {
            /// Outer.inner is a complex struct
            #[toml_example(nesting = section)]
            inner: Option<Inner>,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex struct
# Inner is a config live in Outer
# [inner]
# Inner.a should be a number
# a = 0

"#
        );
        assert_eq!(
            toml::from_str::<Outer>(&Outer::toml_example()).unwrap(),
            Outer::default()
        );
    }

    #[test]
    fn optional_nesting_by_prefix() {
        /// Inner is a config live in Outer
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Inner {
            /// Inner.a should be a number
            a: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Outer {
            /// Outer.inner is a complex struct
            #[toml_example(nesting = prefix)]
            inner: Option<Inner>,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex struct
# Inner is a config live in Outer
# Inner.a should be a number
# inner.a = 0

"#
        );
        assert_eq!(
            toml::from_str::<Outer>(&Outer::toml_example()).unwrap(),
            Outer::default()
        );
    }

    #[test]
    fn optional_nesting_vector() {
        /// Service with specific port
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            port: usize,
        }
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Node {
            /// Services are running in the node
            #[toml_example(nesting)]
            services: Option<Vec<Service>>,
        }
        assert_eq!(
            Node::toml_example(),
            r#"# Services are running in the node
# Service with specific port
# [[services]]
# port should be a number
# port = 0

"#
        );
        assert!(toml::from_str::<Node>(&Node::toml_example()).is_ok());
    }

    #[test]
    fn optional_nesting_hashmap() {
        /// Service with specific port
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            port: usize,
        }
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Node {
            /// Services are running in the node
            #[toml_example(nesting)]
            services: Option<HashMap<String, Service>>,
        }
        assert_eq!(
            Node::toml_example(),
            r#"# Services are running in the node
# Service with specific port
# [services.example]
# port should be a number
# port = 0

"#
        );
        assert!(toml::from_str::<Node>(&Node::toml_example()).is_ok());
    }

    #[test]
    fn nesting_hashmap_with_default_name() {
        /// Service with specific port
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            #[toml_example(default = 80)]
            port: usize,
        }
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Node {
            /// Services are running in the node
            #[toml_example(nesting)]
            #[toml_example(default = http)]
            services: HashMap<String, Service>,
        }
        assert_eq!(
            Node::toml_example(),
            r#"# Services are running in the node
# Service with specific port
[services.http]
# port should be a number
port = 80

"#
        );
        assert!(toml::from_str::<Node>(&Node::toml_example()).is_ok());
    }

    #[test]
    fn nesting_hashmap_with_dash_name() {
        /// Service with specific port
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            #[toml_example(default = 80)]
            port: usize,
        }
        #[derive(TomlExample, Deserialize)]
        #[allow(dead_code)]
        struct Node {
            /// Services are running in the node
            #[toml_example(nesting)]
            #[toml_example(default = http.01)]
            services: HashMap<String, Service>,
        }
        assert_eq!(
            Node::toml_example(),
            r#"# Services are running in the node
# Service with specific port
[services.http-01]
# port should be a number
port = 80

"#
        );
        assert!(toml::from_str::<Node>(&Node::toml_example()).is_ok());
    }

    #[test]
    fn recursive_nesting() {
        #[derive(TomlExample, Default, Debug, Deserialize, PartialEq)]
        struct Outer {
            #[toml_example(nesting)]
            _middle: Middle,
        }
        #[derive(TomlExample, Default, Debug, Deserialize, PartialEq)]
        struct Middle {
            #[toml_example(nesting)]
            _inner: Inner,
        }
        #[derive(TomlExample, Default, Debug, Deserialize, PartialEq)]
        struct Inner {
            _value: usize,
        }
        let example = Outer::toml_example();
        assert_eq!(toml::from_str::<Outer>(&example).unwrap(), Outer::default());
        assert_eq!(
            example,
            r#"[_middle]
[_middle._inner]
_value = 0

"#
        );
    }

    #[test]
    fn recursive_nesting_and_flatten() {
        #[derive(TomlExample, Default, Debug, Deserialize, PartialEq)]
        struct Outer {
            #[toml_example(nesting)]
            middle: Middle,
            #[toml_example(default = false)]
            /// Some toggle
            flag: bool,
        }
        #[derive(TomlExample, Default, Debug, Deserialize, PartialEq)]
        struct Middle {
            #[serde(flatten)]
            #[toml_example(nesting)]
            /// Values of [Inner] are flattened into [Middle]
            inner: Inner,
        }
        #[derive(TomlExample, Default, Debug, Deserialize, PartialEq)]
        struct Inner {
            #[toml_example(nesting)]
            /// [Extra] is flattened into [Middle]
            extra: Extra,
            /// `value` is defined below `extra`, but shown above
            value: usize,
        }
        #[derive(TomlExample, Debug, Deserialize, PartialEq)]
        #[toml_example(default)]
        struct Extra {
            name: String,
        }
        impl Default for Extra {
            fn default() -> Self {
                Self {
                    name: String::from("ferris"),
                }
            }
        }
        let example = Outer::toml_example();
        assert_eq!(toml::from_str::<Outer>(&example).unwrap(), Outer::default());
        assert_eq!(
            example,
            r#"# Some toggle
flag = false

[middle]
# Values of [Inner] are flattened into [Middle]
# `value` is defined below `extra`, but shown above
value = 0

# [Extra] is flattened into [Middle]
[middle.extra]
name = "ferris"

"#
        );
    }

    #[test]
    fn require() {
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
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
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a is an optional number
a = 0

# Config.b is an optional string
b = ""

c = "third"

"#
        );
    }

    #[test]
    fn skip() {
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a is a number
            a: usize,
            #[toml_example(skip)]
            b: usize,
            #[serde(skip)]
            c: usize,
            #[serde(skip_deserializing)]
            d: usize,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a is a number
a = 0

"#
        );
    }

    #[test]
    fn is_enum() {
        fn b() -> AB {
            AB::B
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Config {
            /// Config.ab is an enum
            #[toml_example(enum, default)]
            ab: AB,
            /// Config.ab2 is an enum too
            #[toml_example(is_enum)]
            #[serde(default)]
            ab2: AB,
            /// Config.ab3 is an enum as well
            #[toml_example(is_enum)]
            #[serde(default = "b")]
            ab3: AB,
        }
        #[derive(Debug, Default, Deserialize, PartialEq)]
        enum AB {
            #[default]
            A,
            B,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.ab is an enum
ab = "A"

# Config.ab2 is an enum too
ab2 = "A"

# Config.ab3 is an enum as well
ab3 = "B"

"#
        );
    }

    #[test]
    fn flatten() {
        #[derive(TomlExample)]
        struct ItemWrapper {
            #[toml_example(flatten, nesting)]
            _item: Item,
        }
        #[derive(TomlExample)]
        struct Item {
            _value: String,
        }
        assert_eq!(ItemWrapper::toml_example(), Item::toml_example());
    }

    #[test]
    fn flatten_order() {
        #[derive(TomlExample)]
        struct Outer {
            #[toml_example(nesting)]
            _nested: Item,
            #[toml_example(flatten, nesting)]
            _flattened: Item,
        }
        #[derive(TomlExample)]
        struct Item {
            _value: String,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"_value = ""

[_nested]
_value = ""

"#
        );
    }

    #[test]
    fn multi_attr_escaping() {
        #[derive(TomlExample, Deserialize, PartialEq)]
        struct ConfigWrapper {
            #[toml_example(default = ["hello", "{nice :)\""], require)]
            vec: Option<Vec<String>>,

            #[toml_example(require, default = ["\"\\\n}])", "super (fancy\\! :-) )"])]
            list: Option<[String; 2]>,
        }

        assert_eq!(
            ConfigWrapper::toml_example(),
            r#"vec = ["hello", "{nice :)\""]

list = ["\"\\\n}])", "super (fancy\\! :-) )"]

"#
        );
    }

    #[test]
    fn r_sharp_field() {
        #[derive(TomlExample)]
        #[allow(dead_code)]
        struct Config {
            /// Config.type is a number
            r#type: usize,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.type is a number
type = 0

"#
        );
    }

    #[test]
    fn non_nesting_field_should_be_first() {
        #[derive(TomlExample)]
        #[allow(dead_code)]
        struct Foo {
            a: String,
        }

        #[derive(TomlExample)]
        #[allow(dead_code)]
        struct Bar {
            #[toml_example(nesting)]
            foo: Foo,
            b: String,
        }

        assert_eq!(
            Bar::toml_example(),
            r#"b = ""

[foo]
a = ""

"#
        );
    }

    #[test]
    fn rename() {
        use serde::Serialize;

        #[derive(Deserialize, Serialize, TomlExample)]
        struct Config {
            #[serde(rename = "bb")]
            b: usize,
        }
        assert_eq!(
            Config::toml_example(),
            r#"bb = 0

"#
        );
    }

    #[test]
    fn rename_all() {
        use serde::Serialize;

        #[derive(Deserialize, Serialize, TomlExample)]
        #[serde(rename_all = "kebab-case")]
        struct Config {
            a_a: usize,
        }
        assert_eq!(
            Config::toml_example(),
            r#"a-a = 0

"#
        );
    }

    #[test]
    fn hashset_and_struct() {
        use std::collections::HashMap;

        #[derive(TomlExample)]
        #[allow(dead_code)]
        struct Foo {
            a: String,
        }

        #[derive(TomlExample)]
        #[allow(dead_code)]
        struct Bar {
            /// Default instances doc
            #[toml_example(nesting)]
            default: Foo,

            /// Instances doc
            #[toml_example(nesting)]
            instance: HashMap<String, Foo>,
        }

        assert_eq!(
            Bar::toml_example(),
            r#"# Default instances doc
[default]
a = ""

# Instances doc
[instance.example]
a = ""

"#
        );
    }
}
