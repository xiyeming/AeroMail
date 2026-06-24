use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{Duration, timeout};
use tracing::{debug, instrument};

use crate::error::AeroError;
use crate::models::ai::{AiMcpServer, AiSkill, ToolDefinition, ToolSource};
use crate::services::ai::client::ToolCall;
use crate::services::ai::tools::mcp::{McpStdioClient, McpTool};

pub mod mcp;

/// A callable tool known to the assistant.
#[derive(Debug, Clone)]
pub struct Tool {
    pub definition: ToolDefinition,
    pub handler: ToolHandler,
}

#[derive(Debug, Clone)]
pub enum ToolHandler {
    Mcp { server_id: String },
    Skill { skill_id: String },
}

/// A registry of all available tools, backed by MCP servers and local skills.
pub struct ToolRegistry {
    tools: Vec<Tool>,
    mcp_clients: HashMap<String, Arc<tokio::sync::Mutex<McpStdioClient>>>,
    skills: HashMap<String, AiSkill>,
}

impl ToolRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            mcp_clients: HashMap::new(),
            skills: HashMap::new(),
        }
    }

    /// Returns OpenAI-compatible tool definitions for the system prompt.
    #[must_use]
    pub fn list_openai_tools(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.definition.name,
                        "description": tool.definition.description,
                        "parameters": tool.definition.input_schema,
                    }
                })
            })
            .collect()
    }

    /// Finds a tool by its function name.
    #[must_use]
    pub fn find_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|t| t.definition.name == name)
    }

    /// (Re)loads tools from the provided MCP servers and skills.
    ///
    /// Existing clients are dropped and replaced.
    pub async fn refresh(&mut self, servers: &[AiMcpServer], skills: &[AiSkill]) {
        self.tools.clear();
        self.mcp_clients.clear();
        self.skills.clear();

        for skill in skills.iter().filter(|s| s.is_enabled) {
            self.register_skill(skill);
        }

        for server in servers.iter().filter(|s| s.is_enabled) {
            if let Err(e) = self.connect_server(server).await {
                debug!(server_id = %server.id, error = %e, "failed to connect mcp server");
            }
        }
    }

    fn register_skill(&mut self, skill: &AiSkill) {
        let schema: Value = serde_json::from_str(&skill.input_schema_json).unwrap_or_else(|_| {
            serde_json::json!({
                "type": "object",
                "properties": {},
            })
        });
        self.tools.push(Tool {
            definition: ToolDefinition {
                name: skill.name.clone(),
                description: Some(skill.description.clone()),
                input_schema: schema,
                source: ToolSource::Skill {
                    skill_id: skill.id.clone(),
                },
            },
            handler: ToolHandler::Skill {
                skill_id: skill.id.clone(),
            },
        });
        self.skills.insert(skill.id.clone(), skill.clone());
    }

    async fn connect_server(&mut self, server: &AiMcpServer) -> Result<(), AeroError> {
        if server.transport.to_string() != "stdio" {
            return Err(AeroError::AiToolError(format!(
                "transport {} is not supported yet",
                server.transport
            )));
        }
        let command = server
            .command
            .as_deref()
            .ok_or_else(|| AeroError::AiToolError("missing mcp command".to_string()))?;
        let args = server.args.clone().unwrap_or_default();
        let env: Option<HashMap<String, String>> = server
            .env_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|e| AeroError::AiToolError(format!("invalid env_json: {e}")))?;

        let mut client = McpStdioClient::new(command, &args, env.as_ref())?;
        client.initialize().await?;
        let tools = client.list_tools().await?;
        let client = Arc::new(tokio::sync::Mutex::new(client));
        self.mcp_clients
            .insert(server.id.clone(), Arc::clone(&client));

        for McpTool {
            name,
            description,
            input_schema,
        } in tools
        {
            self.tools.push(Tool {
                definition: ToolDefinition {
                    name,
                    description,
                    input_schema: input_schema.unwrap_or_else(|| {
                        serde_json::json!({
                            "type": "object",
                            "properties": {},
                        })
                    }),
                    source: ToolSource::Mcp {
                        server_id: server.id.clone(),
                    },
                },
                handler: ToolHandler::Mcp {
                    server_id: server.id.clone(),
                },
            });
        }

        Ok(())
    }

    /// Executes a single tool call and returns its text result.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool is unknown, arguments are invalid,
    /// or execution fails.
    #[instrument(skip(self), fields(tool_name = %call.function.name), err(Debug))]
    pub async fn execute(&self, call: &ToolCall) -> Result<String, AeroError> {
        let tool = self.find_tool(&call.function.name).ok_or_else(|| {
            AeroError::AiToolError(format!("unknown tool: {}", call.function.name))
        })?;
        let arguments: Value = serde_json::from_str(&call.function.arguments)
            .map_err(|e| AeroError::AiToolError(format!("invalid tool arguments: {e}")))?;

        match &tool.handler {
            ToolHandler::Mcp { server_id } => {
                let client = self.mcp_clients.get(server_id).ok_or_else(|| {
                    AeroError::AiToolError(format!("mcp client not connected: {server_id}"))
                })?;
                let mut guard = client.lock().await;
                guard.call_tool(&call.function.name, arguments).await
            }
            ToolHandler::Skill { skill_id } => {
                let skill = self.skills.get(skill_id).ok_or_else(|| {
                    AeroError::AiToolError(format!("skill not found: {skill_id}"))
                })?;
                run_skill(skill, arguments).await
            }
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_skill(skill: &AiSkill, arguments: Value) -> Result<String, AeroError> {
    debug!(skill_name = %skill.name, "running skill");
    let mut cmd = Command::new(&skill.command);
    if let Some(args) = &skill.args {
        cmd.args(args);
    }
    if let Some(dir) = &skill.working_dir {
        cmd.current_dir(dir);
    }
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        AeroError::AiToolError(format!("failed to spawn skill {}: {e}", skill.name))
    })?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| AeroError::AiToolError("missing skill stdin".to_string()))?;
    let input = serde_json::to_string(&arguments)
        .map_err(|e| AeroError::AiToolError(format!("failed to serialize skill args: {e}")))?;
    tokio::spawn(async move {
        let _ = stdin.write_all(input.as_bytes()).await;
        let _ = stdin.shutdown().await;
    });

    let duration = Duration::from_secs(u64::from(skill.timeout_seconds.unwrap_or(30)));
    let output = timeout(duration, child.wait_with_output())
        .await
        .map_err(|_| AeroError::AiToolError(format!("skill {} timed out", skill.name)))?
        .map_err(|e| AeroError::AiToolError(format!("failed to read skill output: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AeroError::AiToolError(format!(
            "skill {} exited with {}: {stderr}",
            skill.name,
            output.status.code().unwrap_or(-1)
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| AeroError::AiToolError(format!("skill output is not utf-8: {e}")))?;
    Ok(stdout.trim().to_string())
}
