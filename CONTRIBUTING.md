# Contributing

<details><summary>Show table of contents</summary>

1. [General guidelines and philosophy](#general-guidelines-and-philosophy)
2. [Building / Compilation](#building--compilation)
2.1 [Prerequisites](#prerequisites)
2.2 [Windows](#windows)
2.3 [MacOS](#macos)
2.3 [Linux](#linux)
2.6 [Disabling modulo](#disabling-modulo-gui-features)
3. [Creating a PR](#creating-a-pr)
3.1 [Going to the practice](#going-to-the-practice)
4. [Periodic tasks](#periodic-tasks)

</details>

---

Welcome to Espanso!
We are very happy to have you and we thank you for considering contributing!

We would like to order the contributions like this:

- Before you start hacking, it is important to **ask first** if your idea or bugfix is in order.
  Create an issue or come and say hi in the `#dev` channel for fixing bugs and adding features involved with coding logic, and `#documentation` for updating primarly the website. Join us at [the discord][`espanso` discord].
  We don't bite!.

  It would be sad that you do the effort to clone the project, successfully make the PR, but it wasn't in our plans or there is another PR that is currently addressing that issue.

- After the PR is submitted, the workflows will start to lint, check and test the changes. Please try to stay all green ✅.
- Most of the time we take some time to respond. Sorry, we are few and there SO much to do here!

## General guidelines and philosophy

This is a list of things we would like to have and mantain across time. Please do your best to abide by.

- We are geared towards a mostly-rust codebase, except on interactions with OS native modules (eg. C++ on Windows and Objective-C on macOS). We decided to stay on the native langauges on each platform, but if it's possible to make a change into rust, submit a PR and we'll see what can we do.
- Everything should be explained on the documentation via drawings, markdown files, etc, but it is important to make it clear. There will always be some new guy or gal into the project we want to welcome 😄.
- Use clear variable names and try to avoid confusing abbreviations. Think that your peers may not be fully fluent in english 💬.

[`espanso` discord]: https://discord.gg/4QARseMS6k

## Building / Compilation

This chapter explains the various steps needed to build espanso.

### Prerequisites

These are the basic tools required to build espanso:

- A recent Rust compiler. You can install it following [these instructions](https://www.rust-lang.org/tools/install)
- A C/C++ compiler. There are multiple of them depending on the platform, but espanso officially supports the following:
  - On Windows, you should use the MSVC compiler. The easiest way to install it is by downloading [Visual Studio](https://visualstudio.microsoft.com/) and checking "Desktop development with C++" in the installer.
  Note that [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) alone doesn't fulfill all the requirements for espanso.
  - On macOS, you should use the official build tools that come with Xcode. If you don't want to install Xcode, you should be able to download only the build tools by executing `xcode-select —install` and following the instructions.
  - On Linux, you should use the default C/C++ compiler (it's usually GCC). On Ubuntu/Debian systems, you can install them with `sudo apt install build-essential`

- Espanso heavily relies on [cargo make](https://github.com/sagiegurari/cargo-make) for the various packaging
steps. You can install it by running:

```bash
cargo install rust-script --version "0.7.0"
cargo install --force cargo-make --version 0.37.5
```

### Windows

After installing the prerequisites, you are ready to compile Espanso on Windows.

Espanso supports multiple targets on Windows: plain executable, installer and portable mode. The following sections explain how to build Espanso for these configurations.

#### Plain executable

If you only want to build the "plain" Espanso executable, you can do so by running:

```bash
cargo make --profile release -- build-binary
```

This will create an `espanso` executable in the `target/release` directory.

##### Installer

If you want to build the Installer (the executable that installs Espanso on a machine), you can run:

```bash
cargo make --profile release -- build-windows-installer
```

This will generate the installer in the `target/windows/installer` directory.

##### Portable mode bundle

You can also generate a portable-mode bundle (a self-contained ZIP archive that does not require installation) by running:

```bash
cargo make --profile release -- build-windows-portable
```

This will generate the executable in the `target/windows/portable` directory.
There are README instructions inside!.

### MacOS

After installing the prerequisites, you are ready to build Espanso on macOS.

Espanso supports two targets on macOS: plain executable and App Bundle. For most cases, the App Bundle format is preferrable.

#### App Bundle

You can build the App Bundle by running:

```bash
cargo make --profile release -- create-bundle
```

This will create the `Espanso.app` bundle in the `target/mac` directory.

### Linux

Espanso on Linux comes in two different flavors: one for X11 and one for Wayland.
If you don't know which one to choose, follow [these steps to determine which one you are running](https://unix.stackexchange.com/a/325972).

#### Necessary dependencies

If compiling on a version of Ubuntu X11 before 22.04 (including 22.04):

```bash
sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.0-gtk3-dev
```

If compiling on a version of Ubuntu X11 after 22.04:

```bash
sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.2-dev
```

#### Compiling for X11

##### X11 AppImage

The AppImage is a convenient format to distribute Linux applications, as besides the binary,
it also bundles all the required libraries.

You can create the AppImage by running (this will work on X11 systems):

```bash
cargo make --profile release -- create-app-image
```

You will find the resulting AppImage in the `target/linux/AppImage/out` folder.

##### Binary

You can build the Espanso binary on X11 by running the following command:

```bash
cargo make --profile release -- build-binary
```

You'll then find the `espanso` binary in the `target/release` directory.

#### Compiling on Wayland

You can build Espanso on Wayland by running:

```bash
cargo make --env NO_X11=true --profile release -- build-binary
```

You'll then find the `espanso` binary in the `target/release` directory.

### Advanced

Espanso offers a few flags that might be necessary if you want to further tune the resulting binary.

#### Disabling modulo (GUI features)

Espanso includes a component known as _modulo_, which handles most of the graphical-related parts of the tool.
For example, the Search bar or Forms are handled by it.

If you don't want them, you can pass the `--env NO_MODULO=true` flag to any of the previous `cargo make` commands
to remove support for it.

Keep in mind that espanso was designed with modulo as a first class citizen, so the experience might be far from perfect without it.

## Testing

It is good practice to cover your changes with a test. Also, try to think about corner cases and various ways how your changes could break. Cover those in the tests as well.

Tests can be found in 2 places:

- `/test/` folder (we have this case only in `espanso-migrate`)
- within the same file of the function, in a `tests` mod

## Creating a PR

Firt and foremost: **A good PR makes a change!**. Let's chat some basic theory before going to the practice

Because of this PR-centric strategy and the goal that the reviewers should easily understand your change, the **PR title and description matters** a great deal!

> **Note**
> Try to follow the suggestions in our PR message template to make sure we can quickly focus on the technical merits and impact on the users.

### A PR should limit itself to a single functional change or related set of same changes

Mixing different changes in the same PR will make the review process much harder. A PR might get stuck on one aspect while we would actually like to land another change. Furthermore, if we are forced to revert a change, mixing and matching different aspects makes fixing bugs or regressions much harder.

Thus, please try to **separate out unrelated changes**!
**Don't** mix unrelated refactors with a potentially contested change.
Stylistic fixes and housekeeping can be bundled up into singular PRs.

### Going to the practice

We would like the rust code:

- to be formatted via `rustfmt`

  ```console
  cargo fmt --all
  ```

- to abide by the `clippy` ruleset we currently use

  ```console
  cargo clippy --workspace
  ```

- to approve the tests by running

  ```console
  cargo test --workspace
  ```

- to be compiled with `stable`, not `nightly`
- prefer not to use macros, if possible. Try to use functions or generics.

And C / C++ code:

- we would like to use `clang-format`
- and we would like to use `clang-tidy`

`clang-format` and `clang-tidy` aren't capable of formatting folders recursively,
 so that's why we use [run-clang-format](https://github.com/lmapii/run-clang-format) and
 [run-clang-tidy](https://github.com/lmapii/run-clang-tidy)

To format the code run:

```bash
run-clang-format .clang-format.json
```

And it should format all the `*.c`, `*.h`, `*.cpp`, `*.hpp` and `*.mm` files.

With `clang-tidy` the structure is ready to run:

```bash
run-clang-tidy .clang-tidy.json
```

But it [is necessary to pass the build args](https://github.com/espanso/espanso/pull/1864#issuecomment-1962998631).

Today our submitted code is yet untidy. Work in progress!

### `git`

`git` is a powerful but a bit complex tool to use, and there are many criteria around the internet. We normally:

- squash the commits when we merge a PR
- TODO: setup git hooks with [`rusty-hook`](https://github.com/swellaby/rusty-hook)

## Periodic tasks

Developing a project is sometimes complex, and we use these tools every now and then to check if we
 are doing everything right. These aren't tools that we _need_ to run on CI, but
 it's nice to have them and check if it's allright.

### `cargo udeps` to find unused dependencies

install it by

```bash
# traditional cargo install
cargo install cargo-udeps --locked
# or with cargo binstall
cargo binstall cargo-udeps --locked
```

Nightly toolchain is needed, just for this tool only

```bash
rustup toolchain install nightly
```

and then run:

```bash
cargo +nightly udeps --all-targets
```

There is a [sample issue here](https://github.com/espanso/espanso/issues/1833),
and [its PR there](https://github.com/espanso/espanso/pull/1856)

We know some false positives, running on windows or linux gives:

```bash
unused dependencies:
`espanso-mac-utils v0.1.0 (C:\Users\user\repos\espanso\espanso-mac-utils)`
├─── dependencies
│    ├─── "lazy_static"
│    └─── "regex"
└─── build-dependencies
     └─── "cc"
`espanso-modulo v0.1.0 (C:\Users\user\repos\espanso\espanso-modulo)`
└─── build-dependencies
     └─── "glob"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
```

These are because the windows and linux can't see through the `target = macos`

