{
  buildGoModule,
  fetchFromGitHub,
  lib,
}:
buildGoModule rec {
  pname = "mcphost";
  version = "0.4.4";
  src = fetchFromGitHub {
    owner = "mark3labs";
    repo = pname;
    rev = "v${version}";
    hash = "sha256-TMeKQhr2uLnTGaNCQxsdA8ETLwfIHTyZrOPFNTLmxDo=";
  };

  vendorHash = "sha256-fl/i1MS4pK/U+7n9fqmP0qmF8QOldZsZ/wB5r8VWgVg=";

  meta = {
    description = "A CLI host application that enables Large Language Models (LLMs) to interact with external tools through the Model Context Protocol (MCP).";
    homepage = "https://github.com/mark3labs/mcphost";
    license = lib.licenses.mit;
  };
}
