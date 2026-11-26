use super::{Node, Tool};
use khor_catalog::msg;
use serde_json::{Value, json};

fn schema(fields: &[(&str, &str)], required: &[&str]) -> Value {
    let mut properties = serde_json::Map::new();
    for (name, kind) in fields {
        properties.insert((*name).into(), json!({"type":kind}));
    }
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}

pub(super) const EXECUTE: Tool = Tool {
    name: "execute",
    reads: false,
    about: msg::mcp_execute,
    schema: || {
        schema(
            &[
                ("machine", "string"),
                ("command", "string"),
                ("cwd", "string"),
                ("request", "string"),
                ("timeout_ms", "integer"),
            ],
            &["machine", "command", "request"],
        )
    },
};
pub(super) const STATUS: Tool = Tool {
    name: "execution_status",
    reads: true,
    about: msg::mcp_execution_status,
    schema: || {
        schema(
            &[("machine", "string"), ("task", "string")],
            &["machine", "task"],
        )
    },
};
pub(super) const OUTPUT: Tool = Tool {
    name: "execution_output",
    reads: true,
    about: msg::mcp_execution_output,
    schema: || {
        schema(
            &[
                ("machine", "string"),
                ("task", "string"),
                ("stdout_cursor", "integer"),
                ("stderr_cursor", "integer"),
                ("limit", "integer"),
                ("wait_ms", "integer"),
            ],
            &["machine", "task"],
        )
    },
};
pub(super) const CANCEL: Tool = Tool {
    name: "execution_cancel",
    reads: false,
    about: msg::mcp_execution_cancel,
    schema: || {
        schema(
            &[("machine", "string"), ("task", "string")],
            &["machine", "task"],
        )
    },
};
pub(super) const FORGET: Tool = Tool {
    name: "execution_forget",
    reads: false,
    about: msg::mcp_execution_forget,
    schema: || {
        schema(
            &[("machine", "string"), ("task", "string")],
            &["machine", "task"],
        )
    },
};
pub(super) const TERMINAL_READ: Tool = Tool {
    name: "terminal_read",
    reads: true,
    about: msg::mcp_terminal_read,
    schema: || {
        schema(
            &[
                ("machine", "string"),
                ("session", "string"),
                ("cursor", "integer"),
                ("limit", "integer"),
                ("wait_ms", "integer"),
            ],
            &["session"],
        )
    },
};
pub(super) const TERMINAL_INPUT: Tool = Tool {
    name: "terminal_input",
    reads: false,
    about: msg::mcp_terminal_input,
    schema: || {
        schema(
            &[
                ("machine", "string"),
                ("session", "string"),
                ("request", "string"),
                ("generation", "string"),
                ("revision", "integer"),
                ("text", "string"),
                ("enter", "boolean"),
            ],
            &["session", "request", "generation", "revision", "text"],
        )
    },
};
pub(super) const TERMINAL_ENDED: Tool = Tool {
    name: "terminal_ended",
    reads: true,
    about: msg::mcp_terminal_ended,
    schema: || {
        schema(
            &[("machine", "string"), ("session", "string")],
            &["session"],
        )
    },
};

pub(super) fn call(node: &Node, name: &str, args: &Value) -> Option<Result<String, String>> {
    let string = |key: &str| args[key].as_str().unwrap_or("").to_owned();
    let number = |key: &str, default| args[key].as_u64().unwrap_or(default);
    let request = match name {
        "execute" => crate::command_task::Request::Start {
            token: string("request"),
            command: string("command"),
            argv: None,
            cwd: string("cwd"),
            timeout_ms: args["timeout_ms"].as_u64(),
        },
        "execution_status" => crate::command_task::Request::Status { id: string("task") },
        "execution_output" => crate::command_task::Request::Read {
            id: string("task"),
            stdout_cursor: number("stdout_cursor", 0),
            stderr_cursor: number("stderr_cursor", 0),
            limit: number("limit", 16384).min(usize::MAX as u64) as usize,
            wait_ms: number("wait_ms", 0),
        },
        "execution_cancel" => crate::command_task::Request::Cancel { id: string("task") },
        "execution_forget" => crate::command_task::Request::Forget { id: string("task") },
        "terminal_read" | "terminal_input" | "terminal_ended" => {
            let request = match name {
                "terminal_read" => crate::terminal_control::Request::Read {
                    session: string("session"),
                    cursor: args["cursor"].as_u64(),
                    limit: number("limit", 16384).min(usize::MAX as u64) as usize,
                    wait_ms: number("wait_ms", 0),
                },
                "terminal_input" => crate::terminal_control::Request::Input {
                    session: string("session"),
                    token: string("request"),
                    generation: string("generation"),
                    revision: number("revision", 0),
                    text: string("text"),
                    enter: args["enter"].as_bool().unwrap_or(false),
                },
                _ => crate::terminal_control::Request::Ended {
                    session: string("session"),
                },
            };
            return Some(super::runtime().and_then(|rt| rt.block_on(node.terminal_on(&string("machine"),request))).and_then(|reply| match reply {
                crate::terminal_control::Reply::Read { session, output } => serde_json::to_string(&json!({"session":session,"output":{"text":String::from_utf8_lossy(&output.bytes),"bytes":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,&output.bytes),"cursor":output.cursor,"end":output.end,"reset":output.reset,"generation":output.generation,"input_revision":output.input_revision,"shell_ready":output.shell_ready,"startup_released":output.startup_released}})).map_err(|e|e.to_string()),
                other => serde_json::to_string(&other).map_err(|e|e.to_string()),
            }));
        }
        _ => return None,
    };
    Some(
        super::runtime()
            .and_then(|rt| rt.block_on(node.command_task_on(&string("machine"), request)))
            .and_then(|reply| serde_json::to_string(&reply).map_err(|e| e.to_string())),
    )
}
