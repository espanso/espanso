#!/usr/bin/env pwsh

# Stop on any error
$ErrorActionPreference = "Stop"

# Enable verbose output
# Set-PSDebug -Strict -Trace 1

# Define constants
$INSTALLER_NAME = "Espanso-Win-Installer"
$ARCH = "x86_64"
$TARGET_DIR = "target/windows/installer"
$RESOURCE_DIR = "target/windows/resources"

$BASE_DIR = Get-Location

$ResourcesDirExists = Test-Path -Path $RESOURCE_DIR
if (!$ResourcesDirExists) {
    Write-Error "You need to build the resources before building the windows installer`n Run ./scripts/build_windows_resources.ps1"
}

function Get-TomlVersion {
    $TomlPath = Join-Path $BASE_DIR "espanso/Cargo.toml"

    if (-not (Get-Module -ListAvailable -Name 'PSTOML')) {
        Install-Module -Name 'PSTOML' -Force
    }
    Import-Module 'PSTOML'

    $tomlContent = Get-Content -Path $TomlPath -Raw | ConvertFrom-Toml
    if ($tomlContent.package -and $tomlContent.package.version) {
        return $tomlContent.package.version
    }

    Write-Error "Could not find the version in Cargo.toml"

}

function Main {
    $espansod_path = Join-Path $BASE_DIR "target\windows\resources\espansod.exe"
    $espansod_exists = Test-Path $espansod_path
    if (!$espansod_exists) {
        Write-Error "You need to build the windows resources first.`nPlease run scripts/build_windows_resources.ps1"
    }

    # Clean the output directory
    if (Test-Path $TARGET_DIR) {
        Remove-Item $TARGET_DIR -Recurse -Force
    }

    # Create the target directory
    New-Item -Path $TARGET_DIR -ItemType Directory -Force | Out-Null

    $script_resources_path = Join-Path $BASE_DIR "scripts/resources/windows"
    $template_path = Join-Path $script_resources_path "setupscript.iss"

    try {
        $template = Get-Content $template_path -Raw
    }
    catch {
        Write-Error "Failed to read template file: '$template_path' - $($PSItem.Exception.Message)"
    }

    $version = Get-TomlVersion
    $homepage = "https://espanso.org/"

    $license = Join-Path $BASE_DIR "LICENSE"
    $icon = Join-Path $script_resources_path "icon.ico"
    $cli_helper = Join-Path $script_resources_path "espanso.cmd"
    $output_dir = Join-Path $BASE_DIR $TARGET_DIR

    $include_paths = ""
    Get-ChildItem -Path $RESOURCE_DIR -Filter "*.dll" | ForEach-Object {
        $winpath_dll = $PSItem.FullName
        $include_paths += "Source: `"$winpath_dll`"; DestDir: `"{app}`"; Flags: ignoreversion`r`n"
    }

    $template = $template -replace '{{{app_version}}}', $version
    $template = $template -replace '{{{app_url}}}', $homepage
    $template = $template -replace '{{{app_license}}}', $license
    $template = $template -replace '{{{app_icon}}}', $icon
    $template = $template -replace '{{{cli_helper}}}', $cli_helper
    $template = $template -replace '{{{output_dir}}}', $output_dir
    $template = $template -replace '{{{output_name}}}', "$INSTALLER_NAME-$ARCH"
    $template = $template -replace '{{{executable_path}}}', $espansod_path
    $template = $template -replace '{{{dll_include}}}', $include_paths

    $iss_setup = Join-Path $TARGET_DIR "setupscript.iss"
    $template | Out-File $iss_setup -Encoding UTF8

    # Helpful for debugging in CI, as the iscc errors mention line numbers
    Get-Content $iss_setup | ForEach-Object { $i = 1 } { "$i`t$PSItem"; $i++ }

    # Execute the Inno Setup Compiler
    iscc "$iss_setup"

    Write-Output "Windows Installer Done!"
}

Main @PSBoundParameters
