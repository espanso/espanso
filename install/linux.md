---
title: Install on Linux
layout: default
---
The linux installation depends on the distro you are using. 
Espanso has been tested in the following distros, but you shouldn't 
have many problems making it work on others.

* [Ubuntu/Debian](#installing-on-ubuntu--debian)
* [Manjaro/Arch](#installing-on-manjaro--arch)
* [Others](#installing-on-other-distros)

#### Installing on Ubuntu / Debian

Espanso depends upon the `X11 Record Extension`, the `XDO library` and the `xclip` command, 
so you will need to install those first with the following command:

```
sudo apt update
sudo apt install libxtst6 libxdo3 xclip
```

You can now download the latest espanso release:
```
curl https://github.com/federico-terzi/espanso/releases/latest/download/espanso-linux.tar.gz | tar -xz -C /tmp/
```

And then move it to the `/usr/local/bin/` directory

```
sudo mv /tmp/espanso /usr/local/bin/espanso
```

You should now have espanso installed in your system. To start it, type the following command:

```
espanso start
```

If you now type `:espanso` in any text field, you should see "Hi there!" appear! 

#### Installing on Manjaro / Arch

TODO

#### Installing on Other Distros

TODO