use crate::errors::McpError;
use crate::schema_from_type;
use rmcp::model::{CallToolResult, Content, Tool};
use rmcp::schemars::JsonSchema;
use rmcp::serde_json::Value;
use rmcp::{schemars, serde_json};
use serde::{Deserialize, Serialize};

pub(crate) const CONNECTORS_TOOL_NAME: &str = "connectors";

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
                "Get the connectors specification markdown",
                schema_from_type!(Input),
            ),
        }
    }

    pub async fn execute(&self) -> Result<CallToolResult, McpError> {

        // TODO: Fetch the markdown file here (or in it's own function for testability purposes?)
        
        Ok(CallToolResult {
            content: vec![Content::text("Hello world")],
            is_error: None,
        })
    }
}

// TODO: How the heck do I test this since my only functions are new and an async execute function?
// TODO: Should I break out the logic for fetching the markdown file into it's own function and test that in isolation?
