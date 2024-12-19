use std::fs::{self, create_dir};
use std::fmt::Display;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Result;


pub trait Entry {
    fn create(&self, path: &Path) -> Result<()>;
}

pub trait File: Display {}

impl<T: File> Entry for T {
    fn create(&self, path: &Path) -> Result<()> {
        let mut buffer = fs::File::create_new(path)?;
        write!(buffer, "{}", self)?;
        Ok(())
    }
}

pub type Directory<'a> = HashMap<OsString, &'a dyn Entry>;

impl Entry for Directory<'_> {
    fn create(&self, path: &Path) -> Result<()> {
        create_dir(path)?;
        for (name, entry) in self {
            entry.create(&path.join(name))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    use std::fmt;

    use tempdir::TempDir;

    #[test]
    fn directory_create() {
        const entry_name: &str = "test_directory";

        let temp_dir = TempDir::new(module_path!()).expect("couldn't create temp dir");

        let path = temp_dir.path().join(entry_name);
        assert!(!path.exists(), "test is invalid, '{entry_name}' already exists");

        let entry = Directory::new();
        entry.create(&path).expect(&format!("couldn't create '{entry_name}'"));
        assert!(path.exists(), "didn't create '{entry_name}'");

        temp_dir.close().expect("couldn't close temp dir");
    }

    type StringFile = String;
    impl File for StringFile {}
    #[test]
    fn file_create() {
        const entry_name: &str = "test_file";

        let temp_dir = TempDir::new(module_path!()).expect("Should have been able to create temp dir");

        let path = temp_dir.path().join(entry_name);
        assert!(!path.exists(), "'{entry_name}' exist before test ran");

        const file_contents: &str = "Hello World!";
        let entry = StringFile::from(file_contents);
        entry.create(&path).expect(&format!("Should have created '{entry_name}'"));
        assert!(path.exists(), "'{entry_name}' does not exists");

        let read_file_contents = fs::read_to_string(path)
            .expect("Should have been able to read the file");
        assert_eq!(read_file_contents, file_contents, "'{entry_name}' does not contain the correct contents");

        temp_dir.close().expect("couldn't close temp dir");
    }
}
