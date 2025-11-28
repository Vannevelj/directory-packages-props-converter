#[ctor::ctor]
fn init() {
    // Enable logs during tests; ignore error if already initialized by other test modules
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace"),
    )
    .is_test(true)
    .try_init();
}

mod tests {
    use directory_packages_props_converter::converter::parse_package_version;

    #[test]
    fn parse_package_version_empty_project_no_refs() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
    </ItemGroup>
    <PropertyGroup>
        <TargetFramework>net8.0</TargetFramework>
    </PropertyGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(0, versions.len());
    }

    #[test]
    fn parse_package_version_without_version_attribute_is_ignored() {
        // Should be ignored because `Version` attribute is required for collection
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(0, versions.len());
    }

    #[test]
    fn parse_package_version_with_simple_version() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" Version="1.1.0" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(1, versions.len());

        // Direct assertions against exposed fields
        let p = &versions[0];
        assert_eq!("My.Reference", p.name);
        assert_eq!(Some(semver::Version::parse("1.1.0").unwrap()), p.version);
        assert_eq!("1.1.0", p.fallback_version);
    }

    #[test]
    fn parse_package_version_with_version_range() {
        // Not a valid semver for `semver::Version`, but function should still collect it
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" Version="[1.10.1, 2)" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(1, versions.len());

        // Version ranges are not parseable by semver::Version, so version should be None
        let p = &versions[0];
        assert_eq!("My.Reference", p.name);
        assert!(p.version.is_none());
        assert_eq!("[1.10.1, 2)", p.fallback_version);
    }

    #[test]
    fn parse_package_version_with_multiple_references() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" Version="[1.10.1, 2)" />
        <PackageReference Include="My.Other.Reference" Version="2.0.0" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(2, versions.len());

        // Order-agnostic checks
        let mut saw_range = false;
        let mut saw_simple = false;
        for p in versions.iter() {
            match p.name.as_str() {
                "My.Reference" => {
                    assert!(p.version.is_none());
                    assert_eq!("[1.10.1, 2)", p.fallback_version);
                    saw_range = true;
                }
                "My.Other.Reference" => {
                    assert_eq!(Some(semver::Version::parse("2.0.0").unwrap()), p.version);
                    assert_eq!("2.0.0", p.fallback_version);
                    saw_simple = true;
                }
                other => panic!("Unexpected package name parsed: {}", other),
            }
        }
        assert!(saw_range);
        assert!(saw_simple);
    }

    #[test]
    fn parse_package_version_ignores_other_elements() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <ProjectReference Include="..\\Other\\Other.csproj" />
        <None Include="README.md" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(0, versions.len());
    }
}
