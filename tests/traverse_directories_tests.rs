#[ctor::ctor]
fn init() {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace"),
    )
    .is_test(true)
    .try_init();
}

// Shared mock filesystem module for integration tests in this directory
#[path = "common/mod.rs"]
mod common;

mod tests {
    use std::collections::{HashMap};
    use std::path::PathBuf;

    use directory_packages_props_converter::converter::{traverse_directories, PackageVersion};
    use directory_packages_props_converter::fs::Fs;
    use super::common::MockFs;

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
