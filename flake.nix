{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.stable.latest.default;

        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish -p toml-example-derive || echo "publish toml-example-derive fail"
          sleep 10
          cargo publish -p toml-example
        '';
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            rust
            publishScript
          ];
        };
      }
    );
}
