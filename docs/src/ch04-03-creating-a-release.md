# Creating a release

Creating a release is still a cloudy and ambiguos process, so

> "Sorry about the mess."
>
> ―Han Solo, after using his DL-44 to kill Greedo in the Mos Eisley Cantina

### Step by step

1) checkout a new branch, change the version in `espanso/Cargo.toml`

2) create a pr with the new release: [sample pr](https://github.com/espanso/espanso/pull/2359)

3) when ready, close the pr (merge to `dev`). Checkout to `dev` and add a git
tag with:

```bash
git tag <tagname>
git push origin <tagname>
```

for example `git tag v2.2.2` and `git push origin v2.2.2`

4) Auca needs to build the software (only him can sign the executables)

5) and sign the binaries

- macOS Silicon arch

6) create a release manually in github (mark it as draft, or pre-release)

7) add all the signed binaries, SHAs and add some text.

8) modify the website

- the download links
- the version number

9) lastly, the news!

- make an announcement in the `espanso` discord
