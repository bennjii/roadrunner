use crate::exec::Executor;
use crate::lang::RuntimeError;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::MutexGuard;

use super::ChildWrapper;

pub fn run(exec: &MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError> {
    // Create File and Fill
    let file_dir = exec.allocated_dir.to_string();
    let file_contents: String = exec.src_file.clone();

    match std::fs::write(format!("{}/app.js", file_dir), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    let new_args = exec.commandline_arguments.arguments.clone();

    let now = Instant::now();

    // Execute File
    let execution = match Command::new("bun")
        .current_dir(file_dir.clone())
        .args(["run", "app.js"])
        .args(new_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(val) => val,
        Err(err) => {
            return Err(RuntimeError::InitializationFailure(format!(
                "Command: 'bun run app.js' in '{}': {}",
                file_dir, err
            )))
        }
    };

    Ok(ChildWrapper {
        child: execution,
        start_time: now,
    })
}
