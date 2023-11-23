use std::process::ExitStatus;
use std::time::{Duration, Instant};

use crate::exec::{Executor, TerminalStream, TerminalStreamType};
use crate::lang;
use phf::{phf_map, Map};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::MutexGuard;

pub struct ChildWrapper {
    pub child: Child,
    pub start_time: Instant,
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
    "go" => lang::go::run,
    "shell" => lang::shell::run
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
    pub fn as_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Languages {
    Python,
    Javascript,
    Rust,
    Go,
    C,
    Cpp,
    Shell,
}

impl Languages {
    pub fn as_string(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::Javascript => "javascript",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Shell => "shell"
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
            "shell" => Self::Shell,
            _ => Self::Python,
        }
    }

    pub async fn run(exec: MutexGuard<'_, Executor>) -> Result<ExecutionOutput, RuntimeError> {
        match LANGUAGES.get(Self::as_string(&exec.language)) {
            Some(executor) => {
                let mut execution: ChildWrapper = match executor(&exec) {
                    Ok(val) => val,
                    Err(err) => return Err(err),
                };

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
                let mut child_stdin = execution
                    .child
                    .stdin
                    .take()
                    .expect("Internal error, could not take stdin");

                let stdout_sender = exec.broadcast.0.clone();
                let stderr_sender = exec.broadcast.0.clone();
                let stdin_sender = exec.broadcast.0.clone();

                let stdout_nonce = exec.nonce.clone();
                let stderr_nonce = exec.nonce.clone();
                let stdin_nonce = exec.nonce.clone();

                // Collate STDIN inputs
                let input_vec = exec
                    .terminal_feed
                    .std_cin
                    .iter()
                    .map(|v| match v.terminal_type {
                        TerminalStreamType::StandardInput => {
                            v.pipe_value.as_ref().unwrap().as_str()
                        }
                        _ => "",
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");

                let stdin_thread = tokio::spawn(async move {
                    match child_stdin.write(input_vec.as_bytes()).await {
                        Ok(_) => {
                            println!("Wrote all values.")
                        }
                        Err(error) => {
                            match stdin_sender.send(TerminalStream::new(
                                TerminalStreamType::StandardError,
                                format!("roadrunner_error: {}", error),
                                stdin_nonce.clone(),
                            )) {
                                Ok(val) => println!("[TERM]: Sent output size {}", val),
                                Err(err) => println!("[TERM]: Failed to send output {:?}", err),
                            }
                        }
                    }
                });

                let stdout_thread = tokio::spawn(async move {
                    let mut stdout_lines = BufReader::new(child_stdout).lines();

                    loop {
                        match stdout_lines.next_line().await {
                            Ok(line) => {
                                match line {
                                    Some(line_value) => {
                                        println!("[OKAY_OUTPUT]: {}", line_value);

                                        match stdout_sender.send(TerminalStream::new(
                                            TerminalStreamType::StandardOutput,
                                            line_value,
                                            stdout_nonce.clone(),
                                        )) {
                                            Ok(val) => println!("[TERM]: Sent output size {}", val),
                                            Err(err) => println!("[TERM]: Failed to send output {:?}", err),
                                        }
                                    }
                                    None => {
                                        println!("Consumed nothing.");
                                    }
                                }
                            }
                            Err(error) => {
                                println!("Could not consume, {}", error);
                            }
                        }
                    }
                });

                let stderr_thread = tokio::spawn(async move {
                    let mut stderr_lines = BufReader::new(child_stderr).lines();
                    loop {
                        if let Some(line) = stderr_lines.next_line().await.unwrap() {
                            println!("[ERROR_OUTPUT]: {}", line);

                            match stderr_sender.send(TerminalStream::new(
                                TerminalStreamType::StandardError,
                                line,
                                stderr_nonce.clone(),
                            )) {
                                Ok(val) => println!("[TERM]: Sent output size {}", val),
                                Err(err) => println!("[TERM]: Failed to send output {:?}", err),
                            }
                        }
                    }
                });

                let standard_timeout = Duration::from_secs(5);

                loop {
                    match execution.child.try_wait() {
                        Ok(optional) => match optional {
                            Some(exit_status) => {
                                stdin_thread.abort();
                                stdout_thread.abort();
                                stderr_thread.abort();

                                return Ok(ExecutionOutput {
                                    exit_status,
                                    duration: execution.start_time.elapsed(),
                                });
                            }
                            None => {
                                if execution.start_time.elapsed().ge(&standard_timeout)
                                    && exec.language != Languages::Shell {
                                    // Has run for too long, kill it.
                                    let _ = execution.child.start_kill();
                                }
                            }
                        },
                        Err(err) => {
                            stdin_thread.abort();
                            stdout_thread.abort();
                            stderr_thread.abort();
                            return Err(RuntimeError::Capture(err.to_string()));
                        }
                    }
                }
            }
            None => Err(RuntimeError::NoExecutor),
        }
    }
}
