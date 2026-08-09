# CLI and Runtime Module Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 1,807-line CLI parser and 2,388-line runtime source files with focused command-domain modules while preserving every externally observable behavior.

**Architecture:** Keep `bb-cli` responsible for clap parsing and conversion into `bb_core::Request`, and keep `bb-core` responsible for runtime execution. Within each crate, use a directory module with a small `mod.rs`, command-domain modules, and narrowly shared helpers; move existing code instead of redesigning it.

**Tech Stack:** Rust 1.93+, clap, reqwest, serde/serde_json, Cargo test, rustfmt

## Global Constraints

- Bitbucket Cloud only.
- Supported top-level command groups remain `auth`, `api`, `repo`, `pr`, `pipeline`, `issue`, `wiki`, `completion`, and `version`.
- `bb-cli` delegates request execution to `bb-core`; the implementation remains synchronous/blocking.
- Command names, aliases, flags, defaults, conflicts, help text, request mapping, stdout/stderr routing, JSON envelopes, API endpoints, query parameters, pagination, Git behavior, and exit codes must not change.
- Do not add dependencies, traits, macros, speculative abstractions, or unrelated cleanup.
- Preserve existing tests by moving them with the responsibility they cover; add tests only if extraction exposes an untested observable boundary.
- Keep the public interfaces used outside each directory unchanged: `cli::parse_from`, `bb_core::runtime::STDIN_TOKEN_SENTINEL`, and `bb_core::run`.
- Prefer private or `pub(super)` visibility for extracted implementation details.
- No production Rust source created by this plan may exceed 1,000 lines.

---

### Task 1: Split CLI parsing by command domain

**Files:**
- Delete: `rust/bb-cli/src/cli.rs`
- Create: `rust/bb-cli/src/cli/mod.rs`
- Create: `rust/bb-cli/src/cli/auth.rs`
- Create: `rust/bb-cli/src/cli/api.rs`
- Create: `rust/bb-cli/src/cli/repo.rs`
- Create: `rust/bb-cli/src/cli/pr.rs`
- Create: `rust/bb-cli/src/cli/pipeline.rs`
- Create: `rust/bb-cli/src/cli/issue.rs`
- Create: `rust/bb-cli/src/cli/wiki.rs`
- Create: `rust/bb-cli/src/cli/repository.rs`
- Create: `rust/bb-cli/src/cli/tests.rs`
- Test: `rust/bb-cli/src/cli/tests.rs`
- Test: `rust/bb-cli/tests/cli_smoke.rs`

**Interfaces:**
- Consumes: All clap argument definitions and `bb_core::Request` conversion behavior currently implemented in `rust/bb-cli/src/cli.rs`.
- Produces: `pub fn parse_from<I, T>(args: I) -> Result<bb_core::Request, clap::Error>` in `cli/mod.rs`; private domain command/argument types and conversion functions; repository selection helpers used only by `cli/mod.rs`.

- [x] **Step 1: Record the focused baseline**

Run: `cargo test --manifest-path rust/Cargo.toml -p bb-cli`

Expected: 28 CLI unit tests and 70 CLI smoke tests pass.

- [x] **Step 2: Create the directory module and domain files**

Move the root `Cli`/`Commands` declarations, `parse_from`, argument normalization, and top-level request dispatch into `cli/mod.rs`. Move each command enum, argument structs, and its branch of `map_request` into the matching domain file. Keep `CompletionArgs` in `cli/mod.rs` because it has no runtime command family.

- [x] **Step 3: Isolate global repository selection**

Move `RepositoryTarget`, `parse_repository_target`, `apply_repository_target`, `set_repository_target`, and `unsupported_repository_target` into `cli/repository.rs`. Keep PR ID resolution in `cli/pr.rs` with the PR request conversion that uses it. Preserve exact clap conflict and error behavior.

- [x] **Step 4: Move the parser unit tests**

Move the existing `#[cfg(test)]` test module into `cli/tests.rs`, updating only module paths/imports. Do not delete or weaken assertions.

- [x] **Step 5: Format and run focused verification**

Run: `cargo fmt --manifest-path rust/Cargo.toml --all -- --check`

Run: `cargo test --manifest-path rust/Cargo.toml -p bb-cli`

Expected: rustfmt exits 0; 28 CLI unit tests and 70 CLI smoke tests pass with unchanged help and parsing assertions.

- [x] **Step 6: Confirm the size boundary and commit**

Run: `wc -l rust/bb-cli/src/cli/*.rs`

Expected: every listed production module except `tests.rs` is at most 1,000 lines, and `rust/bb-cli/src/cli.rs` no longer exists.

Commit: `refactor: split CLI parser modules (#38)`

---

### Task 2: Split runtime execution by command domain

**Files:**
- Delete: `rust/bb-core/src/runtime.rs`
- Create: `rust/bb-core/src/runtime/mod.rs`
- Create: `rust/bb-core/src/runtime/auth.rs`
- Create: `rust/bb-core/src/runtime/api.rs`
- Create: `rust/bb-core/src/runtime/repo.rs`
- Create: `rust/bb-core/src/runtime/pr.rs`
- Create: `rust/bb-core/src/runtime/pipeline.rs`
- Create: `rust/bb-core/src/runtime/issue.rs`
- Create: `rust/bb-core/src/runtime/wiki.rs`
- Create: `rust/bb-core/src/runtime/support.rs`
- Test: unit tests colocated in the extracted runtime domain modules
- Test: `rust/bb-cli/tests/cli_smoke.rs`

**Interfaces:**
- Consumes: Task 1's unchanged `bb_core::Request` values and all runtime behavior currently implemented in `rust/bb-core/src/runtime.rs`.
- Produces: `pub fn run<R: BufRead, O: Write, E: Write>(...) -> u8` and `pub const STDIN_TOKEN_SENTINEL` reachable through `bb_core::runtime`; private or `pub(super)` domain handlers; shared parsing/rendering/config helpers in `runtime/support.rs`.

- [x] **Step 1: Record the focused baseline**

Run: `cargo test --manifest-path rust/Cargo.toml -p bb-core`

Expected: 32 `bb-core` unit tests pass.

- [x] **Step 2: Create the runtime directory module**

Move `run`, top-level dispatch, completion/version handling, JSON-error routing, and `wants_json_errors` into `runtime/mod.rs`. Keep `STDIN_TOKEN_SENTINEL` publicly reachable at the same path.

- [x] **Step 3: Move command handlers by domain**

Move each `handle_*` family and its domain-specific constants/helpers into the matching `auth.rs`, `api.rs`, `repo.rs`, `pr.rs`, `pipeline.rs`, `issue.rs`, or `wiki.rs`. Keep PR progress/fetch helpers with `pr.rs`, pipeline selector/build lookup helpers with `pipeline.rs`, API input parsing/validation with `api.rs`, and wiki filesystem/Git helpers with `wiki.rs`.

- [x] **Step 4: Extract only genuinely shared support**

Move configuration/profile lookup, output-mode parsing, JSON projection/printing, required/optional string helpers, optional JSON field assignment, and query collection into `runtime/support.rs`. Use `pub(super)` only where sibling modules require access; do not redesign signatures unless compilation requires ownership-neutral adjustments.

- [x] **Step 5: Move domain unit tests**

Place API input tests in `api.rs`, PR pagination/progress tests in `pr.rs`, and pipeline selector tests in `pipeline.rs`. Do not delete or weaken assertions.

- [x] **Step 6: Format and run focused verification**

Run: `cargo fmt --manifest-path rust/Cargo.toml --all -- --check`

Run: `cargo test --manifest-path rust/Cargo.toml -p bb-core`

Run: `cargo test --manifest-path rust/Cargo.toml -p bb-cli`

Expected: rustfmt exits 0; 32 `bb-core` unit tests, 28 CLI unit tests, and 70 CLI smoke tests pass.

- [x] **Step 7: Confirm the size boundary and commit**

Run: `wc -l rust/bb-core/src/runtime/*.rs`

Expected: every production module is at most 1,000 lines, and `rust/bb-core/src/runtime.rs` no longer exists.

Commit: `refactor: split runtime modules (#38)`

---

## Final Verification

- [x] **Step 1: Review the final diff for scope**

Run: `git diff --check origin/main...HEAD`

Run: `git diff --stat origin/main...HEAD`

Expected: no whitespace errors; changes are limited to the plan/steering files and CLI/runtime module extraction.

- [x] **Step 2: Run full workspace verification**

Run: `cargo fmt --manifest-path rust/Cargo.toml --all -- --check`

Run: `cargo test --manifest-path rust/Cargo.toml`

Expected: rustfmt exits 0 and all workspace tests pass with zero failures.

- [x] **Step 3: Check public help and source sizes**

Run: `cargo run --manifest-path rust/Cargo.toml -p bb-cli --bin bb -- --help`

Run: `wc -l rust/bb-cli/src/cli/*.rs rust/bb-core/src/runtime/*.rs`

Expected: root help contains every supported top-level command, and every production module is at most 1,000 lines.

- [x] **Step 4: Record completion**

Post the commands and observed results to GitHub issue #38 before the final commit/status update. Update `.context/STEERING.md` to record that the refactor is complete and commit the plan/status files as `docs: record module refactor completion (#38)`.
