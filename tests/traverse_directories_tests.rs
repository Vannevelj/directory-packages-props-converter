#[ctor::ctor]
fn init() {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace"),
    )
    .is_test(true)
    .try_init();
}

mod tests {
    use std::{
        collections::HashMap,
        path::PathBuf,
        collections::HashSet
    };
    use directory_packages_props_converter::converter::{
        traverse_directories_with_fs, PackageVersion,
    };
    use directory_packages_props_converter::fs::Fs;

    #[derive(Default)]
    struct MockFs {
        files: HashSet<PathBuf>,
        dirs: HashMap<PathBuf, Vec<PathBuf>>, // directory -> children (files or dirs)
    }

    impl MockFs {
        fn with_dir(mut self, dir: &str, children: Vec<&str>) -> Self {
            let dir_pb = PathBuf::from(dir);
            let child_pbs: Vec<PathBuf> = children.into_iter().map(PathBuf::from).collect();
            self.dirs.insert(dir_pb, child_pbs);
            self
        }

        fn with_file(mut self, path: &str) -> Self {
            self.files.insert(PathBuf::from(path));
            self
        }
    }

    impl Fs for MockFs {
        fn metadata_is_file(&self, path: &std::path::Path) -> bool {
            self.files.contains(&PathBuf::from(path))
        }

        fn read_dir(&self, path: &std::path::Path) -> Vec<PathBuf> {
            self.dirs
                .get(&PathBuf::from(path))
                .cloned()
                .unwrap_or_default()
        }

        fn read_to_string(&self, _path: &std::path::Path) -> std::io::Result<String> {
            Ok(String::from(r##"<Project Sdk="Microsoft.NET.Sdk"></Project>"##))
        }
    }

    fn run_traverse(fs: &impl Fs, root: &PathBuf) -> HashMap<PathBuf, Vec<PackageVersion>> {
        let mut files_of_interest: HashMap<PathBuf, Vec<PackageVersion>> = HashMap::new();
        traverse_directories_with_fs(fs, root, &mut files_of_interest);
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
            .with_dir("root", vec!["root/A", "root/B", "root/not_interesting.xml"]) // files may appear in read_dir listing
            .with_dir("root/A", vec!["root/A/project.csproj", "root/A/ignore.txt"])
            .with_dir("root/B", vec!["root/B/nested"]) // nested dir
            .with_dir("root/B/nested", vec!["root/B/nested/Directory.Build.props"]) // file
            .with_file(csproj.to_str().unwrap())
            .with_file(ignore.to_str().unwrap())
            .with_file(props.to_str().unwrap())
            .with_file(other.to_str().unwrap());

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
            .with_dir("root2", vec!["root2/A", ignore1.to_str().unwrap()])
            .with_dir("root2/A", vec![csproj.to_str().unwrap(), ignore2.to_str().unwrap()])
            .with_file(ignore1.to_str().unwrap())
            .with_file(ignore2.to_str().unwrap())
            .with_file(csproj.to_str().unwrap());

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
            .with_dir("root3", vec![csproj_root.to_str().unwrap(), "root3/sub"]) // file + dir
            .with_dir("root3/sub", vec![props_nested.to_str().unwrap()])
            .with_file(csproj_root.to_str().unwrap())
            .with_file(props_nested.to_str().unwrap());

        let files_of_interest = run_traverse(&mock, &root);
        assert_eq!(2, files_of_interest.len());
        assert!(files_of_interest.contains_key(&csproj_root));
        assert!(files_of_interest.contains_key(&props_nested));
    }
}
