use std::fs::{self, create_dir};
use std::fmt::Display;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Result;
use std::boxed::Box;

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

pub type Directory<'a> = HashMap<OsString, Box<dyn Entry + 'a>>;

impl<'a> Entry for Directory<'a> {
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

    use tempdir::TempDir;

    type StringFile = String;
    impl File for StringFile {}

    type CharFile = char;
    impl File for CharFile {}

    #[test]
    fn file_create() {
        const ENTRY_NAME: &str = "test_file";

        let temp_dir = TempDir::new(module_path!()).expect("should have created temp dir");

        let path = temp_dir.path().join(ENTRY_NAME);
        assert!(!path.exists(), "'{ENTRY_NAME}' exist before test ran");

        const FILE_CONTENTS: &str = "Hello World!";
        let entry = StringFile::from(FILE_CONTENTS);
        entry.create(&path).unwrap_or_else(|_| panic!("should have created '{ENTRY_NAME}'"));
        assert!(path.exists(), "'{ENTRY_NAME}' does not exists");

        let read_file_contents = fs::read_to_string(path)
            .expect("should have been able to read the file");
        assert_eq!(read_file_contents, FILE_CONTENTS, "'{ENTRY_NAME}' does not contain the correct contents");

        temp_dir.close().expect("should have closed temp dir");
    }

    #[test]
    fn directory_create() {
        const ENTRY_NAME: &str = "test_directory";

        let temp_dir = TempDir::new(module_path!()).expect("couldn't create temp dir");

        let path = temp_dir.path().join(ENTRY_NAME);
        assert!(!path.exists(), "test is invalid, '{ENTRY_NAME}' already exists");

        let entry = Directory::new();
        entry.create(&path).unwrap_or_else(|_| panic!("couldn't create '{ENTRY_NAME}'"));
        assert!(path.exists(), "didn't create '{ENTRY_NAME}'");

        temp_dir.close().expect("should have closed temp dir");
    }

    #[test]
    fn sub_directories_create() {
        const ENTRY_NAME: &str = "test_directory";

        let temp_dir = TempDir::new(module_path!()).expect("couldn't create temp dir");

        let path = temp_dir.path().join(ENTRY_NAME);
        assert!(!path.exists(), "test is invalid, '{ENTRY_NAME}' already exists");

        let entry: Directory = ('a'..'d')
            .map(String::from)
            .map(|c| (OsString::from(c), Box::new(Directory::new()) as Box<dyn Entry>))
            .collect();
        entry.create(&path).unwrap_or_else(|_| panic!("couldn't create '{ENTRY_NAME}'"));
        assert!(path.exists(), "didn't create '{ENTRY_NAME}'");

        for (name, _entry) in entry {
            assert!(&path.join(&name).exists(), "didn't create '{ENTRY_NAME}/{name:?}'");
        }

        temp_dir.close().expect("should have closed temp dir");
    }

    #[test]
    fn sub_files_create() {
        const ENTRY_NAME: &str = "test_directory";

        let temp_dir = TempDir::new(module_path!()).expect("couldn't create temp dir");

        let path = temp_dir.path().join(ENTRY_NAME);
        assert!(!path.exists(), "test is invalid, '{ENTRY_NAME}' already exists");

        let entry: Directory = ('a'..'d')
            .map(|c| (OsString::from(c.clone().to_string()), Box::new(CharFile::from(c.clone())) as Box<dyn Entry>))
            .collect();
        entry.create(&path).unwrap_or_else(|_| panic!("couldn't create '{ENTRY_NAME}'"));
        assert!(path.exists(), "didn't create '{ENTRY_NAME}'");

        for (name, _entry) in entry {
            let local_path = &path.join(&name);
            assert!(local_path.exists(), "didn't create '{ENTRY_NAME}/{name:?}'");
            let read_file_contents = fs::read_to_string(local_path)
                .expect("should have been able to read the file");
            assert_eq!(*read_file_contents, *name, "'{ENTRY_NAME}/{name:?}' does not contain the correct contents");
        }

        temp_dir.close().expect("should have closed temp dir");
    }
}
