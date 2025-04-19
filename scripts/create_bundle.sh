#!/usr/bin/env bash

set -Eeuf -o pipefail

main() {
  TARGET_DIR=target/mac/Espanso.app

  rm -rf -- "${TARGET_DIR}"

  local VERSION=$(awk -F '"' '/^version/ { print $2; exit }' espanso/Cargo.toml)

  mkdir -p "${TARGET_DIR}"/Contents
  mkdir -p "${TARGET_DIR}"/Contents/MacOS
  mkdir -p "${TARGET_DIR}"/Contents/Resources

  sed -e "s/VERSION/${VERSION}/" espanso/src/res/macos/Info.plist > "${TARGET_DIR}"/Contents/Info.plist

  /bin/echo "APPL????" > "${TARGET_DIR}"/Contents/PkgInfo

  cp -f espanso/src/res/macos/icon.icns "${TARGET_DIR}"/Contents/Resources/icon.icns

  lipo -create \
    -output "${TARGET_DIR}/Contents/MacOS/espanso" \
    target/x86_64-apple-darwin/release/espanso target/aarch64-apple-darwin/release/espanso
}
main "$@"
