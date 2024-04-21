#!/usr/bin/env sh

echo "Activating feature 'espanso_x11'"

# Devcontainer install scripts always run as root, therefore we don't need to use "sudo".
apt update -y
apt install -y --no-install-recommends libx11-dev libxtst-dev libxkbcommon-dev libdbus-1-dev libwxgtk3.2-dev