use crate::fs::Fs;
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use globset::{Glob, GlobSet, GlobSetBuilder};
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

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct PackageVersion {
    pub name: String,
    pub version: Option<Version>,
    pub fallback_version: String,
}

pub fn parse_path(path: &PathBuf) -> String {
    path.to_owned()
        .into_os_string()
        .into_string()
        .expect("Failed to parse path")
}

pub fn traverse_directories(
    fs_impl: &impl Fs,
    path: &Path,
    files_of_interest: &mut HashMap<PathBuf, Vec<PackageVersion>>,
) {
    // Build ignore set once from root .gitignore and traverse with it
    let ignore_set = build_ignore_set(fs_impl, path);
    traverse_with_ignores(fs_impl, path, path, files_of_interest, ignore_set.as_ref());
}

pub fn build_ignore_set(fs_impl: &impl Fs, root: &Path) -> Option<GlobSet> {
    let gitignore = root.join(".gitignore");
    if !fs_impl.metadata_is_file(&gitignore) {
        return None;
    }

    let Ok(contents) = fs_impl.read_to_string(&gitignore) else {
        return None;
    };

    let mut builder = GlobSetBuilder::new();
    let mut has_any = false;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // ignore negations for now
        if line.starts_with('!') {
            continue;
        }

        // Normalize: if it ends with '/', target directory and all descendants
        // If it starts with '/', anchor to root; otherwise match anywhere under root
        let pattern = if line.ends_with('/') {
            if line.starts_with('/') {
                // Remove leading '/' and trailing '/' and anchor under root
                format!("{}/**", line.trim_start_matches('/').trim_end_matches('/'))
            } else {
                format!("**/{}/**", line.trim_end_matches('/'))
            }
        } else if line.starts_with('/') {
            line.trim_start_matches('/').to_string()
        } else {
            format!("**/{line}")
        };

        // Build a Glob relative to root; globset matches on Path, so we'll pass relative paths
        if let Ok(glob) = Glob::new(&pattern) {
            builder.add(glob);
            has_any = true;
        }
    }

    if !has_any {
        return None;
    }
    builder.build().ok()
}

fn is_ignored(root: &Path, current: &Path, ignore: Option<&GlobSet>) -> bool {
    let Some(set) = ignore else {
        return false;
    };
    let rel = current.strip_prefix(root).unwrap_or(current);
    set.is_match(rel)
}

fn traverse_with_ignores(
    fs_impl: &impl Fs,
    root: &Path,
    path: &Path,
    files_of_interest: &mut HashMap<PathBuf, Vec<PackageVersion>>,
    ignore: Option<&GlobSet>,
) {
    if is_ignored(root, path, ignore) {
        debug!("Ignoring path due to .gitignore: {path:?}");
        return;
    }

    if fs_impl.metadata_is_file(path) {
        let is_interesting = path
            .extension()
            .is_some_and(|ext| ext.to_os_string() == "csproj")
            || path
                .file_name()
                .is_some_and(|name| name == "Directory.Build.props");

        if is_interesting {
            // Gather the versions for each <PackageReference> in the file
            let contents = fs_impl.read_to_string(path).expect("Failed to read file");
            let package_versions = parse_package_version(contents);
            debug!("Found {} packages in {:?}", package_versions.len(), path);
            files_of_interest.insert(path.to_owned(), package_versions);
        }

        return;
    }

    debug!("Diving into new directory: {path:?}");

    for entry_path in fs_impl.read_dir(path) {
        traverse_with_ignores(fs_impl, root, &entry_path, files_of_interest, ignore);
    }
}

pub fn parse_package_version(contents: String) -> Vec<PackageVersion> {
    let xml_document = Document::parse(&contents).expect("Failed to parse XML");

    let package_reference_nodes = xml_document
        .descendants()
        .filter(|node| node.tag_name().name() == "PackageReference");
    let mut package_versions: Vec<PackageVersion> = Vec::new();

    for package_reference_node in package_reference_nodes {
        let version_attribute = package_reference_node.attribute("Version");
        let include_attribute = package_reference_node.attribute("Include");
        debug!("Evaluating <PackageReference>: {package_reference_node:?}");

        if let Some(raw_version) = version_attribute {
            let version = version_attribute.and_then(|attr| Version::parse(attr).ok());

            match (version, include_attribute) {
                (Some(version), Some(name)) => package_versions.push(PackageVersion {
                    name: name.to_string(),
                    version: Some(version),
                    fallback_version: raw_version.to_owned(),
                }),
                (None, Some(name)) => package_versions.push(PackageVersion {
                    name: name.to_string(),
                    version: None,
                    fallback_version: raw_version.to_owned(),
                }),
                _ => (),
            };
        }
    }

    package_versions
}

fn get_version(package: &PackageVersion) -> String {
    match &package.version {
        Some(v) => v.to_owned().to_string(),
        _ => package.fallback_version.to_owned(),
    }
}

pub fn write_directory_packages_props_file(
    files_of_interest: &HashMap<PathBuf, Vec<PackageVersion>>,
    root: &Path,
) {
    let all_references = files_of_interest.values().flatten();
    let mut chosen_references: CaseInsensitiveHashMap<PackageVersion> =
        CaseInsensitiveHashMap::new();

    for reference in all_references {
        let existing_reference = chosen_references.get(reference.name.to_owned());

        match existing_reference {
            Some(existing) if reference.version > existing.version => {
                let old = get_version(existing);
                let new = get_version(existing);

                debug!("Replacing {} {} with {}", reference.name, old, new);
                chosen_references.insert(reference.name.to_owned(), reference.clone());
            }
            None => {
                debug!(
                    "Adding {} {:?}",
                    reference.name,
                    reference.fallback_version.to_owned()
                );
                chosen_references.insert(reference.name.to_owned(), reference.clone());
            }
            _ => (),
        }
    }

    debug!("Finished adding references");

    for (_, chosen_reference) in &chosen_references {
        debug!(
            "Selected {}: {}",
            chosen_reference.name,
            chosen_reference.fallback_version.to_owned()
        );
    }

    // Write output to indicate which references got their version lifted
    for (filename, dependencies) in files_of_interest {
        for dependency in dependencies {
            let selected_dependency = chosen_references
                .get(dependency.name.to_string())
                .expect("Failed to find selected dependency version");

            let old = get_version(dependency);
            let new = get_version(selected_dependency);

            // https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
            if dependency.version != selected_dependency.version {
                info!(
                    "{}: Upgrading {} from \x1b[93m{}\x1b[0m to \x1b[92m{}\x1b[0m",
                    strip_path(filename),
                    dependency.name,
                    old,
                    new
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
    // Remove leading blank lines and spaces.
    .trim_start()
    .to_owned();

    let mut sorted_references: Vec<&PackageVersion> = chosen_references.values().collect();
    sorted_references.sort_unstable_by_key(|dep| dep.name.to_lowercase());
    for package in sorted_references {
        let version_to_write = match &package.version {
            Some(v) => v.to_string(),
            _ => package.fallback_version.to_owned(),
        };

        contents.push_str(
            format!(
                "    <PackageVersion Include=\"{}\" Version=\"{}\" />\n",
                package.name, version_to_write
            )
            .as_str(),
        );
    }

    contents.push_str(
        r#"  </ItemGroup>
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

pub fn strip_version_attributes(files_of_interest: &HashMap<PathBuf, Vec<PackageVersion>>) {
    for file in files_of_interest.keys() {
        let contents = fs::read_to_string(file).expect("Failed to read file");
        let result = replace_package_reference_element(contents);
        let mut file = File::create(file).expect("Failed to open file for writing");
        file.write_all(result.as_bytes())
            .expect("Failed to write <PackageReference> updates");
    }
}

pub fn replace_package_reference_element(contents: String) -> String {
    let regex =
        Regex::new("(?<rest><PackageReference.*)(?<version> Version\\s*=\\s*\".*?\")").unwrap();

    regex.replace_all(&contents, "$rest").to_string()
}
