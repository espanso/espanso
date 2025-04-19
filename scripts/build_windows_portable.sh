#!/usr/bin/env bash

set -Eeuf -o pipefail
set -x

TARGET_DIR=target/windows/portable
RESOURCE_DIR=target/windows/resources

main() {
  # Clean the target directory
  rm -rf -- "${TARGET_DIR}"

  cp -a "${RESOURCE_DIR}" "${TARGET_DIR}"

  # Create the launcher
  echo 'start espansod.exe launcher' > "${TARGET_DIR}/START_ESPANSO.bat"

  # Create the necessary folders
  mkdir -p "${TARGET_DIR}/"{.espanso,.espanso-runtime}

  cat << 'EOF' > "${TARGET_DIR}/README.txt"
Welcome to Espanso (Portable edition)!

To start espanso, you can double click on "START_ESPANSO.bat"  

After the first run, you will see some files in the ".espanso" directory.
This is where your snippets and configurations should be defined.

For more information, please visit the official documentation: 
https://espanso.org/docs/

IMPORTANT: Don't delete any file or directory, otherwise espanso won't work.


FOR ADVANCED USERS:  

Espanso also offers a rich CLI interface. To start it from the terminal, cd into the 
current directory and run "espanso start". You can also run "espanso --help" for more information.

You might have noticed that the directory contains both an "espansod.exe" and an "espanso.cmd" file.
You should generally avoid running "espansod.exe" directly, and instead use the "espanso.cmd"
wrapper (which can simply be run as "espanso" in the terminal). This is needed to correctly manage
STD console handles on Windows.
EOF
}
main "$@"
