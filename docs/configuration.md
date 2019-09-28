---
title: Configuration
layout: docs
---
Following a Unix-like philisophy, **espanso uses files to manage it's configuration**
instead of GUIs. This has many advantages, such as the capability to easily sync your
configurations between machines using [Git](https://git-scm.com/) or cloud services
such as [Dropbox](https://www.dropbox.com/) or [Google Drive](https://www.google.com/drive/).

### Structure

All espanso configurations reside in a folder called `.espanso`, located in the user home directory.
The path of the home directory depends on the Operating System, here's a few examples:
* Windows: `C:\Users\Federico\.espanso\`
* Mac: `/Users/Federico/.espanso/`
* Linux: `/home/federico/.espanso/`

After installing espanso, the `.espanso` directory will contain the following file structure:

```
- default.yml
- user/
- packages/
```

The `default.yml` file contains the **main configurations** and for a basic usage, this is the only file
you will be working with. 

The `user` folder is used to store more advanced user configurations, such as [Application-specific](#application-specific-config) configs.

The `packages` folder stores packages installed using the *package manager* and should not be directly modified
by the user. For more information, check out the [Packages](/docs/packages) section of the documentation.