#!/bin/bash

set -e

echo "Installing cargo-deb"
cargo install cargo-deb --version 1.34.0

cd espanso

echo "Building X11 deb package"
cargo deb -p espanso

echo "Building Wayland deb package"
cargo deb -p espanso --variant wayland -- --features wayland

cd ..
cp espanso/target/debian/espanso_*.deb espanso-debian-x11-amd64.deb
sha256sum espanso-debian-x11-amd64.deb > espanso-debian-x11-amd64-sha256.txt
cp espanso/target/debian/espanso-wayland*.deb espanso-debian-wayland-amd64.deb
sha256sum espanso-debian-wayland-amd64.deb > espanso-debian-wayland-amd64-sha256.txt
ls -la

echo "Copying to mounted volume"
cp espanso-debian-* /shared
