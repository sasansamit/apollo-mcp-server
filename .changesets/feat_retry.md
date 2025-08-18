### Add retry mechanism for polling collections - @DaleSeo PR #261

Right now, the MCP server restarts whenever there's a connectivity issue while polling collections from GraphOS. This causes the entire server to restart instead of handling the error gracefully.

```
Error: Failed to create operation: Error loading collection: error sending request for url (https://graphql.api.apollographql.com/api/graphql)
Caused by:
    Error loading collection: error sending request for url (https://graphql.api.apollographql.com/api/graphql)
```

This PR adds a retry mechanism with exponential backoff in the collection poller. It differentiates between transient errors, which should be retried, and permanent errors, which should fail immediately.

