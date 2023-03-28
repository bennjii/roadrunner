use crate::exec::{Executor};
use std::process::{Command, Stdio, Child};
use crate::lang::RuntimeError;
use tokio::{sync::MutexGuard};

pub fn run(exec: &MutexGuard<Executor>) -> Result<Child, RuntimeError> {
    // Create File and Fill
    let file_dir = format!("{}", exec.allocated_dir);
    let file_contents: String = exec.src_file.clone();

    // Execute File
    match Command::new("cargo init")
        .current_dir(format!("{}/{}", file_dir, exec.id))
        .spawn() {
            Ok(_) => {},
            Err(err) => return Err(RuntimeError::InitializationFailure(err.to_string())),
        };

    match std::fs::write(&format!("{}/{}/src/main.rs", file_dir, exec.id), file_contents) {
        Ok(_) => {},
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    let args = exec.commandline_arguments.arguments.clone();

    // Execute File
    let execution = Command::new("cargo run")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    Ok(execution)
}