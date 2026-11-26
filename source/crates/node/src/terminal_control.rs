//! Bounded terminal observation and coordinated input, shared by CLI and MCP.
use crate::{Node, SessionId, host};
use serde::{Deserialize, Serialize};
use std::{net::TcpStream, path::Path, time::Duration};

pub const MAX_OUTPUT: usize = 64 * 1024;
pub const MAX_INPUT: usize = 32 * 1024;
pub const MAX_WAIT_MS: u64 = 20_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostRequest {
    Read {
        cursor: Option<u64>,
        limit: usize,
        wait_ms: u64,
    },
    Input {
        token: String,
        generation: String,
        revision: u64,
        #[serde(with = "serde_bytes")]
        bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostOutput {
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
    pub generation: String,
    pub cursor: u64,
    pub end: u64,
    pub reset: bool,
    pub input_revision: u64,
    pub shell_ready: bool,
    pub startup_released: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostReply {
    Output(HostOutput),
    Input { generation: String, revision: u64 },
    Refused { why: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Read {
        session: String,
        cursor: Option<u64>,
        limit: usize,
        wait_ms: u64,
    },
    Input {
        session: String,
        token: String,
        generation: String,
        revision: u64,
        text: String,
        enter: bool,
    },
    Ended {
        session: String,
    },
}

impl Request {
    pub fn session(&self) -> &str {
        match self {
            Self::Read { session, .. } | Self::Input { session, .. } | Self::Ended { session } => {
                session
            }
        }
    }
    pub fn is_reading(&self) -> bool {
        !matches!(self, Self::Input { .. })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.session().is_empty() || self.session().len() > 512 {
            return Err("Invalid terminal session ID".into());
        }
        match self {
            Self::Read { limit, wait_ms, .. }
                if *limit == 0 || *limit > MAX_OUTPUT || *wait_ms > MAX_WAIT_MS =>
            {
                Err("Terminal read exceeds its output or wait budget".into())
            }
            Self::Input {
                token,
                generation,
                text,
                enter,
                ..
            } if !crate::once::valid(token)
                || generation.len() != 64
                || !generation.bytes().all(|b| b.is_ascii_hexdigit())
                || text.len() + usize::from(*enter) > MAX_INPUT =>
            {
                Err("Invalid terminal input token or input budget".into())
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reply {
    Read {
        session: khor_core::Session,
        output: HostOutput,
    },
    Input {
        generation: String,
        revision: u64,
    },
    Ended {
        value: Option<crate::live::Ended>,
    },
}

fn host_call(dir: &Path, request: &HostRequest) -> Result<HostReply, String> {
    let address = host::read_host_file(dir)?;
    let mut stream = TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], address.port)),
        Duration::from_secs(3),
    )
    .map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(25)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    host::write_frame(
        &mut stream,
        &host::Hello {
            cookie: address.cookie,
            cols: 0,
            rows: 0,
            control: true,
        },
    )?;
    let welcome: host::Welcome = host::read_frame(&mut stream)?;
    if !welcome.ok {
        return Err(welcome.why);
    }
    if welcome.why != format!("khor-host/{}", host::CONTROL_VERSION) {
        return Err(format!(
            "Terminal monitoring protocol mismatch: {}",
            welcome.why
        ));
    }
    host::write_frame(&mut stream, &host::ClientOp::Version(host::CONTROL_VERSION))?;
    host::write_frame(&mut stream, request)?;
    host::read_frame(&mut stream)
}

pub fn handle(root: &Path, request: Request) -> Result<Reply, String> {
    request.validate()?;
    let node = Node::open(root.to_path_buf())?;
    let id = SessionId(request.session().to_owned());
    let view = node
        .every_session()?
        .into_iter()
        .find(|v| v.session.id == id && v.source.is_none())
        .ok_or_else(|| format!("No terminal session on this machine: {}", id.0))?;
    if let Request::Ended { .. } = request {
        return Ok(Reply::Ended {
            value: node.ended(&id),
        });
    }
    let dir = node
        .session_dir(&id)
        .ok_or_else(|| format!("No terminal host: {}", id.0))?;
    let op = match request {
        Request::Read {
            cursor,
            limit,
            wait_ms,
            ..
        } => {
            if view.session.dead {
                let bytes = node
                    .ended(&id)
                    .and_then(|e| e.screen)
                    .map(|r| r.bytes)
                    .unwrap_or_default();
                let start = bytes.len().saturating_sub(limit);
                return Ok(Reply::Read {
                    session: view.session,
                    output: HostOutput {
                        cursor: bytes.len() as u64,
                        end: bytes.len() as u64,
                        bytes: bytes[start..].to_vec(),
                        reset: true,
                        generation: String::new(),
                        input_revision: 0,
                        shell_ready: false,
                        startup_released: true,
                    },
                });
            }
            HostRequest::Read {
                cursor,
                limit,
                wait_ms,
            }
        }
        Request::Input {
            token,
            generation,
            revision,
            text,
            enter,
            ..
        } => {
            node.require_interactive_owner(&id)?;
            let mut bytes = text.into_bytes();
            if enter {
                bytes.push(b'\r');
            }
            HostRequest::Input {
                token,
                generation,
                revision,
                bytes,
            }
        }
        Request::Ended { .. } => unreachable!(),
    };
    match host_call(&dir, &op)? {
        HostReply::Output(output) => {
            let session = node
                .every_session()?
                .into_iter()
                .find(|v| v.session.id == id && v.source.is_none())
                .map(|v| v.session)
                .unwrap_or(view.session);
            Ok(Reply::Read { session, output })
        }
        HostReply::Input {
            generation,
            revision,
        } => Ok(Reply::Input {
            generation,
            revision,
        }),
        HostReply::Refused { why } => Err(why),
    }
}
