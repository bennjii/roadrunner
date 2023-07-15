use crate::exec::Executor;
use crate::lang::RuntimeError;
use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};
use std::time::Instant;
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
                "Command: 'bun app.js' in '{}': {}",
                file_dir, err
            )))
        }
    };

    let mut outstdin = execution.stdin.as_ref().unwrap();
    let mut writer = BufWriter::new(&mut outstdin);

    // Write all lines of input
    for line in &exec.terminal_feed.std_cin {
        if let Some(reference) = line.sval.as_ref() {
            writer.write_all(reference.as_bytes()).unwrap();
        }
    }

    drop(writer);

    Ok(ChildWrapper {
        child: execution,
        start_time: now,
    })
}
