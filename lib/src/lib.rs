#[doc(hidden)]
pub use toml_example_derive::TomlExample;
pub mod traits;
pub use traits::*;

#[cfg(test)]
mod tests {
    use crate as toml_example;
    use toml_example::TomlExample;

    #[test]
    fn basic() {
        #[derive(TomlExample)]
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
    }

    #[test]
    fn option() {
        #[derive(TomlExample)]
        #[allow(dead_code)]
        struct Config {
            /// Config.a should be a number
            a: Option<usize>,
            /// Config.b is optional string
            b: Option<String>,
        }
        assert_eq!(
            Config::toml_example(),
            r#"# Config.a should be a number
# a = 0
# Config.b is optional string
# b = ""
"#
        );
    }
}
