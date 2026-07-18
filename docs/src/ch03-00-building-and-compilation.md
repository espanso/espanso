# Building and compilation

This chapter explains the various steps needed to build espanso.

### Cross-compilation

At the time of writing, cross-compilation is not guaranteed to be supported. In other words, if you wish to build the `exe` files for Windows, you _must_ do this on a PC running Windows. For Windows, this means WSL cannot be used to compile Espanso.

### Prerequisites

These are the basic tools required to build espanso:

- A recent Rust compiler. You can install it following [these instructions](https://www.rust-lang.org/tools/install)
- A C/C++ compiler. There are multiple of them depending on the platform, but
espanso officially supports the following:
  - On Windows, you should use the MSVC compiler. The easiest way to install
  it is by downloading [Visual Studio](https://visualstudio.microsoft.com/) and checking "Desktop development with C++" in the installer.
  Note that [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  alone doesn't fulfill all the requirements for espanso - it's recommended to download Visual Studio.
    - As of the time of writing, you can NOT build in WSL. You _must_ do the build natively in Windows. You must
    first install Rust for Windows, which requires you to install Microsoft Visual Studio (_Not_ VsCode!).
  - On macOS, you should use the official build tools that come with Xcode. If
  you don't want to install Xcode, you should be able to download only the
  build tools by executing `xcode-select —install` and following the
  instructions.
  - On Linux, you should use the default C/C++ compiler (it's usually GCC). On
  Ubuntu/Debian systems, you can install them with
  `sudo apt install build-essential`

### Installing the app

After installing the prerequisites, the following chapters (Linux, macOS and 
Windows) will guide you on how to install the app with the command line.

### Advanced compilation

Espanso offers a few flags that might be necessary if you want to further tune the resulting binary.

#### Disabling modulo (GUI features)

Espanso includes a component known as _modulo_, which handles most of the graphical-related parts of the tool.
For example, the Search bar or Forms are handled by it.

If you don't want them, you can build without the `modulo` feature flag to remove support for it.

Keep in mind that espanso was designed with modulo as a first class citizen, so the experience might be far from perfect without it.
