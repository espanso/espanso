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

Espanso depends upon the `X11 Record Extension`, the `xdo library` and the `xclip` command, 
so you will need to install those first with the following commands:

```
sudo apt update
sudo apt install libxtst6 libxdo3 xclip
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

Espanso depends upon the `X11 Record Extension`, the `xdo library` and the `xclip` command, 
so you will need to install those first with the following commands:

```
sudo pacman -Sy
sudo pacman -S libxtst xdotool xclip
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