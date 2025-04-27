{
  description = "A cross-platform Text Expander written in Rust";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs =
    {
      self,
      nixpkgs,
      ...
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      eachSystem =
        with nixpkgs.lib;
        f: foldAttrs mergeAttrs { } (map (s: mapAttrs (_: v: { ${s} = v; }) (f s)) supportedSystems);
    in
    eachSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        checks.espanso = self.packages.${system}.espanso.override { buildType = "debug"; };
        formatter = pkgs.nixfmt-rfc-style;
        packages = rec {
          espanso = pkgs.callPackage ./nix/espanso.nix { };
          espanso-wayland = pkgs.callPackage ./nix/espanso.nix {
            waylandSupport = true;
          };
          default = espanso;
        };
        devShells =
          let
            commonRustFlagsEnv = "-C target-cpu=native";
          in
          {
            default = pkgs.mkShell {
              name = "espanso";
              inputsFrom = [ self.checks.${system}.espanso ];
              packages =
                with pkgs;
                [
                  biome
                  cargo-make
                  clang-tools
                  rustfmt
                  rust-script
                ]
                ++ lib.optional (stdenv.isx86_64 && stdenv.isLinux) [ cargo-tarpaulin ];
              shellHook = ''
                export RUST_BACKTRACE="1"
                export RUSTFLAGS="''${RUSTFLAGS:-""} ${commonRustFlagsEnv}";
              '';
            };
          };
      }
    );
}
