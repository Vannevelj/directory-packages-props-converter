mod options;
use log::{debug, info};
use regex::Regex;
use roxmltree::Document;
use semver::Version;
use std::io::Write;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use crate::options::Options as CLIopts;

#[derive(Hash, Eq, PartialEq, Debug)]
struct PackageVersion {
    name: String,
    version: Version,
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = CLIopts::from_args();

    let mut files_of_interest: HashMap<PathBuf, Vec<PackageVersion>> = HashMap::new();

    traverse_directories(&expand_path(&args.path), &mut files_of_interest);
    write_directory_packages_props_file(&files_of_interest, &args.path);
    strip_version_attributes(&files_of_interest);
}

fn expand_path(input: &PathBuf) -> PathBuf {
    let path = parse_path(input);
    let expanded_path: String = shellexpand::tilde(&path).to_string();
    Path::new(&expanded_path).to_owned()
}

fn parse_path(path: &PathBuf) -> String {
    path.to_owned()
        .into_os_string()
        .into_string()
        .expect("Failed to parse path")
}

fn traverse_directories(
    path: &PathBuf,
    files_of_interest: &mut HashMap<PathBuf, Vec<PackageVersion>>,
) {
    let metadata = fs::metadata(path).expect("Failed to retrieve file metadata");
    if metadata.is_file() {
        let is_interesting = path
            .extension()
            .is_some_and(|ext| ext.to_os_string() == "csproj")
            || path
                .file_name()
                .is_some_and(|name| name == "Directory.Build.props");

        if is_interesting {
            // Gather the versions for each <PackageReference> in the file
            let package_versions = parse_package_versions(path);
            files_of_interest.insert(path.to_owned(), package_versions);
        }

        return;
    }

    debug!("Diving into new directory: {:?}", path);

    for entry in fs::read_dir(path).unwrap().flatten() {
        traverse_directories(&entry.path(), files_of_interest);
    }
}

fn parse_package_versions(path: &PathBuf) -> Vec<PackageVersion> {
    let contents = fs::read_to_string(path).expect("Failed to read file");
    let xml_document = Document::parse(&contents).expect("Failed to parse XML");

    let package_reference_nodes = xml_document
        .descendants()
        .filter(|node| node.tag_name().name() == "PackageReference");
    let mut package_versions: Vec<PackageVersion> = Vec::new();

    for package_reference_node in package_reference_nodes {
        let version_attribute = package_reference_node.attribute("Version");
        let include_attribute = package_reference_node.attribute("Include");

        let version = version_attribute.and_then(|attr| Version::parse(attr).ok());

        match (version, include_attribute) {
            (Some(version), Some(name)) => package_versions.push(PackageVersion {
                name: name.to_string(),
                version,
            }),
            _ => (),
        };
    }

    package_versions
}

fn write_directory_packages_props_file(
    files_of_interest: &HashMap<PathBuf, Vec<PackageVersion>>,
    root: &Path,
) {
    let all_references = files_of_interest.values().flatten();
    let mut chosen_references: HashMap<String, &PackageVersion> = HashMap::new();

    for reference in all_references {
        let existing_reference = chosen_references.get(&reference.name);
        match existing_reference {
            Some(existing) if reference.version > existing.version => {
                debug!(
                    "Replacing {} {} with {}",
                    reference.name, existing.version, reference.version
                );
                chosen_references.insert(reference.name.to_owned(), reference);
            }
            None => {
                debug!("Adding {} {}", reference.name, reference.version);
                chosen_references.insert(reference.name.to_owned(), reference);
            }
            _ => (),
        }
    }

    for (_, chosen_reference) in &chosen_references {
        debug!(
            "Selected {}: {}",
            chosen_reference.name, chosen_reference.version
        );
    }

    // Write output to indicate which references got their version lifted
    for (filename, dependencies) in files_of_interest {
        for dependency in dependencies {
            let selected_dependency = chosen_references
                .get(&dependency.name)
                .expect("Failed to find selected dependency version");

            // https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
            if dependency.version != selected_dependency.version {
                info!(
                    "{}: Upgrading {} from \x1b[93m{}\x1b[0m to \x1b[92m{}\x1b[0m",
                    strip_path(&filename),
                    dependency.name,
                    dependency.version,
                    selected_dependency.version
                )
            }
        }
    }

    let directory_packages_props_file = root.join("Directory.Packages.props");
    info!(
        "Writing Directory.Packages.props to {}",
        parse_path(&directory_packages_props_file)
    );
    let mut directory_packages_props_file = File::create(directory_packages_props_file)
        .expect("Failed to create Directory.Packages.props file");

    let mut contents = r#"
<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
  </PropertyGroup>

  <ItemGroup>

"#
    .to_owned();

    let mut sorted_references: Vec<&PackageVersion> =
        chosen_references.into_values().into_iter().collect();
    sorted_references.sort_unstable_by_key(|dep| &dep.name);
    for package in sorted_references {
        contents.push_str(
            format!(
                "    <PackageVersion Include=\"{}\" Version=\"{}\" />\n",
                package.name, package.version
            )
            .as_str(),
        );
    }

    contents.push_str(
        r#"  
  </ItemGroup>
</Project>"#,
    );

    directory_packages_props_file
        .write_all(contents.as_bytes())
        .expect("Failed to write Directory.Packages.props file");
}

fn strip_path(path: &Path) -> String {
    let path = path.file_name().unwrap();
    path.to_str().to_owned().unwrap().to_string()
}

fn strip_version_attributes(files_of_interest: &HashMap<PathBuf, Vec<PackageVersion>>) {
    let re = Regex::new("(?<rest><PackageReference.*)(?<version> Version=\".*?\")").unwrap();

    for (file, _) in files_of_interest {
        let contents = fs::read_to_string(file).expect("Failed to read file");
        let result = re.replace_all(&contents, "$rest").to_string();

        let mut file = File::create(file).expect("Failed to open file for writing");
        file.write_all(result.as_bytes())
            .expect("Failed to write <PackageReference> updates");
    }
}
