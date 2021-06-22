---
title: CLI 
layout: docs
---
Espanso ships with a few useful terminal commands that can be used to interact with it.

### Enable or Disable

Other than using the `toggle_key` shortcut, you can also enable/disable espanso with the following commands:

```bash
# Enable espanso
espanso cmd enable

# Disable espanso
espanso cmd disable

# Toggle between enabled/disabled
espanso cmd toggle
```

These makes it a breeze to control espanso from your desktop environment or other scripts.

### Listing Matches

When integrating espanso with a script, it might be useful to list the available matches. You can so so with the following command:

```
espanso match list
```

Run `espanso match list --help` to find out other interesting options.

### Trigger an injection

Another interesting possibility is injecting a given match from the command line. You can do so with:

```
espanso match exec <trigger>
```

where `<trigger>` is the match trigger.

This is useful if, for example, you want to trigger an expansion from a script.

### Paths

Besides the [overview command](configuration#structure) `espanso path`,
you can also output the exact paths to specific components:

```
espanso path packages # or config, data or default
```

This is useful for including [scripts in packages](packages#scripts-in-packages).
