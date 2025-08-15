### Keycloak OIDC discovery URL transformation - @DaleSeo PR #238

The MCP server currently replaces the entire path when building OIDC discovery URLs. This causes authentication failures for identity providers like Keycloak, which have path-based realms in the URL. This PR updates the URL transformation logic to preserve the existing path from the OAuth server URL.
