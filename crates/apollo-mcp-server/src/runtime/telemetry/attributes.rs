use opentelemetry::Key;
use std::collections::HashSet;
use apollo_mcp_server::generated::telemetry::{TelemetryAttribute, ALL_ATTRS, APOLLO_MCP_OPERATION_ID, APOLLO_MCP_OPERATION_TYPE, APOLLO_MCP_REQUEST_ID, APOLLO_MCP_SUCCESS, APOLLO_MCP_TOOL_NAME};

// impl TelemetryAttribute {
//     pub const fn to_key(self) -> Key {
//         match self {
//             TelemetryAttribute::ToolName => Key::from_static_str(APOLLO_MCP_TOOL_NAME),
//             TelemetryAttribute::OperationId => Key::from_static_str(APOLLO_MCP_OPERATION_ID),
//             TelemetryAttribute::OperationType => Key::from_static_str(APOLLO_MCP_OPERATION_TYPE),
//             TelemetryAttribute::Success => Key::from_static_str(APOLLO_MCP_SUCCESS),
//             TelemetryAttribute::RequestId => Key::from_static_str(APOLLO_MCP_REQUEST_ID),
//         }
//     }
//
//     pub fn included_attributes(omitted: HashSet<TelemetryAttribute>) -> Vec<TelemetryAttribute> {
//         ALL_ATTRS
//             .iter()
//             .copied()
//             .filter(|a| !omitted.contains(a))
//             .collect()
//     }
// }
