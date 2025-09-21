# Creating a release

Creating a release is still a cloudy and ambiguos process, so

> "Sorry about the mess."
>
> ―Han Solo, after using his DL-44 to kill Greedo in the Mos Eisley Cantina

### Step by step

1) Run the `create-release-draft.yml` workflow. Even though the CI is able to 
build the mac executables, only Auca can sign the executables, so this step in 
the workflow is commented out for macOS.

2) Wait until the workflow finishes... meanwhile you build and sign the macos
binaries...

3) Upload the *.dmg, update the description and hit publish!

4) Share the news!

- make an announcement in the `espanso` discord

5) Change the next version of espanso in `espanso/Cargo.toml`. Create a pr with
the new release: [sample pr](https://github.com/espanso/espanso/pull/2359).
When ready, close the pr (merge to `dev`). Checkout to `dev` and add a git
tag with:

```bash
git tag <tagname>
git push origin <tagname>
```

for example `git tag v2.2.2` and `git push origin v2.2.2`
