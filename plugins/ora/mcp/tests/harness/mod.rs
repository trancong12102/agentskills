//! A minimal MCP client, used by the end-to-end suite to drive the real binary
//! over the real protocol.
//!
//! Exercising the tools through stdio rather than by calling their functions is
//! the whole point: the argument enums, the schema, the transport framing and
//! the error mapping only exist on this path, and every bug this suite has
//! caught so far lived in a layer a direct call would have skipped.
//!
//! Panicking is how a test harness reports, so the crate-wide bans on it are
//! lifted here. The rest of the crate's lints are tuned for a long-running
//! server and do not fit a test module either.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

use serde_json::{Value, json};

/// Generous enough for a cold clone of a mid-sized repository, short enough that
/// a hung server fails the run instead of blocking it forever.
const REPLY_TIMEOUT: Duration = Duration::from_secs(120);

pub struct Server {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
    next_id: u64,
}

/// What a tool call produced: either the concatenated text content, or the
/// error the model would see.
pub type ToolResult = Result<String, String>;

impl Server {
    pub fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ora-mcp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Server logs stay visible in the test output.
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn ora-mcp");

        let stdin = child.stdin.take().expect("no stdin");
        let stdout = child.stdout.take().expect("no stdout");

        // Reads on a thread so a silent server times out rather than deadlocking
        // the test process.
        let (tx, lines) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        let mut server = Self {
            child,
            stdin,
            lines,
            next_id: 0,
        };

        let init = server.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "ora-e2e", "version": "0"},
            }),
        );
        assert!(init.get("result").is_some(), "initialize failed: {init}");
        server.notify("notifications/initialized");
        server
    }

    fn send(&mut self, message: &Value) {
        writeln!(self.stdin, "{message}").expect("write to server");
        self.stdin.flush().expect("flush");
    }

    fn notify(&mut self, method: &str) {
        let msg = json!({"jsonrpc": "2.0", "method": method});
        self.send(&msg);
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        self.send(&msg);

        // Skip anything that is not the reply to this id: notifications and
        // server-initiated requests are legal on the same stream.
        loop {
            let line = match self.lines.recv_timeout(REPLY_TIMEOUT) {
                Ok(line) => line,
                Err(RecvTimeoutError::Timeout) => {
                    panic!("no reply to {method} within {REPLY_TIMEOUT:?}")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("server exited before replying to {method}")
                }
            };
            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if value.get("id").and_then(Value::as_u64) == Some(id) {
                return value;
            }
        }
    }

    pub fn list_tools(&mut self) -> Vec<Value> {
        let reply = self.request("tools/list", json!({}));
        reply
            .pointer("/result/tools")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_else(|| panic!("tools/list returned no tools: {reply}"))
    }

    pub fn call(&mut self, tool: &str, arguments: Value) -> ToolResult {
        let reply = self.request("tools/call", json!({"name": tool, "arguments": arguments}));

        // A rejected argument arrives as a JSON-RPC error, a failed tool run as
        // a result flagged isError. Both are failures to the model, so both are
        // Err here.
        if let Some(err) = reply.pointer("/error/message").and_then(Value::as_str) {
            return Err(err.to_string());
        }

        let text: String = reply
            .pointer("/result/content")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("{tool} returned no content: {reply}"))
            .iter()
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("");

        if reply
            .pointer("/result/isError")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Err(text);
        }
        Ok(text)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Asserts on a successful call, printing the payload when it does not match so
/// a failure is diagnosable without a rerun.
#[track_caller]
pub fn assert_ok_contains(result: &ToolResult, case: &str, needles: &[&str]) {
    let body = match result {
        Ok(body) => body,
        Err(e) => panic!("[{case}] expected success, got error: {e}"),
    };
    for needle in needles {
        assert!(
            body.contains(needle),
            "[{case}] missing {needle:?} in:\n{}",
            head(body)
        );
    }
}

#[track_caller]
pub fn assert_err_contains(result: &ToolResult, case: &str, needles: &[&str]) {
    let message = match result {
        Err(message) => message,
        Ok(body) => panic!("[{case}] expected an error, got:\n{}", head(body)),
    };
    for needle in needles {
        assert!(
            message.contains(needle),
            "[{case}] missing {needle:?} in error:\n{}",
            head(message)
        );
    }
}

fn head(body: &str) -> String {
    let cut = body.char_indices().nth(1200).map_or(body.len(), |(i, _)| i);
    match body.get(..cut) {
        Some(shown) if cut < body.len() => format!("{shown}\n…[{} bytes total]", body.len()),
        _ => body.to_string(),
    }
}
