# directory-packages-props-converter

Converts your projects to use [Central Package Management](https://devblogs.microsoft.com/nuget/introducing-central-package-management/). `<PackageReference>` dependencies have their `Version` removed in each individual `.csproj` file. Instead, a `Directory.Packages.props` file is created in the root folder which contains the version for each separate dependency.

## Getting started

Download the binary from the [releases](https://github.com/Vannevelj/directory-packages-props-converter/releases). 

Mac:

```sh
chmod +x ./directory-packages-props-converter
./directory-packages-props-converter .
```

If your Mac prevents you from running the executable, right-click the executable and select "Open" to override the default block.

---

Windows:

```sh
.\directory-packages-props-converter.exe .
```


Once you've ran the script, you'll see a new `Directory.Packages.props` like this:

```xml
<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
  </PropertyGroup>

  <ItemGroup>

    <PackageVersion Include="MSTest.TestAdapter" Version="3.0.2" />
    <PackageVersion Include="MSTest.TestFramework" Version="3.0.2" />
    <PackageVersion Include="Microsoft.AspNetCore.Http.Abstractions" Version="2.2.0" />
    <PackageVersion Include="Microsoft.AspNetCore.Mvc" Version="2.2.0" />
    <PackageVersion Include="Microsoft.CodeAnalysis" Version="4.4.0" />
  
  </ItemGroup>
</Project>
```

and corresponding `.csproj` changes like this

```diff
<ItemGroup>
-    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.4.1" />
-    <PackageReference Include="MSTest.TestAdapter" Version="3.0.2" />
+    <PackageReference Include="Microsoft.NET.Test.Sdk" />
+    <PackageReference Include="MSTest.TestAdapter" />
</ItemGroup>
```

## Notes

* When multiple versions are detected for a particular dependency, the highest version number is used
* Supports `.csproj` and `Directory.Build.props` files
* Partial support for version ranges like `[1.1.0, 2]`: they will be included in `Directory.Packages.props` but without trying to discover the largest version