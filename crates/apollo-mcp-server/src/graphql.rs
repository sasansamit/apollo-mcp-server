//! Execute GraphQL operations from an MCP tool

use crate::errors::McpError;
use apollo_compiler::response::serde_json_bytes::serde_json;
use apollo_compiler::response::serde_json_bytes::serde_json::Value;
use reqwest::header::{HeaderMap, HeaderValue};
use rmcp::model::{CallToolResult, Content, ErrorCode};

pub struct Request<'a> {
    pub input: Value,
    pub endpoint: &'a str,
    pub headers: HeaderMap,
}

/// Able to be executed as a GraphQL operation
pub trait Executable {
    /// Get the persisted query ID to be executed, if any
    fn persisted_query_id(&self) -> Option<String>;

    /// Get the operation to execute
    fn operation(&self, input: Value) -> Result<String, McpError>;

    /// Get the variables to execute the operation with
    fn variables(&self, input: Value) -> Result<Value, McpError>;

    /// Get the headers to execute the operation with
    fn headers(&self, default_headers: &HeaderMap<HeaderValue>) -> HeaderMap<HeaderValue>;

    /// Get the operation name
    fn operation_name(&self) -> Option<String>;

    /// Execute as a GraphQL operation using the endpoint and headers
    async fn execute(&self, request: Request<'_>) -> Result<CallToolResult, McpError> {
        let client_metadata = serde_json::json!({
            "type": "mcp",
            "version": std::env!("CARGO_PKG_VERSION")
        });

        let mut request_body = if let Some(id) = self.persisted_query_id() {
            serde_json::json!({
                "variables": self.variables(request.input)?,
                "extensions": {
                    "persistedQuery": {
                        "version": 1,
                        "sha256Hash": id,
                    },
                    "ApolloClientMetadata": client_metadata,
                },
            })
        } else {
            serde_json::json!({
                "query": self.operation(request.input.clone())?,
                "variables": self.variables(request.input)?,
                "extensions": {
                    "ApolloClientMetadata": client_metadata,
                },
            })
        };

        if let Some(op_name) = self.operation_name() {
            if let Some(obj) = request_body.as_object_mut() {
                obj.insert(
                    "operationName".to_string(),
                    serde_json::Value::String(op_name),
                );
            }
        }

        reqwest::Client::new()
            .post(request.endpoint)
            .headers(self.headers(&request.headers))
            .body(request_body.to_string())
            .send()
            .await
            .map_err(|reqwest_error| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("Failed to send GraphQL request: {reqwest_error}"),
                    None,
                )
            })?
            .json::<Value>()
            .await
            .map_err(|reqwest_error| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("Failed to read GraphQL response body: {reqwest_error}"),
                    None,
                )
            })
            .map(|json| CallToolResult {
                content: vec![Content::json(&json).unwrap_or(Content::text(json.to_string()))],
                is_error: Some(
                    json.get("errors")
                        .filter(|value| !matches!(value, Value::Null))
                        .is_some()
                        && json
                            .get("data")
                            .filter(|value| !matches!(value, Value::Null))
                            .is_none(),
                ),
            })
    }
}
