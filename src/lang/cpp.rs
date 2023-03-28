use crate::exec::{Executor};
use std::process::{Command, Stdio, Child};
use crate::lang::RuntimeError;
use tokio::sync::MutexGuard;

pub fn run(exec: &MutexGuard<Executor>) -> Result<Child, RuntimeError> {
    // Create File and Fill
    let file_dir = format!("{}", exec.allocated_dir);
    let file_contents: String = exec.src_file.clone();

    match std::fs::write(&format!("{}/main.cpp", file_dir), file_contents) {
        Ok(_) => {},
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }


    // Compile File
    let compiler = match Command::new("g++")
        .current_dir(format!("{}", file_dir))
        .args(["-o", "exec.out", "main.cpp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(val) => val,
            Err(err) => return Err(RuntimeError::InitializationFailure(format!("Command: 'g++ {} {} ' in '{}': {}", format!("-o exec.out"), format!("main.cpp"),  file_dir, err.to_string()))),
        };

    let err_output = String::from_utf8(compiler.wait_with_output().unwrap().stderr).unwrap();
    println!("Running gcc {} {}: {:?}", format!("-o exec"), format!("main.cpp"), err_output);

    let new_args = exec.commandline_arguments.arguments.clone();

    // Execute File
    let execution = match Command::new("./exec.out")
        .current_dir(format!("{}", file_dir))
        .args(new_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(val) => val,
            Err(err) => return Err(RuntimeError::InitializationFailure(format!("Command: './exec.out' in '{}': {}", file_dir, err.to_string()))),
        };

    Ok(execution)
}