use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub trait TomlExample {
    /// structure to toml example
    fn toml_example() -> String;

    /// structure, which is nesting or flatten inside other structure, to a toml example
    /// There will be a section `{label_format.0}{label}{lable_format.1}` for the example of struct, and `prefix` will add `# ` if it is a optional.
    fn toml_example_with_prefix(label: &str, label_format: (&str, &str), prefix: &str) -> String;

    fn to_toml_example<P: AsRef<Path>>(file_name: P) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        file.write_all(Self::toml_example().as_bytes())?;
        Ok(())
    }
}
