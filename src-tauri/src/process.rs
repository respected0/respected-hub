//! Running a child process with a request on stdin, a hard time limit and bounded output.
use std::{io::{Read, Write}, process::Command, time::{Duration, Instant}};

pub(crate) struct PumpOutput { pub stdout: Vec<u8>, pub stderr: Vec<u8>, pub success: bool }

pub(crate) enum PumpError { Spawn(String), Missing(&'static str), Io(String), Timeout, Reader(&'static str), TooLarge }

impl PumpError {
    pub(crate) fn message(&self) -> String {
        match self {
            PumpError::Spawn(e) => format!("Engine could not start: {e}"),
            PumpError::Missing(what) => (*what).to_string(),
            PumpError::Io(e) => e.clone(),
            PumpError::Timeout => "Engine request timed out; inspect task state before retrying".into(),
            PumpError::Reader(which) => format!("Engine {which} reader failed"),
            PumpError::TooLarge => "Engine output exceeded the allowed size".into(),
        }
    }
}

/// Kill the engine and everything it started (a plain `kill` leaves grandchildren running and writing files).
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        let _ = Command::new("taskkill").args(["/T", "/F", "/PID", &child.id().to_string()]).creation_flags(0x08000000).output();
    }
    let _ = child.kill();
    let _ = child.wait();
}

pub(crate) fn pump(mut command: Command, request: Option<Vec<u8>>, timeout: Duration, max_output: usize) -> Result<PumpOutput, PumpError> {
    let mut child = command.spawn().map_err(|e| PumpError::Spawn(e.to_string()))?;
    // The request goes in on its own thread: a child that never reads its stdin must not stall us before the time limit starts.
    let writer = child.stdin.take().map(|mut input| std::thread::spawn(move || {
        let result = request.map_or(Ok(()), |payload| input.write_all(&payload));
        drop(input);
        result
    }));
    let stdout = child.stdout.take().ok_or(PumpError::Missing("Missing stdout"))?;
    let stderr = child.stderr.take().ok_or(PumpError::Missing("Missing stderr"))?;
    let limit = max_output.saturating_add(1) as u64;
    let out = std::thread::spawn(move || { let mut bytes = Vec::new(); stdout.take(limit).read_to_end(&mut bytes).map(|_| bytes) });
    let err = std::thread::spawn(move || { let mut bytes = Vec::new(); stderr.take(1 << 20).read_to_end(&mut bytes).map(|_| bytes) });
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| PumpError::Io(e.to_string()))? { break status; }
        if Instant::now() >= deadline { kill_tree(&mut child); return Err(PumpError::Timeout); }
        std::thread::sleep(Duration::from_millis(25));
    };
    // a grandchild that inherited the pipes must not hold the caller (and the engine gate) forever
    let grace = Instant::now() + Duration::from_secs(10);
    while !(out.is_finished() && err.is_finished()) {
        if Instant::now() >= grace { return Err(PumpError::Io("Engine output pipe is still held by a child process".into())); }
        std::thread::sleep(Duration::from_millis(25));
    }
    if let Some(writer) = writer {
        // the child is gone, so the pipe is closed and the writer has either finished or failed with a broken pipe
        if let Ok(Err(error)) = writer.join() { if error.kind() != std::io::ErrorKind::BrokenPipe { return Err(PumpError::Io(error.to_string())); } }
    }
    let bytes = out.join().map_err(|_| PumpError::Reader("output"))?.map_err(|e| PumpError::Io(e.to_string()))?;
    let error_bytes = err.join().map_err(|_| PumpError::Reader("error"))?.map_err(|e| PumpError::Io(e.to_string()))?;
    if bytes.len() > max_output { return Err(PumpError::TooLarge); }
    Ok(PumpOutput { stdout: bytes, stderr: error_bytes, success: status.success() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;

    fn python(code: &str) -> Command { let mut command = Command::new("python"); command.args(["-c", code]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()); command }

    #[test]
    fn stdin_round_trips_and_exit_status_is_reported() {
        let out = pump(python("import sys; sys.stdout.write(sys.stdin.read().upper())"), Some(b"abc".to_vec()), Duration::from_secs(20), 1024).ok().unwrap();
        assert!(out.success && out.stdout == b"ABC");
    }

    #[test]
    fn a_child_that_never_reads_a_large_request_is_stopped_by_the_time_limit() {
        let started = Instant::now();
        let result = pump(python("import time; time.sleep(60)"), Some(vec![b'x'; 4 * 1024 * 1024]), Duration::from_secs(2), 1024);
        assert!(matches!(result, Err(PumpError::Timeout)));
        assert!(started.elapsed() < Duration::from_secs(15), "the time limit started late");
    }

    #[test]
    fn output_over_the_limit_is_refused() {
        let result = pump(python("print('x' * 5000)"), None, Duration::from_secs(20), 100);
        assert!(matches!(result, Err(PumpError::TooLarge)));
    }
}
