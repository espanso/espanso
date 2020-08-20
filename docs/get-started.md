---
title: Getting Started
layout: docs
---
In this section, we will cover the basics of espanso to get you started immediately.
Make sure to [install espanso](/install) before diving into the next sections.

### Starting espanso

If you followed the [Windows](/install/win/) or [MacOS](/install/mac/) installation correctly, 
**espanso will be automatically started** when you power up your computer. There are times, 
however, when you may need to start espanso explicitly, such as when you're using Linux.

It's very easy to check if espanso is currently running: if you're using **MacOS** or **Windows**, 
you should see the **icon in the status bar**. If you don't see it, or if you're using **Linux**, 
another way to check it is to **open a terminal** and type:

```bash
espanso status
```

If you see "`espanso is not running`", then you'll need to start espanso manually with the following command:

```bash
espanso start
```

At this point you are ready to use espanso. Open any typing application and type `:espanso`, 
you should see `Hi there!` appear.

If you don't see it, make sure espanso is currently running. You could also try to repeat the installation procedure.

### Understanding Matches

espanso works by **detecting** your keypresses and **replacing** them when they match a specific keyword, called *trigger*.

![How espanso works](/assets/images/match1.png)

The rule that associate the *trigger* with the *replaced text* is called **Match** and is a core concept of espanso.
Matches are very flexible and powerful to solve many tasks. 
You can learn all about Matches in their [documentation](/docs/matches/) page.

![Match](/assets/images/match2.png)

espanso ships with very few built-in Matches to give you the maximum flexibility, but you can expand its capabilities
in two ways: creating your own **custom matches** or **installing packages**. 
Both of these possibilities are introduced below.

### Configuration

espanso uses a **file-based configuration** approach, following the Unix philosophy. All configuration files
reside in the `espanso` directory, whose location depends on the current OS:

* Linux: `$XDG_CONFIG_HOME/espanso` (e.g. `/home/user/.config/espanso`)
* macOS: `$HOME/Library/Preferences/espanso` (e.g. `/Users/user/Library/Preferences/espanso`)
* Windows: `{FOLDERID_RoamingAppData}\espanso` (e.g. `C:\Users\user\AppData\Roaming\espanso`)

> By default, configuration folder is hidden in most systems. To open it, copy path of your configuration folder and paste it in the address bar (aka path bar) of your file manager/explorer.

A quick way to find the path of your configuration folder is by using the following command:

```bash
espanso path
```

While this folder may contain many different files, let's focus on the most important one: `default.yml`.

The `default.yml` file contain the **main espanso configuration** and uses the widely
spread [YAML](https://en.wikipedia.org/wiki/YAML) syntax.
It can be used to change various settings as well as **creating custom matches**.
You can learn all about espanso's configuration by visiting the [documentation](/docs/configuration).

### Creating your own Match

That's enough theory for now, let's start with some action! Let's say you write a lot of emails, and you're
tired of writing the greetings at the end, so you decide to speed up the process.

We will configure espanso so that every time you type `:br`, it will be expanded to:

```bash
Best Regards,
Jon Snow
```

By now you should know that we need to **define a Match**.

With your favourite text editor, open the `espanso/default.yml` file, introduced previously in the 
[Configuration](#configuration) section. You should see something like:

```yml
# espanso configuration file

# This is the default configuration file, change it as you like it
# You can refer to the official documentation:
# https://espanso.org/docs/

# Matches are the substitution rules, when you type the "trigger" string
# it gets replaced by the "replace" string.
matches:
  # Simple text replacement
  - trigger: ":espanso"
    replace: "Hi there!"

...
```

We need to define a new Match, so in the `matches:` section, add the following code:

```yml
  - trigger: ":br"
    replace: "Best Regards,\nJon Snow"
```

**Make sure to include the indentation**, otherwise it won't be valid YAML syntax. You should get something like:

```yml
# espanso configuration file

# This is the default configuration file, change it as you like it
# You can refer to the official documentation:
# https://espanso.org/docs/

# Matches are the substitution rules, when you type the "trigger" string
# it gets replaced by the "replace" string.
matches:
  # Simple text replacement
  - trigger: ":espanso"
    replace: "Hi there!"

  - trigger: ":br"
    replace: "Best Regards,\nJon Snow"

...
```

We're almost there! After every configuration change, **espanso must be restarted**. Open a terminal and type:

```bash
espanso restart
```

Now try to type `:br` anywhere. If you did everything correctly, you should see `Best Regards` appear!

> In version 0.5.1, espanso introduced the `edit` subcommand which makes editing configuration files much easier. 
Take a look at [Quick Editing](/docs/configuration/#quick-editing) if you are interested.

### Understanding Packages

Custom matches are amazing, but sometimes it can be tedious to define Matches for every **common operation**,
and even more when you want to **share them with other people**.

espanso offers an easy way to **share and reuse matches** with other people, **packages**. In fact,
they are so important that espanso includes a **build-in package manager** and a **store**,
the [espanso hub](https://hub.espanso.org/).

If you are lucky enough, someone might have already written a **package** to include the matches you need!
Otherwise, you can create a package and publish it on the hub, for more information check out the
[Packages](/docs/packages/) documentation.

### Installing a Package

Let's say you want to **add some emojis** to espanso, such that when you type `:ok` it gets expanded to 👍.

A solution would be to install the [Basic Emojis](https://hub.espanso.org/packages/basic-emojis/) package from the
[espanso hub](https://hub.espanso.org/) store. Open a terminal and type:

```bash
espanso install basic-emojis
```

At this point, as we did with custom matches, we need to **restart espanso**. We can do so with the command:

```bash
espanso restart
```

If you now type `:ook` into any text field, you should see 👍👍👍👍 appear!

### Useful shortcuts

Let's conclude this introduction with the most important shortcut espanso offers, the **toggle shortcut**:

There are times when you may want to **disable espanso to avoid an unwanted expansion**. This can be
easily accomplished by quickly **double pressing the `ALT` key** ( Option on MacOS ). You should then 
see a notification showing "Espanso disabled".

At this point, espanso will be disabled and will not expand any match.

To **re-enable** it, double press the `ALT` key again.


#### Customizing the Toggle Key

By default, espanso can be temporarily disabled & enabled by pressing the Alt key twice, resulting in a
notification saying "espanso disabled." Pressing Alt twice again will enable it, and you'll receive a
notification saying "espanso enabled." This does not turn off espanso, it simply disables it until
you enable it again.

If you'd like to customize this behavior, simply add the `toggle_key` option to your previously mentioned
`default.yml` and set it to `OFF` or one of the available options:

|              |             |               |              |
|--------------|-------------|---------------|--------------|
| `CTRL`       | `ALT`       | `SHIFT`       | `META`       |
| `LEFT_CTRL`  | `LEFT_ALT`  | `LEFT_SHIFT`  | `LEFT_META`  |
| `RIGHT_CTRL` | `RIGHT_ALT` | `RIGHT_SHIFT` | `RIGHT_META` |


Using the above example `default.yml`, if I wanted to make it so espanso is disabled/enabled when I press
the left control key twice, I add `toggle_key: LEFT_CTRL` to the file like so, and then, pressing
the left control key twice disables/enables espanso.

```yml
# espanso configuration file

# This is the default configuration file, change it as you like it
# You can refer to the official documentation:
# https://espanso.org/docs/
toggle_key: LEFT_CTRL

# Matches are the substitution rules, when you type the "trigger" string
# it gets replaced by the "replace" string.
matches:
  # Simple text replacement
  - trigger: ":espanso"
    replace: "Hi there!"

  - trigger: ":br"
    replace: "Best Regards,\nJon Snow"

...
```

And if you'd rather it not be possible to disable it on accident, just turn it off like so:

```yml
toggle_key: OFF
```

After the changes are made, if you used `espanso edit` it will automatically restart. If not, issue a `espanso restart` and you'll be ready to go!
