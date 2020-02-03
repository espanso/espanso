#!/bin/bash

echo "Testing espanso..."
cd espanso
cargo test --release

echo "Building espanso and packaging deb"
cargo deb

cd ..
cp espanso/target/debian/espanso*.deb espanso-debian-amd64.deb
sha256sum espanso-debian-amd64.deb > espanso-debian-amd64-sha256.txt
ls -la

echo "Copying to mounted volume"
cp espanso-debian-* /shared
