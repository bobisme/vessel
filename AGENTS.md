# botty

Project type: cli
Tools: `beads`, `maw`, `crit`, `botbus`, `botty`
Reviewer roles: security

<!-- Add project-specific context below: architecture, conventions, key files, etc. -->


## Daily Development Workflow

### Starting a Work Session

1. **Check for new work** and triage if needed:
   ```bash
   br ready                    # See what's actionable
   botbus history botty        # Check for messages from other agents
   jj git fetch                # Fetch latest from remote
   ```

2. **Triage new issues** (if any were filed):
   - Read the actual code to assess feasibility
   - Check for existing infrastructure you can leverage
   - Estimate complexity and update priority if needed
   - Add implementation notes to the bead description
   ```bash
   br show <issue-id>
   br update <issue-id> --priority=2 --description="Updated with implementation notes"
   ```

3. **Pick work** based on priority and scope:
   - Prefer P2 over P3
   - Consider batching related features for a release
   - Bugs before features (users are affected now)

4. **Start working** (jj tracks changes automatically):
   ```bash
   jj describe -m "wip: working on <feature>"
   ```

### Feature Development Loop

For each feature, follow this cycle:

1. **Start the work**:
   ```bash
   br update <issue-id> --status=in_progress
   ```

2. **Implement the feature**:
   - Read existing code to understand patterns
   - Make minimal, focused changes
   - Avoid over-engineering or premature abstraction
   - Follow existing conventions (file structure, naming, error handling)

3. **Test thoroughly**:
   ```bash
   just test                   # Unit + integration tests
   cargo test <test-name>      # Specific test
   just build                  # Verify it builds
   ```
   - Add unit tests for new functions
   - Add integration/CLI tests for new commands
   - Do manual testing for UX features
   - Verify all tests pass before committing

4. **Describe your changes** with semantic message:
   ```bash
   jj describe -m "feat(scope): description

   Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
   ```

5. **Close the bead**:
   ```bash
   br close <issue-id> --reason="Implemented in commit <sha>. [brief summary]"
   ```

6. **Continue to next feature** or prepare for release
   - jj automatically creates a new working copy
   - Each feature gets its own commit

### Batch Releases

Instead of releasing after each feature, batch multiple features into a release:

1. Work on 2-4 related features
2. Test everything together
3. Bump version, tag, and release as one unit
4. **Only then** announce on #botty

This creates coherent releases with clear themes (e.g., "testing improvements").

### Bug Investigation Workflow

1. **Understand the symptom**: Read the bug report carefully
2. **Find the code**: Use `grep`, `rg`, or `ast-grep` to locate relevant code
3. **Reproduce locally**: Try to trigger the bug yourself
4. **Identify root cause**: Read the code, trace the execution path
5. **Design minimal fix**: Target the root cause, avoid over-engineering
6. **Test the fix**: Verify it solves the problem without breaking anything
7. **Consider edge cases**: What else might be affected?

### End of Session Checklist

Before ending a work session:

```bash
jj status                    # Check working copy state
jj diff                      # Review changes
br sync --flush-only         # Export beads to JSONL

# Describe the beads update commit
jj describe -m "chore(beads): update issue tracking

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# Push to main (if working directly on main)
jj git push --bookmark main
```

**Note**: If you've been making multiple commits, you may want to squash them before pushing. See the Release Workflow section for details on preparing changes for review.

## Release Workflow

This section covers the full release cycle: creating a feature branch, implementing changes, getting review, and releasing.

### 1. Start a Feature Branch

```bash
# Create a new commit for your work
jj new -m "wip: description of change"

# Create a bookmark for the feature
jj bookmark create feature-name

# Work on your changes...
jj describe -m "feat(scope): description of change"
```

### 2. Request Code Review

After completing your changes and ensuring tests pass:

```bash
# Verify build and tests
just build && just test

# Create a review
crit reviews create --title "feat(scope): description of change"
# Note the review ID (e.g., cr-xxxx)
```

**Spawn specialist reviewers** using the code-review skill (`~/.claude/skills/code-review/SKILL.md`):

- **Security reviewer** (always): Looks for injection, auth issues, resource exhaustion, etc.
- **Architecture reviewer** (for structural changes): Evaluates design, abstractions, maintainability

The skill has ready-to-use prompts for spawning these subagents.

### 3. Address Review Feedback

Monitor botbus for reviewer completion:

```bash
botbus history general
```

For each thread raised:

```bash
# View threads
crit threads list <review_id>
crit threads show <thread_id>

# Respond (set your agent identity first)
export BOTBUS_AGENT=<your-agent>
crit comments add <thread_id> "Response explaining fix or rationale"

# After addressing, resolve with reason
crit threads resolve <thread_id> --reason "Fixed: description"
crit threads resolve <thread_id> --reason "Won't fix: rationale"
crit threads resolve <thread_id> --reason "Deferred: created bead bd-xxx"
```

### 4. Get Approval

Reviewers vote with:

```bash
crit lgtm <review_id> -m "Reason"    # Approve
crit block <review_id> -r "Reason"   # Block
```

### 5. Merge and Release

Once approved (LGTM votes, no blocking votes, all threads resolved):

```bash
# Approve and merge the review
crit reviews approve <review_id>
crit reviews merge <review_id>

# Bump version in Cargo.toml (edit manually or with sed)
# e.g., 0.2.0 → 0.3.0

# Update commit message
jj describe -m "chore: bump version to X.Y.Z

Co-Authored-By: Claude <noreply@anthropic.com>"

# Move main bookmark forward and push
jj bookmark set main -r @
jj git push --bookmark main

# Tag the release and push tag
jj tag set vX.Y.Z -r main
git push origin vX.Y.Z

# Install locally
just install

# Verify
botty --version

# Announce on botbus
export BOTBUS_AGENT=<your-agent>
botbus send botty "Released vX.Y.Z - [summary of changes]"
```

### Quick Reference

| Stage | Key Commands |
|-------|--------------|
| Start feature | `jj new -m "wip: ..."` then `jj bookmark create name` |
| Create review | `crit reviews create --title "..."` |
| View threads | `crit threads list <review_id>` |
| Respond | `crit comments add <thread_id> "..."` |
| Resolve | `crit threads resolve <thread_id> --reason "..."` |
| Approve/merge | `crit reviews approve <id> && crit reviews merge <id>` |
| Release | bump version → `jj bookmark set main` → push → tag → `just install` |
<!-- botbox:managed-start -->
## Botbox Workflow

**New here?** Read [worker-loop.md](.agents/botbox/worker-loop.md) first — it covers the complete triage → start → work → finish cycle.

**All tools have `--help`** with usage examples. When unsure, run `<tool> --help` or `<tool> <command> --help`.

### Directory Structure (maw v2)

This project uses a **bare repo** layout. Source files live in workspaces under `ws/`, not at the project root.

```
project-root/          ← bare repo (no source files here)
├── ws/
│   ├── default/       ← main working copy (AGENTS.md, .beads/, src/, etc.)
│   ├── frost-castle/  ← agent workspace (isolated jj commit)
│   └── amber-reef/    ← another agent workspace
├── .jj/               ← jj repo data
├── .git/              ← git data (core.bare=true)
├── AGENTS.md          ← stub redirecting to ws/default/AGENTS.md
└── CLAUDE.md          ← symlink → AGENTS.md
```

**Key rules:**
- `ws/default/` is the main workspace — beads, config, and project files live here
- Agent workspaces (`ws/<name>/`) are isolated jj commits for concurrent work
- Use `maw exec <ws> -- <command>` to run commands in a workspace context
- Use `maw exec default -- br|bv ...` for beads commands (always in default workspace)
- Use `maw exec <ws> -- crit ...` for review commands (always in the review's workspace)
- Never run `br`, `bv`, `crit`, or `jj` directly — always go through `maw exec`

### Beads Quick Reference

| Operation | Command |
|-----------|---------|
| View ready work | `maw exec default -- br ready` |
| Show bead | `maw exec default -- br show <id>` |
| Create | `maw exec default -- br create --actor $AGENT --owner $AGENT --title="..." --type=task --priority=2` |
| Start work | `maw exec default -- br update --actor $AGENT <id> --status=in_progress --owner=$AGENT` |
| Add comment | `maw exec default -- br comments add --actor $AGENT --author $AGENT <id> "message"` |
| Close | `maw exec default -- br close --actor $AGENT <id>` |
| Add dependency | `maw exec default -- br dep add --actor $AGENT <blocked> <blocker>` |
| Sync | `maw exec default -- br sync --flush-only` |
| Triage (scores) | `maw exec default -- bv --robot-triage` |
| Next bead | `maw exec default -- bv --robot-next` |

**Required flags**: `--actor $AGENT` on mutations, `--author $AGENT` on comments.

### Workspace Quick Reference

| Operation | Command |
|-----------|---------|
| Create workspace | `maw ws create <name>` |
| List workspaces | `maw ws list` |
| Merge to main | `maw ws merge <name> --destroy` |
| Destroy (no merge) | `maw ws destroy <name>` |
| Run jj in workspace | `maw exec <name> -- jj <jj-args...>` |

**Avoiding divergent commits**: Each workspace owns ONE commit. Only modify your own.

| Safe | Dangerous |
|------|-----------|
| `jj describe` (your working copy) | `jj describe main -m "..."` |
| `maw exec <your-ws> -- jj describe -m "..."` | `jj describe <other-change-id>` |

If you see `(divergent)` in `jj log`:
```bash
jj abandon <change-id>/0   # keep one, abandon the divergent copy
```

### Beads Conventions

- Create a bead before starting work. Update status: `open` → `in_progress` → `closed`.
- Post progress comments during work for crash recovery.
- **Push to main** after completing beads (see [finish.md](.agents/botbox/finish.md)).

### Identity

Your agent name is set by the hook or script that launched you. Use `$AGENT` in commands.
For manual sessions, use `<project>-dev` (e.g., `myapp-dev`).

### Claims

When working on a bead, stake claims to prevent conflicts:

```bash
bus claims stake --agent $AGENT "bead://<project>/<id>" -m "<id>"
bus claims stake --agent $AGENT "workspace://<project>/<ws>" -m "<id>"
bus claims release --agent $AGENT --all  # when done
```

### Reviews

Use `@<project>-<role>` mentions to request reviews:

```bash
maw exec $WS -- crit reviews request <review-id> --reviewers $PROJECT-security --agent $AGENT
bus send --agent $AGENT $PROJECT "Review requested: <review-id> @$PROJECT-security" -L review-request
```

The @mention triggers the auto-spawn hook for the reviewer.

### Cross-Project Communication

**Don't suffer in silence.** If a tool confuses you or behaves unexpectedly, post to its project channel.

1. Find the project: `bus history projects -n 50` (the #projects channel has project registry entries)
2. Post question or feedback: `bus send --agent $AGENT <project> "..." -L feedback`
3. For bugs, create beads in their repo first
4. **Always create a local tracking bead** so you check back later:
   ```bash
   maw exec default -- br create --actor $AGENT --owner $AGENT --title="[tracking] <summary>" --labels tracking --type=task --priority=3
   ```

See [cross-channel.md](.agents/botbox/cross-channel.md) for the full workflow.

### Session Search (optional)

Use `cass search "error or problem"` to find how similar issues were solved in past sessions.


### Design Guidelines

- [CLI tool design for humans, agents, and machines](.agents/botbox/design/cli-conventions.md)

### Workflow Docs

- [Ask questions, report bugs, and track responses across projects](.agents/botbox/cross-channel.md)
- [Close bead, merge workspace, release claims, sync](.agents/botbox/finish.md)
- [groom](.agents/botbox/groom.md)
- [Verify approval before merge](.agents/botbox/merge-check.md)
- [Turn specs/PRDs into actionable beads](.agents/botbox/planning.md)
- [Validate toolchain health](.agents/botbox/preflight.md)
- [Create and validate proposals before implementation](.agents/botbox/proposal.md)
- [Report bugs/features to other projects](.agents/botbox/report-issue.md)
- [Reviewer agent loop](.agents/botbox/review-loop.md)
- [Request a review](.agents/botbox/review-request.md)
- [Handle reviewer feedback (fix/address/defer)](.agents/botbox/review-response.md)
- [Explore unfamiliar code before planning](.agents/botbox/scout.md)
- [Claim bead, create workspace, announce](.agents/botbox/start.md)
- [Find work from inbox and beads](.agents/botbox/triage.md)
- [Change bead status (open/in_progress/blocked/done)](.agents/botbox/update.md)
- [Full triage-work-finish lifecycle](.agents/botbox/worker-loop.md)
<!-- botbox:managed-end -->
