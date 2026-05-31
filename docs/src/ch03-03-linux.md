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

#### Compiling for X11

##### X11 AppImage

The AppImage is a convenient format to distribute Linux applications, as besides the binary,
it also bundles all the required libraries.

You can create the AppImage by running (this will work on X11 systems):

```console
cargo build --release --no-default-features --features modulo,vendored-tls
./scripts/create_app_image.sh
```

You will find the resulting AppImage in the `target/linux/AppImage/out` folder.

##### Binary

You can build the Espanso binary on X11 by running the following command:

```console
cargo build --release --no-default-features --features modulo,vendored-tls
```

You'll then find the `espanso` binary in the `target/release` directory.

#### Compiling on Wayland

You can build Espanso on Wayland by running:

```bash
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

