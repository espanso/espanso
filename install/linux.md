---
title: Install on Linux
layout: docs
---
The linux installation depends on the distro you are using. 
Espanso has been tested in the following distros, but you shouldn't 
have many problems making it work on others.

* [Ubuntu/Debian](#installing-on-ubuntu--debian)
* [Manjaro/Arch](#installing-on-manjaro--arch)

#### Wayland support

Currently espanso supports X11 systems only.

### Installing on Ubuntu / Debian

Espanso depends upon the `X11 Record Extension`, the `xdo library`, the `xclip` command and
the `libnotify-bin` library, so you will need to install 
those first with the following commands:

```
sudo apt update
sudo apt install libxtst6 libxdo3 xclip libnotify-bin
```

You can now download the latest espanso release:
```
curl -L https://github.com/federico-terzi/espanso/releases/latest/download/espanso-linux.tar.gz | tar -xz -C /tmp/
```

> If you want to verify the correctness of the archive, in the [Github Releases](https://github.com/federico-terzi/espanso/releases/) page you will find the **SHA256** hash in the file `espanso-linux-sha256.txt`.

And then move it to the `/usr/local/bin/` directory

```
sudo mv /tmp/espanso /usr/local/bin/espanso
```

> If you want to avoid using `sudo`, you can move espanso in the `~/.local/bin` directory instead. Make sure that the `~/.local/bin` directory is in the `PATH`. If not present, you may need to reboot the system.

You should now have espanso installed in your system. To start it, type the following command:

```
espanso start
```

If you now type `:espanso` in any text field, you should see "Hi there!" appear! 

At this point, you are ready to read the [Getting Started](/docs/get-started/) tutorial.

### Installing on Manjaro / Arch

There are multiple ways to install espanso on Arch: the preferred method is by using the [AUR](#installing-from-aur) package,
but you can also install it manually from the [prebuilt executables](#installing-from-the-prebuilt-release).

#### Installing from AUR

The official way to install espanso on Arch-based systems is by using one of the AUR packages, currently maintained by [Scrumplex](https://scrumplex.net/). There are multiple options available:

* [espanso-bin](https://aur.archlinux.org/packages/espanso-bin/) - Pre-compiled version for x64 systems.
* [espanso](https://aur.archlinux.org/packages/espanso/) - Builds from the latest (stable) release.
* [espanso-git](https://aur.archlinux.org/packages/espanso-git/) - Builds from latest commit ( in master branch ).

If you are using a 64 bit machine, you should prefer `espanso-bin` as it is much faster to install. If you have a 32 bit system, or prefer compiling espanso
from sources, you should use the `espanso` package instead.

When you are ready, you can install espanso with:

```
git clone https://aur.archlinux.org/espanso-bin.git
cd espanso-bin
makepkg -si
```
 
#### Installing from the prebuilt release

Espanso depends upon the `X11 Record Extension`, the `xdo library`, the `xclip` command and the
`libnotify` library, so you will need to install those first with the following commands:

```
sudo pacman -Sy
sudo pacman -S libxtst xdotool xclip libnotify
```

You can now download the latest espanso release:
```
curl -L https://github.com/federico-terzi/espanso/releases/latest/download/espanso-linux.tar.gz | tar -xz -C /tmp/
```

> If you want to verify the correctness of the archive, in the [Github Releases](https://github.com/federico-terzi/espanso/releases/) page you will find the **SHA256** hash in the file `espanso-linux-sha256.txt`.

And then move it to the `/usr/local/bin/` directory

```
sudo mv /tmp/espanso /usr/local/bin/espanso
```

> If you want to avoid using `sudo`, you can move espanso in the `~/.local/bin` directory instead. Make sure that the `~/.local/bin` directory is in the `PATH`. If not present, you may need to reboot the system.

You should now have espanso installed in your system. To start it, type the following command:

```
espanso start
```

If you now type `:espanso` in any text field, you should see "Hi there!" appear! 

At this point, you are ready to read the [Getting Started](/docs/get-started/) tutorial.