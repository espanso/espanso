#!/usr/bin/env bash
set -e
TARGET_DIR=$(pwd)/target/linux/AppImage
BUILD_DIR=$TARGET_DIR/build
OUTPUT_DIR=$TARGET_DIR/out
BASE_DIR=$(pwd)

rm -Rf "$TARGET_DIR"
mkdir -p $OUTPUT_DIR $BUILD_DIR

pushd $OUTPUT_DIR
$BASE_DIR/scripts/vendor-app-image/linuxdeploy*.AppImage --appimage-extract-and-run -e "$BASE_DIR/$EXEC_PATH" \
  -d "$BASE_DIR/espanso/src/res/linux/espanso.desktop" \
  -i "$BASE_DIR/espanso/src/res/linux/icon.png" \
  --appdir $BUILD_DIR \
  --output appimage
chmod +x ./Espanso*.AppImage

./Espanso*.AppImage --appimage-extract
rm -Rf ./Espanso*.AppImage squashfs-root/usr/lib/libgmodule*
$BASE_DIR/scripts/vendor-app-image/appimagetool*.AppImage --appimage-extract-and-run -v squashfs-root
rm -Rf squashfs-root

popd
