### Prevent server restarts while polling collections - @DaleSeo PR #261

Right now, the MCP server restarts whenever there's a connectivity issue while polling collections from GraphOS. This causes the entire server to restart instead of handling the error gracefully.

```
Error: Failed to create operation: Error loading collection: error sending request for url (https://graphql.api.apollographql.com/api/graphql)
Caused by:
    Error loading collection: error sending request for url (https://graphql.api.apollographql.com/api/graphql)
```

This PR prevents server restarts by distinguishing between transient errors and permanent errors.

