#!/usr/bin/env bash

set -Eeuf -o pipefail

readonly BASE_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
readonly TARGET_DIR=${BASE_DIR}/target/linux/AppImage
readonly BUILD_DIR=${TARGET_DIR}/build
readonly OUTPUT_DIR=${TARGET_DIR}/out

main() {
  : "${BASE_DIR:?BASE_DIR is not set}"
  : "${TARGET_DIR:?TARGET_DIR is not set}"
  if [[ "${1:-}" == "--" ]]; then
    shift
  fi
  if [[ $# -gt 2 ]]; then
    echo "Usage: $0 [espanso_bin] [output_name]" >&2
    return 1
  fi
  local espanso_bin=${1:-${BASE_DIR}/target/release/espanso}
  local output_name=${2:-}
  local linuxdeploy=""
  local espanso_appimage=""
  local appimagetool=""

  # Optional output name (e.g. "Espanso-X11" or "Espanso-Wayland").
  # Accepts an optional .AppImage suffix.
  if [[ -n "${output_name}" ]]; then
    output_name=${output_name%.AppImage}
    if [[ -z "${output_name}" ]]; then
      echo "ERROR: invalid output name: ${2:-}" >&2
      return 1
    fi
  fi

  if [[ -n "${output_name}" && ! "${output_name}" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
    echo "ERROR: invalid output name: ${output_name}" >&2
    return 1
  fi

  # Allow relative paths for espanso_bin
  if [[ "${espanso_bin}" != /* ]]; then
    espanso_bin="${BASE_DIR}/${espanso_bin}"
  fi

  if [[ ! -f "${espanso_bin}" ]]; then
    echo "ERROR: No such file or directory: ${espanso_bin}" >&2
    return 1
  fi

  # Workaround for modern binutils (.relr.dyn) with the strip binary bundled
  # in the linuxdeploy AppImage (from 2022), which fails on newer objects.
  # Skipping the strip step avoids corrupting the bundled binaries.
  export NO_STRIP=1

  rm -rf -- "${TARGET_DIR:?}"
  mkdir -p "${OUTPUT_DIR}"
  mkdir -p "${BUILD_DIR}"

  echo "Building AppImage into ${OUTPUT_DIR}"
  pushd "${OUTPUT_DIR}" > /dev/null || return 1

  linuxdeploy=$(
    find "${BASE_DIR}/scripts/vendor-app-image" \
      -maxdepth 1 \
      -name 'linuxdeploy*.AppImage' \
      -print \
      -quit
  )
  if [[ -z "${linuxdeploy}" ]]; then
    echo "ERROR: linuxdeploy AppImage not found in ${BASE_DIR}/scripts/vendor-app-image" >&2
    popd > /dev/null || true
    return 1
  fi
  "${linuxdeploy}" --appimage-extract-and-run -e "${espanso_bin}" \
    -d "${BASE_DIR}"/espanso/src/res/linux/espanso.desktop \
    -i "${BASE_DIR}"/espanso/src/res/linux/espanso.png \
    --appdir "${BUILD_DIR}" \
    --output appimage

  find . -maxdepth 1 -name 'Espanso*.AppImage' -exec chmod +x -- {} \;

  # Apply a workaround to fix this issue: https://github.com/federico-terzi/espanso/issues/900
  # See: https://github.com/project-slippi/Ishiiruka/issues/323#issuecomment-977415376
  echo "Applying patch for libgmodule"

  espanso_appimage=$(find . -maxdepth 1 -name 'Espanso*.AppImage' -print -quit)

  if [[ -z "${espanso_appimage}" ]]; then
    echo "ERROR: linuxdeploy did not produce an AppImage in ${OUTPUT_DIR}" >&2
    popd > /dev/null || true
    return 1
  fi

  "${espanso_appimage}" --appimage-extract

  find . -maxdepth 1 -name 'Espanso*.AppImage' -delete
  find squashfs-root/usr/lib -maxdepth 1 -name 'libgmodule*' -delete

  appimagetool=$(
    find "${BASE_DIR}/scripts/vendor-app-image" \
      -maxdepth 1 \
      -name 'appimagetool*.AppImage' \
      -print \
      -quit
  )
  if [[ -z "${appimagetool}" ]]; then
    echo "ERROR: appimagetool AppImage not found in ${BASE_DIR}/scripts/vendor-app-image" >&2
    popd > /dev/null || true
    return 1
  fi
  "${appimagetool}" --appimage-extract-and-run -v squashfs-root
  rm -rf -- squashfs-root

  if [[ -n "${output_name}" ]]; then
    find . -maxdepth 1 -name 'Espanso*.AppImage' ! -name "${output_name}.AppImage" \
      -exec mv -- {} "${output_name}.AppImage" \; -quit
    # Remove any leftover differently-named AppImages so out/ holds a single result.
    find . -maxdepth 1 -name 'Espanso*.AppImage' ! -name "${output_name}.AppImage" -delete
    if [[ ! -f "${output_name}.AppImage" ]]; then
      echo "ERROR: expected AppImage ${output_name}.AppImage was not created in ${OUTPUT_DIR}" >&2
      popd > /dev/null || true
      return 1
    fi
    chmod +x -- "${output_name}.AppImage"
  fi

  popd > /dev/null
}
main "$@"
