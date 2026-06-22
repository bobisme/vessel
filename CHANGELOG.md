# Changelog

## [0.17.5] - 2026-06-22

### Security
- Bound IPC request frames and `SendBytes` payloads to prevent server memory
  exhaustion (CWE-400). The server previously accumulated an unbounded line via
  `read_line` before parsing, and decoded `SendBytes.data` base64 into an
  unbounded `Vec<u8>`, so a same-user or spawned-agent client on the owner-only
  control socket could exhaust memory and deny the shared control plane. The
  server now caps each newline-delimited frame at 1 MiB (rejecting and closing
  the connection before parse/dispatch) and independently rejects oversized
  `SendBytes` payloads before allocating the decode buffer.

## [0.17.4] - 2026-06-19

### Fixed
- Build failure on macOS/BSD: cast `TIOCSCTTY` to the `c_ulong` type that
  `libc::ioctl` expects (it is `c_uint` there), fixing an `E0308` mismatched
  types error in `src/pty.rs`. Linux was unaffected.

### Added
- `send-keys` now accepts the `space` key name (sends a literal space byte,
  `0x20`). Previously there was no way to send a space, since a bare `" "`
  argument is trimmed away during key parsing.

## [0.17.3] - 2026-04-22

### Security
- Bind the Unix control socket under a restrictive umask (`0o177`) so the inode
  is created owner-only atomically. Closes a race window between `bind()` and
  the subsequent `set_permissions(0o600)` call during which a local user on a
  multi-user parent directory (notably the `/tmp/vessel-$UID.sock` fallback)
  could `connect()` and drive the server with unauthenticated `Spawn`
  requests. The existing `set_permissions` call is retained as a
  belt-and-suspenders safeguard.

## [0.17.2] - 2026-04-15

### Added
- `vessel wait --exited --any` to return as soon as any listed agent exits and print which agent IDs had exited when the wait completed

## [0.17.1] - 2026-03-26

### Changed
- Switch `asupersync` dependency from git rev to crates.io v0.2.9, enabling publication to crates.io
- Update `select!` macro patterns for `Select::new().await` returning `Result<Either<A,B>, SelectError>` (asupersync v0.2.9 API change)
- Handle new `broadcast::RecvError::PolledAfterCompletion` variant (asupersync v0.2.9)

## [0.17.0] - 2026-03-05

### Changed
- Rename crate from `botty-pty` to `vessel-pty`
- Default runtime switched to `asupersync`; tokio remains available via `runtime-tokio` feature

### Added
- `asupersync` runtime backend: feature-gated async runtime using asupersync for cancel-correct async I/O
- Runtime abstraction module (`src/runtime.rs`) re-exporting active runtime primitives
- `select!` macro compatible with both tokio and asupersync runtimes

## [0.16.1] - 2026-02-18

### Fixed
- `wait --exited` now supports multiple agent IDs
- Stable screen detection for `wait --stable`

## [0.16.0] - 2026-02-10

### Added
- `vessel tail` command for streaming agent output
- `vessel events` and `vessel subscribe` for event streaming
- PTY reader background task for real-time transcript and screen updates

## [0.13.2] - 2026-01-28

### Fixed
- Server shutdown respects running agents (SIGTERM/SIGINT ignored when agents are active)
- `view` pane identity uses agent ID instead of pane title
