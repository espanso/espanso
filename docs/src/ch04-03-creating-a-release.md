# Creating a release

Creating a release is still a cloudy and ambiguos process, so 

> "Sorry about the mess."
> 
> ―Han Solo, after using his DL-44 to kill Greedo in the Mos Eisley Cantina

### Step by step

1) create a pr with the new release:
[sample pr] (missing-link)

2) change the version in `espanso/Cargo.toml`

3) Auca needs to build the software (only him can sign the executables)

3) and sign the binaries

- macOS Silicon arch
- Windows

4) create a release manually in github (mark it as draft, or pre-release)

5) add all the signed binaries, SHAs and add some text.

x) modify the website

- the download links
- the version number

lastly, the news!

- make an announcement in the `espanso` discord


