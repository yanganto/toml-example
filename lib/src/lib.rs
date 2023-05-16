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
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a should be a number
a = 7

# Config.b should be a string
b = "seven"

c = "default"

"#
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
            /// Outer.inner is a complex sturct
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex sturct
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
            /// Outer.inner is a complex sturct
            #[toml_example(nesting)]
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex sturct
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
            /// Outer.inner is a complex sturct
            #[toml_example(nesting = section)]
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex sturct
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
            /// Outer.inner is a complex sturct
            #[toml_example(nesting = prefix)]
            inner: Inner,
        }
        assert_eq!(
            Outer::toml_example(),
            r#"# Outer.inner is a complex sturct
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
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            port: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
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
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
        #[allow(dead_code)]
        struct Service {
            /// port should be a number
            port: usize,
        }
        #[derive(TomlExample, Deserialize, Default, PartialEq, Debug)]
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
}
