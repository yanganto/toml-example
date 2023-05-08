#[doc(hidden)]
pub use toml_example_derive::TomlExample;
pub mod traits;
pub use traits::*;

#[cfg(test)]
mod tests {
    use crate as toml_example;
    use serde_derive::Deserialize;
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
            toml::from_str::<Config>(Config::toml_example()).unwrap(),
            Config::default()
        )
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
            toml::from_str::<Config>(Config::toml_example()).unwrap(),
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
        assert!(toml::from_str::<Config>(Config::toml_example()).is_ok())
    }

    #[test]
    fn sturct_doc() {
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
            toml::from_str::<Config>(Config::toml_example()).unwrap(),
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
"#
        );
        assert_eq!(
            toml::from_str::<Config>(Config::toml_example()).unwrap(),
            Config::default()
        )
    }
}
