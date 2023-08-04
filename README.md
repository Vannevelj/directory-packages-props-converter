# directory-packages-props-converter

Converts your projects to use [Central Package Management](https://devblogs.microsoft.com/nuget/introducing-central-package-management/). `<PackageReference>` dependencies have their `Version` removed in each individual `.csproj` file. Instead, a `Directory.Packages.props` file is created in the root folder which contains the version for each separate dependency.

## Getting started

Download the binary from the [releases](https://github.com/Vannevelj/directory-packages-props-converter/releases). 

Mac:

```sh
chmod +x ./directory-packages-props-converter
./directory-packages-props-converter .
```

Windows:

```sh
.\directory-packages-props-converter.exe .
```

## Notes

* When multiple versions are detected for a particular dependency, the highest version number is used
* Supports `.csproj` and `Directory.Build.props` files
* Partial support for version ranges like `[1.1.0, 2]`: they will be included in `Directory.Packages.props` but without trying to discover the largest version