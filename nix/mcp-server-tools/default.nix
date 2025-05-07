{
  pkgs,
  lib,
}: let
  node-tools = pkgs.callPackage ./node-generated {};

  # Auto extract the server tools so that we don't have to add them manually
  # every time.
  node-mcp-server-tools = lib.attrsets.filterAttrs (name: val: lib.strings.hasPrefix "@modelcontextprotocol/" name) node-tools;
in
  builtins.attrValues node-mcp-server-tools
