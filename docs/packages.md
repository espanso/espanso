---
title: Packages
layout: docs
---
**Packages** makes it easy to reuse Matches created by other people or **share** yours with the world. Thanks
to [espanso hub](https://hub.espanso.org/), the official **espanso package repository**, and the **built-in
package manager**, using them is a breeze.

### Install a Package

Packages can be installed in various ways, but the easiest choice is [espanso hub](#from-espanso-hub).

#### From espanso hub

Navigate to [espanso hub](https://hub.espanso.org/) and choose the package you want to install.

Let's say you want to install the [Basic Emojis](https://hub.espanso.org/packages/basic-emojis/) package, 
open a terminal and type:

```
espanso install basic-emojis
```

Then don't forget to **restart** espanso using:

```
espanso restart
```

#### From a Repository

Currently packages can only be installed from the [espanso hub](https://hub.espanso.org/), but direct 
installation from a repository is coming soon. Follow the status on this [Issue](https://github.com/federico-terzi/espanso/issues/55).

### Listing Installed Packages

To list installed packages, open a terminal and type:

```
espanso package list
```

### Removing a Package

To remove a package, open a terminal and type:

```
espanso package remove <package-name>
```

Where `<package-name>` is the name of the package. You can obtain that name [listing installed packages](#listing-installed-packages).

Then don't forget to **restart** espanso using:

```
espanso restart
```

### Creating a Package

**Note:** espanso is in the alpha stage right now and sometimes things are not very polished. The current
package creation process will be improved in the future.

In their basic form, Packages are just **configuration files**, very similar to 
[those you've seen before](/docs/configuration), with a few *metadata* to describe them.

Packages are hosted on **GitHub repositories**. To create one, go ahead and 
fork the [espanso package example](https://github.com/federico-terzi/espanso-package-example/) repo.

From that example you can create your own package in a few easy steps:

1. Change the `simple-package` directory name to your desired **package name** (only alphanumeric characters and '-' are
allowed)
2. Change the **metadata** in the header of the `simple-package/README.md` file, where:
    * `package_name` is the name of the package ( must be the same as the directory )
    * `package_title` human friendly version of the package name.
    * `package_desc` a **short** description of the package
    * `package_version` the version of the package, you should not change this one when you start.
    * `package_author` your name
    * `package_repo` the repo address of this package, should match the final github address

3. Then after the `---` you can write the package description using the [Markdown](https://github.com/adam-p/markdown-here/wiki/Markdown-Cheatsheet) syntax.
4. Modify the `package.yml` file contained in the `0.1.0` folder (matching the package version) by **changing the name** and **including your Matches**. For example:
   ```yml
    # Simple package

    name: simple-package 
    parent: default

    matches:
    - trigger: ":hw"
      replace: "hello world"
    ```

#### Publishing on espanso hub

After following all these steps, you can request to publish your package to [espanso hub](http://hub.espanso.org)
by opening an [Issue](https://github.com/federico-terzi/espanso-hub/issues) with the following information:
* The package **repository** url