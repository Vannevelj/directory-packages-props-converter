# Change Log
All notable changes to this project will be documented in this file.

## [0.3.3] - 2025-06-22
* Order dependencies case-insensitively

## [0.3.2] - 2025-02-06
* Remove blank lines in the output

## [0.3.1] - 2024-03-15
* Added Linux versions of the released binaries.

## [0.3.0] - 2023-10-20
* Updating the `<PackageReference>` is more tolerant towards additional whitespace

## [0.2.0] - 2023-10-20
* Fixed a panic when a range version is used

## [0.1.0] - 2023-08-04
* Fixed a panic when no version could be found
* Added a CLI option `--log-level` which can be used to tweak the log level (e.g. `debug`, `info`, `error`)

## [0.0.3] - 2023-08-04
* When a version range along the lines of `[1.1.0, 2]` is encountered, it will include it in `Directory.Packages.props`

## [0.0.2] - 2023-08-04
* Fixed an issue where it wouldn't correctly determine the most recent package version
* Dependencies are written in alphabetical order to `Directory.Packages.props`
* Support `Directory.Build.props`
* Fixed some whitespace in the `Directory.Packages.props` output
* Added colour coding to the terminal output

## [0.0.1] - 2023-08-03
* Initial release
