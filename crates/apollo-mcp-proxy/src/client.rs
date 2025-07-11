use crate::server::ProxyServer;
use rmcp::transport::stdio;
use rmcp::{
    ServiceExt,
    model::{ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
};
use std::error::Error;
use tracing::{error, info};

pub async fn start_client(url: &str) -> Result<(), Box<dyn Error>> {
    let transport = StreamableHttpClientTransport::from_uri(url);
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "apollo mcp proxy client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    let client = match client_info.serve(transport).await.inspect_err(|e| {
        error!("proxy client error: {:?}", e);
    }) {
        Ok(client) => client,
        Err(e) => {
            error!("proxy client startup error: {:?}", e);
            return Err(e.into());
        }
    };

    let server_info = client.peer_info();
    info!("Connected to server: {server_info:#?}");

    let proxy_server = ProxyServer::new(client.peer().clone(), client.peer_info());
    let stdio_transport = stdio();
    let server = proxy_server.serve(stdio_transport).await?;

    server.waiting().await?;
    Ok(())
}
