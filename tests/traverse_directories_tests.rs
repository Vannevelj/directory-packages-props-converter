#[ctor::ctor]
fn init() {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace"),
    )
    .is_test(true)
    .try_init();
}

mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use directory_packages_props_converter::converter::{
        traverse_directories_with_fs, Fs, PackageVersion,
    };

    #[derive(Default)]
    struct MockFs {
        files: HashMap<PathBuf, String>,
        dirs: HashMap<PathBuf, Vec<PathBuf>>, // directory -> children (files or dirs)
    }

    impl MockFs {
        fn with_dir(mut self, dir: &str, children: Vec<&str>) -> Self {
            let dir_pb = PathBuf::from(dir);
            let child_pbs: Vec<PathBuf> = children.into_iter().map(PathBuf::from).collect();
            self.dirs.insert(dir_pb, child_pbs);
            self
        }

        fn with_file(mut self, path: &str, contents: &str) -> Self {
            self.files.insert(PathBuf::from(path), contents.to_string());
            self
        }
    }

    impl Fs for MockFs {
        fn metadata_is_file(&self, path: &std::path::Path) -> bool {
            self.files.contains_key(&PathBuf::from(path))
        }

        fn read_dir(&self, path: &std::path::Path) -> Vec<PathBuf> {
            self.dirs
                .get(&PathBuf::from(path))
                .cloned()
                .unwrap_or_default()
        }

        fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
            self.files
                .get(&PathBuf::from(path))
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
        }
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

        let csproj_xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Pkg.One" Version="1.2.3" />
  </ItemGroup>
  <PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup>
</Project>
"#;

        let props_xml = r#"
<Project>
  <ItemGroup>
    <PackageReference Include="Pkg.Range" Version="[1.0, 2)" />
    <PackageReference Include="Pkg.Two" Version="2.0.0" />
  </ItemGroup>
</Project>
"#;

        let mock = MockFs::default()
            .with_dir("root", vec!["root/A", "root/B", "root/not_interesting.xml"]) // files may appear in read_dir listing
            .with_dir("root/A", vec!["root/A/project.csproj", "root/A/ignore.txt"])
            .with_dir("root/B", vec!["root/B/nested"]) // nested dir
            .with_dir("root/B/nested", vec!["root/B/nested/Directory.Build.props"]) // file
            .with_file(csproj.to_str().unwrap(), csproj_xml)
            .with_file(ignore.to_str().unwrap(), "hello")
            .with_file(props.to_str().unwrap(), props_xml)
            .with_file(other.to_str().unwrap(), "<Project></Project>");

        let mut files_of_interest: HashMap<PathBuf, Vec<PackageVersion>> = HashMap::new();
        traverse_directories_with_fs(&mock, &root, &mut files_of_interest);

        // We expect exactly the csproj and the Directory.Build.props entries
        assert_eq!(2, files_of_interest.len());
        assert!(files_of_interest.contains_key(&csproj));
        assert!(files_of_interest.contains_key(&props));
        assert!(!files_of_interest.contains_key(&other));
    }
}
