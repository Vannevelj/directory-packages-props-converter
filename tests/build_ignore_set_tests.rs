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
    use std::path::PathBuf;

    use super::common::MockFs;
    use directory_packages_props_converter::converter::build_ignore_set;

    #[test]
    fn no_gitignore_file_means_no_exclusions() {
        let root = PathBuf::from("root_none");
        let mock = MockFs::default();

        let ignore = build_ignore_set(&mock, &root);
        assert!(ignore.is_none());
    }

    #[test]
    fn empty_gitignore_has_no_effect() {
        let root = PathBuf::from("root_empty");
        let mock = MockFs::default().with_file("root_empty/.gitignore", "");

        let ignore = build_ignore_set(&mock, &root);
        assert!(ignore.is_none());
    }

    #[test]
    fn comments_only_gitignore_is_ignored() {
        let root = PathBuf::from("root_comments");
        let mock =
            MockFs::default().with_file("root_comments/.gitignore", "# comment\n  \n# another\n");

        let ignore = build_ignore_set(&mock, &root);
        assert!(ignore.is_none());
    }

    #[test]
    fn malformed_patterns_are_skipped() {
        let root = PathBuf::from("root_malformed");
        // Both lines are malformed for globset; they should be dropped
        let mock = MockFs::default().with_file("root_malformed/.gitignore", "[\n**/*[\n");

        let ignore = build_ignore_set(&mock, &root);
        assert!(ignore.is_none());
    }

    #[test]
    fn negations_are_currently_ignored() {
        let root = PathBuf::from("root_neg");
        // Lines starting with '!' are ignored by our implementation
        let mock =
            MockFs::default().with_file("root_neg/.gitignore", "!keep.csproj\n!bin/keep.csproj\n");

        let ignore = build_ignore_set(&mock, &root);
        assert!(ignore.is_none());
    }

    #[test]
    fn valid_pattern_returns_some() {
        let root = PathBuf::from("root_valid");
        let mock = MockFs::default().with_file("root_valid/.gitignore", "bin/\n");

        let ignore = build_ignore_set(&mock, &root);
        assert!(ignore.is_some());
    }
}
