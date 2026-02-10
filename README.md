# botty

`botty` is a PTY-based runtime for spawning, controlling, and observing interactive terminal agents over a Unix socket.

It is designed for AI orchestrators, test harnesses, and automation systems that need real terminal semantics (not just stdout pipes).

## What botty is (and is not)

- **Is:** local control plane for interactive worker processes (`spawn`, `send`, `wait`, `snapshot`, `events`, `attach`, `view`).
- **Is:** good for multi-agent workflows, TUI testing, and reproducible terminal automation.
- **Is not:** container runtime, distributed scheduler, or durable job queue.

## Requirements

- Linux with Unix sockets + PTY support
- Rust 1.85+ (for building from source)
- `tmux` (optional, only for `botty view`)

## Install

From this repository:

```bash
cargo install --locked --path .
```

From git tag:

```bash
cargo install --locked --git https://github.com/bobisme/botty --tag v0.12.1
```

## Quick start (2 minutes)

```bash
# 1) Spawn a worker shell
botty spawn --name demo -- bash

# 2) Send a command (+ Enter)
botty send demo "echo hello from botty" -n

# 3) Wait for expected output, then inspect the virtual screen
botty wait demo --contains "hello from botty" --timeout 5
botty snapshot demo

# 4) Clean up (SIGTERM by default; use --force for hard kill)
botty kill demo --force

# 5) Stop server when done
botty shutdown
```

## Mental model

```text
Agent  = PTY process + transcript ring + virtual screen
Server = owns all agent state, listens on Unix socket
Client = stateless CLI sending JSON requests
View   = tmux dashboard; panes run read-only attach streams
```

Key implications:

- `snapshot` reflects current terminal state (best for assertions).
- `tail`/`dump` reflect transcript bytes (useful for logs/streaming).
- State is in-memory in the server process (no persistence across server restart).

## Core command map

### Lifecycle

```bash
botty spawn --name worker --label batch --timeout 60 -- bash
botty list
botty list --all --format json
botty kill worker
botty kill --label batch --force
botty kill --all --force
botty signal worker --signal USR1
```

### Input/output

```bash
botty send worker "make test" -n
botty send-bytes worker 1b5b41           # up arrow
botty send-keys worker ctrl-c enter
botty tail worker -f
botty tail worker --raw
botty snapshot worker
botty snapshot worker --raw
botty dump worker --format jsonl
```

### Synchronization and assertions

```bash
botty wait worker --contains "READY" --timeout 30
botty wait worker --stable 200 --contains "$ "
botty wait worker --exited
botty assert worker --contains "PASS"
botty assert worker --not-contains "ERROR"
```

### Streaming and observability

```bash
botty events --output
botty subscribe --id worker --prefix
botty subscribe --label batch --format jsonl
botty attach worker
botty attach worker --readonly
botty view
botty view --mode windows
botty view --label batch
```

### One-off command execution

```bash
botty exec -- git status --short
botty exec --timeout 120 -- cargo test
```

### Recording and replay scaffolding

```bash
botty spawn --name rec --record -- bash
botty send rec "echo hi" -n
botty recording rec --format pretty
botty gen-test rec > replay.sh
chmod +x replay.sh
```

## Orchestration patterns

Spawn dependencies:

```bash
# Wait for setup to exit before starting app
botty spawn --name setup -- ./setup.sh
botty spawn --name app --after setup -- ./run-app.sh

# Wait for output from another agent before spawning
botty spawn --name db -- ./start-db.sh
botty spawn --name api --wait-for db:READY -- ./start-api.sh
```

Recommended cleanup for automation:

```bash
botty kill --label batch --force
```

## Output formats for automation

Many commands support `--format text|json|pretty`.

- `text`: compact, pipe-friendly
- `json`: structured envelope (`{"<key>": ..., "advice": [...]}`)
- `pretty`: human-oriented terminal output

Example:

```bash
botty list --format json | jq '.agents[] | {id, state, labels}'
```

## Server behavior

- Server auto-starts for most regular commands.
- `events` and `subscribe` do **not** auto-start (they expect an existing server/session).
- Default socket path: `/run/user/$UID/botty.sock` (fallback `/tmp/botty-$UID.sock`).
- Override with `BOTTY_SOCKET` or `--socket`.

## Troubleshooting

```bash
botty doctor
```

If you hit stale socket/session issues:

```bash
botty shutdown
tmux kill-session -t botty 2>/dev/null || true
```

Notes:

- `kill` sends SIGTERM by default; some interactive shells ignore it. Use `--force` for deterministic teardown.
- For TUI inspection, prefer `snapshot` or `attach --readonly` over plain `tail`.

## Development

```bash
just build
just test
```

Relevant docs:

- `AGENTS.md` - contributor + agent workflow
- `docs/testing.md` - testing approach and scenarios
