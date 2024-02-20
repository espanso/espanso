# Compilation

This document tries to explain the various steps needed to build espanso. (Work in progress).

## Prerequisites

These are the basic tools required to build espanso:

* A recent Rust compiler. You can install it following [these instructions](https://www.rust-lang.org/tools/install)
* A C/C++ compiler. There are multiple of them depending on the platform, but espanso officially supports the following:
  * On Windows, you should use the MSVC compiler. The easiest way to install it is by downloading [Visual Studio](https://visualstudio.microsoft.com/) and checking "Desktop development with C++" in the installer.
  Note that [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) alone doesn't fulfill all the requirements for espanso.
  * On macOS, you should use the official build tools that come with Xcode. If you don't want to install Xcode, you should be able to download only the build tools by executing `xcode-select —install` and following the instructions.
  * On Linux, you should use the default C/C++ compiler (it's usually GCC). On Ubuntu/Debian systems, you can install them with `sudo apt install build-essential`

* Espanso heavily relies on [cargo make](https://github.com/sagiegurari/cargo-make) for the various packaging
steps. You can install it by running:

```bash
cargo install rust-script --version "0.7.0"
cargo install --force cargo-make --version 0.37.5
```

## Windows

After installing the prerequisites, you are ready to compile Espanso on Windows.

Espanso supports multiple targets on Windows: plain executable, installer and portable mode. The following sections explain how to build Espanso for these configurations.

### Plain executable

If you only want to build the "plain" Espanso executable, you can do so by running:

```bash
cargo make --profile release -- build-binary
```

This will create an `espanso` executable in the `target/release` directory.

### Installer

If you want to build the Installer (the executable that installs Espanso on a machine), you can run:

```bash
cargo make --profile release -- build-windows-installer
```

This will generate the installer in the `target/windows/installer` directory.

### Portable mode bundle

You can also generate a portable-mode bundle (a self-contained ZIP archive that does not require installation) by running:

```bash
cargo make --profile release -- build-windows-portable
```

This will generate the executable in the `target/windows/portable` directory.
There are README instructions inside!.

## MacOS

After installing the prerequisites, you are ready to build Espanso on macOS.

Espanso supports two targets on macOS: plain executable and App Bundle. For most cases, the App Bundle format is preferrable.

### App Bundle

You can build the App Bundle by running:

```bash
cargo make --profile release -- create-bundle
```

This will create the `Espanso.app` bundle in the `target/mac` directory.

## Linux

Espanso on Linux comes in two different flavors: one for X11 and one for Wayland.
If you don't know which one to choose, follow [these steps to determine which one you are running](https://unix.stackexchange.com/a/325972).

### Necessary dependencies

If compiling on a version of Ubuntu X11 before 22.04 (including 22.04):

* `sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.0-gtk3-dev`

If compiling on a version of Ubuntu X11 after 22.04:

* `sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.2-dev`

### Compiling for X11

#### X11 AppImage

The AppImage is a convenient format to distribute Linux applications, as besides the binary,
it also bundles all the required libraries.

You can create the AppImage by running (this will work on X11 systems):

```bash
cargo make --profile release -- create-app-image
```

You will find the resulting AppImage in the `target/linux/AppImage/out` folder.

#### Binary

You can build the Espanso binary on X11 by running the following command:

```bash
cargo make --profile release -- build-binary
```

You'll then find the `espanso` binary in the `target/release` directory.

### Compiling on Wayland

You can build Espanso on Wayland by running:

```bash
cargo make --env NO_X11=true --profile release -- build-binary
```

You'll then find the `espanso` binary in the `target/release` directory.

## Advanced

Espanso offers a few flags that might be necessary if you want to further tune the resulting binary.

### Disabling modulo (GUI features)

Espanso includes a component known as _modulo_, which handles most of the graphical-related parts of the tool.
For example, the Search bar or Forms are handled by it.

If you don't want them, you can pass the `--env NO_MODULO=true` flag to any of the previous `cargo make` commands
to remove support for it.

Keep in mind that espanso was designed with modulo as a first class citizen, so the experience might be far from perfect without it.
