# Building and compilation

This chapter explains the various steps needed to build espanso.

### Prerequisites

These are the basic tools required to build espanso:

- A recent Rust compiler. You can install it following [these instructions](https://www.rust-lang.org/tools/install)
- A C/C++ compiler. There are multiple of them depending on the platform, but
espanso officially supports the following:
  - On Windows, you should use the MSVC compiler. The easiest way to install
  it is by downloading [Visual Studio](https://visualstudio.microsoft.com/) and checking "Desktop development with C++" in the installer.
  Note that [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  alone doesn't fulfill all the requirements for espanso.
  - On macOS, you should use the official build tools that come with Xcode. If
  you don't want to install Xcode, you should be able to download only the
  build tools by executing `xcode-select —install` and following the
  instructions.
  - On Linux, you should use the default C/C++ compiler (it's usually GCC). On
  Ubuntu/Debian systems, you can install them with
  `sudo apt install build-essential`

- Espanso heavily relies on [cargo make](https://github.com/sagiegurari/cargo-make) for the various packaging
steps. You can install it by running:

```bash
cargo install rust-script
cargo install --force cargo-make --version 0.37.5
```

### Installing the app

After installing the prerequisites, the following chapters (linux, macos and 
windows) will guide you on how to install the app with the commandline

### Advanced compilation

Espanso offers a few flags that might be necessary if you want to further tune the resulting binary.

#### Disabling modulo (GUI features)

Espanso includes a component known as _modulo_, which handles most of the graphical-related parts of the tool.
For example, the Search bar or Forms are handled by it.

If you don't want them, you can pass the `--env NO_MODULO=true` flag to any of the previous `cargo make` commands
to remove support for it.

Keep in mind that espanso was designed with modulo as a first class citizen, so the experience might be far from perfect without it.
