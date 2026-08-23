#[macro_use]
mod str_enum;

pub(crate) mod ast;
pub(crate) mod docs;
pub(crate) mod pkg;
pub(crate) mod repo;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, ContentBlock};

/// Every tool returns one compact text block. Guidance the model can't infer
/// from the payload rides along in the same block rather than in the tool
/// description, so it is read on every call instead of once at load time.
pub(crate) fn text(body: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(body.into())])
}

pub(crate) fn invalid(msg: impl Into<String>) -> McpError {
    McpError::invalid_params(msg.into(), None)
}

pub(crate) fn failed(msg: impl Into<String>) -> McpError {
    McpError::internal_error(msg.into(), None)
}
