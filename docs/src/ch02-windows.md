# Windows

After installing the prerequisites, you are ready to compile Espanso on Windows.

Espanso supports multiple targets on Windows: plain executable, installer and portable mode. The following sections explain how to build Espanso for these configurations.

#### Plain executable

If you only want to build the "plain" Espanso executable, you can do so by running:

```bash
cargo make --profile release -- build-binary
```

This will create an `espanso` executable in the `target/release` directory.

##### Installer

If you want to build the Installer (the executable that installs Espanso on a machine), you can run:

```bash
cargo make --profile release -- build-windows-installer
```

This will generate the installer in the `target/windows/installer` directory.

##### Portable mode bundle

You can also generate a portable-mode bundle (a self-contained ZIP archive that does not require installation) by running:

```bash
cargo make --profile release -- build-windows-portable
```

This will generate the executable in the `target/windows/portable` directory.
There are README instructions inside!.
