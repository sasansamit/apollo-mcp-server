### Setting input_schema properties to empty when operation has no args ([Issue #136](https://github.com/apollographql/apollo-mcp-server/issues/136)) ([PR #212](https://github.com/apollographql/apollo-mcp-server/pull/212))

To support certain scenarios where a client fails on an omitted `properties` field within `input_schema`, setting the field to an empty map (`{}`) instead. While a missing `properties` field is allowed this will unblock
certain users and allow them to use the MCP server.
