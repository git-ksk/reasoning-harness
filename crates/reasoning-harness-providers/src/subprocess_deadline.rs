use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use reasoning_harness_core::ResolutionAdapterErrorKind;

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(5);

fn spawn_error_kind(error: &std::io::Error) -> ResolutionAdapterErrorKind {
    match error.kind() {
        std::io::ErrorKind::NotFound => ResolutionAdapterErrorKind::Unavailable,
        std::io::ErrorKind::PermissionDenied => ResolutionAdapterErrorKind::PermissionDenied,
        _ => ResolutionAdapterErrorKind::Transport,
    }
}

fn remaining(started: Instant, timeout: Duration) -> Option<Duration> {
    timeout.checked_sub(started.elapsed())
}

fn poll_delay(started: Instant, timeout: Duration) -> Result<Duration, ResolutionAdapterErrorKind> {
    remaining(started, timeout)
        .filter(|remaining| !remaining.is_zero())
        .map(|remaining| remaining.min(PROCESS_POLL_INTERVAL))
        .ok_or(ResolutionAdapterErrorKind::Timeout)
}

fn terminate_without_waiting(mut child: Child) {
    let _ = child.kill();
    // Reaping is best-effort and deliberately detached. A descendant may retain inherited pipes;
    // the invoking thread must never join cleanup work after its Harness-owned deadline expires.
    let _ = thread::Builder::new()
        .name("reason-subprocess-reaper".into())
        .spawn(move || {
            let _ = child.wait();
        });
}

fn spawn_writer(mut stdin: ChildStdin, payload: Vec<u8>) -> Receiver<std::io::Result<()>> {
    let (tx, rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let result = stdin.write_all(&payload);
        drop(stdin);
        let _ = tx.send(result);
    });
    rx
}

fn spawn_eof_reader(
    mut stdout: ChildStdout,
    response_limit: usize,
) -> Receiver<std::io::Result<Vec<u8>>> {
    let (tx, rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        // Keep draining stdout even after the retained response reaches its bounded limit.
        // Stopping reads at the limit could fill the pipe and make an otherwise finite child
        // block until the deadline, incorrectly classifying an oversized response as Timeout.
        let mut retained = Vec::with_capacity(response_limit.min(64 * 1024));
        let mut buffer = [0_u8; 8 * 1024];
        let result = loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break Ok(retained),
                Ok(read) => {
                    if retained.len() < response_limit {
                        let keep = (response_limit - retained.len()).min(read);
                        retained.extend_from_slice(&buffer[..keep]);
                    }
                }
                Err(error) => break Err(error),
            }
        };
        let _ = tx.send(result);
    });
    rx
}

#[derive(Debug)]
enum LineReadError {
    Io,
    Eof,
    TooLarge,
}

fn read_one_bounded_line(stdout: ChildStdout, max_bytes: usize) -> Result<Vec<u8>, LineReadError> {
    let mut reader = BufReader::new(stdout);
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|_| LineReadError::Io)?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Err(LineReadError::Eof)
            } else {
                Ok(bytes)
            };
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |index| index + 1);
        if bytes.len().saturating_add(take) > max_bytes {
            return Err(LineReadError::TooLarge);
        }
        bytes.extend_from_slice(&available[..take]);
        reader.consume(take);
        if newline.is_some() {
            while bytes
                .last()
                .is_some_and(|byte| *byte == b'\n' || *byte == b'\r')
            {
                bytes.pop();
            }
            return Ok(bytes);
        }
    }
}

fn spawn_line_reader(
    stdout: ChildStdout,
    max_response_bytes: usize,
) -> Receiver<Result<Vec<u8>, LineReadError>> {
    let (tx, rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = tx.send(read_one_bounded_line(stdout, max_response_bytes));
    });
    rx
}

fn observe_writer(
    receiver: &Receiver<std::io::Result<()>>,
    completed: &mut bool,
) -> Result<(), ResolutionAdapterErrorKind> {
    if *completed {
        return Ok(());
    }
    match receiver.try_recv() {
        Ok(Ok(())) => {
            *completed = true;
            Ok(())
        }
        Ok(Err(_)) | Err(TryRecvError::Disconnected) => Err(ResolutionAdapterErrorKind::Transport),
        Err(TryRecvError::Empty) => Ok(()),
    }
}

fn wait_for_writer(
    receiver: &Receiver<std::io::Result<()>>,
    started: Instant,
    timeout: Duration,
) -> Result<(), ResolutionAdapterErrorKind> {
    let remaining = remaining(started, timeout).ok_or(ResolutionAdapterErrorKind::Timeout)?;
    match receiver.recv_timeout(remaining) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(ResolutionAdapterErrorKind::Transport)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => Err(ResolutionAdapterErrorKind::Timeout),
    }
}

pub(crate) fn run_to_exit(
    command: &mut Command,
    payload: Vec<u8>,
    started: Instant,
    timeout: Duration,
    max_response_bytes: usize,
) -> Result<Vec<u8>, ResolutionAdapterErrorKind> {
    let response_limit = max_response_bytes
        .checked_add(1)
        .ok_or(ResolutionAdapterErrorKind::PolicyDenied)?;
    if remaining(started, timeout).is_none() {
        return Err(ResolutionAdapterErrorKind::Timeout);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| spawn_error_kind(&error))?;
    let stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            terminate_without_waiting(child);
            return Err(ResolutionAdapterErrorKind::Transport);
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_without_waiting(child);
            return Err(ResolutionAdapterErrorKind::Transport);
        }
    };
    let writer = spawn_writer(stdin, payload);
    let reader = spawn_eof_reader(stdout, response_limit);
    let mut writer_completed = false;

    let status = loop {
        if let Err(kind) = observe_writer(&writer, &mut writer_completed) {
            terminate_without_waiting(child);
            return Err(kind);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(_) => {
                terminate_without_waiting(child);
                return Err(ResolutionAdapterErrorKind::Transport);
            }
        }
        let delay = match poll_delay(started, timeout) {
            Ok(delay) => delay,
            Err(kind) => {
                terminate_without_waiting(child);
                return Err(kind);
            }
        };
        thread::sleep(delay);
    };

    if !status.success() {
        return Err(ResolutionAdapterErrorKind::Protocol);
    }
    if !writer_completed {
        wait_for_writer(&writer, started, timeout)?;
    }
    let remaining = remaining(started, timeout).ok_or(ResolutionAdapterErrorKind::Timeout)?;
    let output = match reader.recv_timeout(remaining) {
        Ok(Ok(output)) => output,
        Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err(ResolutionAdapterErrorKind::Transport);
        }
        Err(mpsc::RecvTimeoutError::Timeout) => return Err(ResolutionAdapterErrorKind::Timeout),
    };
    if output.len() > max_response_bytes {
        return Err(ResolutionAdapterErrorKind::Protocol);
    }
    Ok(output)
}

pub(crate) fn run_until_line(
    command: &mut Command,
    payload: Vec<u8>,
    started: Instant,
    timeout: Duration,
    max_response_bytes: usize,
) -> Result<Vec<u8>, ResolutionAdapterErrorKind> {
    if max_response_bytes == 0 {
        return Err(ResolutionAdapterErrorKind::PolicyDenied);
    }
    if remaining(started, timeout).is_none() {
        return Err(ResolutionAdapterErrorKind::Timeout);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| spawn_error_kind(&error))?;
    let stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            terminate_without_waiting(child);
            return Err(ResolutionAdapterErrorKind::Transport);
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_without_waiting(child);
            return Err(ResolutionAdapterErrorKind::Transport);
        }
    };
    let writer = spawn_writer(stdin, payload);
    let reader = spawn_line_reader(stdout, max_response_bytes);
    let mut writer_completed = false;
    let mut line = None;

    loop {
        if let Err(kind) = observe_writer(&writer, &mut writer_completed) {
            terminate_without_waiting(child);
            return Err(kind);
        }
        if line.is_none() {
            match reader.try_recv() {
                Ok(Ok(bytes)) => line = Some(bytes),
                Ok(Err(LineReadError::TooLarge)) => {
                    terminate_without_waiting(child);
                    return Err(ResolutionAdapterErrorKind::Protocol);
                }
                Ok(Err(LineReadError::Io | LineReadError::Eof))
                | Err(TryRecvError::Disconnected) => {
                    terminate_without_waiting(child);
                    return Err(ResolutionAdapterErrorKind::Transport);
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if writer_completed && let Some(line) = line.take() {
            terminate_without_waiting(child);
            return Ok(line);
        }
        match child.try_wait() {
            Ok(_) => {}
            Err(_) => {
                terminate_without_waiting(child);
                return Err(ResolutionAdapterErrorKind::Transport);
            }
        }
        let delay = match poll_delay(started, timeout) {
            Ok(delay) => delay,
            Err(kind) => {
                terminate_without_waiting(child);
                return Err(kind);
            }
        };
        thread::sleep(delay);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

    use super::*;

    fn script(body: &str, name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "reason-subprocess-deadline-{}-{name}.sh",
            std::process::id()
        ));
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[test]
    fn large_write_to_non_reader_is_bounded_by_absolute_deadline() {
        let path = script("sleep 1", "blocked-write");
        let started = Instant::now();
        let mut command = Command::new(&path);
        let result = run_to_exit(
            &mut command,
            vec![b'x'; 2 * 1024 * 1024],
            started,
            Duration::from_millis(40),
            1024,
        );
        fs::remove_file(path).ok();
        assert_eq!(result.unwrap_err(), ResolutionAdapterErrorKind::Timeout);
        assert!(started.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn inherited_stdout_descendant_does_not_extend_deadline_cleanup() {
        let path = script(
            "cat >/dev/null
sleep 1 &
exit 0",
            "inherited-stdout",
        );
        let started = Instant::now();
        let mut command = Command::new(&path);
        let result = run_to_exit(
            &mut command,
            b"request".to_vec(),
            started,
            Duration::from_millis(40),
            1024,
        );
        fs::remove_file(path).ok();
        assert_eq!(result.unwrap_err(), ResolutionAdapterErrorKind::Timeout);
        assert!(started.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn oversized_stdout_is_drained_and_classified_as_protocol() {
        let path = script(
            "cat >/dev/null
head -c 131072 /dev/zero",
            "oversized-stdout",
        );
        let started = Instant::now();
        let mut command = Command::new(&path);
        let result = run_to_exit(
            &mut command,
            b"request".to_vec(),
            started,
            Duration::from_secs(10),
            1024,
        );
        fs::remove_file(path).ok();
        // This test isolates response-size classification rather than scheduler performance.
        // Receiving Protocol proves the reader drained the finite oversized stream before the
        // deliberately generous deadline.
        assert_eq!(result.unwrap_err(), ResolutionAdapterErrorKind::Protocol);
    }
}
