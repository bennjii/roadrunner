use crate::lang::RuntimeError;
use crate::{exec::Executor, lang::ChildWrapper};
use std::{
    process::{Command as LinearCommand, Stdio},
    time::Instant,
};
use tokio::process::Command;
use tokio::sync::MutexGuard;

pub fn run(exec: &MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError> {
    // Create File and Fill
    let file_dir = exec.allocated_dir.to_string();
    let file_contents: String = exec.src_file.clone();

    match std::fs::write(format!("{}/main.cpp", file_dir), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    // Compile File
    let compiler = match LinearCommand::new("g++")
        .current_dir(&file_dir)
        .args(["-o", "exec.out", "main.cpp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(val) => val,
        Err(err) => {
            return Err(RuntimeError::InitializationFailure(format!(
                "Command: 'g++ {} {} ' in '{}': {}",
                "-o exec.out", "main.cpp", file_dir, err
            )))
        }
    };

    let err_output = String::from_utf8(compiler.wait_with_output().unwrap().stderr).unwrap();
    println!("Running gcc -o exec main.cpp: {:?}", err_output);

    let new_args = exec.commandline_arguments.arguments.clone();
    let now = Instant::now();

    // Execute File
    let execution = match Command::new("./exec.out")
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
                "Command: './exec.out' in '{}': {}",
                file_dir, err
            )))
        }
    };

    Ok(ChildWrapper {
        child: execution,
        start_time: now,
    })
}
