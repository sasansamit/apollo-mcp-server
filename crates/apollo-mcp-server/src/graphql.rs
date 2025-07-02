//! Execute GraphQL operations from an MCP tool

use crate::errors::McpError;
use reqwest::header::{HeaderMap, HeaderValue};
use rmcp::model::{CallToolResult, Content, ErrorCode};
use rmcp::serde_json::{self, Map, Value};

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

        let mut request_body = Map::from_iter([(
            String::from("variables"),
            self.variables(request.input.clone())?,
        )]);

        if let Some(id) = self.persisted_query_id() {
            request_body.insert(
                String::from("extensions"),
                serde_json::json!({
                    "persistedQuery": {
                        "version": 1,
                        "sha256Hash": id,
                    },
                    "ApolloClientMetadata": client_metadata,
                }),
            );
        } else {
            request_body.insert(
                String::from("query"),
                Value::String(self.operation(request.input)?),
            );
            request_body.insert(
                String::from("extensions"),
                serde_json::json!({
                    "ApolloClientMetadata": client_metadata,
                }),
            );
        }

        if let Some(op_name) = self.operation_name() {
            request_body.insert("operationName".to_string(), Value::String(op_name));
        }

        reqwest::Client::new()
            .post(request.endpoint)
            .headers(self.headers(&request.headers))
            .body(Value::Object(request_body).to_string())
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
