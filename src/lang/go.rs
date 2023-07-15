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

    Command::new("go")
        .current_dir(&file_dir)
        .args(["mod", "init", "roadrunner.com/task"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    match std::fs::write(format!("{}/task.go", file_dir), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    // Compile File
    let compiler = match Command::new("go")
        .current_dir(&file_dir)
        .args(["build"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(val) => val,
        Err(err) => {
            return Err(RuntimeError::InitializationFailure(format!(
                "Command: 'go run .' in '{}': {}",
                file_dir, err
            )))
        }
    };

    let err_output = String::from_utf8(compiler.wait_with_output().unwrap().stderr).unwrap();
    println!("Running go build: {:?}", err_output);

    let now = Instant::now();
    let new_args = exec.commandline_arguments.arguments.clone();

    // Execute File
    let execution = match Command::new("./task")
        .current_dir(&file_dir)
        .args(new_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(val) => val,
        Err(err) => {
            return Err(RuntimeError::InitializationFailure(format!(
                "Command: './task' in '{}': {}",
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
