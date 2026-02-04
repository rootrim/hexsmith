{
  description =
    "Hexsmith - interactive tui for editing and using payloads, shellcodes and any other hex codes on binaries";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) { inherit system; };

        naersk' = pkgs.callPackage naersk { };

      in rec {
        defaultPackage = naersk'.buildPackage {
          pname = "hexsmith";
          version = "0.1.0";
          src = ./.;
        };

        buildInputs = with pkgs; [ openssl mold ];
        nativeBuildInputs = with pkgs; [ pkg-config ];

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rust-analyzer
            rustfmt
            openssl
            pkg-config
            mold
          ];
        };

      });
}
