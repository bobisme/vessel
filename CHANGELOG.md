# Changelog

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
