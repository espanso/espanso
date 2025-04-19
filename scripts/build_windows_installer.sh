#!/usr/bin/env bash

set -Eeuf -o pipefail
set -x

readonly INSTALLER_NAME="Espanso-Win-Installer"
readonly TARGET_DIR=$(realpath ./target/windows/installer)
readonly RESOURCE_DIR=$(realpath ./target/windows/resources)

log() {
  printf '%s\n' "$*" >&2
}

err() {
  log "$*"
  exit 1
}

toml_value_for_key_in_section() {
  local key=$1
  local section=$2
  awk \
    -F= \
    -v key="^${key} =" \
    -v section="^\\\[${section}\\\]" \
    '
      flag && $0 ~ key {
        value=$2
        sub(/^ *"?/, "", value)
        sub(/"$/, "", value)
        print value
        exit
      }
      $0 ~ section { flag++ }
    '
}

# convert /d/a/espanso/espanso/LICENSE
# to      D:\a\espanso\espanso\LICENSE
winpath() {
  sed 's|^/\([a-zA-Z]\)|\U\1:|; s|/|\\|g'
}

main() {
  # Clean the target directory
  rm -rf -- "${TARGET_DIR}"

  # Create the target directory
  mkdir -p "${TARGET_DIR}"

  local project_path=$(pwd)
  local script_resources_path=${project_path}/scripts/resources/windows
  local template_path=${script_resources_path}/setupscript.iss
  local template=$(< "${template_path}")

  local espanso_toml_path=${project_path}/espanso/Cargo.toml

  local arch=$(arch)

  local version=$(toml_value_for_key_in_section version package < "${espanso_toml_path}")
  local homepage=$(toml_value_for_key_in_section homepage package < "${espanso_toml_path}")

  local license=$(winpath <<< "${project_path}/LICENSE")

  local icon=$(winpath <<< "${script_resources_path}/icon.ico")
  local cli_helper=$(winpath <<< "${script_resources_path}/espanso.cmd")
  local exec_path=$(winpath <<< "${RESOURCE_DIR}"/espansod.exe)
  local output_dir=$(winpath <<< "${TARGET_DIR}")

  local include_paths=""
  while read -r dll; do
    local winpath_dll=$(winpath <<< "${dll}")
    include_paths+="Source: \"${winpath_dll}\"; DestDir: \"{{app}}\"; Flags: ignoreversion"$'\r\n'
  done < <(find "${RESOURCE_DIR}" -name '*.dll')

  : "${template//"{{{app_version}}}"/"${version}"}"
  : "${_//"{{{app_url}}}"/"${homepage}"}"
  : "${_//"{{{app_license}}}"/"${license}"}"
  : "${_//"{{{app_icon}}}"/"${icon}"}"
  : "${_//"{{{cli_helper}}}"/"${cli_helper}"}"
  : "${_//"{{{output_dir}}}"/"${output_dir}"}"
  : "${_//"{{{output_name}}}"/"${INSTALLER_NAME}-${arch}"}"
  : "${_//"{{{executable_path}}}"/"${exec_path}"}"
  : "${_//"{{{dll_include}}}"/"${include_paths}"}"

  template=${_}

  local iss_setup=${TARGET_DIR}/setupscript.iss
  printf '%s' "${template}" > "${iss_setup}"

  # Helpful for debugging in CI, as the iscc errors mention line numbers
  cat -n "${iss_setup}"

  iscc "${iss_setup}"
}
main "$@"
