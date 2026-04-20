//! clock-mcp — a Model Context Protocol server that exposes wall-clock time
//! and duration math to an AI assistant over stdio.

mod duration;
mod tools;

use anyhow::Result;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use serde::Serialize;
use tracing_subscriber::EnvFilter;

use tools::{
    convert_timezone::{self, ConvertTimezoneRequest},
    now::{self, NowRequest},
    time_between::{self, TimeBetweenRequest},
    time_since::{self, TimeSinceRequest},
    time_until::{self, TimeUntilRequest},
    ToolError,
};

#[derive(Clone)]
pub struct ClockServer {
    // Populated by the #[tool_router] macro; read by #[tool_handler].
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl Default for ClockServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl ClockServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Get the current wall-clock time. Accepts an optional IANA timezone (e.g. \"America/Denver\"); defaults to UTC. Returns ISO 8601 string, Unix timestamp (seconds), and the resolved timezone."
    )]
    async fn now(
        &self,
        Parameters(req): Parameters<NowRequest>,
    ) -> Result<CallToolResult, McpError> {
        wrap(now::run(req))
    }

    #[tool(
        description = "Given a target ISO 8601 / RFC 3339 datetime, returns the duration from now until then. Negative if the target is in the past."
    )]
    async fn time_until(
        &self,
        Parameters(req): Parameters<TimeUntilRequest>,
    ) -> Result<CallToolResult, McpError> {
        wrap(time_until::run(req))
    }

    #[tool(
        description = "Given a past ISO 8601 / RFC 3339 datetime, returns the duration from then until now. Negative if the input is actually in the future."
    )]
    async fn time_since(
        &self,
        Parameters(req): Parameters<TimeSinceRequest>,
    ) -> Result<CallToolResult, McpError> {
        wrap(time_since::run(req))
    }

    #[tool(
        description = "Given two ISO 8601 / RFC 3339 datetimes (start, end), returns the duration between them. Negative if end precedes start."
    )]
    async fn time_between(
        &self,
        Parameters(req): Parameters<TimeBetweenRequest>,
    ) -> Result<CallToolResult, McpError> {
        wrap(time_between::run(req))
    }

    #[tool(
        description = "Given an ISO 8601 / RFC 3339 datetime and a target IANA timezone, returns the equivalent local time in that zone."
    )]
    async fn convert_timezone(
        &self,
        Parameters(req): Parameters<ConvertTimezoneRequest>,
    ) -> Result<CallToolResult, McpError> {
        wrap(convert_timezone::run(req))
    }
}

#[tool_handler]
impl ServerHandler for ClockServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
            .with_instructions(
                "Wall-clock and duration-math tools.\n\n\
                 Tools:\n\
                 - now: current time (optional IANA timezone)\n\
                 - time_until: duration from now → target datetime\n\
                 - time_since: duration from past datetime → now\n\
                 - time_between: duration from one datetime → another\n\
                 - convert_timezone: re-express an instant in another IANA timezone\n\n\
                 All datetimes use ISO 8601 / RFC 3339 with a timezone offset.",
            )
    }
}

fn wrap<T: Serialize>(result: Result<T, ToolError>) -> Result<CallToolResult, McpError> {
    match result {
        Ok(value) => {
            let content = Content::json(value).map_err(|e| {
                McpError::internal_error(format!("failed to serialize response: {e}"), None)
            })?;
            Ok(CallToolResult::success(vec![content]))
        }
        Err(err) => {
            let content = Content::json(&err).map_err(|e| {
                McpError::internal_error(format!("failed to serialize error: {e}"), None)
            })?;
            Ok(CallToolResult::error(vec![content]))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Log to stderr so we never pollute the stdout MCP channel.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("clock-mcp {} starting", env!("CARGO_PKG_VERSION"));

    let service = ClockServer::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("failed to start stdio server: {e:?}");
    })?;

    service.waiting().await?;
    Ok(())
}
