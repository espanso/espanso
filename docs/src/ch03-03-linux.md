# linux

Espanso on Linux comes in two different flavors: one for X11 and one for Wayland.
If you don't know which one to choose, follow [these steps to determine which one you are running](https://unix.stackexchange.com/a/325972).

#### Necessary dependencies

If compiling on a version of Ubuntu X11 before 22.04 (including 22.04):

```bash
sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.*-dev
```

If compiling on a version of Ubuntu X11 after 22.04:

```bash
sudo apt install libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.*-dev
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

