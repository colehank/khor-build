use super::*;
use khor_node::{command_task, terminal_control};

struct Arguments {
    machine: String,
    positional: Vec<String>,
    options: std::collections::BTreeMap<String, String>,
}
impl Arguments {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut parsed = Self {
            machine: String::new(),
            positional: Vec::new(),
            options: Default::default(),
        };
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if arg == "--on" {
                parsed.machine = iter.next().ok_or_else(|| usage_text().to_owned())?.clone();
            } else if arg.starts_with("--") {
                if ![
                    "--request",
                    "--command",
                    "--cwd",
                    "--timeout-ms",
                    "--stdout-cursor",
                    "--stderr-cursor",
                    "--limit",
                    "--wait-ms",
                    "--cursor",
                    "--revision",
                    "--generation",
                    "--text",
                    "--enter",
                ]
                .contains(&arg.as_str())
                {
                    return Err(msg::unknown_action(arg));
                }
                let value = if arg == "--enter" {
                    "true".to_owned()
                } else {
                    iter.next().ok_or_else(|| usage_text().to_owned())?.clone()
                };
                if parsed.options.insert(arg.clone(), value).is_some() {
                    return Err(usage_text().into());
                }
            } else {
                parsed.positional.push(arg.clone());
            }
        }
        Ok(parsed)
    }
    fn text(&self, name: &str) -> String {
        self.options.get(name).cloned().unwrap_or_default()
    }
    fn number(&self, name: &str, default: u64) -> Result<u64, String> {
        self.options
            .get(name)
            .map(|s| s.parse().map_err(|_| usage_text().to_owned()))
            .unwrap_or(Ok(default))
    }
    fn id(&self) -> Result<String, String> {
        match self.positional.as_slice() {
            [id] => Ok(id.clone()),
            _ => Err(usage_text().into()),
        }
    }
}

fn print_json(value: &impl serde::Serialize) -> Result<(), String> {
    let data = serde_json::to_value(value).map_err(|e| e.to_string())?;
    record_data(data.clone());
    println!("{data}");
    Ok(())
}

pub(super) fn exec(args: &[String]) -> Result<(), String> {
    let args = Arguments::parse(args)?;
    if !args.positional.is_empty()
        || args.text("--command").is_empty()
        || args.text("--request").is_empty()
    {
        return Err(usage_text().into());
    }
    let reply = rt()?.block_on(
        node()?.command_task_on(
            &args.machine,
            command_task::Request::Start {
                token: args.text("--request"),
                command: args.text("--command"),
                argv: None,
                cwd: args.text("--cwd"),
                timeout_ms: args
                    .options
                    .get("--timeout-ms")
                    .map(|_| args.number("--timeout-ms", 0))
                    .transpose()?,
            },
        ),
    )?;
    print_json(&reply)
}

pub(super) fn execution(args: &[String]) -> Result<(), String> {
    let Some((action, args)) = args.split_first() else {
        return Err(usage_text().into());
    };
    let args = Arguments::parse(args)?;
    let id = args.id()?;
    let request = match action.as_str() {
        "status" => command_task::Request::Status { id },
        "read" => command_task::Request::Read {
            id,
            stdout_cursor: args.number("--stdout-cursor", 0)?,
            stderr_cursor: args.number("--stderr-cursor", 0)?,
            limit: args.number("--limit", 16384)?.min(usize::MAX as u64) as usize,
            wait_ms: args.number("--wait-ms", 0)?,
        },
        "cancel" => command_task::Request::Cancel { id },
        "forget" => command_task::Request::Forget { id },
        _ => return Err(usage_text().into()),
    };
    print_json(&rt()?.block_on(node()?.command_task_on(&args.machine, request))?)
}

pub(super) fn terminal(args: &[String]) -> Result<(), String> {
    let Some((action, args)) = args.split_first() else {
        return Err(usage_text().into());
    };
    let args = Arguments::parse(args)?;
    let session = args.id()?;
    let request = match action.as_str() {
        "read" => terminal_control::Request::Read {
            session,
            cursor: args
                .options
                .get("--cursor")
                .map(|_| args.number("--cursor", 0))
                .transpose()?,
            limit: args.number("--limit", 16384)?.min(usize::MAX as u64) as usize,
            wait_ms: args.number("--wait-ms", 0)?,
        },
        "input" => {
            if !args.options.contains_key("--revision") {
                return Err(usage_text().into());
            }
            terminal_control::Request::Input {
                session,
                token: args.text("--request"),
                generation: args.text("--generation"),
                revision: args.number("--revision", 0)?,
                text: args.text("--text"),
                enter: args.options.contains_key("--enter"),
            }
        }
        "ended" => terminal_control::Request::Ended { session },
        _ => return Err(usage_text().into()),
    };
    let reply = rt()?.block_on(node()?.terminal_on(&args.machine, request))?;
    match reply {
        terminal_control::Reply::Read { session, output } => print_json(
            &serde_json::json!({"session":session,"output":{"text":String::from_utf8_lossy(&output.bytes),"bytes":output.bytes,"cursor":output.cursor,"end":output.end,"reset":output.reset,"generation":output.generation,"input_revision":output.input_revision,"shell_ready":output.shell_ready,"startup_released":output.startup_released}}),
        ),
        other => print_json(&other),
    }
}

/// `run` is one command task, with its streams forwarded while its receipt is
/// retained. It creates no session row; explicit `open` creates the terminal.
pub(super) fn run_once(node: &Node, words: &[String]) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .display()
        .to_string();
    let token = khor_node::link::fresh_hex()?;
    let runtime = rt()?;
    let reply = runtime.block_on(node.command_task_on(
        "",
        command_task::Request::Start {
            token,
            command: String::new(),
            argv: Some(words.to_vec()),
            cwd,
            timeout_ms: None,
        },
    ))?;
    if in_request() {
        return print_json(&reply);
    }
    eprintln!("task: {}", reply.task.id);
    let code=runtime.block_on(async {
        let mut stdout=0;let mut stderr=0;let mut interrupted=false;
        let mut reply=reply;
        loop {
            let request=command_task::Request::Read{id:reply.task.id.clone(),stdout_cursor:stdout,stderr_cursor:stderr,limit:65536,wait_ms:200};
            reply=tokio::select! {
                result=node.command_task_on("",request)=>result?,
                signal=tokio::signal::ctrl_c(), if !interrupted=>{
                    signal.map_err(|e|e.to_string())?;
                    interrupted=true;
                    node.command_task_on("",command_task::Request::Cancel{id:reply.task.id.clone()}).await?;
                    node.command_task_on("",command_task::Request::Read{id:reply.task.id.clone(),stdout_cursor:stdout,stderr_cursor:stderr,limit:65536,wait_ms:0}).await?
                }
            };
            if reply.stdout.truncated || reply.stderr.truncated { eprintln!("{}",msg::command_output_retired()); }
            for (is_stderr,output) in [(false,&reply.stdout),(true,&reply.stderr)] {
                let bytes=output.raw_bytes()?;
                let written=if is_stderr {std::io::Write::write_all(&mut std::io::stderr().lock(),&bytes)} else {std::io::Write::write_all(&mut std::io::stdout().lock(),&bytes)};
                if let Err(error)=written {
                    let _=node.command_task_on("",command_task::Request::Cancel{id:reply.task.id.clone()}).await;
                    if error.kind()==std::io::ErrorKind::BrokenPipe {return Ok(141);}
                    return Err(error.to_string());
                }
            }
            stdout=reply.stdout.next;stderr=reply.stderr.next;
            if reply.task.phase.terminal() && stdout==reply.stdout.end && stderr==reply.stderr.end {break;}
        }
        if let Some(error)=&reply.task.error {eprintln!("{error}");}
        Ok::<_,String>(if interrupted {130} else {reply.task.exit.unwrap_or(1)})
    })?;
    CLI_EXIT.with(|value| value.set(code));
    Ok(())
}
