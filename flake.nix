{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        rustWithSrc = rustToolchain.override {
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };
        deps = [ pkgs.openssl.dev ];
      in
      with pkgs;
      {
        defaultPackage = rustPlatform.buildRustPackage {
          name = "pfp";
          src = ./.;
          cargoLock = {
            lockFileContents = builtins.readFile ./Cargo.lock;
          };
          buildInputs = deps;
          nativeBuildInputs = [ pkgs.pkg-config ];
        };
        devShells.default = mkShell {
          buildInputs = [
            rustWithSrc
            sccache
            cargo-make
            lldb
          ]
          ++ deps;
          nativeBuildInputs = [ pkg-config ];
          shellHook = ''
            export RUSTC_WRAPPER="${sccache}/bin/sccache"
          '';
        };
        formatter = alejandra;
      }
    );
}
