//! Opt-in single-process evaluation. The controller owns the peer socket;
//! losing it terminates this process even when the controller is SIGKILLed.
//! macOS additionally forbids process creation, including detached children.
#[cfg(unix)]
use std::io::{Read, Write};

#[cfg(unix)]
pub(crate) const READY: &[u8] = b"reason-evaluation-lifeline-v1\n";

#[cfg(unix)]
pub(crate) fn install(fd: i32, no_fork: bool) -> Result<(), String> {
    use std::os::{fd::FromRawFd, unix::net::UnixStream};
    if fd < 3 {
        return Err("evaluation lifeline requires an inherited socket".into());
    }
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) };
    if duplicate < 0 {
        return Err("evaluation lifeline socket is unavailable".into());
    }
    let mut stream = unsafe { UnixStream::from_raw_fd(duplicate) };
    if !stream.peer_addr().is_ok_and(|a| a.is_unnamed())
        || !stream.local_addr().is_ok_and(|a| a.is_unnamed())
        || unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0
    {
        return Err("evaluation lifeline requires a private socket pair".into());
    }
    if no_fork {
        forbid_process_creation()?;
    }
    // The controller persists the launch identity before acknowledging GO.
    // An already-dead controller can never release provider work.
    stream
        .write_all(READY)
        .map_err(|_| "evaluation lifeline handshake failed")?;
    let mut go = [0];
    stream
        .read_exact(&mut go)
        .map_err(|_| "evaluation controller disappeared before launch")?;
    if go != [b'G'] {
        return Err("invalid evaluation controller acknowledgment".into());
    }
    std::thread::Builder::new()
        .name("evaluation-lifeline".into())
        .spawn(move || {
            let mut byte = [0];
            loop {
                match stream.read(&mut byte) {
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    // EOF, unexpected input and I/O failure all revoke execution.
                    _ => std::process::exit(125),
                }
            }
        })
        .map_err(|_| "cannot start evaluation lifeline monitor")?;
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn install(_fd: i32, _no_fork: bool) -> Result<(), String> {
    Err("evaluation lifeline requires a Unix runtime".into())
}

#[cfg(target_os = "macos")]
fn forbid_process_creation() -> Result<(), String> {
    use std::ffi::CString;
    unsafe extern "C" {
        fn sandbox_init(
            profile: *const libc::c_char,
            flags: u64,
            error: *mut *mut libc::c_char,
        ) -> i32;
        fn sandbox_free_error(error: *mut libc::c_char);
    }
    let profile = CString::new("(version 1)(allow default)(deny process-fork)").unwrap();
    let mut error = std::ptr::null_mut();
    let result = unsafe { sandbox_init(profile.as_ptr(), 0, &mut error) };
    if !error.is_null() {
        unsafe { sandbox_free_error(error) };
    }
    if result == 0 {
        Ok(())
    } else {
        Err("macOS evaluation process-creation restriction unavailable".into())
    }
}

#[cfg(not(target_os = "macos"))]
fn forbid_process_creation() -> Result<(), String> {
    Err("no-descendant evaluation requires the macOS process restriction".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_lifeline_is_rejected_without_starting_monitor() {
        assert!(install(-1, false).is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_process_restriction_blocks_fork_in_an_isolated_test_process() {
        const CHILD: &str = "REASON_TEST_NO_FORK_CHILD";
        if std::env::var_os(CHILD).is_some() {
            forbid_process_creation().unwrap();
            let pid = unsafe { libc::fork() };
            if pid == 0 {
                unsafe { libc::_exit(99) };
            }
            if pid > 0 {
                unsafe {
                    libc::kill(pid, libc::SIGKILL);
                    libc::waitpid(pid, std::ptr::null_mut(), 0);
                }
            }
            assert_eq!(pid, -1, "fork must be denied before a descendant exists");
            assert!(
                std::process::Command::new("/usr/bin/true").spawn().is_err(),
                "posix_spawn must also be denied"
            );
            return;
        }
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "evaluation_supervision::tests::macos_process_restriction_blocks_fork_in_an_isolated_test_process"])
            .env(CHILD, "1").status().unwrap();
        assert!(status.success());
    }
}
