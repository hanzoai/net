// DISABLED - rmcp client API changed between 0.6 and 0.8
// These functions use outdated rmcp 0.6 client-side API that is incompatible with rmcp 0.8
// Zoo Node has been upgraded to rmcp 0.8 but these client functions haven't been updated yet
// TODO: Update to rmcp 0.8 client API or remove if not needed

/*
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, ClientCapabilities, ClientInfo, Implementation},
    transport::{SseClientTransport, StreamableHttpClientTransport, TokioChildProcess},
    ServiceExt,
};
*/
use serde_json::Value;
use std::collections::HashMap;
// use tokio::process::Command;

type Result<T> = std::result::Result<T, String>;

// DISABLED - uses rmcp 0.6 API
/// Run MCP tool via child process (command)
#[allow(dead_code)]
pub async fn run_tool_via_command(
    _command: String,
    _tool: String,
    _env: HashMap<String, String>,
    _parameters: serde_json::Map<String, Value>,
) -> Result<String> {
    Err("MCP client functions temporarily disabled - rmcp 0.6 to 0.8 migration pending".to_string())
}

/*
pub async fn run_tool_via_command(
    command: String,
    tool: String,
    env: HashMap<String, String>,
    parameters: serde_json::Map<String, Value>,
) -> Result<CallToolResult> {
    // Parse command into parts
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command provided".to_string());
    }

    let program = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    // Create tokio command
    let mut cmd = Command::new(program);
    cmd.kill_on_drop(true);
    cmd.envs(env);
    cmd.args(args);

    // Create child process transport and service
    let service = ()
        .serve(TokioChildProcess::new(cmd).map_err(|e| format!("Failed to create child process: {:?}", e))?)
        .await
        .map_err(|e| format!("Failed to create service: {:?}", e))?;

    // Initialize
    service.peer_info();

    // Call the tool
    let call_tool_result = service
        .call_tool(CallToolRequestParam {
            name: tool.into(),
            arguments: Some(parameters),
        })
        .await;

    // Gracefully shut down
    let _ = service
        .cancel()
        .await
        .inspect_err(|e| log::error!("error cancelling service: {:?}", e));

    Ok(call_tool_result.map_err(|e| format!("Tool call failed: {:?}", e))?)
}
*/

// DISABLED - uses rmcp 0.6 API
/// Run MCP tool via SSE (Server-Sent Events)
#[allow(dead_code)]
pub async fn run_tool_via_sse(
    _url: String,
    _tool: String,
    _parameters: serde_json::Map<String, Value>,
) -> Result<String> {
    Err("MCP client functions temporarily disabled - rmcp 0.6 to 0.8 migration pending".to_string())
}

/*
pub async fn run_tool_via_sse(
    url: String,
    tool: String,
    parameters: serde_json::Map<String, Value>,
) -> Result<CallToolResult> {
    // Create SSE transport
    let transport = SseClientTransport::start(url)
        .await
        .inspect_err(|e| log::error!("error starting sse transport: {:?}", e))
        .map_err(|e| format!("Failed to create SSE transport: {:?}", e))?;

    // Create client info
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "zoo_node_sse_client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            icons: None,
            title: None,
            website_url: None,
        },
    };

    // Create service
    let client = client_info
        .serve(transport)
        .await
        .inspect_err(|e| log::error!("SSE client connection error: {:?}", e))
        .map_err(|e| format!("Failed to create client: {:?}", e))?;

    // Initialize
    let _ = client.peer_info();

    // Call the tool
    let call_tool_result = client
        .call_tool(CallToolRequestParam {
            name: tool.into(),
            arguments: Some(parameters),
        })
        .await
        .inspect_err(|e| log::error!("error calling tool: {:?}", e));

    // Gracefully shut down
    let _ = client
        .cancel()
        .await
        .inspect_err(|e| log::error!("error cancelling sse service: {:?}", e));

    Ok(call_tool_result.map_err(|e| format!("Tool call failed: {:?}", e))?)
}
*/

// DISABLED - uses rmcp 0.6 API
/// Run MCP tool via HTTP
#[allow(dead_code)]
pub async fn run_tool_via_http(
    _url: String,
    _tool: String,
    _parameters: serde_json::Map<String, Value>,
) -> Result<String> {
    Err("MCP client functions temporarily disabled - rmcp 0.6 to 0.8 migration pending".to_string())
}

/*
pub async fn run_tool_via_http(
    url: String,
    tool: String,
    parameters: serde_json::Map<String, Value>,
) -> Result<CallToolResult> {
    // Create HTTP transport
    let transport = StreamableHttpClientTransport::from_uri(url);

    // Create client info
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "zoo_node_http_client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            icons: None,
            title: None,
            website_url: None,
        },
    };

    // Create service
    let client = client_info
        .serve(transport)
        .await
        .inspect_err(|e| log::error!("HTTP client connection error: {:?}", e))
        .map_err(|e| format!("Failed to create client: {:?}", e))?;

    // Initialize
    let _ = client.peer_info();

    // Call the tool
    let call_tool_result = client
        .call_tool(CallToolRequestParam {
            name: tool.into(),
            arguments: Some(parameters),
        })
        .await
        .inspect_err(|e| log::error!("error calling tool: {:?}", e));

    // Gracefully shut down
    let _ = client
        .cancel()
        .await
        .inspect_err(|e| log::error!("error cancelling http service: {:?}", e));

    Ok(call_tool_result.map_err(|e| format!("Tool call failed: {:?}", e))?)
}
*/
