#!/usr/bin/env pwsh

# Stop on any error
$ErrorActionPreference = "Stop"

# Enable verbose output
# Set-PSDebug -Strict -Trace 1

$TARGET_DIR = "target/windows/portable"
$RESOURCE_DIR = "target/windows/resources"

# Check if the resources were previously built
if (-not (Test-Path $RESOURCE_DIR)) {
    Write-Error "You need to build the windows resources first.`nPlease run scripts/build_windows_resources.ps1"
    exit 1
}

function Main {
    # Clean the target directory
    if (Test-Path $TARGET_DIR) {
        Remove-Item $TARGET_DIR -Recurse -Force
    }

    # Remove the portable folder if found
    if (Test-Path "target/windows/espanso-portable") {
        Remove-Item "target/windows/espanso-portable" -Recurse -Force
    }

    # Copy the resources directory
    Copy-Item -Path $RESOURCE_DIR -Destination $TARGET_DIR -Recurse -Force

    # Create the launcher script
    $launcherContent = 'start espansod.exe launcher'
    $launcherContent | Out-File "$TARGET_DIR/START_ESPANSO.bat" -Encoding ASCII

    New-Item -Path "$TARGET_DIR/.espanso" -ItemType Directory -Force | Out-Null
    New-Item -Path "$TARGET_DIR/.espanso-runtime" -ItemType Directory -Force | Out-Null

    $readmeContent = @"
Welcome to Espanso (Portable edition)!

To start espanso, you can double click on "START_ESPANSO.bat"

After the first run, you will see some files in the ".espanso" directory.
This is where your snippets and configurations should be defined.

For more information, please visit the official documentation:
https://espanso.org/docs/

IMPORTANT: Don't delete any file or directory, otherwise espanso won't work.


FOR ADVANCED USERS:

Espanso also offers a rich CLI interface. To start it from the terminal, cd into the
current directory and run "espanso start". You can also run "espanso --help" for more information.

You might have noticed that the directory contains both an "espansod.exe" and an "espanso.cmd" file.
You should generally avoid running "espansod.exe" directly, and instead use the "espanso.cmd"
wrapper (which can simply be run as "espanso" in the terminal). This is needed to correctly manage
STD console handles on Windows.
"@
    $readmeContent | Out-File "$TARGET_DIR/README.txt" -Encoding UTF8

    Rename-Item -Path $TARGET_DIR -NewName espanso-portable
    Compress-Archive target/windows/espanso-portable target/windows/Espanso-Win-Portable-x86_64.zip -Force

    Write-Output "Espanso Portable created!"
}

Main @PSBoundParameters
