#[ctor::ctor]
fn init() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace"),
    );
}

mod tests {
    use directory_packages_props_converter::converter::{replace_package_reference_element};

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
