#!/usr/bin/env bash

set -Eeuf -o pipefail
set -x

readonly WINDOWS_KITS_LOCATION="C:/Program Files (x86)/Windows Kits/10/bin"
readonly CERTIFICATE_TARGET_DIR=target/codesign

# Inspired by: https://github.com/dlemstra/code-sign-action/blob/main/index.ts#L143
get_signtool_location() {
  find "${WINDOWS_KITS_LOCATION}" \
    -type f \
    -path '*/x86/*' \
    -name 'signtool.exe' |
    sort |
    tail -1
}

main() {
  rm -rf -- "${CERTIFICATE_TARGET_DIR}"
  mkdir -p "${CERTIFICATE_TARGET_DIR}"

  local signtool_path=$(get_signtool_location)
  printf "using signtool location: %s\n" signtool_path

  local target_exe_path=${TARGET_SIGNTOOL_FILE}
  printf "signing file: %s\n" target_exe_path

  local certificate_pwd=${CODESIGN_PWD}
  local cross_signed_certificate_b64=${CODESIGN_CROSS_SIGNED_B64}

  local cross_signed_certificate_path=${CERTIFICATE_TARGET_DIR}/SectigoPublicCodeSigningRootR46_AAA.crt
  base64 \
    -o "${cross_signed_certificate_path}" \
    <<< "${cross_signed_certificate_b64}"
  trap "rm -f -- '${cross_signed_certificate_path}'" EXIT

  local codesign_certificate_path=${CERTIFICATE_TARGET_DIR}/codesign.pfx
  base64 \
    -o "${codesign_certificate_path}" \
    <<< "${CODESIGN_CERTIFICATE_B64}"
  trap "rm -f -- '${codesign_certificate_path}'" EXIT

  "${signtool_path}" \
    sign \
    /fd \
    SHA256 \
    /p \
    "${certificate_pwd}" \
    /ac \
    "${cross_signed_certificate_path}" \
    /f \
    "${CODESIGN_CERTIFICATE_B64}" \
    /tr \
    http://timestamp.sectigo.com/rfc3161 \
    /td \
    sha256 \
    "${target_exe_path}"
}
