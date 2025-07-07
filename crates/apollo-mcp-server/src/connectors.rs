use crate::errors::McpError;
use crate::schema_from_type;
// use reqwest::header::HeaderMap;
use rmcp::model::{CallToolResult, Content, ErrorCode, Tool};
use rmcp::schemars::JsonSchema;
use rmcp::serde_json::Value;
use rmcp::{schemars, serde_json};
use serde::{Deserialize, Serialize};

pub(crate) const CONNECTORS_TOOL_NAME: &str = "connectors-spec";

#[derive(Clone)]
pub struct Connectors {
    pub tool: Tool,
}

#[derive(JsonSchema, Deserialize, Serialize)]
pub struct Input {
    /// The GraphQL document
    #[serde(default = "default_input")]
    input: String,
}

fn default_input() -> String {
    "{}".to_string()
}

impl Connectors {

    pub fn new() -> Self {
        Self {
            tool: Tool::new(
                CONNECTORS_TOOL_NAME,
                "This tool fetches the specification which describes how to use Apollo Connectors in a graphql schema to send an HTTP request. A user may refer to an Apollo Connector as 'Apollo Connector', 'REST Connector', or even just 'Connector'. Treat these all as synonyms for the same thing. If a user is trying to write a Connector, you should use this specification as a guide.",
                schema_from_type!(Input),
            ),
        }
    }

    pub async fn execute(&self) -> Result<CallToolResult, McpError> {

        let content = reqwest::Client::new()
            .get(r"https://raw.githubusercontent.com/apollographql/router/refs/heads/am/connectorsllmmd/connectors-llm/connector-llm.md")
            .send()
            .await
            .map_err(|err| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("Failed to query for connectors markdown file: {err}"),
                    None,
                )
            })
            .map( async |response| {
                response.text().await
            });

        let result = match content {
            Ok(fut) => fut.await,
            Err(_) => todo!(),
        };

        let result = match result {
            Ok(value) => value,
            Err(_) => todo!(),
        };

        println!("{:?}", result);
        
        Ok(CallToolResult {
            content: vec![Content::text(result)],
            is_error: None,
        })
    }
}

// TODO: How the heck do I test this since my only functions are new and an async execute function?
// TODO: Should I break out the logic for fetching the markdown file into it's own function and test that in isolation?
