use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ExitStatus},
    thread,
    time::Duration,
};

use crate::exec::{Executor, TerminalStream, TerminalStreamType};
use crate::lang;
use phf::{phf_map, Map};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use tokio::sync::MutexGuard;
use wait_timeout::ChildExt;

pub struct ChildWrapper {
    pub child: Child,
    pub duration: Duration,
}

#[derive(Clone, Debug, Copy)]
pub struct ExecutionOutput {
    pub exit_status: ExitStatus,
    pub duration: Duration,
}

impl Serialize for ExecutionOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_struct("execution_output", 2)?;
        seq.serialize_field("exit_status", &self.exit_status.to_string())?;
        seq.serialize_field("duration", &self.duration.as_nanos())?;
        seq.end()
    }
}

type LanguageExecutor = fn(&MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError>;

static LANGUAGES: Map<&'static str, LanguageExecutor> = phf_map! {
    "python" => lang::python::run,
    "javascript" => lang::javascript::run,
    "rust" => lang::rust::run,
    "c" => lang::c::run,
    "cpp" => lang::cpp::run,
//    "go" => lang::go::run,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RuntimeError {
    NoExecutor,
    Capture(String),
    WriteFailed(String),
    InitializationFailure(String),
    ParseInput(String),
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
    Cpp,
}

impl Languages {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::Javascript => "javascript",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
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
            _ => Self::Python,
        }
    }

    pub fn run(exec: MutexGuard<Executor>) -> Result<ExecutionOutput, RuntimeError> {
        match LANGUAGES.get(Self::to_string(&exec.language)) {
            Some(executor) => {
                let mut execution: ChildWrapper = match executor(&exec) {
                    Ok(val) => val,
                    Err(err) => return Err(err),
                };

                let input_vec = exec
                    .terminal_feed
                    .std_cin
                    .iter()
                    .map(|v| match v.terminal_type {
                        TerminalStreamType::StandardInput => v.sval.as_ref().unwrap().as_str(),
                        _ => "",
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");

                let mut input = execution.child.stdin.take().unwrap();
                input.write_all(input_vec.as_bytes()).unwrap();

                let child_stdout = execution
                    .child
                    .stdout
                    .take()
                    .expect("Internal error, could not take stdout");
                let child_stderr = execution
                    .child
                    .stderr
                    .take()
                    .expect("Internal error, could not take stderr");

                let sender = exec.broadcast.0.clone();
                let sender2 = exec.broadcast.0.clone();

                let nonce = exec.nonce.clone();
                let nonce_copy = exec.nonce.clone();

                let stdout_thread = thread::spawn(move || {
                    let stdout_lines = BufReader::new(child_stdout).lines();
                    for line in stdout_lines {
                        let line = line.unwrap();
                        println!("[OKA:OUTPUT]: {}", line);

                        match sender.send(TerminalStream::new(
                            TerminalStreamType::StandardOutput,
                            line,
                            nonce.clone(),
                        )) {
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

                        match sender2.send(TerminalStream::new(
                            TerminalStreamType::StandardError,
                            line,
                            nonce_copy.clone(),
                        )) {
                            Ok(val) => println!("[TERM]: Sent output size {}", val),
                            Err(err) => println!("[TERM]: Failed to send output {:?}", err),
                        }
                    }
                });

                let standard_timeout = Duration::from_secs(2);

                let try_exit = execution
                    .child
                    .wait_timeout(standard_timeout)
                    .expect("Internal error, failed to wait on child");

                stdout_thread.join().unwrap();
                stderr_thread.join().unwrap();

                match try_exit {
                    Some(status) => Ok(ExecutionOutput {
                        exit_status: status,
                        duration: execution.duration,
                    }),
                    None => {
                        execution.child.kill().unwrap();
                        let status = execution.child.wait().unwrap();

                        Ok(ExecutionOutput {
                            exit_status: status,
                            duration: execution.duration,
                        })
                    }
                }
            }
            None => Err(RuntimeError::NoExecutor),
        }
    }
}
