//! Runtime abstraction layer.
//!
//! Re-exports async primitives from the active runtime (tokio or asupersync)
//! behind a unified interface. During migration, both runtimes can coexist
//! via feature flags; once migration is complete, the tokio path is removed.

// Exactly one runtime must be enabled.
#[cfg(all(feature = "runtime-tokio", feature = "runtime-asupersync"))]
compile_error!("features `runtime-tokio` and `runtime-asupersync` are mutually exclusive");

#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-asupersync")))]
compile_error!("exactly one of `runtime-tokio` or `runtime-asupersync` must be enabled");

// ── tokio backend ────────────────────────────────────────────────────────

#[cfg(feature = "runtime-tokio")]
pub mod net {
    pub use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
    pub use tokio::net::{UnixListener, UnixStream};
}

#[cfg(feature = "runtime-tokio")]
pub mod io {
    pub use tokio::io::{
        AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, stdin, stdout,
    };
}

#[cfg(feature = "runtime-tokio")]
pub mod sync {
    pub use tokio::sync::{Mutex, broadcast};
}

#[cfg(feature = "runtime-tokio")]
pub mod time {
    pub use tokio::time::{Duration, interval, sleep, timeout};
}

#[cfg(feature = "runtime-tokio")]
pub mod signal {
    pub use tokio::signal::unix::{SignalKind, signal};
}

#[cfg(feature = "runtime-tokio")]
pub mod task {
    pub use tokio::spawn;
    pub use tokio::task::{JoinHandle, spawn_blocking};
}

/// Re-export the select macro.
#[cfg(feature = "runtime-tokio")]
macro_rules! select {
    ($($tt:tt)*) => { tokio::select! { $($tt)* } };
}

#[cfg(feature = "runtime-tokio")]
pub(crate) use select;

// ── asupersync backend (stubs — filled in during migration) ──────────────

#[cfg(feature = "runtime-asupersync")]
pub mod net {
    pub use asupersync::net::{UnixListener, UnixStream};
    pub use asupersync::net::{UnixOwnedReadHalf as OwnedReadHalf, UnixOwnedWriteHalf as OwnedWriteHalf};
}

#[cfg(feature = "runtime-asupersync")]
pub mod io {
    pub use asupersync::io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufReader};
    // TODO: stdin, stdout — ChildStdin/ChildStdout exist but may not cover raw terminal I/O
}

#[cfg(feature = "runtime-asupersync")]
pub mod sync {
    pub use asupersync::sync::Mutex;
    pub use asupersync::channel::broadcast;
}

#[cfg(feature = "runtime-asupersync")]
pub mod time {
    pub use std::time::Duration;
    // TODO: sleep, timeout, interval wrappers around asupersync::time
    //       These need wall_now() prepended and Cx for some operations.
}

#[cfg(feature = "runtime-asupersync")]
pub mod signal {
    pub use asupersync::signal::{sigterm, sigwinch};
    // TODO: map SignalKind-based API if needed
}

#[cfg(feature = "runtime-asupersync")]
pub mod task {
    // TODO: spawn (needs RuntimeHandle), spawn_blocking, JoinHandle
}
