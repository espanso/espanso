_default:
    just --list -u

alias f := format
alias fmt := format
alias t := test
alias r := ready
alias l := lint

format:
    cargo fmt --all

lint:
    cargo clippy --workspace

test:
    cargo test --workspace

# When you finished your feature, run this to run the CI on local
ready:
    just format
    just lint
    just test
    just build

[windows]
build-binary:
    cargo make --profile release -- build-binary

[windows]
plain-installer:
    cargo make --profile release -- build-windows-installer

[windows]
portable:
    cargo make --profile release -- build-windows-portable

[macos]
bundle:
    cargo make --profile release -- create-bundle

[linux]
x11-appimage:
    cargo make --profile release -- create-app-image

[linux]
x11-binary:
    cargo make --profile release -- build-binary

[linux]
wayland-binary:
    cargo make --env NO_X11=true --profile release -- build-binary

