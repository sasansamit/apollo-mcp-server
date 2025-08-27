### fix: Add missing token propagation for execute tool - @DaleSeo PR #298

The execute tool is not forwarding JWT authentication tokens to upstream GraphQL endpoints, causing authentication failures when using this tool with protected APIs.
