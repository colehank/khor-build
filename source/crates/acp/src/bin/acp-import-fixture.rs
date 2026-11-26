//! Closed stdio fixture for real catalog/fork wire tests. No model or files
//! outside the caller's explicit private fixture directory are accessed.
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("full");
    let directory = args
        .get(2)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    for line in std::io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let method = request["method"].as_str().unwrap_or("");
        if method == "notifications/initialized" {
            continue;
        }
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("methods.jsonl"))
            .unwrap();
        writeln!(log, "{}", request).unwrap();
        let result = match method {
            "initialize" => {
                let mut caps =
                    json!({"loadSession":true,"sessionCapabilities":{"list":{},"resume":{}}});
                if mode != "no-archive" {
                    caps["sessionCapabilities"]["archive"] = json!({});
                }
                if mode != "no-fork" {
                    caps["sessionCapabilities"]["fork"] = json!({});
                }
                json!({"protocolVersion":1,"agentCapabilities":caps,"agentInfo":{"name":"arbitrary-provider","version":"fixture"}})
            }
            "session/list" => {
                if request["params"]["cursor"].is_null() {
                    json!({"sessions":[
                        {"sessionId":"original-a","cwd":directory,"title":"same title"},
                        {"sessionId":"archived","cwd":directory,"title":"must be hidden","archived":true}
                    ],"nextCursor":"next"})
                } else {
                    json!({"sessions":[{"sessionId":"original-a","cwd":directory,"title":"same title"},{"sessionId":"original-b","cwd":directory,"title":"same title"}]})
                }
            }
            "session/fork" => {
                if mode == "no-fork" {
                    panic!("An unadvertised fork must never be sent");
                }
                let id = if mode == "bad-fork" {
                    request["params"]["sessionId"].clone()
                } else {
                    json!("independent-branch")
                };
                json!({"sessionId":id})
            }
            "session/prompt" => json!({"stopReason":"end_turn"}),
            "session/cancel" => continue,
            _ => panic!("Unexpected provider operation {method}"),
        };
        println!(
            "{}",
            json!({"jsonrpc":"2.0","id":request["id"],"result":result})
        );
        std::io::stdout().flush().unwrap();
    }
}
