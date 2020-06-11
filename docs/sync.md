---
title: Synchronization
layout: docs
---
After using espanso for a while, you may need to synchronize your configuration between devices. Luckly for you, the espanso
file-based configuration makes it pretty easy to accomplish using a Cloud Storage service (such as Dropbox, Google Drive, ecc)
or even GitHub!

> From now on, I will only mention "Dropbox folder" for brevity, but you can apply the same procedure for every service.

The general idea, which applies to all operating systems, is the following:

* Move the espanso configuration folder inside your Dropbox folder (also a subdirectory is perfectly fine)
* Create a **symbolic link** in the original position, pointing to the synced folder.

The specific commands depend on you OS:

### Windows

By default, the espanso configuration folder resides in this folder (change "user" with your username):

```
C:\Users\user\AppData\Roaming\espanso
```

The first step is moving this folder in your Dropbox folder, for example in:

```
C:\Users\user\Dropbox\espanso
```

Now you need to create a **symbolic link**. Open the Command Prompt and type the following command, making sure you specify the correct paths:

```
mklink /J "C:\Users\user\AppData\Roaming\espanso" "C:\Users\user\Dropbox\espanso"
```

Now restart espanso and you should be ready to go!

### macOS

By default, the espanso configuration folder resides in this folder

```
$HOME/Library/Preferences/espanso
```

The first step is moving this folder in your Dropbox folder, for example in:

```
$HOME/Dropbox/espanso
```

Now you need to create a **symbolic link**. Open the Terminal and type the following command, making sure you specify the correct paths:

> Note: Before running the following command, make sure that there is no folder called `espanso` in the `Preferences` folder, as otherwise it will create another nested folder `espanso/espanso` (which is wrong).

```
ln -s "$HOME/Dropbox/espanso" "$HOME/Library/Preferences/espanso"
```

Now restart espanso and you should be ready to go!

### Linux

By default, the espanso configuration folder resides in this folder (change "user" with your username):

```
/home/user/.config/espanso
```

The first step is moving this folder in your Dropbox folder, for example in:

```
/home/user/Dropbox/espanso
```

Now you need to create a **symbolic link**. Open the Terminal and type the following command, making sure you specify the correct paths:

```
ln -s "/home/user/Dropbox/espanso" "/home/user/.config/espanso"
```

Now restart espanso and you should be ready to go!
