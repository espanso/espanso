set -e

TARGET_DIR=target/universal

rm -Rf $TARGET_DIR

mkdir -p $TARGET_DIR

lipo -create -output "$TARGET_DIR/espanso" ./target/x86_64-apple-darwin/release/espanso ./target/aarch64-apple-darwin/release/espanso