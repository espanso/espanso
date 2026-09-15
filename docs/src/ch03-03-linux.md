# linux

Espanso on Linux comes in two different flavors: one for X11 and one for Wayland.
The variant you build or install **must match your active session**, otherwise Espanso may install but silently fail to work.

#### Determining your session type

Run the following command to check whether you are running X11 or Wayland:

```console
echo $XDG_SESSION_TYPE
```

This prints either `x11` or `wayland`. If the variable is empty, follow [these steps to determine which one you are running](https://unix.stackexchange.com/a/325972).

If you are on Wayland (the default on Fedora and recent GNOME-based distributions), build or install the Wayland variant. If you are on X11, use the X11 variant.

#### Necessary dependencies

If compiling on a version of Ubuntu X11 before 22.04 (including 22.04):

```console
sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.*-dev
```

If compiling on a version of Ubuntu X11 after 22.04:

```console
sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.*-dev
```

#### Compiling on X11

##### X11 AppImage

The AppImage is a convenient format to distribute Linux applications, as besides the binary,
it also bundles all the required libraries.

You can create the AppImage by running the following commands from the repository root
(build the X11 variant; run it on an X11 session):

```console
cargo build --release --no-default-features --features modulo,vendored-tls
./scripts/create_app_image.sh target/release/espanso Espanso-X11
```

You will find `Espanso-X11.AppImage` in the `target/linux/AppImage/out` folder.

##### X11 binary

You can build the Espanso binary on X11 by running the following command:

```console
cargo build --release --no-default-features --features modulo,vendored-tls
```

You'll then find the `espanso` binary in the `target/release` directory.

#### Compiling on Wayland

##### Wayland AppImage

To create the Wayland AppImage, run the following commands from the repository root,
building with the `wayland` feature and passing a
custom output name to avoid clashing with the X11 one:

```console
cargo build --release --no-default-features --features wayland,modulo,vendored-tls
./scripts/create_app_image.sh target/release/espanso Espanso-Wayland
```

You will find `Espanso-Wayland.AppImage` in the `target/linux/AppImage/out` folder.

> **Note:** Each run of `create_app_image.sh` clears `target/linux/AppImage/`
> (including `out/`), so copy the X11 AppImage elsewhere before building
> the Wayland one if you need both.

##### Wayland binary

You can build the Espanso binary on Wayland by running the following command:

```console
cargo build --release --no-default-features --features wayland,modulo,vendored-tls
```

You'll then find the `espanso` binary in the `target/release` directory.

#### Using nix for compilation

We do have nix for building the repo and flakes to track the dependencies.

Do this to build the `release` mode

```
nix build
```

The binary will be located at `/your-espanso-repo-folder/result/bin/espanso`

And this to run:

```
nix run
```

