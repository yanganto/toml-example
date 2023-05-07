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
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a is a list of number
a = [ 0, ]
# Config.b is a list of string
b = [ "", ]
"#
        );
        assert!(toml::from_str::<Config>(Config::toml_example()).is_ok())
    }
}
