mod options;
use std::{path::{PathBuf, Path}, collections::HashMap, fs};

use log::debug;
use roxmltree::Document;
use structopt::StructOpt;

use crate::options::Options as CLIopts;

struct PackageVersion {
    name: String,
    version: String,
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = CLIopts::from_args();

    let mut files_of_interest: HashMap<String, Vec<PackageVersion>> = HashMap::new();

    // look for interesting files (.csproj / Directory.Build.props)
    traverse_directories(&expand_path(args.path), &mut files_of_interest);
    
    // Write a Directory.Packages.props file
    // Update the <PackageReference> elements to remove the Version property
    // Write output to indicate which references got their version lifted

    for (file_name, package_versions) in files_of_interest {
        for package_version in package_versions {
            println!("{}: {} ({})", file_name, package_version.name, package_version.version);
        }
    }
}

fn expand_path(input: PathBuf) -> PathBuf {
    let path = parse_path(input);
    let expanded_path: String = shellexpand::tilde(&path).to_string();
    Path::new(&expanded_path).to_owned()
}

fn parse_path(path: PathBuf) -> String {
    path.to_owned().into_os_string().into_string().expect("Failed to parse path")
}

fn traverse_directories(
    path: &PathBuf,
    files_of_interest: &mut HashMap<String, Vec<PackageVersion>>,
) {
    let metadata = fs::metadata(path).expect("Failed to retrieve file metadata");
    if metadata.is_file() {
        let is_interesting = path.extension().is_some_and(|ext| ext.to_os_string() == "csproj");

        if is_interesting {
            // Gather the versions for each <PackageReference> in the file
            let filename = parse_path(path.to_owned());
            let package_versions = parse_package_versions(path);
            files_of_interest.insert(filename.to_owned(), package_versions);
        }

        return;
    }

    debug!("Diving into new directory: {:?}", path);

    for entry in fs::read_dir(path).unwrap().flatten() {
        let directory_name = parse_path(entry.path());

        debug!("Evaluating {}", directory_name);
        traverse_directories(&entry.path(), files_of_interest);
    }
}

fn parse_package_versions(path: &PathBuf) -> Vec<PackageVersion> {
    let contents = fs::read_to_string(path).expect("Failed to read file");
    let xml_document = Document::parse(&contents).expect("Failed to parse XML");

    let package_reference_nodes = xml_document.descendants().filter(|node| node.tag_name().name() == "PackageReference");
    let mut package_versions: Vec<PackageVersion> = Vec::new();

    for package_reference_node in package_reference_nodes {
        let version_attribute = package_reference_node.attribute("Version");
        let include_attribute = package_reference_node.attribute("Include");
        
        let package_version = match (version_attribute, include_attribute) {
            (Some(version), Some(name)) => Some(PackageVersion { name: name.to_string(), version: version.to_string()}),
            _ => None
        };

        if let Some(package_version) = package_version {
            package_versions.push(package_version);
        }
    }

    package_versions
}