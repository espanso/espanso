---
title: Configuration
layout: docs
---
Following a Unix-like philosophy, **espanso uses files to manage its configuration**
instead of GUIs. This has many advantages, such as the capability to easily sync your
configurations between machines using [Git](https://git-scm.com/) or cloud services
such as [Dropbox](https://www.dropbox.com/) or [Google Drive](https://www.google.com/drive/).

### Structure

All espanso configurations reside in a folder called `espanso`, whose location varies between Operating Systems:
* Linux: `$XDG_CONFIG_HOME/espanso` (e.g. `/home/user/.config/espanso`)
* macOS: `$HOME/Library/Preferences/espanso` (e.g. `/Users/user/Library/Preferences/espanso`)
* Windows: `{FOLDERID_RoamingAppData}\espanso` (e.g. `C:\Users\user\AppData\Roaming\espanso`)

A quick way to find the path of your configuration folder is by using the following command:

```
espanso path
```

The `espanso` directory will contain the following file structure:

```
- default.yml
- user/
```

The `default.yml` file contains the **main configurations** and for a basic usage, this is the only file
you will be working with. You can find a list of all the possible options in the [Options](#options) section. 

The `user` folder is used to store more advanced user configurations, such as [Application-specific](#application-specific-configurations) configs.

### Quick Editing

Introduced in version 0.5.1, espanso now ships with the `edit` subcommand, which makes editing configuration files much more convenient. Let's see how it works:

If you open a terminal and type:

```
espanso edit
```

the default system editor (Notepad on Windows and Nano on Unix systems) will be spawned, editing the `default.yml`.
Then, after you saved the file and exited the editor, **espanso will automatically restart, loading the new changes**.

#### Customizing the editor

If you want to use another editor, customizing it is super easy, just specify your choice in the `EDITOR` (or `VISUAL`)
environment variables, such as:

```
EDITOR=/usr/bin/vim
```

#### Editing files in the user/ directory

If you invoke `espanso edit` without further arguments, it will open the `default.yml` file. But what if you want to edit
files in the `user/*` directory? Luckily, you can simply specify the name as an additional argument (without the extension).

For example, if you want to edit the `user/emails.yml` file, you can type:

```
espanso edit emails
```

Note that the last command also allows the user to create a new file in the `user/` directory if it doesn't already exist.

### Organizing Matches

After creating a lot of matches, you may wonder if there's a way to keep them organized in multiple files instead of creating a long list in the `default.yml` configuration.
Luckily, you can split your matches into multiple files by placing them in the `user/` folder!

Let's say you want to create a file for your email signatures. Create the `user/emails.yml` file with the following content:

```yml
name: emails
parent: default

matches:
  - trigger: ":sig"
    replace: |
      Best regards,
      Jon Snow
```

After restarting espanso (by using the `espanso restart` command), you can now use the `:sig` trigger as you would have done by inserting it into the `default.yml` configuration.

This is made possible by the `parent: default` instruction, which tells espanso to merge the current matches into the default configuration.

You can create as many files as you want, and keep all your matches well organized :)

### Application-Specific Configurations

Sometimes you may need to make espanso behave **differently** with some applications. For example, you may want to have
a different set of Matches for an application, or you may need to change the `backend` option for compatibility
purposes.

For such cases, espanso offers the **Application Specific** configurations, that is configurations that are valid
only for some applications which match specific **filters**.

Let's say you want to add some Matches for emojis, but only when using the Telegram desktop app.

You can create a `telegram.yml` file in the `espanso/user` folder, with the following content:

```yml
filter_title: "Telegram"

matches:
  - trigger: ":ok"
    replace: "👍"
```

After restarting espanso with `espanso restart`, you are ready to test the new configuration.

Navigate to Telegram and type `:ok`, you should see your emoji appear. If you then move to another application
and try again, you should not see it!

The key here is the `filter_title` option, that basically means: "If the current application contains `Telegram`
in the title, use this configuration instead of the `default.yml` one"

**Note**: app-specific configurations don't support all options, refer to the [table](#options) below to find out more.

#### Available Filters

espanso supports various filters, but their support depends on the Operating System used. You can refer to this table:

Filter | Description | Windows Support | MacOS Support | Linux Support
--- | --- | --- | --- | ---
`filter_title` | Filter based on the current Window title | Full support | Uses the App identifier instead of the Window title | Full support
`filter_exec` | Filter based on the current application executable path | Full support | Full support | Partial support
`filter_class` | Filter based on the current Window class | Uses the application executable path instead | Uses the App identifier instead | Full support

The `filter_title`, `filter_exec` and `filter_class` filters support a full **regex** as parameter, but make sure to escape the special characters properly.

#### Finding the right filters

To make it easier to find the right filters, espanso offers the `detect` subcommand. Open a terminal and type:

```
espanso detect
```

Now, while leaving it running, move to the desired application and then come back to the terminal. You should see
an output like:

```
Detected change, current window has properties:
==> Title: 'Telegram (1828)'
==> Class: 'TelegramDesktop'
==> Executable: '/snap/telegram-desktop/953/bin/Telegram'
```

These are the parameters espanso detected for your target application, so you can now use them to create the 
perfect filter.

### Customizing the Toggle Key

By default, espanso can be temporarily disabled and enabled by pressing the Alt key twice, resulting in a
notification saying "espanso disabled." Pressing Alt twice again will enable it, and you'll receive a
notification saying "espanso enabled."

If you'd like to customize the key, simply add the `toggle_key` option to your
`default.yml` configuration and set it to one of the available options:

|              |             |               |              |
|--------------|-------------|---------------|--------------|
| `CTRL`       | `ALT`       | `SHIFT`       | `META`       |
| `LEFT_CTRL`  | `LEFT_ALT`  | `LEFT_SHIFT`  | `LEFT_META`  |
| `RIGHT_CTRL` | `RIGHT_ALT` | `RIGHT_SHIFT` | `RIGHT_META` |

And if you'd rather turn it off, you can do so with:

```yml
toggle_key: OFF
```

### Hiding the Icon

You can hide the espanso icon on macOS and Windows by adding the following option to your `default.yml` file:

```yaml
show_icon: false
```

### Hiding the Notifications

You can hide the espanso notifications by adding the following option to your `default.yml` config:

```yaml
show_notifications: false
```

**Note for macOS users:** if you only want to hide the *SecureInput* notifications, please read the following section.1

### macOS Notification for SecureInput

On macOS there is a security feature known as `SecureInput`, which blocks text expanders from detecting input when entering text in sensitive areas, such as password fields (but also other apps, even the Terminal if configured).

As a result, espanso will not work in those situations, and espanso will trigger a notification (as well as logging it) to warn the user if an app triggers SecureInput.  If you want to disable the notification, just add the following line in your config file:

`secure_input_notification: false`

### Options

Here's a list of all options available for the configuration file:

Option | Description | Possible Values | Default | App-Specific 
--- | --- | --- | --- | ---
`backend` | The typing engine used. `Inject` simulate keypresses, `Clipboard` simulates a copy/paste, `Auto` is available on Linux only and combines the two previous. | `Clipboard`, `Inject` or `Auto` (Linux only) | `Inject` on Win and macOS, `Auto` on Linux | Yes
`auto_restart` | Restart when the configuration changes | `true`/`false` | `true` | No
`backspace_limit` | How many backspace espanso tracks to correct misspelled keywords | int | `3` | No
`enable_active` | Disable the active mode for the current configuration | `true`/`false` | `true` | Yes
`enable_passive` | Disable the passive mode for the current configuration | `true`/`false` | `false` | Yes
`parent` | The target for the current configuration file, mainly used in packages | string | `self` | Yes
`ipc_server_port` | Windows only. Set the daemon listening port | int | `34982` | No
`exclude_default_entries` | Used in app-specific configs, avoid parent matches and global variables | `true`/`false` | `false` | Yes
`toggle_key` | Change the key used to toggle espanso active mode | `CTRL`, `ALT`, `SHIFT`, `META`, `LEFT_CTRL`, `LEFT_ALT`, `LEFT_SHIFT`, `LEFT_META`, `RIGHT_CTRL`, `RIGHT_ALT`, `RIGHT_SHIFT`, `RIGHT_META`, `OFF`| `ALT` | No
`passive_key` | Change the key used to trigger passive mode | `CTRL`, `ALT`, `SHIFT`, `META`, `LEFT_CTRL`, `LEFT_ALT`, `LEFT_SHIFT`, `LEFT_META`, `RIGHT_CTRL`, `RIGHT_ALT`, `RIGHT_SHIFT`, `RIGHT_META`, `OFF`| `OFF` | No
`secure_input_notification` | Enable/Disable the Secure Input notification on macOS | `true`/`false` | `true` | No
`show_icon` | Show/Hide the icon in the status bar on macOS and Windows | `true`/`false` | `true` | No
`show_notifications` | Show/Hide the notifications| `true`/`false` | `true` | No
`undo_backspace` | Enable/Disable the Backspace-To-Undo feature | `true`/`false` | `true`| Yes
`fast_inject` |  Use a faster injection mechanism (Linux only). It uses XSendEvent API rather than XTestFakeKeyEvent API, which is faster but incompatible with some applications.| `true`/`false` | `true`| Yes

