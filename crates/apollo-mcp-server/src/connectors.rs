use crate::errors::McpError;
use crate::schema_from_type;
use rmcp::model::{CallToolResult, Content, ErrorCode, Tool};
use rmcp::schemars::JsonSchema;
use rmcp::serde_json::Value;
use rmcp::{schemars, serde_json};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::fs;

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

    /// Fetch the Apollo Connectors specification from GitHub.
    ///
    /// The method asynchronously fetches the specification from the
    /// [Apollo Router](https://github.com/apollographql/router) repository.
    ///
    /// # Errors
    ///
    /// The method returns an error if the HTTP request fails.
    async fn fetch_specification() -> Result<String, Box<dyn std::error::Error>> {
        // Check if there's a local file path in environment variable
        if let Ok(local_path) = env::var("CONNECTORS_SPEC_PATH") {
            println!("📁 Using local specification file: {local_path}");
            match fs::read_to_string(&local_path).await {
                Ok(content) => {
                    println!("✅ Successfully read local file");
                    return Ok(content);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to read local file '{local_path}': {e}");
                    eprintln!("🌐 Falling back to remote URL...");
                }
            }
        }
        // Fallback to HTTP request
        println!("🌐 Fetching specification from remote URL...");
        let response = reqwest::get(r"https://raw.githubusercontent.com/apollographql/router/refs/heads/am/connectorsllmmd/connectors-llm/connector-llm.md")
            .await?;
        
        // Convert the reqwest::Error to Box<dyn std::error::Error>
        let text = response.text().await?;
        Ok(text)
    }

    /// Execute the tool.
    ///
    /// The method asynchronously fetches the Apollo Connectors specification from GitHub and returns it as a `CallToolResult`.
    ///
    /// # Errors
    ///
    /// The method returns an MCP error if the spec fetch fails.
    pub async fn execute(&self) -> Result<CallToolResult, McpError> {
        let result = Self::fetch_specification().await;

        match result {
            Ok(specification) => {
                println!("Specification fetched successfully");
                Ok(CallToolResult {
                    content: vec![Content::text(specification)],
                    is_error: None,
                })
            }
            Err(err) => {
                println!("Failed to fetch connectors specification: {err}");
                Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("Failed to fetch connectors specification: {err}"),
                    None,
                ))
            }
        }
    }
}

// TODO: How the heck do I test this since my only functions are new and an async execute function?
// TODO: Should I break out the logic for fetching the markdown file into it's own function and test that in isolation?
