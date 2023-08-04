mod options;
use directory_packages_props_converter::converter::{
    parse_path, strip_version_attributes, traverse_directories,
    write_directory_packages_props_file, PackageVersion,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use crate::options::Options as CLIopts;

fn main() {
    let args = CLIopts::from_args();

    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, args.log_level),
    );

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
