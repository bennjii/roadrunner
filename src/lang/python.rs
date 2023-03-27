use crate::exec::{Executor, TerminalStreamType};
use std::process::{Command, Stdio, ExitStatus};
use crate::exec::TerminalStream;
use std::io::BufReader;
use crate::lang::RuntimeError;
use tokio::sync::MutexGuard;
use std::thread;
use std::io::{Write, BufRead};

pub fn run(exec: MutexGuard<Executor>) -> Result<ExitStatus, RuntimeError> {
    // Create File and Fill
    let file_dir = format!("{}", exec.allocated_dir);
    let file_contents: String = exec.src_file.clone();

    match std::fs::write(&format!("{}/{}.py", file_dir, exec.id), file_contents) {
        Ok(_) => {},
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    let mut new_args = exec.commandline_arguments.arguments.clone();
    new_args.insert(0, format!("{}/{}.py", file_dir, exec.id));

    // Execute File
    let mut execution = Command::new("python3")
        .args(new_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let input_vec = exec.terminal_feed.std_cin.iter().map(| v | {
        match v.terminal_type {
            TerminalStreamType::StandardInput => v.value.as_str(),
            _ => ""
        }
    }).collect::<Vec<&str>>().join("\n");

    let mut input = execution.stdin.take().unwrap();
    input.write_all(input_vec.as_bytes()).unwrap();

    let child_stdout = execution
        .stdout
        .take()
        .expect("Internal error, could not take stdout");
    let child_stderr = execution
        .stderr
        .take()
        .expect("Internal error, could not take stderr");

    let sender = exec.broadcast.0.clone();
    let sender2 = exec.broadcast.0.clone();

    let stdout_thread = thread::spawn(move || {
        let stdout_lines = BufReader::new(child_stdout).lines();
        for line in stdout_lines {
            let line = line.unwrap();
            println!("[OKA:OUTPUT]: {}", line);

            match sender.send(TerminalStream::new(TerminalStreamType::StandardOutput, line)) {
                Ok(val) => println!("[TERM]: Sent output size {}", val),
                Err(err) => println!("[TERM]: Failed to send output {:?}", err),
            }
        }
    });

    let stderr_thread = thread::spawn(move || {
        let stderr_lines = BufReader::new(child_stderr).lines();
        for line in stderr_lines {
            let line = line.unwrap();
            println!("[ERR:OUTPUT]: {}", line);

            match sender2.send(TerminalStream::new(TerminalStreamType::StandardError, line)) {
                Ok(val) => println!("[TERM]: Sent output size {}", val),
                Err(err) => println!("[TERM]: Failed to send output {:?}", err),
            }
        }
    });

    let status = execution
        .wait()
        .expect("Internal error, failed to wait on child");

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    Ok(status)
}