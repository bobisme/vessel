//! PTY creation and management.
//!
//! Provides helpers for spawning processes in pseudo-terminals.
//!
//! # Safety
//!
//! This module uses unsafe code for PTY operations (fork, ioctl, dup2).
//! These are fundamental operations that cannot be done safely.

#![allow(unsafe_code)]

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::pty::{openpty, OpenptyResult, Winsize};
use nix::sys::signal::{self, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{execvp, fork, setsid, ForkResult, Pid};
use std::ffi::CString;
use std::os::unix::io::{AsRawFd, OwnedFd, RawFd};
use thiserror::Error;

/// Errors that can occur during PTY operations.
#[derive(Debug, Error)]
pub enum PtyError {
    #[error("failed to open PTY: {0}")]
    OpenPty(#[source] nix::Error),

    #[error("failed to fork: {0}")]
    Fork(#[source] nix::Error),

    #[error("failed to create session: {0}")]
    Setsid(#[source] nix::Error),

    #[error("failed to set controlling terminal: {0}")]
    SetControllingTerminal(#[source] nix::Error),

    #[error("failed to change directory: {0}")]
    Chdir(#[source] std::io::Error),

    #[error("failed to exec: {0}")]
    Exec(#[source] nix::Error),

    #[error("command is empty")]
    EmptyCommand,

    #[error("invalid command string: {0}")]
    InvalidCommand(#[source] std::ffi::NulError),

    #[error("failed to send signal: {0}")]
    Signal(#[source] nix::Error),

    #[error("failed to wait: {0}")]
    Wait(#[source] nix::Error),
}

/// Result of spawning a process in a PTY.
pub struct PtyProcess {
    /// The master side of the PTY.
    pub master: OwnedFd,
    /// The child process ID.
    pub pid: Pid,
    /// Terminal size.
    pub size: Winsize,
}

impl PtyProcess {
    /// Get the raw file descriptor of the master PTY.
    #[must_use]
    pub fn master_fd(&self) -> RawFd {
        self.master.as_raw_fd()
    }

    /// Send a signal to the child process.
    pub fn signal(&self, sig: Signal) -> Result<(), PtyError> {
        signal::kill(self.pid, sig).map_err(PtyError::Signal)
    }

    /// Check if the child process has exited without blocking.
    /// Returns `Some(exit_code)` if exited, None if still running.
    pub fn try_wait(&self) -> Result<Option<i32>, PtyError> {
        match waitpid(self.pid, Some(WaitPidFlag::WNOHANG)).map_err(PtyError::Wait)? {
            WaitStatus::Exited(_, code) => Ok(Some(code)),
            WaitStatus::Signaled(_, sig, _) => Ok(Some(128 + sig as i32)),
            // All other states (StillAlive, Stopped, Continued, etc.) mean not exited yet
            _ => Ok(None),
        }
    }

    /// Wait for the child process to exit (blocking).
    pub fn wait(&self) -> Result<i32, PtyError> {
        match waitpid(self.pid, None).map_err(PtyError::Wait)? {
            WaitStatus::Exited(_, code) => Ok(code),
            WaitStatus::Signaled(_, sig, _) => Ok(128 + sig as i32),
            status => {
                tracing::warn!(?status, "unexpected wait status");
                Ok(-1)
            }
        }
    }

    /// Resize the PTY.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), PtyError> {
        let winsize = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // TIOCSWINSZ ioctl
        unsafe {
            let ret = libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &winsize);
            if ret < 0 {
                return Err(PtyError::SetControllingTerminal(nix::Error::last()));
            }
        }
        Ok(())
    }
}

/// Minimal set of environment variables always provided to spawned agents.
///
/// These are captured from the server's environment at spawn time.
/// Explicit `--env` values override these.
const ESSENTIAL_ENV_VARS: &[&str] = &[
    "PATH",                    // command resolution
    "HOME",                    // home directory
    "USER",                    // current user
    "TERM",                    // terminal type (critical for PTY)
    "SHELL",                   // default shell
    "LANG",                    // locale / character encoding
    "XDG_RUNTIME_DIR",         // systemd, D-Bus, Wayland sockets
    "DBUS_SESSION_BUS_ADDRESS", // systemd-run --user needs session bus
];

/// Environment configuration for spawning.
///
/// The environment is always cleared before setting vars.
/// Essential vars (PATH, HOME, USER, TERM, SHELL, LANG) are set from
/// the server's environment, then explicit vars are applied on top.
#[derive(Debug, Default)]
pub struct SpawnEnv {
    /// Environment variables to set (key, value pairs).
    /// These override essential vars if they share a key.
    pub vars: Vec<(String, String)>,
}

/// Spawn a command in a new PTY.
///
/// # Arguments
///
/// * `cmd` - Command and arguments to execute
/// * `rows` - Terminal height in rows
/// * `cols` - Terminal width in columns
///
/// # Returns
///
/// A `PtyProcess` containing the master FD and child PID.
pub fn spawn(cmd: &[String], rows: u16, cols: u16) -> Result<PtyProcess, PtyError> {
    spawn_with_env(cmd, rows, cols, &SpawnEnv::default(), None)
}

/// Spawn a command in a new PTY with custom environment.
///
/// # Arguments
///
/// * `cmd` - Command and arguments to execute
/// * `rows` - Terminal height in rows
/// * `cols` - Terminal width in columns
/// * `env` - Environment configuration
/// * `cwd` - Optional working directory for the child process
///
/// # Returns
///
/// A `PtyProcess` containing the master FD and child PID.
pub fn spawn_with_env(
    cmd: &[String],
    rows: u16,
    cols: u16,
    env: &SpawnEnv,
    cwd: Option<&str>,
) -> Result<PtyProcess, PtyError> {
    if cmd.is_empty() {
        return Err(PtyError::EmptyCommand);
    }

    let winsize = Winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    // Capture essential env vars from server before forking.
    // Explicit vars override these (collected into a map for dedup).
    let explicit_keys: std::collections::HashSet<&str> =
        env.vars.iter().map(|(k, _)| k.as_str()).collect();
    // Used in child branch after fork; compiler warns because parent branch returns early.
    #[allow(unused_variables)]
    let essential: Vec<(String, String)> = ESSENTIAL_ENV_VARS
        .iter()
        .filter(|k| !explicit_keys.contains(**k))
        .filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v)))
        .collect();

    // Open a new PTY pair
    let OpenptyResult { master, slave } = openpty(&winsize, None).map_err(PtyError::OpenPty)?;

    // Fork the process
    match unsafe { fork() }.map_err(PtyError::Fork)? {
        ForkResult::Parent { child } => {
            // Parent: close slave, keep master
            drop(slave);

            // Set master to non-blocking mode for async I/O
            let flags = fcntl(&master, FcntlArg::F_GETFL).map_err(PtyError::OpenPty)?;
            let mut flags = OFlag::from_bits_retain(flags);
            flags.insert(OFlag::O_NONBLOCK);
            fcntl(&master, FcntlArg::F_SETFL(flags)).map_err(PtyError::OpenPty)?;

            Ok(PtyProcess {
                master,
                pid: child,
                size: winsize,
            })
        }
        ForkResult::Child => {
            // Child: set up the terminal and exec.
            //
            // CRITICAL: After fork(), the child must NEVER return from this
            // function. If any step fails, it must _exit() immediately.
            // Returning would let the child continue executing the parent's
            // code (e.g., test runner logic), causing hangs and zombies.

            // Close master in child
            drop(master);

            // Create a new session
            if setsid().is_err() {
                unsafe { libc::_exit(1) };
            }

            // Set the slave as the controlling terminal
            unsafe {
                if libc::ioctl(slave.as_raw_fd(), libc::TIOCSCTTY, 0) < 0 {
                    libc::_exit(1);
                }
            }

            // Redirect stdin/stdout/stderr to the slave
            let slave_fd = slave.as_raw_fd();
            unsafe {
                if libc::dup2(slave_fd, libc::STDIN_FILENO) < 0
                    || libc::dup2(slave_fd, libc::STDOUT_FILENO) < 0
                    || libc::dup2(slave_fd, libc::STDERR_FILENO) < 0
                {
                    libc::_exit(1);
                }
            }

            // Close the original slave fd if it's not one of 0, 1, 2
            if slave_fd > 2 {
                drop(slave);
            }

            // Set up environment: clear everything, then set essential + explicit vars.
            // SAFETY: We're in a forked child process before exec, so modifying
            // environment is safe (no other threads exist in this process).
            unsafe {
                for (key, _) in std::env::vars() {
                    std::env::remove_var(&key);
                }
                for (key, value) in &essential {
                    std::env::set_var(key, value);
                }
                for (key, value) in &env.vars {
                    std::env::set_var(key, value);
                }
            }

            // Change working directory if requested
            if let Some(dir) = cwd {
                if std::env::set_current_dir(dir).is_err() {
                    unsafe { libc::_exit(1) };
                }
            }

            // Convert command to CStrings — _exit on failure
            let Ok(prog) = CString::new(cmd[0].as_str()) else {
                unsafe { libc::_exit(1) };
            };
            let args: Vec<CString> = match cmd
                .iter()
                .map(|s| CString::new(s.as_str()))
                .collect::<Result<_, _>>()
            {
                Ok(args) => args,
                Err(_) => unsafe { libc::_exit(1) },
            };

            // Exec the command — only returns on error
            let _: Result<std::convert::Infallible, _> = execvp(&prog, &args);
            unsafe { libc::_exit(127) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_spawn_echo() {
        let pty = spawn(&["sh".into(), "-c".into(), "echo hello".into()], 24, 80).unwrap();

        // Wait for child to exit
        let exit_code = pty.wait().unwrap();
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_spawn_exit_code() {
        let pty = spawn(&["sh".into(), "-c".into(), "exit 42".into()], 24, 80).unwrap();
        let exit_code = pty.wait().unwrap();
        assert_eq!(exit_code, 42);
    }

    #[test]
    fn test_spawn_empty_command() {
        let result = spawn(&[], 24, 80);
        assert!(matches!(result, Err(PtyError::EmptyCommand)));
    }

    #[test]
    fn test_try_wait() {
        let pty = spawn(&["sleep".into(), "0.1".into()], 24, 80).unwrap();

        // Should still be running
        let result = pty.try_wait().unwrap();
        assert!(result.is_none());

        // Wait for it to finish
        std::thread::sleep(Duration::from_millis(200));

        // Now it should be done
        let result = pty.try_wait().unwrap();
        assert_eq!(result, Some(0));
    }
}
