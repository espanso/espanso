#!/bin/bash

set -e

if [[ -z "$VERSION" ]]; then
  echo "Missing target VERSION environment variable, please specify it"
  exit 1
fi
# Removing the v suffix, if present
VERSION=${VERSION#"v"}

rm -Rf target/homebrew
mkdir -p target/homebrew/artifacts

echo "Targeting version $VERSION"
echo "Downloading macOS artifacts"

gh release download v$VERSION --pattern "Espanso-Mac*" --dir target/homebrew/artifacts

echo "Reading artifacts hashes"
INTEL_SHA=$(awk '{print $1}' target/homebrew/artifacts/Espanso-Mac-Intel.zip.sha256.txt)
M1_SHA=$(awk '{print $1}' target/homebrew/artifacts/Espanso-Mac-M1.zip.sha256.txt)

echo "Cloning tap repository"
pushd target/homebrew
git clone git@github.com:espanso/homebrew-espanso.git

pushd homebrew-espanso
echo "Rendering formula template"

sed "s/{{{VERSION}}}/$VERSION/g; s/{{{INTEL_SHA}}}/$INTEL_SHA/g; s/{{{M1_SHA}}}/$M1_SHA/g" \
    ../../../scripts/resources/macos/formula_template.rb > ./Casks/espanso.rb

echo "Committing version update"
git add Casks/espanso.rb
git commit -m "Version bump: $VERSION"

echo "Pushing changes"
git push

echo "Done!"
