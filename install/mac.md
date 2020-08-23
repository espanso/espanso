---
title: Install on macOS
layout: docs
---
The easiest way to install espanso on macOS is by using the [Homebrew](https://brew.sh/)
package manager, but you can also do it manually.

### Using Homebrew

The first thing to do is to add the official espanso *tap* to Homebrew with
the following command:

```
brew tap federico-terzi/espanso
```

Then you can install espanso with:

```
brew install espanso
```

To make sure that espanso was correctly installed, you can open a terminal and type:

```
espanso --version
```

At this point, you have to [Enable Accessibility](#enabling-accessibility) to use espanso.

### Manually

Download `espanso-mac.tar.gz` from the [Releases page](https://github.com/federico-terzi/espanso/releases):

```
curl -sOL https://github.com/federico-terzi/espanso/releases/latest/download/espanso-mac.tar.gz
```

Extract the binary:

```
tar -xzf espanso-mac.tar.gz
```

Create a folder to house the binary:

```
sudo mkdir -p /usr/local/espanso/bin
sudo cp espanso /usr/local/espanso/bin/espanso
```

Create a symbolic link in your `/usr/local/bin` folder:

```
sudo ln -s /usr/local/espanso/bin/espanso /usr/local/bin
```

To make sure that espanso was correctly installed, you can open a terminal and type:

```
espanso --version
```

**Important:** Some of the most recent espanso features require also `modulo`, so make sure to follow the installation instructions from the [modulo docs](https://github.com/federico-terzi/modulo#macos).

At this point, you have to [Enable Accessibility](#enabling-accessibility) to use espanso.

### Enabling Accessibility

Because espanso uses the macOS [Accessibility API](https://developer.apple.com/library/archive/documentation/Accessibility/Conceptual/AccessibilityMacOSX/)
to work, you need to authorize it using the following procedure:

Open a terminal and type the command:

```
espanso register
```

A dialog should show up, click on "Open System Preferences", as shown here:

![Accessibility Prompt](/assets/images/accessibility-prompt.png)

Then, in the "Privacy" panel click on the Lock icon (1) to enable edits and 
then check "espanso" (2), as shown in the picture:

![Accessibility Settings](/assets/images/accessibility-macos-enable.png)

Now open the terminal again and type:

```
espanso start
```

If everything goes well, you should see the espanso icon appear in the status bar:

![macOS status bar icon](/assets/images/espanso-icon-macos-statusbar.png)

If you now type `:espanso` in any text field, you should see "Hi there!" appear! 

At this point, you are ready to read the [Getting Started](/docs/get-started/) tutorial.
