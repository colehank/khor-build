//! Explicit command-task and persistent-terminal intents at every entry point.
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    CommandTask(crate::command_task::Request),
    Terminal(crate::terminal_control::Request),
}

impl Action {
    pub fn is_reading(&self) -> bool {
        match self {
            Self::CommandTask(request) => request.is_reading(),
            Self::Terminal(request) => request.is_reading(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::CommandTask(request) => request.validate(),
            Self::Terminal(request) => request.validate(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reply {
    CommandTask(crate::command_task::Reply),
    Terminal(crate::terminal_control::Reply),
}

pub fn handle(root: &Path, action: Action) -> Result<Reply, String> {
    action.validate()?;
    match action {
        Action::CommandTask(request) => {
            crate::command_task::handle(root, request).map(Reply::CommandTask)
        }
        Action::Terminal(request) => {
            crate::terminal_control::handle(root, request).map(Reply::Terminal)
        }
    }
}
