# Windows

## Prerequisites

Compilation of Espanso on Windows requires you to have Rust installed on native Windows (not WSL).

To do this, download and run the official Rust installer for Windows. The Rust installer will inform you that you must
have a copy of Microsoft Visual Studio installed, which contains a bunch of necessary libraries.  
The Visual Studio Community Edition is free. Visual Studio _Code_ is a different program and does not contain the
libraries needed for compilation.

Once you have Visual Studio and Rust installed you are ready to compile Espanso on Windows.

You do not need to use Visual Studio for editing and building the code. Rust simply needs to use Visual Studio's files
to compile Espanso.

## Building

Espanso supports multiple targets on Windows:

1. Plain executable
2. Installer
3. Portable mode

The following sections explain how to build Espanso for these configurations.

### 1. Plain executable

If you only want to build the "plain" Espanso executable, you can do so by running the following command in a
PowerShell or Command Prompt (or in your IDE of choice):

```console
cargo build --no-default-features --features modulo,native-tls --release
```

This will create an `espanso.exe` executable in the `target/release` directory. You can use this as the command-line
application, or you can run the GUI by running this command in a PowerShell terminal:

```powershell
.\target\windows\resources\espansod.exe launcher
```

> _Note:_ While you can't run this command in your WSL shell itself, you could still host your code in WSL! First, you
must [map your WSL distro as a mapped network drive in Windows](https://stackoverflow.com/a/78707619). After you have
done that, navigate to your WSL `espanso` directory using Windows File Explorer. Then, <kbd>shift</kbd> +
<kbd>right-click</kbd> --> `Open PowerShell window here` and then you can run `cargo` commands to run the build. Your
code will be compiled in PowerShell by the copy of Rust installed in Windows, and you should be able to edit and
manage your code in your WSL environment. But your mileage may vary: build times using this method can be
substantially longer than compiling code hosted in the native Windows file system. You may find that building without
WSL is a better experience. But at least it _is_ possible to host your code in WSL if you prefer. It is recommended to
copy the built executables _out_ of WSL and run them from Windows directly, instead of trying to run the executables
while they are stored within your WSL directory. They _will_ run, but performance will suffer greatly. It seems that
any "cross-OS" connection between Windows and WSL introduces lots of latency, which is far from ideal.


### 2. & 3. Prerequisite: Build the resources

Before building the installer or portable versions of Espanso you must prepare the files. Run the
`build_windows_resources.ps1` script in Powershell, first setting the `EXEC_PATH` variable to point to your target
executable. In this example, the `release` executable is used:

```powershell
$env:EXEC_PATH = "target\release\espanso.exe"
scripts\build_windows_resources.ps1
```

If you wanted to use the debug target, you would run this command before running `build_windows_resources` instead:

```powershell
$env:EXEC_PATH = "target\debug\espanso.exe"
```

This will create files in `target\windows\resources`.

> **Note:** You may get an error in in PowerShell saying _"running scripts is disabled on this system."_ To get around
this, change the execution policy to allow yourself to run scripts:
>
> ```powershell
> Get-ExecutionPolicy -Scope CurrentUser  # This will simply print what the script-running policy is *currently* set to.
> Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope CurrentUser
> ```
>
> It _may_ be considered unsafe to leave this security setting disabled, which is why Microsoft has it disabled by
> default. So you may choose to restore it back to the previous setting, which is printed out by the previous
> commands. More info can be found [here](https:/go.microsoft.com/fwlink/?LinkID=135170).

### 2. Installer

If you want to build the Installer, the executable that installs Espanso on a
machine, first build the executable and resources as outlined above. Afterwards, run:

```powershell
Install-Module -Name 'PSTOML' -Force # just needed once
scripts\build_windows_installer.ps1
```

This will generate the installer in the `target/windows` directory.

### 3. Portable mode bundle

You can also generate a portable-mode bundle (a self-contained ZIP archive that
does not require installation) by first building the executable as per above, and
then running:

```powershell
scripts\build_windows_portable.ps1
```

This will generate the zip file in the `target/windows` directory.
There are README instructions inside!


## A brief side note about cross-compilation and WSL:

If you try to run these Rust commands in WSL directly the build will _say_ it completes but the resulting application
will not run when installed on Windows, as it likely defaulted to a Linux build target. Rust supports
cross-compilation, but the compilation method used for building Espanso for Windows (MSVC) requires files that are
proprietary to Microsoft and only bundled with Visual Studio. It would not be trivial to change Espanso off of MSVC
and migrate to an architecture that is friendlier to cross-compilation. If you are new to the Espanso project and
bring experience with Windows applications and cross-compiling Rust applications across multiple platforms, then
please feel free to suggest improvements. Others be warned: here be dragons - It is currently easier to build natively
for each supported platform.
