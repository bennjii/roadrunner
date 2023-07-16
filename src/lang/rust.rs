use crate::exec::Executor;
use crate::lang::RuntimeError;
use std::process::Command as LinearCommand;
use std::{process::Stdio, time::Instant};
use tokio::{process::Command, sync::MutexGuard};

use super::ChildWrapper;

pub fn run(exec: &MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError> {
    // Create File and Fill
    let file_dir = exec.allocated_dir.to_string();
    let file_contents: String = exec.src_file.clone();

    // Execute File
    let mut init = match LinearCommand::new("cargo")
        .args(["init", "--name", "job"])
        .current_dir(&file_dir)
        .spawn()
    {
        Ok(child) => child,
        Err(err) => return Err(RuntimeError::InitializationFailure(err.to_string())),
    };

    match init.wait() {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::InitializationFailure(err.to_string())),
    }

    match std::fs::write(format!("{}/src/main.rs", &file_dir), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    // Execute File
    let mut execution = LinearCommand::new("cargo")
        .args(["build", "--release"])
        .current_dir(&file_dir)
        .spawn()
        .unwrap();

    match execution.wait() {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::InitializationFailure(err.to_string())),
    }

    let args = exec.commandline_arguments.arguments.clone();
    let now = Instant::now();

    // Execute File
    let execution = Command::new("./target/release/job")
        .current_dir(file_dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    Ok(ChildWrapper {
        child: execution,
        start_time: now,
    })
}
