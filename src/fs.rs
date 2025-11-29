use std::fs;
use std::path::{Path, PathBuf};

// Abstraction over filesystem to enable testing traversal without real IO
pub trait Fs {
    fn metadata_is_file(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> Vec<PathBuf>;
    fn read_to_string(&self, path: &Path) -> std::io::Result<String>;
}

pub struct RealFs;

impl Fs for RealFs {
    fn metadata_is_file(&self, path: &Path) -> bool {
        fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
    }

    fn read_dir(&self, path: &Path) -> Vec<PathBuf> {
        fs::read_dir(path)
            .map(|rd| rd.flatten().map(|e| e.path()).collect())
            .unwrap_or_default()
    }

    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}
