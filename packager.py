import subprocess
import sys
import os
import platform
import hashlib
import click
import shutil
import toml
from dataclasses import dataclass

PACKAGER_TARGET_DIR = "target/packager"

@dataclass
class PackageInfo:
    name: str
    version: str
    description: str
    publisher: str
    url: str

@click.group()
def cli():
    pass

@cli.command()
@click.option('--skipcargo', default=False, is_flag=True, help="Skip cargo release build")
def build(skipcargo):
    """Build espanso distribution"""
    # Check operating system
    TARGET_OS = "macos"
    if platform.system() == "Windows":
        TARGET_OS = "windows"
    elif platform.system() == "Linux":
        TARGET_OS = "linux"

    print("Detected OS:", TARGET_OS)

    print("Loading info from Cargo.toml")
    cargo_info = toml.load("Cargo.toml")
    package_info = PackageInfo(cargo_info["package"]["name"],
                               cargo_info["package"]["version"],
                               cargo_info["package"]["description"],
                               cargo_info["package"]["authors"][0],
                               cargo_info["package"]["homepage"])
    print(package_info)

    if not skipcargo:
        print("Building release version...")
        subprocess.run(["cargo", "build", "--release"])
    else:
        print("Skipping build")

    if TARGET_OS == "windows":
        build_windows(package_info)
    elif TARGET_OS == "macos":
        build_mac(package_info)


def build_windows(package_info):
    print("Starting packaging process for Windows...")

    # Check Inno Setup
    try:
        subprocess.run(["iscc"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except FileNotFoundError:
        raise Exception("Could not find Inno Setup compiler. Please install it from here: http://www.jrsoftware.org/isdl.php")

    print("Clearing target dirs")

    # Clearing previous build directory
    if os.path.isdir(PACKAGER_TARGET_DIR):
        print("Cleaning packager temp directory...")
        shutil.rmtree(PACKAGER_TARGET_DIR)

    TARGET_DIR = os.path.join(PACKAGER_TARGET_DIR, "win")
    os.makedirs(TARGET_DIR, exist_ok=True)

    INSTALLER_NAME = "espanso-setup"

    # Inno setup
    shutil.copy("packager/win/modpath.iss", os.path.join(TARGET_DIR, "modpath.iss"))

    print("Processing inno setup template")
    with open("packager/win/setupscript.iss", "r") as iss_script:
        content = iss_script.read()

        # Replace variables
        content = content.replace("{{{app_name}}}", package_info.name)
        content = content.replace("{{{app_version}}}", package_info.version)
        content = content.replace("{{{app_publisher}}}", package_info.publisher)
        content = content.replace("{{{app_url}}}", package_info.url)
        content = content.replace("{{{app_license}}}",  os.path.abspath("LICENSE"))
        content = content.replace("{{{app_icon}}}",  os.path.abspath("packager/win/icon.ico"))
        content = content.replace("{{{executable_path}}}",  os.path.abspath("target/release/espanso.exe"))
        content = content.replace("{{{output_dir}}}",  os.path.abspath(TARGET_DIR))
        content = content.replace("{{{output_name}}}",  INSTALLER_NAME)

        with open(os.path.join(TARGET_DIR, "setupscript.iss"), "w") as output_script:
            output_script.write(content)

    print("Compiling installer with Inno setup")
    subprocess.run(["iscc", os.path.abspath(os.path.join(TARGET_DIR, "setupscript.iss"))])


def build_mac(package_info):
    print("Starting packaging process for MacOS...")

    print("Clearing target dirs")

    # Clearing previous build directory
    if os.path.isdir(PACKAGER_TARGET_DIR):
        print("Cleaning packager temp directory...")
        shutil.rmtree(PACKAGER_TARGET_DIR)

    TARGET_DIR = os.path.join(PACKAGER_TARGET_DIR, "mac")
    os.makedirs(TARGET_DIR, exist_ok=True)

    print("Compressing release to archive...")
    target_name = f"espanso-mac-{package_info.version}.tar.gz"
    archive_target = os.path.abspath(os.path.join(TARGET_DIR, target_name))
    subprocess.run(["tar",
                    "-C", os.path.abspath("target/release"),
                    "-cvf",
                    archive_target,
                    "espanso",
                    ])
    print(f"Created archive: {archive_target}")

    print("Processing Homebrew formula template")
    with open("packager/mac/espanso.rb", "r") as formula_template:
        content = formula_template.read()

        # Replace variables
        content = content.replace("{{{app_desc}}}", package_info.description)
        content = content.replace("{{{app_url}}}", package_info.url)
        content = content.replace("{{{app_version}}}", package_info.version)

        # Calculate hash
        with open(archive_target, "rb") as f:
            bytes = f.read()
            readable_hash = hashlib.sha256(bytes).hexdigest()
            content = content.replace("{{{release_hash}}}", readable_hash)

        with open(os.path.join(TARGET_DIR, "espanso.rb"), "w") as output_script:
            output_script.write(content)

    print("Done!")

if __name__ == '__main__':
    print("[[ espanso packager ]]")

    # Check python version 3
    if sys.version_info[0] < 3:
        raise Exception("Must be using Python 3")

    cli()