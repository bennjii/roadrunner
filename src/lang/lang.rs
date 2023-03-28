use std::{process::{ExitStatus, Child}, io::{Write, BufReader, BufRead}, thread};

use crate::exec::{Executor, TerminalStreamType, TerminalStream};
use phf::{Map, phf_map};
use serde::{Serialize, Deserialize};
use crate::lang;
use tokio::sync::MutexGuard;

type LanguageExecutor = fn(&MutexGuard<Executor>) -> Result<Child, RuntimeError>;

static LANGUAGES: Map<&'static str, LanguageExecutor> = phf_map! {
    "python" => lang::python::run,
    "javascript" => lang::javascript::run,
    "rust" => lang::rust::run,
//    "go" => lang::go::run,
    "c" => lang::c::run,
    "cpp" => lang::cpp::run,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RuntimeError {
    NoExecutor, Capture(String), WriteFailed(String), InitializationFailure(String), ParseInput(String)
}

impl RuntimeError {
    pub fn to_string(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type")]
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

    pub fn from_string(language: &str) -> Languages {
        match language {
            "python" => Self::Python, 
            "javascript" => Self::Javascript,
            "rust" => Self::Rust,
            "go" => Self::Go,
            "c" => Self::C,
            "cpp" => Self::Cpp,
            _ => Self::Python
        }
    }

    pub fn run(exec: MutexGuard<Executor>) -> Result<ExitStatus, RuntimeError> {
        match LANGUAGES.get(Self::to_string(&exec.language)) {
            Some(executor) => {
                let mut execution: Child = match executor(&exec) {
                    Ok(val) => val,
                    Err(err) => return Err(err),
                };
                
                let input_vec = exec.terminal_feed.std_cin.iter().map(| v | {
                    match v.terminal_type {
                        TerminalStreamType::StandardInput => v.value.as_str(),
                        _ => ""
                    }
                }).collect::<Vec<&str>>().join("\n");
            
                let mut input = execution.stdin.take().unwrap();
                input.write_all(input_vec.as_bytes()).unwrap();
            
                let child_stdout = execution
                    .stdout
                    .take()
                    .expect("Internal error, could not take stdout");
                let child_stderr = execution
                    .stderr
                    .take()
                    .expect("Internal error, could not take stderr");
            
                let sender = exec.broadcast.0.clone();
                let sender2 = exec.broadcast.0.clone();
            
                let stdout_thread = thread::spawn(move || {
                    let stdout_lines = BufReader::new(child_stdout).lines();
                    for line in stdout_lines {
                        let line = line.unwrap();
                        println!("[OKA:OUTPUT]: {}", line);
            
                        match sender.send(TerminalStream::new(TerminalStreamType::StandardOutput, line)) {
                            Ok(val) => println!("[TERM]: Sent output size {}", val),
                            Err(err) => println!("[TERM]: Failed to send output {:?}", err),
                        }
                    }
                });
            
                let stderr_thread = thread::spawn(move || {
                    let stderr_lines = BufReader::new(child_stderr).lines();
                    for line in stderr_lines {
                        let line = line.unwrap();
                        println!("[ERR:OUTPUT]: {}", line);
            
                        match sender2.send(TerminalStream::new(TerminalStreamType::StandardError, line)) {
                            Ok(val) => println!("[TERM]: Sent output size {}", val),
                            Err(err) => println!("[TERM]: Failed to send output {:?}", err),
                        }
                    }
                });
            
                let status = execution
                    .wait()
                    .expect("Internal error, failed to wait on child");
            
                stdout_thread.join().unwrap();
                stderr_thread.join().unwrap();
            
                Ok(status)
            }
            None => {
                Err(RuntimeError::NoExecutor)
            }
        }
    }
}