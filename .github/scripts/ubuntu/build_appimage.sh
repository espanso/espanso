#!/usr/bin/env bash

set -Eeuf -o pipefail

log() {
  printf '%s\n' "$*" >&2
}

STAGE_DIR=""

cleanup() {
  if [[ -n "${STAGE_DIR:-}" && -d "${STAGE_DIR}" ]]; then
    rm -rf -- "${STAGE_DIR}" || true
  fi
}

main() {
  local features name spec src_root repo_root

  repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
  src_root=${repo_root}
  STAGE_DIR=$(mktemp -d)
  trap cleanup EXIT

  log "Testing espanso (X11)..."
  pushd "${src_root}" > /dev/null || exit 1
  cargo test \
    --workspace \
    --exclude espanso-modulo \
    --exclude espanso-ipc \
    --no-default-features \
    --features modulo,vendored-tls \
    --release

  log "Testing espanso (Wayland)..."
  cargo test \
    --workspace \
    --exclude espanso-modulo \
    --exclude espanso-ipc \
    --no-default-features \
    --features wayland,modulo,vendored-tls \
    --release

  # Each create_app_image.sh run clears target/linux/AppImage/out,
  # so stage each result immediately to STAGE_DIR.
  for spec in "modulo,vendored-tls:Espanso-X11" "wayland,modulo,vendored-tls:Espanso-Wayland"; do
    features=${spec%%:*}
    name=${spec##*:}
    log "Building ${name} espanso and creating AppImage"
    cargo build \
      --no-default-features \
      --features "$features" \
      --release
    bash "${src_root}/scripts/create_app_image.sh" "${src_root}/target/release/espanso" "$name"
    rm -f -- "${STAGE_DIR}/${name}.AppImage"
    find "${src_root}/target/linux/AppImage/out" -maxdepth 1 -type f -name "${name}*.AppImage" -exec cp -- {} "${STAGE_DIR}/${name}.AppImage" \; -quit
    test -f "${STAGE_DIR}/${name}.AppImage" || { log "ERROR: ${name} AppImage was not created"; exit 1; }
  done

  popd > /dev/null || exit 1

  log "Built AppImages:"
  ls -la -- "${STAGE_DIR}/Espanso-X11.AppImage" "${STAGE_DIR}/Espanso-Wayland.AppImage"

  log "Copying to mounted volume"
  test -d /shared || { log "ERROR: /shared is not mounted, cannot export AppImages"; exit 1; }
  test -f "${STAGE_DIR}/Espanso-X11.AppImage" || { log "ERROR: Espanso-X11.AppImage is missing, cannot export"; exit 1; }
  test -f "${STAGE_DIR}/Espanso-Wayland.AppImage" || { log "ERROR: Espanso-Wayland.AppImage is missing, cannot export"; exit 1; }
  cp -- "${STAGE_DIR}/Espanso-X11.AppImage" "${STAGE_DIR}/Espanso-Wayland.AppImage" /shared/
  test -f "/shared/Espanso-X11.AppImage" || { log "ERROR: failed to export Espanso-X11.AppImage to /shared"; exit 1; }
  test -f "/shared/Espanso-Wayland.AppImage" || { log "ERROR: failed to export Espanso-Wayland.AppImage to /shared"; exit 1; }
}
main "$@"
