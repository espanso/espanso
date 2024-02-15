#!/bin/bash

set -e

echo "Testing espanso..."
cd espanso
cargo make --profile release -- test-binary

echo "Building espanso and creating AppImage"
cargo make --profile release -- create-app-image

cd ..
cp espanso/target/linux/AppImage/out/Espanso-*.AppImage Espanso-X11.AppImage
sha256sum Espanso-X11.AppImage > Espanso-X11.AppImage.sha256.txt
ls -la

echo "Copying to mounted volume"
cp Espanso-X11* /shared
