use crate::exec::Executor;
use crate::lang::RuntimeError;
use std::{
    io::{BufWriter, Write},
    process::{Command, Stdio},
    time::Instant,
};
use tokio::sync::MutexGuard;

use super::ChildWrapper;

pub fn run(exec: &MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError> {
    // Create File and Fill
    let file_dir = exec.allocated_dir.to_string();
    let file_contents: String = exec.src_file.clone();

    // Execute File
    match Command::new("cargo init")
        .current_dir(format!("{}/{}", file_dir, exec.id))
        .spawn()
    {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::InitializationFailure(err.to_string())),
    };

    match std::fs::write(
        format!("{}/{}/src/main.rs", file_dir, exec.id),
        file_contents,
    ) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    let args = exec.commandline_arguments.arguments.clone();
    let now = Instant::now();

    // Execute File
    let execution = Command::new("cargo run")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

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
