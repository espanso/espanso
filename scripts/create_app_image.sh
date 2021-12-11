#!/usr/bin/env bash
set -e
TOOL_DIR=$(pwd)/target/linux/linuxdeploy
TARGET_DIR=$(pwd)/target/linux/AppImage
BUILD_DIR=$TARGET_DIR/build
OUTPUT_DIR=$TARGET_DIR/out
BASE_DIR=$(pwd)

mkdir -p $TOOL_DIR

if ls $TOOL_DIR/*.AppImage 1> /dev/null 2>&1; then
  echo "Skipping download of linuxdeploy"
else
  echo "Downloading linuxdeploy tool"
  wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage -P "$TOOL_DIR"
  chmod +x $TOOL_DIR/linuxdeploy*.AppImage
fi

rm -Rf "$TARGET_DIR"
mkdir -p $OUTPUT_DIR
mkdir -p $BUILD_DIR

echo Building AppImage into $OUTPUT_DIR
pushd $OUTPUT_DIR
$TOOL_DIR/linuxdeploy*.AppImage --appimage-extract-and-run -e "$BASE_DIR/$EXEC_PATH" \
  -d "$BASE_DIR/espanso/src/res/linux/espanso.desktop" \
  -i "$BASE_DIR/espanso/src/res/linux/icon.png" \
  --library "/usr/lib/x86_64-linux-gnu/libglib-2.0.so.0" \
  --appdir $BUILD_DIR \
  --output appimage
chmod +x ./Espanso*.AppImage
popd