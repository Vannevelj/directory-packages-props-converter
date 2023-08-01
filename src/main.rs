mod options;
use std::{path::{PathBuf, Path}, collections::{HashSet, HashMap}, fs, ffi::{OsStr, OsString}};

use log::{debug, info};
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
    traverse_directories(&args.path, &mut files_of_interest);
    
    // Write a Directory.Packages.props file
    // Update the <PackageReference> elements to remove the Version property
    // Write output to indicate which references got their version lifted

    for (key, value) in files_of_interest {
        println!("{}", key);
    }
}

// fn expand_path(input: PathBuf) -> PathBuf {
//     let path = parse_path(input);
//     let expanded_path: String = shellexpand::tilde(&path).to_string();
//     Path::new(&expanded_path).to_owned()
// }

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
            let package_versions: Vec<PackageVersion> = Vec::new();
            files_of_interest.insert(filename.to_owned(), package_versions);
        }

        return;
    }

    debug!("Diving into new directory: {:?}", path);

    for entry in fs::read_dir(path).unwrap().flatten() {
        let directory_name = parse_path(entry.path());

        info!("Evaluating {}", directory_name);
        traverse_directories(&entry.path(), files_of_interest);
    }
}