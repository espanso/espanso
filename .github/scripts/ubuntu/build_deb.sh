#!/usr/bin/env bash

set -Eeuf -o pipefail

log() {
  printf '%s\n' "$*" >&2
}

main() {
  log "Installing cargo-deb"
  cargo install cargo-deb

  pushd espanso

  log "Building X11 deb package"
  cargo deb --package espanso -- --no-default-features --features modulo,vendored-tls

  log "Building Wayland deb package"
  cargo deb --package espanso --variant wayland -- --no-default-features --features modulo,vendored-tls,wayland

  popd

  find ./espanso/target/debian -name 'espanso_*.deb' -exec cp {} espanso-debian-x11-amd64.deb \; -quit

  find ./espanso/target/debian -name 'espanso-wayland*.deb' -exec cp {} espanso-debian-wayland-amd64.deb \; -quit

  log "Copying to mounted volume"
  find . -maxdepth 1 -name 'espanso-debian-*' -exec cp -t /shared {} +
}
main "$@"
