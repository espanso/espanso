set -e

[ $BUILD_ARCH != "current" ] && FINAL_EXEC_PATH=$(echo $EXEC_PATH | sed "s/target\//target\/$BUILD_ARCH\//g") || FINAL_EXEC_PATH=$EXEC_PATH

TARGET_DIR=target/mac/Espanso.app
VERSION=$(awk -F '"' '/version/ { print $2; exit; }' espanso/Cargo.toml)

rm -Rf $TARGET_DIR
mkdir -p $TARGET_DIR/Contents/MacOS $TARGET_DIR/Contents/Resources

sed -e "s/VERSION/$VERSION/" espanso/src/res/macos/Info.plist > $TARGET_DIR/Contents/Info.plist
echo "APPL????" > $TARGET_DIR/Contents/PkgInfo
cp -f espanso/src/res/macos/icon.icns $TARGET_DIR/Contents/Resources/
cp -f $FINAL_EXEC_PATH $TARGET_DIR/Contents/MacOS/espanso
