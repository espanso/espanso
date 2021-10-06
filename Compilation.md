# Compilation

This document tries to explain the various steps needed to build espanso. (Work in progress).

# Prerequisites

These are the basic tools required to build espanso:

* A recent Rust compiler. You can install it following these instructions: https://www.rust-lang.org/tools/install
* A C/C++ compiler. There are multiple of them depending on the platform, but espanso officially supports the following:
  * On Windows, you should use the MSVC compiler. The easiest way to install it is by downloading Visual Studio and checking "Desktop development with C++" in the installer: https://visualstudio.microsoft.com/
  * On macOS, you should use the official build tools that come with Xcode. If you don't want to install Xcode, you should be able to download only the build tools by executing `xcode-select —install` and following the instructions.
  * On Linux, you should use the default C/C++ compiler (it's usually GCC). On Ubuntu/Debian systems, you can install them with `sudo apt install build-essential`

* Espanso heavily relies on [cargo make](https://github.com/sagiegurari/cargo-make) for the various packaging
steps. You can install it by running:

```
cargo install --force cargo-make
```

# Linux

Espanso on Linux comes in two different flavors: one for X11 and one for Wayland. 
If you don't know which one to choose, follow these steps to determine which one you are running: https://unix.stackexchange.com/a/325972

## Compiling for X11

### Necessary packages

If compiling on Ubuntu X11:
* `sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.0-gtk3-dev`

### AppImage

The AppImage is a convenient format to distribute Linux applications, as besides the binary, 
it also bundles all the required libraries.

You can create the AppImage by running (this will work on X11 systems):

```
cargo make create-app-image --profile release
```

You will find the resulting AppImage in the `target/linux/AppImage/out` folder.

### Binary

TODO

## Compiling on Wayland

TODO

## Advanced

Espanso offers a few flags that might be necessary if you want to further tune the resulting binary.

### Disabling modulo (GUI features)

Espanso includes a component known as _modulo_, which handles most of the graphical-related parts of the tool.
For example, the Search bar or Forms are handled by it.

If you don't want them, you can pass the `--env NO_MODULO=true` flag to any of the previous `cargo make` commands
to remove support for it.

Keep in mind that espanso was designed with modulo as a first class citizen, so the experience might be far from perfect without it.
