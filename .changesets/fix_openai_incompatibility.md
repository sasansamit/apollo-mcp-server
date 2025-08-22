### fix: generate openAI-compatible json schemas for list types - @DaleSeo PR #272

The MCP server is generating JSON schemas that don't match OpenAI's function calling specification. It puts `oneOf` at the array level instead of using `items` to define the JSON schemas for the GraphQL list types. While some other LLMs are more flexible about this, it technically violates the [JSON Schema specification](https://json-schema.org/understanding-json-schema/reference/array) that OpenAI strictly follows.

This PR updates the list type handling logic to move `oneOf` inside `items` for GraphQL list types.
