use crate::exec::{Executor};
use std::process::{Command, Stdio, Child};
use crate::lang::RuntimeError;
use tokio::sync::MutexGuard;

pub fn run(exec: &MutexGuard<Executor>) -> Result<Child, RuntimeError> {
    // Create File and Fill
    let file_dir = format!("{}", exec.allocated_dir);
    let file_contents: String = exec.src_file.clone();

    match std::fs::write(&format!("{}/app.js", file_dir), file_contents) {
        Ok(_) => {},
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    let new_args = exec.commandline_arguments.arguments.clone();

    // Execute File
    let execution = match Command::new("~/.bun/bin/bun")
        .current_dir(file_dir.clone())
        .args(["run", "app.js"])
        // .args(new_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(val) => val,
            Err(err) => {
                panic!();
                return Err(RuntimeError::InitializationFailure(format!("Command: 'bun app.js' in '{}': {}", file_dir, err.to_string())))
            }
        };

    Ok(execution)
}