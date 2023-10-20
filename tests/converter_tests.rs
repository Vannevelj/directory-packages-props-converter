#[ctor::ctor]
fn init() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace"),
    );
}

mod tests {
    use directory_packages_props_converter::converter::{parse_package_version, replace_package_reference_element};

    #[test]
    fn parse_package_version_version() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" Version="1.1.0" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(1, versions.len());
    }

    #[test]
    fn parse_package_version_version_range() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" Version="[1.10.1, 2]" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(1, versions.len());
    }

    #[test]
    fn parse_package_version_version_mixed_with_range() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" Version="[1.10.1, 2]" />
        <PackageReference Include="My.Other.Reference" Version="2.0.0" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(2, versions.len());
    }

    #[test]
    fn parse_package_version_version_mixed_with_halfopen_range() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
    <ItemGroup>
        <PackageReference Include="My.Reference" Version="[1.10.1, 2)" />
        <PackageReference Include="My.Other.Reference" Version="2.0.0" />
    </ItemGroup>
</Project>"#;

        let versions = parse_package_version(xml.to_string());
        assert_eq!(2, versions.len());
    }

    #[test]
    fn replace_package_reference_element_regular() {
        let element = r#"<PackageReference Include="My.Reference" Version="1.1.2" />"#;

        let replacement = replace_package_reference_element(element.to_string());
        assert_eq!(r#"<PackageReference Include="My.Reference" />"#, replacement);
    }

    #[test]
    fn replace_package_reference_element_with_range() {
        let element = r#"<PackageReference Include="My.Reference" Version="[1.10.1, 2)" />"#;

        let replacement = replace_package_reference_element(element.to_string());
        assert_eq!(r#"<PackageReference Include="My.Reference" />"#, replacement);
    }

    #[test]
    fn replace_package_reference_element_with_spacing_before_value() {
        let element = r#"<PackageReference Include="My.Reference" Version= "[1.10.1, 2)" />"#;

        let replacement = replace_package_reference_element(element.to_string());
        assert_eq!(r#"<PackageReference Include="My.Reference" />"#, replacement);
    }

    #[test]
    fn replace_package_reference_element_with_spacing_after_key() {
        let element = r#"<PackageReference Include="My.Reference" Version ="[1.10.1, 2)" />"#;

        let replacement = replace_package_reference_element(element.to_string());
        assert_eq!(r#"<PackageReference Include="My.Reference" />"#, replacement);
    }

    #[test]
    fn replace_package_reference_element_with_odd_spacing() {
        let element = r#"<PackageReference Include="My.Reference" Version =   "[1.10.1, 2)"  />"#;

        let replacement = replace_package_reference_element(element.to_string());
        assert_eq!(r#"<PackageReference Include="My.Reference"  />"#, replacement);
    }
}
