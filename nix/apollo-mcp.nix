{
  apple-sdk,
  cargo-zigbuild,
  crane,
  fetchzip,
  lib,
  perl,
  pkg-config,
  pkgs,
  rename,
  stdenv,
  toolchain,
  zig,
}: let
  graphqlFilter = path: _type: builtins.match ".*graphql$" path != null;
  testFilter = path: _type: builtins.match ".*snap$" path != null;
  srcFilter = path: type:
    (graphqlFilter path type) || (testFilter path type) || (craneLib.filterCargoSources path type);

  # Crane options
  src = pkgs.lib.cleanSourceWith {
    src = ../.;
    filter = srcFilter;
    name = "source"; # Be reproducible, regardless of the directory name
  };

  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
  craneCommonArgs = {
    inherit src;
    pname = "apollo-mcp";
    strictDeps = true;

    nativeBuildInputs = [perl pkg-config];
    buildInputs = [];

    # Meta information about the packages
    meta = {
      description = "Apollo MCP Server";
      homepage = "https://www.apollographql.com/docs/apollo-mcp-server";
      license = lib.licenses.mit;
      mainProgram = "apollo-mcp-server";
      platforms = lib.platforms.all;
    };
  };

  # Build the package
  apollo-mcp = craneLib.buildPackage (craneCommonArgs // {
    cargoArtifacts = craneLib.buildDepsOnly craneCommonArgs;
  });

  # Build for cross-compilation
  crossBuild = target: let
    crossCraneLib = (crane.mkLib pkgs).overrideToolchain (toolchain.override {
      targets = [target];
    });
  in
    crossCraneLib.buildPackage (craneCommonArgs // {
      cargoArtifacts = crossCraneLib.buildDepsOnly craneCommonArgs;
    });

  # Development dependencies
  nativeDependencies = with pkgs; [
    cargo-zigbuild
    zig
  ];

  # Runtime dependencies
  dependencies = with pkgs; [
    # Add any runtime dependencies here
  ];

  # Cache items for CI
  cache = [
    apollo-mcp
  ];

  # Test checks
  checks = {
    test = craneLib.cargoTest (craneCommonArgs // {
      cargoArtifacts = craneLib.buildDepsOnly craneCommonArgs;
    });
  };
in {
  inherit apollo-mcp crossBuild nativeDependencies dependencies cache checks;
  packages = {
    inherit apollo-mcp;
    builder = crossBuild;
  };
}