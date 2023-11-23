use crate::exec::Executor;
use crate::lang::RuntimeError;
use std::{process::Stdio, time::Instant};
use tokio::{process::Command, sync::MutexGuard};

use super::ChildWrapper;

pub fn run(exec: &MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError> {
    // Create File and Fill
    let file_dir = exec.allocated_dir.to_string();
    let file_contents: String = exec.src_file.clone();

    match std::fs::write(format!("{}/run.sh", &file_dir), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    let now = Instant::now();

    // Execute File
    let execution = Command::new("sh")
        .args(["-x", &format!("{}/run.sh", &file_dir)])
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
