#!/usr/bin/env bash

set -Eeuf -o pipefail

main() {
  echo "Installing cargo-deb"
  cargo install cargo-deb

  pushd espanso

  echo "Building X11 deb package"
  cargo deb --package espanso -- --no-default-features --features modulo,vendored-tls

  echo "Building Wayland deb package"
  cargo deb --package espanso --variant wayland -- --no-default-features --features modulo,vendored-tls,wayland
  cp espanso/target/debian/espanso_*.deb espanso-debian-x11-amd64.deb

  popd

  sha256sum espanso-debian-x11-amd64.deb > espanso-debian-x11-amd64-sha256.txt
  cp espanso/target/debian/espanso-wayland*.deb espanso-debian-wayland-amd64.deb
  sha256sum espanso-debian-wayland-amd64.deb > espanso-debian-wayland-amd64-sha256.txt

  echo "Copying to mounted volume"
  cp espanso-debian-* /shared
}
main "$@"
