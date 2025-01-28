# Dev recipes and cookbook

Developing a project is sometimes complex, and we use these tools every now and then to check if we
 are doing everything right. These aren't tools that we _need_ to run on CI, but
 it's nice to have them and check if it's allright.

## `cargo udeps` to find unused dependencies

install it by

```bash
# traditional cargo install
cargo install cargo-udeps --locked
# or with cargo binstall
cargo binstall cargo-udeps --locked
```

Nightly toolchain is needed, just for this tool only

```bash
rustup toolchain install nightly
```

and then run:

```bash
cargo +nightly udeps --all-targets
```

There is a [sample issue here](https://github.com/espanso/espanso/issues/1833),
and [its PR there](https://github.com/espanso/espanso/pull/1856)
We know some false positives, running on windows or linux gives:

```bash
unused dependencies:
`espanso-mac-utils v0.1.0 (C:\Users\user\repos\espanso\espanso-mac-utils)`
├─── dependencies
│    ├─── "lazy_static"
│    └─── "regex"
└─── build-dependencies
     └─── "cc"
`espanso-modulo v0.1.0 (C:\Users\user\repos\espanso\espanso-modulo)`
└─── build-dependencies
     └─── "glob"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
```
These are because the windows and linux can't see through the `target = macos`

## check for outdated dependencies

install it by
```bash
# traditional cargo install
cargo install cargo-outdated --locked
# or with cargo binstall
cargo binstall cargo-outdated --locked
```
An run it with
```bash
cargo outdated
```
