# Developer docs


## Install mdBook: 

- from source
```
cargo install mdbook
```

- if you have `cargo-binstall`
```
cargo binstall mdbook
```

- or if you have `scoop`

```
scoop install main/mdbook
```

- in nix-shell

```
nix-shell -p mdbook
```

## run the book locally

Run mdbook build in this directory.
```
mdbook build --open
```

Or build it and open the generated `build/index.html`

you can also `watch` it

```
mdbook watch --open
```

