#!/usr/bin/env pwsh

# Stop on any error
$ErrorActionPreference = "Stop"

# Enable verbose output
# Set-PSDebug -Strict -Trace 1

$TARGET_DIR = "target/windows/resources"

function Main {
    if ([string]::IsNullOrEmpty($env:EXEC_PATH)) {
        Write-Output "EXEC_PATH is a required environment variable for this script`n"
        Write-Output 'Please do $env:EXEC_PATH = ".\target\release\espanso.exe" for release'
        Write-Output 'or $env:EXEC_PATH = ".\target\debug\espanso.exe" for debug mode'
    }

    $EXEC_PATH = Test-Path -Path $env:EXEC_PATH
    if (!$EXEC_PATH) {
        Write-Error "Could not find executable $env:EXEC_PATH`n"
    }

    # Find vcruntime140_1.dll
    $vcruntime_dll = Get-ChildItem -Path "C:\Program Files\Microsoft Visual Studio" -Recurse -Filter "vcruntime140_1.dll" |
        Where-Object { $_.FullName -like "*\VC\Redist\MSVC\*" -and $_.FullName -like "*\x64\*" } |
        Select-Object -First 1 -ExpandProperty FullName

    # Fail if it doesn't exists
    if (-not $vcruntime_dll) {
        Write-Error "Could not find vcruntime140_1.dll"
    }

    # Clean the output directory
    if (Test-Path $TARGET_DIR) {
        Remove-Item $TARGET_DIR -Recurse -Force
    }

    Write-Output "Building windows resources..."
    New-Item -Path $TARGET_DIR -ItemType Directory -Force | Out-Null

    $tooldir = Split-Path $vcruntime_dll -Parent

    Get-ChildItem -Path $tooldir -Filter "*.dll" | Copy-Item -Destination $TARGET_DIR

    Copy-Item -Path $env:EXEC_PATH -Destination "$TARGET_DIR/espansod.exe"

    # Create the command helper script
    $commandContent = '@"%~dp0espansod.exe" %*'
    $commandContent | Out-File "$TARGET_DIR/espanso.cmd" -Encoding ASCII

    Write-Output "Build Windows Resources Done!"
}

Main @PSBoundParameters
