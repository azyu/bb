# AGENTS.md

> **Note:** If you use multiple coding assistants, make `CLAUDE.md` and `GEMINI.md` symlinks to this file.

## Project Structure

Current repository state:
- `docs/references.md`: baseline research for Bitbucket CLI scope, API references, and MVP direction.
- `TASKS.md`: work item tracker for agent-level execution status.
- `PLAN.md`: high-level plan tracker (phases, success criteria, current focus).

Project goal (source of truth: `docs/references.md`):
- Build a Bitbucket CLI similar to `gh` and `tea`.
- Keep first implementation focused on **Bitbucket Cloud**.

If you add source code, keep layout simple and explicit:
- Put runtime code in one top-level code directory (for example `src/` or language-standard equivalent).
- Put tests in one clear test location (for example `tests/` or language-standard equivalent).
- Update this file once the toolchain is chosen.

## Multi-Agent Coordination

When multiple agents split work, use these files as the single source of execution state:
- `PLAN.md`: tracks objective, phase order, success criteria, and current phase owner.
- `TASKS.md`: tracks actionable tasks as checkboxes (`- [ ]`, `- [x]`) with owner and blocker notes.

Mandatory startup rule for every agent task:
1. Read `PLAN.md` first.
2. Read `TASKS.md` second.
3. Only then start implementation.

Update rules during work:
- Before starting a task, assign the owner and add `(in progress)` on that task line.
- If plan/sequence changed, update `PLAN.md` before coding continues.
- On completion, change checkbox to `- [x]` and sync any follow-up work items.

## Build & Development

Primary toolchain: **Go 1.26+**

Useful current commands:
- List tracked/untracked files quickly:
  ```bash
  rg --files -uu
  ```
- Read planning state before any implementation:
  ```bash
  sed -n '1,240p' PLAN.md
  sed -n '1,240p' TASKS.md
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
  go run ./cmd/bb --help
  ```
- Run all tests:
  ```bash
  go test ./...
  ```
- Run package coverage:
  ```bash
  go test -cover ./internal/...
  ```
- Format Go files:
  ```bash
  gofmt -w ./cmd ./internal
  ```

## Code Standards

### Do
- Read `PLAN.md` and `TASKS.md` before any implementation task.
- Keep changes directly tied to the current task; avoid opportunistic refactors.
- Prefer the smallest implementation that satisfies requirements.
- When technical choices, API scopes, endpoint usage, or architecture assumptions change, update `docs/references.md` if the change affects project direction.
- Keep the first release Cloud-only unless explicitly asked otherwise.
- Mirror proven CLI shape from references (`auth`, `repo`, `pr`, `pipeline`, `issue`, `wiki`, `api`, `completion`).
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

Current minimum checklist (docs/bootstrap phase):
1. Ensure files are where expected:
   ```bash
   rg --files -uu
   ```
2. Re-open planning files and verify status is current:
   ```bash
   sed -n '1,240p' PLAN.md
   sed -n '1,240p' TASKS.md
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

Use file-scoped checks first when possible (e.g. `go test ./internal/api`).

## Testing

Testing uses Go's standard testing package.

Rules:
- Prefer fast, file-scoped tests first.
- For bug fixes, reproduce with a failing test before implementing the fix.
- Do not claim a fix is complete until the reproduction test passes.
- Target 80%+ coverage for active internal packages (`./internal/...`).

## Commit & PR Guidelines

- Keep each change set focused on one goal.
- Include verification commands actually run.
- If a command could not be run, state that explicitly.
- Document assumptions and unresolved questions in the PR description.
- When work is completed normally, create a commit for the finished scope.
- Before committing, ensure `PLAN.md` and `TASKS.md` reflect final status.
- Suggested commit flow:
  ```bash
  git add AGENTS.md TASKS.md PLAN.md
  git commit -m "docs: define multi-agent plan/task workflow"
  ```

## Secrets & Environment

- Never commit access tokens, OAuth secrets, or credentials.
- Never hardcode Bitbucket credentials in source code or docs.
- Use local environment configuration that is excluded from version control.

## Known Gotchas

- Bitbucket Cloud and Data Center APIs differ significantly; do not mix them accidentally.
- For list endpoints, rely on API-provided pagination (`next`) instead of hand-built page URLs.
- Keep auth design aligned with current Bitbucket Cloud recommendations; avoid deprecated defaults.
