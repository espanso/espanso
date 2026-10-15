#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
test_dir=$(mktemp -d /tmp/espanso-shutdown-test.XXXXXX)
trap 'rm -rf "$test_dir"' EXIT
clang++ -std=c++17 -Wno-deprecated-declarations -Wno-int-conversion \
    -Iespanso-ui/src/mac \
    espanso-ui/tests/mac_shutdown.mm \
    espanso-ui/src/mac/native.mm espanso-ui/src/mac/AppDelegate.mm \
    espanso-mac-utils/src/native.mm \
    -framework Cocoa -framework Carbon -framework IOKit -framework ApplicationServices \
    -o "$test_dir/shutdown"
"$test_dir/shutdown" worker
"$test_dir/shutdown" menu
"$test_dir/shutdown" main-thread
"$test_dir/shutdown" headless
