_default:
    just --list -u

alias f := format
alias fmt := format
alias t := test
alias r := ready
alias l := lint

build-binary:
    echo "building binary"

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
