# AGENTS.md

> **Note:** If you use multiple coding assistants, make `CLAUDE.md` and `GEMINI.md` symlinks to this file.

## Project Structure

Current repository state:
- `README.MD`: primary user-facing overview and quick start (English).
- `README.ko-kr.MD`: Korean user-facing overview and quick start.
- `rust/bb-cli`: CLI parsing, process behavior, and binary entrypoint.
- `rust/bb-core`: runtime, config, Bitbucket client, rendering, and command handlers.
- `docs/references.md`: baseline research for Bitbucket CLI scope, API references, and MVP direction.
- `docs/spec.md`: canonical technical specification for the active implementation target, including agent-oriented CLI rules.
- `docs/command-contracts.md`: command-by-command behavior contract for the Cloud CLI surface.
- `.context/STEERING.md`: high-level plan tracker (phases, success criteria, current focus).

Project goal (source of truth: `docs/references.md`):
- Build a Bitbucket CLI similar to `gh` and `tea`.
- Keep first implementation focused on **Bitbucket Cloud**.

The toolchain and layout are already chosen:
- Rust workspace rooted at `rust/`
- CLI entrypoint in `rust/bb-cli`
- shared implementation in `rust/bb-core`
- integration-style CLI smoke tests in `rust/bb-cli/tests`

## Multi-Agent Coordination

GitHub Issues on `azyu/bb-cli` is the single source of truth for actionable work, status, ownership, blockers, and follow-ups. Use the `gh` CLI. Open issues labeled `backlog` are the actionable queue; issues labeled `archive` are historical records and are not actionable.

Repository planning remains in `.context/STEERING.md`, which tracks objectives, phase order, success criteria, and current focus without duplicating executable task lists.

Mandatory startup rule for every agent task:
1. Read `.context/STEERING.md`.
2. Read the issue you are working on with `gh issue view <number> -R azyu/bb-cli --comments`. If no issue number was given, pick one from `gh issue list -R azyu/bb-cli --state open --label backlog`, or open a new issue for the work first.
3. Read `docs/spec.md` for the current technical spec and agent-facing behavior constraints.
4. Only then start implementation.

If there is any ambiguity about command behavior, output contracts, or agent-facing constraints, resolve it against `docs/spec.md` before changing code.

Document roles:
- `README.MD` and `README.ko-kr.MD` are for end users.
- `AGENTS.md` is for coding agents and contributors working in this repository.
- `docs/spec.md` is the implementation source of truth.
- `docs/command-contracts.md` is the command behavior reference.

Update rules during work:
- Before starting implementation, take ownership with `gh issue edit <number> -R azyu/bb-cli --add-assignee @me` and a `Claimed by <agent-name>: <one-line plan>` comment, then re-read the comments to confirm no competing claim landed after yours. The comment is what identifies the owner — assistants can share one authenticated `gh` account, so the assignee alone does not.
- If plan or sequence changes, update `.context/STEERING.md` before coding continues.
- On completion, comment the verification results on the issue, then close it with `gh issue close <number> -R azyu/bb-cli --reason completed` once the work has landed — immediately for a direct commit, only after merge when the work went through a PR. An open PR can still fail review or CI, and open `backlog` issues are the queue, so closing at PR creation would drop unfinished work out of it.
- Open follow-up issues for remaining work. GitHub has no parent/child issues here, so cross-link the follow-ups and the closed issue by number in both directions.

## Build & Development

Primary toolchain: **Rust 1.93+**

Useful current commands:
- List tracked/untracked files quickly:
  ```bash
  rg --files -uu
  ```
- Read planning and task state before any implementation:
  ```bash
  sed -n '1,240p' .context/STEERING.md
  gh issue view <number> -R azyu/bb-cli --comments
  sed -n '1,260p' docs/spec.md
  ```
- Review project reference:
  ```bash
  sed -n '1,240p' docs/references.md
  ```
- Review command contracts:
  ```bash
  sed -n '1,260p' docs/command-contracts.md
  ```
- Run CLI locally:
  ```bash
  cargo run --manifest-path rust/Cargo.toml -p bb-cli --bin bb -- --help
  ```
- Run all tests:
  ```bash
  cargo test --manifest-path rust/Cargo.toml
  ```
- Format Rust files:
  ```bash
  cargo fmt --manifest-path rust/Cargo.toml --all
  ```

## Code Standards

### Do
- Read `.context/STEERING.md`, the applicable GitHub issue, and `docs/spec.md` before any implementation task.
- Keep changes directly tied to the current task; avoid opportunistic refactors.
- Prefer the smallest implementation that satisfies requirements.
- When technical behavior changes, update `docs/spec.md` in the same change.
- When technical choices, API scopes, endpoint usage, or architecture assumptions change, update `docs/references.md` if the change affects project direction.
- Keep the first release Cloud-only unless explicitly asked otherwise.
- Mirror proven CLI shape from references (`auth`, `repo`, `pr`, `pipeline`, `issue`, `wiki`, `api`, `completion`).
- When a command maps directly to a Bitbucket Cloud REST operation, prefer the Bitbucket API operation/resource naming over wrapper synonyms (for example `get`/`update` instead of `view`/`edit`).
- Implement API pagination using Bitbucket `next` links.
- Support both human-readable output and JSON output for automation.
- Keep non-interactive behavior deterministic with explicit flags when needed.

### Don’t
- Don’t silently assume requirements when multiple interpretations exist; state assumptions.
- Don’t implement Bitbucket Data Center support in Cloud MVP work.
- Don’t use deprecated auth paths as the default design.
- Don’t add abstractions before a clear second use-case exists.
- Don’t change unrelated files or formatting.

### Write Operation Safety
- Use a dedicated test repository for write operations (`bb pr create`, `bb pipeline run`, `bb issue create`, `bb issue update`, `bb wiki put`) during development.
- Keep production repos on read-only tokens unless write access is explicitly needed.

## After Code Changes

Always verify at the smallest meaningful scope first.

Current minimum checklist:
1. Ensure files are where expected:
   ```bash
   rg --files -uu
   ```
2. Re-open planning state and verify the issue is current:
   ```bash
   sed -n '1,240p' .context/STEERING.md
   gh issue view <number> -R azyu/bb-cli --comments
   ```
3. Re-open changed docs and check for coherence:
   ```bash
   sed -n '1,240p' docs/references.md
   ```
   If the task changed technical assumptions, ensure `docs/references.md` is updated in the same change.
4. If `AGENTS.md` changed, re-read it for internal consistency:
   ```bash
   sed -n '1,260p' AGENTS.md
   ```

Use file-scoped checks first when possible (e.g. `cargo test --manifest-path rust/Cargo.toml -p bb-core`).

## Testing

Testing uses Rust's standard testing support via Cargo.

Rules:
- Prefer fast, file-scoped tests first.
- For bug fixes, reproduce with a failing test before implementing the fix.
- Do not claim a fix is complete until the reproduction test passes.
- Prefer focused crate/package tests before running the full workspace.

## Commit & PR Guidelines

- Keep each change set focused on one goal.
- Include verification commands actually run.
- If a command could not be run, state that explicitly.
- Document assumptions and unresolved questions in the PR description.
- When work is completed normally, create a commit for the finished scope.
- Before committing, ensure `.context/STEERING.md` and the GitHub issue reflect final status. "Final status" here means the verification results are posted on the issue; closing it comes after the commit or PR exists, since the closing comment references them.
- Include the issue reference (`#<number>`) in the commit message or PR description when applicable.

## Secrets & Environment

- Never commit access tokens, OAuth secrets, or credentials.
- Never hardcode Bitbucket credentials in source code or docs.
- Use local environment configuration that is excluded from version control.

## Known Gotchas

- Bitbucket Cloud and Data Center APIs differ significantly; do not mix them accidentally.
- For list endpoints, rely on API-provided pagination (`next`) instead of hand-built page URLs.
- Keep auth design aligned with current Bitbucket Cloud recommendations; avoid deprecated defaults.
