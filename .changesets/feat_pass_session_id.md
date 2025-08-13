### feat: Pass `remote-mcp` mcp-session-id header along to GraphQL request - @damassi PR #236

This adds support for passing the `mcp-session-id` header through from `remote-mcp` via the MCP client config. This header [originates from the underlying `@modelcontextprotocol/sdk` library](https://github.com/modelcontextprotocol/typescript-sdk/blob/a1608a6513d18eb965266286904760f830de96fe/src/client/streamableHttp.ts#L182), invoked from `remote-mcp`.

With this change it is possible to correlate requests from MCP clients through to the final GraphQL server destination.
