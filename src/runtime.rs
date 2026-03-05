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
    pub use tokio::time::{Duration, Instant, interval, sleep, timeout};
}

#[cfg(feature = "runtime-tokio")]
pub mod signal {
    pub use tokio::signal::unix::{Signal, SignalKind, signal};
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

// ── asupersync backend ───────────────────────────────────────────────────

// TODO(asupersync): The asupersync backend stubs below are scaffolding for
// the runtime swap. They document the exact API mapping needed. Each module
// maps tokio's ambient-authority API to asupersync's capability-based API
// using Cx::current() to bridge the gap.
//
// Remaining work to activate this backend:
//
// 1. Mutex wrapper — Cx::current() + unwrap LockError
// 2. broadcast wrapper — Cx::current() for send/recv
// 3. select! macro — nested Select<A, Select<B, ...>> with Box::pin
// 4. stdin/stdout — spawn_blocking shim over raw fd
// 5. interval — async tick() wrapper using sleep
// 6. spawn — global RuntimeHandle via OnceLock
// 7. Entry point — RuntimeBuilder::new().build()?.block_on(...)
// 8. read_line — standalone fn, not trait method (call-site changes)

#[cfg(feature = "runtime-asupersync")]
pub mod net {
    pub use asupersync::net::{UnixListener, UnixStream};
    pub use asupersync::net::{
        UnixOwnedReadHalf as OwnedReadHalf, UnixOwnedWriteHalf as OwnedWriteHalf,
    };
}

#[cfg(feature = "runtime-asupersync")]
pub mod io {
    pub use asupersync::io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufReader};
    // read_line is a standalone function, not a trait method:
    //   asupersync::io::read_line(reader, buf) -> ReadLine future
    // stdin/stdout: no process-own stdio — use spawn_blocking shim
}

#[cfg(feature = "runtime-asupersync")]
pub mod sync {
    pub use asupersync::sync::Mutex;
    pub use asupersync::channel::broadcast;
}

#[cfg(feature = "runtime-asupersync")]
pub mod time {
    pub use std::time::Duration;
    // sleep: asupersync::time::sleep(wall_now(), duration)
    // timeout: asupersync::time::timeout(wall_now(), duration, future)
    // interval: asupersync::time::interval(wall_now(), period) — tick(now) is SYNC
    // Instant: use std::time::Instant (asupersync uses its own Time type)
}

#[cfg(feature = "runtime-asupersync")]
pub mod signal {
    pub use asupersync::signal::{Signal, SignalKind, signal};
}

#[cfg(feature = "runtime-asupersync")]
pub mod task {
    // spawn: RuntimeHandle::spawn(future) — need global handle
    // spawn_blocking: asupersync::spawn_blocking(f) — standalone, no Cx needed
    // JoinHandle: asupersync::runtime::JoinHandle
}
