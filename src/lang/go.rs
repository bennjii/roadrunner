use crate::exec::Executor;
use crate::lang::RuntimeError;
use std::process::Command as LinearCommand;
use std::{process::Stdio, time::Instant};
use tokio::process::Command;
use tokio::sync::MutexGuard;

use super::ChildWrapper;

pub fn run(exec: &MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError> {
    // Create File and Fill
    let file_dir = exec.allocated_dir.to_string();
    let file_contents: String = exec.src_file.clone();

    match LinearCommand::new("go")
        .current_dir(&file_dir)
        .args(["mod", "init", "roadrunner.com/task"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut val) => val.wait().unwrap(),
        Err(err) => {
            return Err(RuntimeError::InitializationFailure(format!(
                "Command: 'go mod init roadrunner.com/task' in '{}': {}",
                file_dir, err
            )))
        }
    };

    match std::fs::write(format!("{}/task.go", file_dir), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    // Compile File
    let compiler = match LinearCommand::new("go")
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

    Ok(ChildWrapper {
        child: execution,
        start_time: now,
    })
}
