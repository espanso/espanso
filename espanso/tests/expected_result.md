This should be the standard response to `espanso` CLI

to run this test
`cargo test --test cli_tests`

expected output:
This is the result! if you modify this the test will fail
```console
$ espanso
espanso [VERSION]
Federico Terzi
A Privacy-first, Cross-platform Text Expander

USAGE:
    espanso [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       
            Prints help information

    -v               
            Sets the level of verbosity

    -V, --version    
            Prints version information


OPTIONS:


SUBCOMMANDS:
    cmd           Send a command to the espanso daemon.
    edit          Shortcut to open the default text editor to edit config files
    env-path      Add or remove the 'espanso' command from the PATH
    help          Prints this message or the help of the given subcommand(s)
    install       Install a package
    log           Print the daemon logs.
    match         List and execute matches from the CLI
    migrate       Automatically migrate legacy config files to the new v2 format.
    package       package-management commands
    path          Prints all the espanso directory paths to easily locate configuration and matches.
    restart       Restart the espanso service
    service       A collection of commands to manage the Espanso service (for example, enabling auto-start on system
                  boot).
    start         Start espanso as a service
    status        Check if the espanso daemon is running or not.
    stop          Stop espanso service
    uninstall     Remove a package
    workaround    A collection of workarounds to solve some common problems.

```