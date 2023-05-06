{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    dependency-refresh.url = "github:yanganto/dependency-refresh";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils, dependency-refresh }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.stable.latest.default;
        dr = dependency-refresh.defaultPackage.${system};

        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish -p toml-example-derive || echo "publish toml-example-derive fail"
          sleep 10
          cargo publish -p toml-example
        '';
        updateDependencyScript = pkgs.writeShellScriptBin "update-dependency" ''
          dr ./Cargo.toml
          if [ -f "Cargo.toml.old" ]; then
            rm Cargo.toml.old
            exit 1
          fi
        '';
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            rust

            dr
            publishScript
            updateDependencyScript
          ];
        };
      }
    );
}
