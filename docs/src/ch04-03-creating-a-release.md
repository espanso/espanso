# Creating a release

Creating a release is still a cloudy and ambiguos process, so

> "Sorry about the mess."
>
> ―Han Solo, after using his DL-44 to kill Greedo in the Mos Eisley Cantina

### Step by step

> Important!
>
> Make sure you have the version bumped and merged into `dev` branch. Either 
> major, minor of bugfix. Previous a release, of just after to schedule the next 
> release. You can do this with:
> ```bash
> git tag <tagname>
> git push origin <tagname>
> ```
>
> for example `git tag v2.2.2` and `git push origin v2.2.2`

1) Run the `create-release-draft.yml` workflow. Even though the CI is able to 
build the mac executables, only Auca can sign the executables, so this step in 
the workflow is commented out for macOS.

2) Wait until the workflow finishes... meanwhile you build and sign the macos
binaries...

3) Upload the *.dmg, update the description and hit publish!

4) Share the news!

- make an announcement in the `espanso` discord
