import subprocess
import sys
import os
import platform
import hashlib
import click
import shutil
import toml
import hashlib
import glob
import urllib.request
from dataclasses import dataclass

PACKAGER_TARGET_DIR = "target/packager"

@dataclass
class PackageInfo:
    name: str
    version: str
    description: str
    publisher: str
    url: str
    modulo_version: str

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
                               cargo_info["package"]["homepage"],
                               cargo_info["modulo"]["version"])
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

def calculate_sha256(file):
    with open(file, "rb") as f:
        b = f.read() # read entire file as bytes
        readable_hash = hashlib.sha256(b).hexdigest()
        return readable_hash

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

    modulo_url = "https://github.com/federico-terzi/modulo/releases/download/v{0}/modulo-win.exe".format(package_info.modulo_version)
    modulo_sha_url = "https://github.com/federico-terzi/modulo/releases/download/v{0}/modulo-win.exe.sha256.txt".format(package_info.modulo_version)
    print("Pulling modulo depencency from:", modulo_url)
    modulo_target_file = os.path.join(TARGET_DIR, "modulo.exe")
    urllib.request.urlretrieve(modulo_url, modulo_target_file)
    print("Pulling SHA signature from:", modulo_sha_url)
    modulo_sha_file = os.path.join(TARGET_DIR, "modulo.sha256")
    urllib.request.urlretrieve(modulo_sha_url, modulo_sha_file)
    print("Checking signatures...")
    expected_sha = None
    with open(modulo_sha_file, "r") as sha_f:
        expected_sha = sha_f.read()
    actual_sha = calculate_sha256(modulo_target_file)
    if actual_sha != expected_sha:
        raise Exception("Modulo SHA256 is not matching")

    print("Gathering CRT DLLs...")
    msvc_dirs = glob.glob("C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\*\\VC\\Redist\\MSVC\\*")
    print("Found Redists: ", msvc_dirs)

    print("Determining best redist...")

    if len(msvc_dirs) == 0:
        raise Exception("Cannot find redistributable dlls")

    msvc_dir = None

    for curr_dir in msvc_dirs:
        dll_files = glob.glob(curr_dir + "\\x64\\*CRT\\*.dll")
        print("Found dlls", dll_files, "in", curr_dir)
        if any("vcruntime140_1.dll" in x.lower() for x in dll_files):
            msvc_dir = curr_dir
            break

    if msvc_dir is None:
        raise Exception("Cannot find redist with VCRUNTIME140_1.dll")

    print("Using: ", msvc_dir)

    dll_files = glob.glob(msvc_dir + "\\x64\\*CRT\\*.dll")

    print("Found DLLs:")
    include_list = []
    for dll in dll_files:
        print("Including: "+dll)
        include_list.append("Source: \""+dll+"\"; DestDir: \"{app}\"; Flags: ignoreversion")

    print("Including modulo")
    include_list.append("Source: \""+modulo_target_file+"\"; DestDir: \"{app}\"; Flags: ignoreversion")

    include = "\r\n".join(include_list)

    INSTALLER_NAME = f"espanso-win-installer"

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
        content = content.replace("{{{dll_include}}}",  include)

        with open(os.path.join(TARGET_DIR, "setupscript.iss"), "w") as output_script:
            output_script.write(content)

    print("Compiling installer with Inno setup")
    subprocess.run(["iscc", os.path.abspath(os.path.join(TARGET_DIR, "setupscript.iss"))])

    print("Calculating the SHA256")
    sha256_hash = hashlib.sha256()
    with open(os.path.abspath(os.path.join(TARGET_DIR, INSTALLER_NAME+".exe")),"rb") as f:
        # Read and update hash string value in blocks of 4K
        for byte_block in iter(lambda: f.read(4096),b""):
            sha256_hash.update(byte_block)

        hash_file = os.path.abspath(os.path.join(TARGET_DIR, "espanso-win-installer-sha256.txt"))
        with open(hash_file, "w") as hf:
            hf.write(sha256_hash.hexdigest())


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
    target_name = f"espanso-mac.tar.gz"
    archive_target = os.path.abspath(os.path.join(TARGET_DIR, target_name))
    subprocess.run(["tar",
                    "-C", os.path.abspath("target/release"),
                    "-cvf",
                    archive_target,
                    "espanso",
                    ])
    print(f"Created archive: {archive_target}")

    print("Calculating the SHA256")
    sha256_hash = hashlib.sha256()
    with open(archive_target,"rb") as f:
        # Read and update hash string value in blocks of 4K
        for byte_block in iter(lambda: f.read(4096),b""):
            sha256_hash.update(byte_block)

        hash_file = os.path.abspath(os.path.join(TARGET_DIR, "espanso-mac-sha256.txt"))
        with open(hash_file, "w") as hf:
            hf.write(sha256_hash.hexdigest())

    modulo_sha_url = "https://github.com/federico-terzi/modulo/releases/download/v{0}/modulo-mac.sha256.txt".format(package_info.modulo_version)
    print("Pulling SHA signature from:", modulo_sha_url)
    modulo_sha_file = os.path.join(TARGET_DIR, "modulo.sha256")
    urllib.request.urlretrieve(modulo_sha_url, modulo_sha_file)
    modulo_sha = None
    with open(modulo_sha_file, "r") as sha_f:
        modulo_sha = sha_f.read()
    if modulo_sha is None:
        raise Exception("Cannot determine modulo SHA")

    print("Processing Homebrew formula template")
    with open("packager/mac/espanso.rb", "r") as formula_template:
        content = formula_template.read()

        # Replace variables
        content = content.replace("{{{app_desc}}}", package_info.description)
        content = content.replace("{{{app_url}}}", package_info.url)
        content = content.replace("{{{app_version}}}", package_info.version)
        content = content.replace("{{{modulo_version}}}", package_info.modulo_version)
        content = content.replace("{{{modulo_sha}}}", modulo_sha)

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