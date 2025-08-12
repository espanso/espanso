#!/usr/bin/env bash

set -Eeuf -o pipefail

main() {
  local dirs
  dirs=(
    espanso
    espanso-detect
    espanso-ui
    espanso-inject
    espanso-ipc
    espanso-config
    espanso-match
    espanso-clipboard
    espanso-render
    espanso-info
    espanso-modulo
    espanso-mac-utils
    espanso-kvs
    espanso-engine
    espanso-package
  )
  local exts
  exts=(c h cc hh cpp)

  local find_exts=("(")
  for ext in "${exts[@]}"; do
    find_exts+=('-name' "*.${ext}" '-o')
  done
  # remove last element
  unset 'find_exts[${#find_exts[@]}-1]'
  find_exts+=(")")

  find "${dirs[@]}" "${find_exts[@]}" -exec clang-format -i --verbose {} +
}
main "$@"
