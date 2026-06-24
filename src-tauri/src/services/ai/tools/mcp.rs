use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::{debug, instrument};

use crate::error::AeroError;

#[derive(Serialize)]
struct JsonRpcRequest<T> {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: T,
}

#[derive(Deserialize)]
struct JsonRpcResponse<T> {
    id: u64,
    #[serde(default)]
    result: Option<T>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct McpInitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: Value,
    #[serde(rename = "serverInfo")]
    pub server_info: Value,
}

impl Default for McpInitializeResult {
    fn default() -> Self {
        Self {
            protocol_version: String::new(),
            capabilities: Value::Null,
            server_info: Value::Null,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: Option<Value>,
}

#[derive(Deserialize, Debug, Default)]
struct ListToolsResult {
    tools: Vec<McpTool>,
}

#[derive(Deserialize, Debug)]
struct ToolContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
struct CallToolResult {
    content: Vec<ToolContent>,
    #[serde(default)]
    is_error: bool,
}

/// A minimal MCP client using stdio transport.
pub struct McpStdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpStdioClient {
    /// Spawns an MCP server process and prepares to communicate over stdio.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be spawned or its stdio captured.
    pub fn new(
        command: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
    ) -> Result<Self, AeroError> {
        debug!(command = %command, "spawning mcp server");
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());
        if let Some(vars) = env {
            for (k, v) in vars {
                cmd.env(k, v);
            }
        }

        let mut child = cmd.spawn().map_err(|e| {
            AeroError::AiToolError(format!("failed to spawn mcp server {command}: {e}"))
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AeroError::AiToolError("missing mcp stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AeroError::AiToolError("missing mcp stdout".to_string()))?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    /// Sends the initialize handshake and waits for the server response.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or the server rejects initialization.
    #[instrument(skip(self), err(Debug))]
    pub async fn initialize(&mut self) -> Result<McpInitializeResult, AeroError> {
        #[derive(Serialize)]
        struct InitParams {
            #[serde(rename = "protocolVersion")]
            protocol_version: String,
            capabilities: Value,
            #[serde(rename = "clientInfo")]
            client_info: Value,
        }

        let params = InitParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: serde_json::json!({}),
            client_info: serde_json::json!({
                "name": "AeroMail",
                "version": env!("CARGO_PKG_VERSION"),
            }),
        };
        let result = self.request("initialize", params).await?;
        self.notify("notifications/initialized", serde_json::json!({}))
            .await?;
        Ok(result)
    }

    /// Lists tools advertised by the MCP server.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    #[instrument(skip(self), err(Debug))]
    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>, AeroError> {
        let result: ListToolsResult = self.request("tools/list", serde_json::json!({})).await?;
        Ok(result.tools)
    }

    /// Calls a tool on the MCP server with the supplied arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the tool reports an error.
    #[instrument(skip(self), fields(tool_name = %name), err(Debug))]
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<String, AeroError> {
        #[derive(Serialize)]
        struct CallParams {
            name: String,
            arguments: Value,
        }

        let params = CallParams {
            name: name.to_string(),
            arguments,
        };
        let result: CallToolResult = self.request("tools/call", params).await?;
        if result.is_error {
            return Err(AeroError::AiToolError(format!(
                "tool {name} returned an error"
            )));
        }
        let text = result
            .content
            .into_iter()
            .filter_map(|c| {
                if c.content_type == "text" {
                    c.text
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(text)
    }

    async fn request<T: Serialize, R: serde::de::DeserializeOwned + Default>(
        &mut self,
        method: &str,
        params: T,
    ) -> Result<R, AeroError> {
        let id = self.next_id;
        self.next_id += 1;
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };
        let line = serde_json::to_string(&request)
            .map_err(|e| AeroError::AiToolError(format!("failed to serialize request: {e}")))?;
        debug!(method = %method, id = id, "sending mcp request");
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| AeroError::AiToolError(format!("failed to write to mcp stdin: {e}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| AeroError::AiToolError(format!("failed to write newline: {e}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| AeroError::AiToolError(format!("failed to flush mcp stdin: {e}")))?;

        let mut buffer = String::new();
        loop {
            buffer.clear();
            let read =
                self.stdout.read_line(&mut buffer).await.map_err(|e| {
                    AeroError::AiToolError(format!("failed to read mcp stdout: {e}"))
                })?;
            if read == 0 {
                return Err(AeroError::AiToolError(
                    "mcp server closed stdout".to_string(),
                ));
            }
            let line = buffer.trim();
            if line.is_empty() {
                continue;
            }
            debug!(response = %line, "received mcp line");
            let response: JsonRpcResponse<R> = serde_json::from_str(line).map_err(|e| {
                AeroError::AiToolError(format!("failed to parse mcp response: {e}"))
            })?;
            if response.id != id {
                continue;
            }
            if let Some(err) = response.error {
                return Err(AeroError::AiToolError(format!(
                    "mcp error {}: {}",
                    err.code, err.message
                )));
            }
            return response.result.ok_or_else(|| {
                AeroError::AiToolError("missing result in mcp response".to_string())
            });
        }
    }

    async fn notify<T: Serialize>(&mut self, method: &str, params: T) -> Result<(), AeroError> {
        #[derive(Serialize)]
        struct JsonRpcNotification<P> {
            jsonrpc: &'static str,
            method: String,
            params: P,
        }

        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
        };
        let line = serde_json::to_string(&notification).map_err(|e| {
            AeroError::AiToolError(format!("failed to serialize notification: {e}"))
        })?;
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| AeroError::AiToolError(format!("failed to write notification: {e}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| AeroError::AiToolError(format!("failed to write newline: {e}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| AeroError::AiToolError(format!("failed to flush notification: {e}")))?;
        Ok(())
    }
}

impl Drop for McpStdioClient {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}
