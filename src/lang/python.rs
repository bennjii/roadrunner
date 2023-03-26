use crate::exec::Executor;
use std::process::{Command, Stdio};
use crate::exec::TerminalStream;
use std::io::BufReader;
use crate::lang::RuntimeError;
use tokio::sync::MutexGuard;
use std::thread;
use std::io::{Read, Write, BufRead};

pub fn run(exec: MutexGuard<Executor>) -> Result<MutexGuard<Executor>, RuntimeError> {
    // Create File and Fill
    let file_name = format!("{}.py", exec.id);

    let mut new_args = exec.commandline_arguments.arguments.clone();
    new_args.insert(0, format!("{}.py", file_name));

    // Execute File
    let mut execution = Command::new("python3")
        .args(new_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let input_vec = exec.terminal_feed.std_cin.iter().map(| v | {
        match v {
            TerminalStream::StandardInput(v) => v.as_str(),
            _ => ""
        }
    }).collect::<Vec<&str>>().join("\n");

    let mut input = execution.stdin.take().unwrap();
    input.write_all(input_vec.as_bytes());

    let child_stdout = execution
        .stdout
        .take()
        .expect("Internal error, could not take stdout");
    let child_stderr = execution
        .stderr
        .take()
        .expect("Internal error, could not take stderr");

//    let stdout_thread = thread::spawn(move || {
//        let stdout_lines = BufReader::new(child_stdout).lines();
//        for line in stdout_lines {
//            let line = line.unwrap();
//            exec.broadcast.0.send(TerminalStream::StandardOutput(line));
//        }
//    });
//
//    let stderr_thread = thread::spawn(move || {
//        let stderr_lines = BufReader::new(child_stderr).lines();
//        for line in stderr_lines {
//            let line = line.unwrap();
//            exec.broadcast.0.send(TerminalStream::StandardError(line));
//        }
//    });

    Ok(exec)
}