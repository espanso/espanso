#!/bin/bash

set -e

echo "Testing espanso..."
cd espanso
cargo make test-binary --profile release

echo "Building espanso and creating AppImage"
cargo make create-app-image --profile release

cd ..
cp espanso/target/linux/AppImage/out/Espanso-*.AppImage Espanso-X11.AppImage
sha256sum Espanso-X11.AppImage > Espanso-X11.AppImage.sha256.txt
ls -la

echo "Copying to mounted volume"
cp Espanso-X11* /shared
