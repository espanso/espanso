#!/usr/bin/env bash

set -Eeuf -o pipefail

main() {
  if [[ -z "${VERSION}" ]]; then
    echo "Missing target VERSION environment variable, please specify it"
    exit 1
  fi

  # Removing the v suffix, if present
  local VERSION=${VERSION#"v"}

  rm -rf -- target/homebrew
  mkdir -p target/homebrew/artifacts

  echo "Targeting version ${VERSION}"
  echo "Downloading macOS artifacts"

  gh release download v"${VERSION}" --pattern "Espanso-Mac*" --dir target/homebrew/artifacts

  echo "Reading artifacts hashes"
  SHA256=$(awk -F ' ' '{print $1}' target/homebrew/artifacts/Espanso-Mac-Universal.zip.sha256.txt)

  echo "Cloning tap repository"

  pushd target/homebrew
  git clone git@github.com:espanso/homebrew-espanso.git

  pushd homebrew-espanso
  echo "Rendering formula template"

  sed < ../../../scripts/resources/macos/formula_template.rb \
    -e "s/{{{VERSION}}}/${VERSION}/g" \
    -e "s/{{{SHA256}}}/${SHA256}/g" \
    > ./Casks/espanso.rb

  echo "Committing version update"
  git add Casks/espanso.rb
  git commit -m "Version bump: ${VERSION}"

  echo "Pushing changes"
  git push

  echo "Done!"
}
main "$@"
