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
  find ./espanso/target/debian -name 'espanso_*.deb' -exec cp {} espanso-debian-x11-amd64.deb \; -quit

  popd

  sha256sum espanso-debian-x11-amd64.deb > espanso-debian-x11-amd64-sha256.txt
  find ./espanso/target/debian -name 'espanso-wayland*.deb' -exec cp {} espanso-debian-wayland-amd64.deb \; -quit
  sha256sum espanso-debian-wayland-amd64.deb > espanso-debian-wayland-amd64-sha256.txt

  echo "Copying to mounted volume"
  find . -maxdepth 1 -name 'espanso-debian-*' -exec cp -t /shared {} +
}
main "$@"
