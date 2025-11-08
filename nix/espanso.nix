{
  lib,
  stdenv,
  rustPlatform,
  coreutils,
  dbus,
  libpng,
  libX11,
  libXi,
  libxkbcommon,
  libXtst,
  openssl,
  pkg-config,
  setxkbmap,
  wl-clipboard,
  wxGTK32,
  xclip,
  xdotool,
  xorg,
  waylandSupport ? false,
  buildType ? "release",
}:
# espanso with wayland support is only supported on linux systems
assert stdenv.hostPlatform.isDarwin -> !waylandSupport;
let
  espansoCargo = with builtins; (fromTOML (readFile ../espanso/Cargo.toml));
  # per default espanso is built with X11 support
  x11Support = stdenv.hostPlatform.isLinux && !waylandSupport;
in
rustPlatform.buildRustPackage {
  pname = espansoCargo.package.name;
  inherit (espansoCargo.package) version;

  cargoLock.lockFile = ../Cargo.lock;
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../espanso
      (lib.fileset.fromSource (lib.sources.sourceByRegex ../. [ "^espanso-.*" ]))
      ../scripts
      ../Cargo.lock
      ../Cargo.toml
    ];
  };

  buildInputs = [
    libpng
    wxGTK32
  ]
  ++ lib.optionals stdenv.hostPlatform.isLinux [
    dbus
    libxkbcommon
    openssl
    setxkbmap
  ]
  ++ lib.optionals waylandSupport [
    wl-clipboard
  ]
  ++ lib.optionals x11Support [
    xorg.libxcb.dev
    libX11
    libXi
    libXtst
    xclip
    xdotool
  ];
  nativeBuildInputs = [
    pkg-config
    wxGTK32
  ];

  inherit buildType;

  buildNoDefaultFeatures = true;
  buildFeatures = [
    "modulo"
  ]
  ++ lib.optionals waylandSupport [
    "wayland"
  ]
  ++ lib.optionals stdenv.hostPlatform.isLinux [
    "vendored-tls"
  ]
  ++ lib.optional stdenv.hostPlatform.isDarwin [
    "native-tls"
  ];

  postPatch = lib.optionalString stdenv.hostPlatform.isDarwin ''
    substituteInPlace scripts/create_bundle.sh \
      --replace-fail target/mac/ $out/Applications/ \
      --replace-fail /bin/echo ${coreutils}/bin/echo
    patchShebangs scripts/create_bundle.sh
    substituteInPlace espanso/src/res/macos/Info.plist \
      --replace-fail "<string>espanso</string>" "<string>${placeholder "out"}/Applications/Espanso.app/Contents/MacOS/espanso</string>"
    substituteInPlace espanso/src/path/macos.rs  espanso/src/path/linux.rs \
      --replace-fail '"/usr/local/bin/espanso"' '"${placeholder "out"}/bin/espanso"'
  '';

  postInstall = lib.optionalString stdenv.hostPlatform.isDarwin ''
    ${stdenv.shell} ./scripts/create_bundle.sh $out/bin/espanso
  '';

  meta = {
    description = "A cross-platform Text Expander written in Rust";
    homepage = "https://espanso.org/";
    license = lib.licenses.gpl3Plus;
    mainProgram = "espanso";
  };
}
