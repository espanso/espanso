#!/bin/bash

set -e

echo "Installing cargo-generate-rpm"
cargo install cargo-generate-rpm --version 0.14.0

cd espanso

echo "Building Wayland"
cargo make --profile release --env NO_X11=true build-binary

echo "Building Wayland RPM package"
cargo generate-rpm -p espanso

cd ..
cp espanso/target/generate-rpm/*.rpm espanso-fedora-wayland-amd64.rpm
sha256sum espanso-fedora-wayland-amd64.rpm > espanso-fedora-wayland-amd64-sha256.txt
ls -la

echo "Copying to mounted volume"
cp espanso-debian-* /shared
