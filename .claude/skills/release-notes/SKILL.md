---
name: release-notes
description: Draft espanso release notes between two git tags. Use when the user asks to write/generate/draft release notes or a changelog for a version range (e.g. "release notes for v2.4.0", "write release notes between v2.3.0 and v2.4.0", "draft the changelog for the latest release").
user-invocable: true
allowed-tools:
  - Bash(git log *)
  - Bash(git tag *)
  - Bash(git describe *)
---

# /release-notes — Draft espanso release notes

Arguments passed: `$ARGUMENTS` — optionally a tag range like `v2.3.0..v2.4.0`,
or a single new tag/version. If empty, use the two most recent tags
(`git tag --sort=-creatordate | head -2`) as the range, oldest..newest.

## Steps

1. Resolve the range. If `$ARGUMENTS` isn't already `FROM..TO`, figure out
   FROM (previous tag) and TO (target tag or `HEAD` if the release isn't
   tagged yet).
2. Get the commit list: `git log FROM..TO --oneline`.
3. Curate — this is the part that matters, don't just dump the log:
   - Drop noise: `Merge pull request`, `:arrow_up: update flake`,
     `retrigger CI`, pure `chore: fix rustfmt formatting` /
     `chore: fmt` commits, and internal governance commits (ownership
     changes, branch protection, CI-only tweaks) unless the user asks to
     keep them.
   - Keep anything user-facing: features, bug fixes, security/CVE
     dependency bumps, docs, and notable platform-specific work
     (macOS/Windows/Linux/Wayland/Nix).
   - Merge duplicate/iterative commits on the same change into one bullet
     (e.g. several "fix rustfmt" + "retrigger CI" commits chained after a
     real fix belong to that fix's bullet, not their own lines).
   - Reference the PR number in parens when the commit subject has one
     (`(#1234)`); otherwise link the commit short hash.

## Output format

Always use this structure (adjust section presence to what the range
actually contains — omit empty sections, don't pad):

```
**Highlights:** <2-3 sentence summary of the release's theme>

### ✨ New Features
- <user-facing addition> (#PR)

### 🐛 Fixes
- <bug fix, platform-tagged if relevant> (#PR)

### 🔒 Security
- <dependency/CVE fix> (#PR)

### 🛠 Improvements
- <internal-but-visible improvement, CI/build/tooling users would notice> (#PR)

### 📚 Documentation
- <README/docs change> (#PR)
```

## After drafting

Show the draft in chat first. Ask whether to save it to a file (e.g.
`CHANGELOG.md` or a scratch file) — don't write files unprompted, and never
create a GitHub release or push tags yourself.
