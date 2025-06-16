# Windows

After installing the prerequisites, you are ready to compile Espanso on Windows.

Espanso supports multiple targets on Windows: plain executable, installer and portable mode. The following sections explain how to build Espanso for these configurations.

#### Plain executable

If you only want to build the "plain" Espanso executable, you can do so by running:

```console
cargo build --no-default-features --features modulo,native-tls --release
```

This will create an `espanso` executable in the `target/release` directory.

##### Build the resources

There is a setup step you need to make before building the installer and portable
versions of espanso. After compiling a binary (Release for example)

```powershell
$env:EXEC_PATH = "target\release\espanso.exe"
scripts\build_windows_resources.ps1
```

The variable `$env:EXEC_PATH` can be load of your OS environment variables, and
the alterntive path of release, would be:

```powershell
$env:EXEC_PATH = "target\debug\espanso.exe"
```

##### Installer

If you want to build the Installer (the executable that installs Espanso on a
machine), first build the executable as per above. Afterwards, you can run:

```powershell
Install-Module -Name 'PSTOML' -Force # just needed once
scripts\build_windows_installer.ps1
```

This will generate the installer in the `target/windows/installer` directory.

##### Portable mode bundle

You can also generate a portable-mode bundle (a self-contained ZIP archive that
does not require installation) by first building the executable as per above, and
then running:

```powershell
scripts\build_windows_portable.ps1
```

This will generate the executable in the `target/windows/portable` directory.
There are README instructions inside!.
