#!/usr/bin/env bash

set -Eeuf -o pipefail

readonly BASE_DIR=$(pwd)
readonly TARGET_DIR=${BASE_DIR}/target/linux/AppImage
readonly BUILD_DIR=${TARGET_DIR}/build
readonly OUTPUT_DIR=${TARGET_DIR}/out

main() {
  local espanso_bin=${1:-${BASE_DIR}/target/release/espanso}

  rm -rf -- "${TARGET_DIR}"
  mkdir -p "${OUTPUT_DIR}"
  mkdir -p "${BUILD_DIR}"

  echo "Building AppImage into ${OUTPUT_DIR}"
  pushd "${OUTPUT_DIR}"

  linuxdeploy=$(
    find "${BASE_DIR}"/scripts/vendor-app-image \
      -maxdepth 1 \
      -name 'linuxdeploy*.AppImage' \
      -print \
      -quit
  )
  "${linuxdeploy}" --appimage-extract-and-run -e "${espanso_bin}" \
    -d "${BASE_DIR}"/espanso/src/res/linux/espanso.desktop \
    -i "${BASE_DIR}"/espanso/src/res/linux/icon.png \
    --appdir "${BUILD_DIR}" \
    --output appimage

  find . -maxdepth 1 -name 'Espanso*.AppImage' -exec chmod +x {} \; -quit

  # Apply a workaround to fix this issue: https://github.com/federico-terzi/espanso/issues/900
  # See: https://github.com/project-slippi/Ishiiruka/issues/323#issuecomment-977415376
  echo "Applying patch for libgmodule"

  espanso_appimage=$(find . -maxdepth 1 -name 'Espanso*.AppImage' -print -quit)

  "${espanso_appimage}" --appimage-extract

  find . -maxdepth 1 -name 'Espanso*.AppImage' -delete -quit
  find squashfs-root/usr/lib -maxdepth 1 -name 'libgmodule*' -delete -quit

  appimagetool=$(
    find "${BASE_DIR}"/scripts/vendor-app-image \
      -maxdepth 1 \
      -name 'appimagetool*.AppImage' \
      -print \
      -quit
  )
  "${appimagetool}" --appimage-extract-and-run -v squashfs-root
  rm -rf -- squashfs-root
}
main "$@"
