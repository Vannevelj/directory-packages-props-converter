#[ctor::ctor]
fn init() {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace"),
    )
    .is_test(true)
    .try_init();
}

mod tests {
    use std::collections::{HashMap};
    use std::path::PathBuf;

    use directory_packages_props_converter::converter::{traverse_directories, PackageVersion};
    use directory_packages_props_converter::fs::Fs;

    #[derive(Default)]
    struct MockFs {
        files_with_contents: HashMap<PathBuf, String>,
        dirs: HashMap<PathBuf, Vec<PathBuf>>, // directory -> children (files or dirs)
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
            // Build components progressively: root, root/A, root/A/B, ...
            let mut parts = full_path.components();
            if let Some(first) = parts.next() {
                let mut cur = PathBuf::from(first.as_os_str());
                // The first component should be a directory key as traversal starts at root
                self.ensure_dir(&cur);

                for comp in parts {
                    let next = cur.join(comp.as_os_str());
                    // If this is the last component and represents a file, don't ensure it as a dir
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

        // Public helper used by tests: provide a file path like "dir/sub/file.ext" and optional content
        // It will create the necessary directory entries and add the file content
        fn with_tree_file(mut self, path: &str, content: Option<String>) -> Self {
            let pb = PathBuf::from(path);
            // default minimal project XML when not specified
            let content = content.unwrap_or_else(|| r##"<Project Sdk="Microsoft.NET.Sdk"></Project>"##.to_string());
            self.add_path_hierarchy(&pb, true);
            self.files_with_contents.insert(pb, content);
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
            Ok(self.files_with_contents.get(&PathBuf::from(path)).unwrap().to_string())
        }
    }

    fn run_traverse(fs: &impl Fs, root: &PathBuf) -> HashMap<PathBuf, Vec<PackageVersion>> {
        let mut files_of_interest: HashMap<PathBuf, Vec<PackageVersion>> = HashMap::new();
        traverse_directories(fs, root, &mut files_of_interest);
        files_of_interest
    }

    #[test]
    fn traverse_collects_interesting_files() {
        // Build a fake tree under root
        // root
        // ├─ A
        // │  ├─ project.csproj (1 semver ref)
        // │  └─ ignore.txt
        // ├─ B
        // │  └─ nested
        // │     └─ Directory.Build.props (2 refs: one range, one semver)
        // └─ not_interesting.xml (ignored)

        let root = PathBuf::from("root");
        let csproj = PathBuf::from("root/A/project.csproj");
        let ignore = PathBuf::from("root/A/ignore.txt");
        let props = PathBuf::from("root/B/nested/Directory.Build.props");
        let other = PathBuf::from("root/not_interesting.xml");

        let mock = MockFs::default()
            .with_tree_file(csproj.to_str().unwrap(), None)
            .with_tree_file(ignore.to_str().unwrap(), None)
            .with_tree_file(props.to_str().unwrap(), None)
            .with_tree_file(other.to_str().unwrap(), None);

        let files_of_interest = run_traverse(&mock, &root);

        // We expect exactly the csproj and the Directory.Build.props entries
        assert_eq!(2, files_of_interest.len());
        assert!(files_of_interest.contains_key(&csproj));
        assert!(files_of_interest.contains_key(&props));
        assert!(!files_of_interest.contains_key(&other));
    }

    #[test]
    fn traverse_ignores_non_interesting_files_in_all_levels() {
        // Ensure random files and xml not named Directory.Build.props are ignored
        let root = PathBuf::from("root2");
        let ignore1 = PathBuf::from("root2/notes.txt");
        let ignore2 = PathBuf::from("root2/A/some.xml");
        let csproj = PathBuf::from("root2/A/app.csproj");

        let mock = MockFs::default()
            .with_tree_file(ignore1.to_str().unwrap(), None)
            .with_tree_file(ignore2.to_str().unwrap(), None)
            .with_tree_file(csproj.to_str().unwrap(), None);

        let files_of_interest = run_traverse(&mock, &root);
        assert_eq!(1, files_of_interest.len());
        assert!(files_of_interest.contains_key(&csproj));
        assert!(!files_of_interest.contains_key(&ignore1));
        assert!(!files_of_interest.contains_key(&ignore2));
    }

    #[test]
    fn traverse_multiple_interesting_files_across_levels() {
        // Add interesting files at root and nested paths
        let root = PathBuf::from("root3");
        let csproj_root = PathBuf::from("root3/app.csproj");
        let props_nested = PathBuf::from("root3/sub/Directory.Build.props");

        let mock = MockFs::default()
            .with_tree_file(csproj_root.to_str().unwrap(), None)
            .with_tree_file(props_nested.to_str().unwrap(), None);

        let files_of_interest = run_traverse(&mock, &root);
        assert_eq!(2, files_of_interest.len());
        assert!(files_of_interest.contains_key(&csproj_root));
        assert!(files_of_interest.contains_key(&props_nested));
    }

    #[test]
    fn traverse_respects_gitignore_bin_obj() {
        // .gitignore excludes bin/ and obj/ directories
        let root = PathBuf::from("root_git");
        let gitignore = "bin/\nobj/\n";

        let app = PathBuf::from("root_git/A/app.csproj");
        let bin_csproj = PathBuf::from("root_git/A/bin/skip.csproj");
        let obj_props = PathBuf::from("root_git/A/obj/Directory.Build.props");

        let mock = MockFs::default()
            .with_tree_file("root_git/.gitignore", Some(gitignore.to_string()))
            .with_tree_file(app.to_str().unwrap(), None)
            .with_tree_file(bin_csproj.to_str().unwrap(), None)
            .with_tree_file(obj_props.to_str().unwrap(), None);

        let files_of_interest = run_traverse(&mock, &root);
        assert!(files_of_interest.contains_key(&app));
        assert!(!files_of_interest.contains_key(&bin_csproj));
        assert!(!files_of_interest.contains_key(&obj_props));
        assert_eq!(1, files_of_interest.len());
    }

    #[test]
    fn traverse_gitignore_root_anchored_file() {
        // .gitignore excludes a specific file at the root path using an anchored pattern
        let root = PathBuf::from("root_git2");
        let gitignore = "/A/skip.csproj\n";

        let keep = PathBuf::from("root_git2/B/skip.csproj");
        let skip = PathBuf::from("root_git2/A/skip.csproj");

        let mock = MockFs::default()
            .with_tree_file("root_git2/.gitignore", Some(gitignore.to_string()))
            .with_tree_file(skip.to_str().unwrap(), None)
            .with_tree_file(keep.to_str().unwrap(), None);

        let files_of_interest = run_traverse(&mock, &root);
        assert!(!files_of_interest.contains_key(&skip));
        assert!(files_of_interest.contains_key(&keep));
        assert_eq!(1, files_of_interest.len());
    }
}
