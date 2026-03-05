//! botty — PTY-based Agent Runtime
//!
//! A tmux-style, user-scoped PTY server for running and coordinating
//! interactive agents as terminal programs.

// Error documentation is deferred - the errors are self-explanatory from types
#![allow(clippy::missing_errors_doc)]

pub mod attach;
pub mod cli;
pub mod client;
pub mod output;
pub mod protocol;
pub mod pty;
pub mod runtime;
pub mod server;
pub mod sys;
pub mod telemetry;
pub mod testing;
pub mod view;

/// Check whether `systemd-run --user` is available.
///
/// Result is cached after the first call (process-lifetime).
pub fn has_systemd_run() -> bool {
    use std::sync::OnceLock;
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        std::process::Command::new("systemd-run")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

pub use attach::{run_attach, AttachConfig, AttachError};
pub use cli::{parse_key_notation, parse_key_sequence, Cli, Command};
pub use client::{default_socket_path, Client, ClientError};
pub use output::{json_envelope, resolve_format, text_record, OutputFormat};
pub use protocol::{AgentInfo, AgentState, DumpFormat, Event, ExitReason, RecordedCommand, Request, ResourceLimits, Response};
pub use server::{Server, ServerError};
pub use testing::{AgentHandle, TestError, TestHarness};
pub use view::{TmuxView, ViewError, ViewMode};
