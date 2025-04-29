#!/usr/bin/env bash

set -Eeuf -o pipefail
# TODO
set -x

log() {
  printf '%s\n' "$*" >&2
}

err() {
  log "$*"
  exit 1
}

readonly TARGET_DIR="target/windows/resources"

main() {
  if test -z "${EXEC_PATH:-}"; then
    err "EXEC_PATH is a required environment variable for this script"
  fi

  # Clean the target directory
  rm -rf -- "${TARGET_DIR}"

  # Create the target directory
  mkdir -p "${TARGET_DIR}"

  local vcruntime_dll=$(
    find "/c/Program Files/Microsoft Visual Studio" \
      -path "*/VC/Redist/MSVC/*" \
      -path '*/x64/*' \
      -name "vcruntime140_1.dll" -print -quit
  )
  local tooldir=$(dirname "${vcruntime_dll}")

  find "${tooldir}" -name '*.dll' -exec cp -t "${TARGET_DIR}" {} +

  cp "${EXEC_PATH}" "${TARGET_DIR}/espansod.exe"

  echo '@"%~dp0espansod.exe" %*' > "${TARGET_DIR}/espanso.cmd"
}
main "$@"
