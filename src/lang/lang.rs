use crate::exec::Executor;
use phf::{Map, phf_map};
use crate::lang;
use tokio::sync::MutexGuard;

type LanguageExecutor = fn(MutexGuard<Executor>) -> Result<MutexGuard<Executor>, RuntimeError>;

static LANGUAGES: Map<&'static str, LanguageExecutor> = phf_map! {
    "python" => lang::python::run,
//    "javascript" => lang::javascript::run,
//    "rust" => lang::rust::run,
//    "go" => lang::go::run,
//    "c" => lang::c::run,
//    "cpp" => lang::cpp::run,
};

#[derive(Debug)]
pub enum RuntimeError {
    NoExecutor, Capture(&'static str)
}

#[derive(Clone, Copy)]
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

    pub fn run(mut exec: MutexGuard<Executor>) -> Result<MutexGuard<Executor>, RuntimeError> {
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