#!/usr/bin/env bash

# Creates an app bundle for MacOS
#
# Optionally accepts a path to an espanso executable as the first argument: if
# not provided, by default it expects to find release binaries for both
# x86_64-darwin and aarch64-darwin and package these in a universal binary for
# the app bundle

set -Eeuf -o pipefail

readonly TARGET_DIR=target/mac/Espanso.app

main() {
  # Pass in the binary to bundle as "$1"; default to universal
  local espanso_bin=${1:-universal}

  rm -rf -- "${TARGET_DIR}"

  local VERSION=$(awk -F '"' '/^version/ { print $2; exit }' espanso/Cargo.toml)

  mkdir -p "${TARGET_DIR}"/Contents
  mkdir -p "${TARGET_DIR}"/Contents/MacOS
  mkdir -p "${TARGET_DIR}"/Contents/Resources

  sed -e "s/VERSION/${VERSION}/" espanso/src/res/macos/Info.plist > "${TARGET_DIR}"/Contents/Info.plist

  /bin/echo "APPL????" > "${TARGET_DIR}"/Contents/PkgInfo

  cp -f espanso/src/res/macos/icon.icns "${TARGET_DIR}"/Contents/Resources/icon.icns

  if [[ "${espanso_bin}" != universal ]]; then
    cp "${espanso_bin}" "${TARGET_DIR}/Contents/MacOS/espanso"
    return
  fi

  lipo -create \
    -output "${TARGET_DIR}/Contents/MacOS/espanso" \
    target/aarch64-apple-darwin/release/espanso target/x86_64-apple-darwin/release/espanso
}
main "$@"
