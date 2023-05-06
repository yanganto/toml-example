#[doc(hidden)]
pub use toml_example_derive::TomlExample;
pub mod traits;
pub use traits::*;

#[cfg(test)]
mod tests {
    use toml_example::TomlExample;
    use crate as toml_example;

    #[test]
    fn basic() {
        #[derive(TomlExample)]
        struct Config {
            _a: usize,
            _b: String,
        }
        assert_eq!(Config::to_example(), "# Toml example for Config".to_string());
    }
}
