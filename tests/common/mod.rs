use std::collections::HashMap;
use std::path::PathBuf;

use directory_packages_props_converter::fs::Fs;

#[derive(Default, Debug)]
pub struct MockFs {
    pub files_with_contents: HashMap<PathBuf, String>,
    pub dirs: HashMap<PathBuf, Vec<PathBuf>>, // directory -> children (files or dirs)
}

impl MockFs {
    // Ensure a directory exists in the map
    fn ensure_dir(&mut self, dir: &PathBuf) {
        self.dirs.entry(dir.clone()).or_insert_with(Vec::new);
    }

    // Add a child path to a parent's children list if not present
    fn add_child_unique(&mut self, parent: &PathBuf, child: &PathBuf) {
        let entry = self.dirs.entry(parent.clone()).or_insert_with(Vec::new);
        if !entry.iter().any(|c| c == child) {
            entry.push(child.clone());
        }
    }

    // Create directory hierarchy for a given path (file or directory) and register parent->child links
    fn add_path_hierarchy(&mut self, full_path: &PathBuf, is_file: bool) {
        let mut parts = full_path.components();
        if let Some(first) = parts.next() {
            let mut cur = PathBuf::from(first.as_os_str());
            // The first component should be a directory key as traversal starts at root
            self.ensure_dir(&cur);

            for comp in parts {
                let next = cur.join(comp.as_os_str());
                let last = &next == full_path;
                if last && is_file {
                    // Register file under its parent
                    self.add_child_unique(&cur, &next);
                } else {
                    // A directory along the way
                    self.ensure_dir(&next);
                    self.add_child_unique(&cur, &next);
                    cur = next;
                }
            }
        }
    }

    // Public helper: provide a file path like "dir/sub/file.ext" and optional content
    // It will create the necessary directory entries and add the file content
    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        let pb = PathBuf::from(path);
        self.add_path_hierarchy(&pb, true);
        self.files_with_contents.insert(pb, content.to_string());
        self
    }
}

impl Fs for MockFs {
    fn metadata_is_file(&self, path: &std::path::Path) -> bool {
        self.files_with_contents.contains_key(&PathBuf::from(path))
    }

    fn read_dir(&self, path: &std::path::Path) -> Vec<PathBuf> {
        self.dirs
            .get(&PathBuf::from(path))
            .cloned()
            .unwrap_or_default()
    }

    fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
        Ok(
            self.files_with_contents
                .get(&PathBuf::from(path))
                .expect("file not found in mock")
                .to_string(),
        )
    }
}
