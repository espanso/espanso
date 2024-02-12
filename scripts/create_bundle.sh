set -e

FINAL_EXEC_PATH=$EXEC_PATH

if [ $BUILD_ARCH != "current" ]; then
  FINAL_EXEC_PATH=$(echo $EXEC_PATH | sed "s/target\//target\/$BUILD_ARCH\//g")
fi

TARGET_DIR=target/mac/Espanso.app

rm -Rf $TARGET_DIR

VERSION=$(cat Cargo.toml | grep version | head -1 | awk -F '"' '{ print $2 }')

mkdir -p $TARGET_DIR/Contents
mkdir -p $TARGET_DIR/Contents/MacOS
mkdir -p $TARGET_DIR/Contents/Resources

sed	-e "s/VERSION/$VERSION/" espanso/src/res/macos/Info.plist > $TARGET_DIR/Contents/Info.plist

/bin/echo "APPL????" > $TARGET_DIR/Contents/PkgInfo

cp -f espanso/src/res/macos/icon.icns $TARGET_DIR/Contents/Resources/icon.icns
cp -f $FINAL_EXEC_PATH $TARGET_DIR/Contents/MacOS/espanso