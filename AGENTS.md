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

This project uses the botbox multi-agent workflow.

### Identity

Every command that touches bus or crit requires `--agent <name>`.
Use `<project>-dev` as your name (e.g., `terseid-dev`). Agents spawned by `agent-loop.sh` receive a random name automatically.
Run `bus whoami --agent $AGENT` to confirm your identity.

### Lifecycle

**New to the workflow?** Start with [worker-loop.md](.agents/botbox/worker-loop.md) — it covers the complete triage → start → work → finish cycle.

Individual workflow docs:

- [Close bead, merge workspace, release claims, sync](.agents/botbox/finish.md)
- [groom](.agents/botbox/groom.md)
- [Verify approval before merge](.agents/botbox/merge-check.md)
- [Validate toolchain health](.agents/botbox/preflight.md)
- [Report bugs/features to other projects](.agents/botbox/report-issue.md)
- [Reviewer agent loop](.agents/botbox/review-loop.md)
- [Request a review](.agents/botbox/review-request.md)
- [Handle reviewer feedback (fix/address/defer)](.agents/botbox/review-response.md)
- [Claim bead, create workspace, announce](.agents/botbox/start.md)
- [Find work from inbox and beads](.agents/botbox/triage.md)
- [Change bead status (open/in_progress/blocked/done)](.agents/botbox/update.md)
- [Full triage-work-finish lifecycle](.agents/botbox/worker-loop.md)

### Quick Start

```bash
AGENT=<project>-dev   # or: AGENT=$(bus generate-name)
bus whoami --agent $AGENT
br ready
```

### Beads Conventions

- Create a bead for each unit of work before starting.
- Update status as you progress: `open` → `in_progress` → `closed`.
- Reference bead IDs in all bus messages.
- Sync on session end: `br sync --flush-only`.

### Mesh Protocol

- Include `-L mesh` on bus messages.
- Claim bead: `bus claims stake --agent $AGENT "bead://$BOTBOX_PROJECT/<bead-id>" -m "<bead-id>"`.
- Claim workspace: `bus claims stake --agent $AGENT "workspace://$BOTBOX_PROJECT/$WS" -m "<bead-id>"`.
- Claim agents before spawning: `bus claims stake --agent $AGENT "agent://role" -m "<bead-id>"`.
- Release claims when done: `bus claims release --agent $AGENT --all`.

### Spawning Agents

1. Check if the role is online: `bus agents`.
2. Claim the agent lease: `bus claims stake --agent $AGENT "agent://role"`.
3. Spawn with an explicit identity (e.g., via botty or agent-loop.sh).
4. Announce with `-L spawn-ack`.

### Reviews

- Use `crit` to open and request reviews.
- If a reviewer is not online, claim `agent://reviewer-<role>` and spawn them.
- Reviewer agents loop until no pending reviews remain (see review-loop doc).

### Cross-Project Feedback

When you encounter issues with tools from other projects:

1. Query the `#projects` registry: `bus inbox --agent $AGENT --channels projects --all`
2. Find the project entry (format: `project:<name> repo:<path> lead:<agent> tools:<tool1>,<tool2>`)
3. Navigate to the repo, create beads with `br create`
4. Post to the project channel: `bus send <project> "Filed beads: <ids>. <summary> @<lead>" -L feedback`

See [report-issue.md](.agents/botbox/report-issue.md) for details.

### Stack Reference

| Tool | Purpose | Key commands |
|------|---------|-------------|
| bus | Communication, claims, presence | `send`, `inbox`, `claim`, `release`, `agents` |
| maw | Isolated jj workspaces | `ws create`, `ws merge`, `ws destroy` |
| br/bv | Work tracking + triage | `ready`, `create`, `close`, `--robot-next` |
| crit | Code review | `review`, `comment`, `lgtm`, `block` |
| botty | Agent runtime | `spawn`, `kill`, `tail`, `snapshot` |

### Loop Scripts

Scripts in `scripts/` automate agent loops:

| Script | Purpose |
|--------|---------|
| `agent-loop.sh` | Worker: sequential triage-start-work-finish |
| `dev-loop.sh` | Lead dev: triage, parallel dispatch, merge |
| `reviewer-loop.sh` | Reviewer: review loop until queue empty |
| `spawn-security-reviewer.sh` | Spawn a security reviewer |

Usage: `bash scripts/<script>.sh <project-name> [agent-name]`
<!-- botbox:managed-end -->
