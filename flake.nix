{
  description = "makemp4";

  nixConfig = {
    extra-substituters = [
      "https://profidev.cachix.org"
    ];

    extra-trusted-public-keys = [
      "profidev.cachix.org-1:xdwadal2vlCD50JtDTy8NwjOJvkOtjdjy1y91ElU9GE="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      nix-filter,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
          config.allowUnfree = true;
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "makemp4";
          version = "0.1.0";

          src = nix-filter {
            root = ./.;
            include = [
              "src"
              "Cargo.toml"
              "Cargo.lock"
            ];
          };

          runtimeDependencies = with pkgs; [
            makemkv
          ];

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          doCheck = false;
        };
      }
    );
}
