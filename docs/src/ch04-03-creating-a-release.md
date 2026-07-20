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

1) Run the `create-release-draft.yml` workflow. The CI builds, codesigns and
notarizes the macOS DMG automatically, using Auca's Apple Developer ID
Application certificate (individual enrollment — espanso doesn't have a
registered legal entity, so an org-owned Apple Developer account isn't an
option) stored in the repo's GitHub Actions secrets (`MACOS_CERTIFICATE`,
`MACOS_CERTIFICATE_PWD`, `MACOS_CERTIFICATE_NAME`, `MACOS_CI_KEYCHAIN_PWD`,
`APPLE_ID`, `APPLE_APP_SPECIFIC_PASSWORD`, `APPLE_TEAM_ID`). As with
Federico's certificate before it, this ties signing to one maintainer's
Apple ID; if Auca's certificate ever expires, is revoked, or he steps away,
someone will need to re-enroll and refresh these secrets. If the secrets are
ever missing or stale, the `macos` job step "Codesign app bundle" will fail
and the DMG will need to be signed manually as a fallback.

2) Wait until the workflow finishes...

3) Update the description and hit publish!

4) Share the news!

- make an announcement in the `espanso` discord
