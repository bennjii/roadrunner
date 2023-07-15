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

    match std::fs::write(format!("{}/main.c", file_dir), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    // Compile File
    let compiler = match Command::new("gcc")
        .current_dir(&file_dir)
        .args(["-o", "exec.out", "main.c"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(val) => val,
        Err(err) => {
            return Err(RuntimeError::InitializationFailure(format!(
                "Command: 'gcc {} {} ' in '{}': {}",
                "-o exec.out", "main.c", file_dir, err
            )))
        }
    };

    let err_output = String::from_utf8(compiler.wait_with_output().unwrap().stderr).unwrap();
    println!("Running gcc -o exec main.c: {:?}", err_output);

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
