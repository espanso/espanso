TARGET_DIR=target/mac/Espanso.app

rm -Rf $TARGET_DIR

VERSION=$(cat espanso/Cargo.toml | grep version | head -1 | awk -F '"' '{ print $2 }')

mkdir -p $TARGET_DIR/Contents
mkdir -p $TARGET_DIR/Contents/MacOS
mkdir -p $TARGET_DIR/Contents/Resources

sed	-e "s/VERSION/$VERSION/" espanso/src/res/macos/Info.plist > $TARGET_DIR/Contents/Info.plist

/bin/echo "APPL????" > $TARGET_DIR/Contents/PkgInfo

cp -f espanso/src/res/macos/icon.icns $TARGET_DIR/Contents/Resources/icon.icns
cp -f $EXEC_PATH $TARGET_DIR/Contents/MacOS/espanso