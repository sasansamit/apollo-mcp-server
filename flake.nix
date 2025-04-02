{
  description = "MCP Support for Apollo Tooling";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-24.11";

    # Rust overlay for toolchain / building deterministically
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Overlay for common architecture support
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      toolchain = fenix.packages.${system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-Hn2uaQzRLidAWpfmRwSRdImifGUCAb9HeAqTYFXWeQk=";
      };
    in {
      devShells.default = pkgs.mkShell {
        buildInputs =
          [
            toolchain
          ]
          ++ (with pkgs; [
            # For local LLM testing
            ollama
          ]);
      };
    });
}
