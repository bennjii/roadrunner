use crate::exec::Executor;
use crate::lang::RuntimeError;
use std::{process::Stdio, time::Instant};
use tokio::{process::Command, sync::MutexGuard};

use super::ChildWrapper;

pub fn run(exec: &MutexGuard<Executor>) -> Result<ChildWrapper, RuntimeError> {
    // Create File and Fill
    let file_dir = exec.allocated_dir.to_string();

    let print_override = "from __future__ import print_function
try:
    # python2
    import __builtin__
except ImportError:
    # python3
    import builtins as __builtin__
    
def print(*args, **kwargs):
    return __builtin__.print(*args, **kwargs, flush=True)\n";

    let file_contents: String = format!(
        // Provide default print() which will flush by default.
        "{}{}",
        print_override,
        exec.src_file.clone()
    );

    match std::fs::write(format!("{}/{}.py", file_dir, exec.id), file_contents) {
        Ok(_) => {}
        Err(err) => return Err(RuntimeError::WriteFailed(err.to_string())),
    }

    let mut new_args = exec.commandline_arguments.arguments.clone();
    new_args.insert(0, format!("{}/{}.py", file_dir, exec.id));

    let now = Instant::now();

    // Execute File
    let execution = Command::new("python3")
        .args(new_args)
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
