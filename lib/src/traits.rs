use std::fs::File;
use std::io::prelude::*;

pub trait TomlExample {
    /// structure to toml example
    fn toml_example() -> String;
    fn toml_example_with_prefix(label: &str, prefix: &str) -> String;
    fn to_toml_example(file_name: &str) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        file.write_all(Self::toml_example().as_bytes())?;
        Ok(())
    }
}
