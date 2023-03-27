use std::process::ExitStatus;

use crate::exec::Executor;
use phf::{Map, phf_map};
use serde::{Serialize, Deserialize};
use crate::lang;
use tokio::sync::MutexGuard;

type LanguageExecutor = fn(MutexGuard<Executor>) -> Result<ExitStatus, RuntimeError>;

static LANGUAGES: Map<&'static str, LanguageExecutor> = phf_map! {
    "python" => lang::python::run,
//    "javascript" => lang::javascript::run,
//    "rust" => lang::rust::run,
//    "go" => lang::go::run,
//    "c" => lang::c::run,
//    "cpp" => lang::cpp::run,
};

#[derive(Debug, Clone)]
pub enum RuntimeError {
    NoExecutor, Capture(String), WriteFailed(String)
}

impl RuntimeError {
    pub fn to_string(self) -> String {
        format!("")
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Languages {
    Python,
    Javascript,
    Rust,
    Go,
    C,
    Cpp
}

impl Languages {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::Javascript => "javascript",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp"
        }
    }

    pub fn run(exec: MutexGuard<Executor>) -> Result<ExitStatus, RuntimeError> {
        match LANGUAGES.get(Self::to_string(&exec.language)) {
            Some(executor) => {
                executor(exec)
            }
            None => {
                Err(RuntimeError::NoExecutor)
            }
        }
    }
}