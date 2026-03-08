# TASKS

- [x] Re-baseline planning/docs for Rust migration (`.context/STEERING.md`, `.context/TASKS.md`, `docs/references.md`) - owner: agent
- [x] Bootstrap Cargo workspace with `bb-cli` and `bb-core` crates - owner: agent
- [x] Implement shared Rust foundations (errors, config, version, output routing, git repo inference, Bitbucket client, pagination) - owner: agent
- [x] Port `bb version`, root help, and `bb completion` to Rust - owner: agent
- [x] Port `bb auth` commands to Rust - owner: agent
- [x] Port `bb api` and `bb repo list` to Rust - owner: agent
- [x] Port `bb pr list/create/merge` to Rust - owner: agent
- [x] Port `bb pipeline list/run` to Rust - owner: agent
- [x] Port `bb issue list/create/update` to Rust - owner: agent
- [x] Port `bb wiki list/get/put` to Rust - owner: agent
- [x] Rewrite README/build instructions and command docs for the Rust implementation - owner: agent
- [x] Convert CI and release workflows from Go to Rust - owner: agent
- [x] Expand release workflow to publish Linux amd64/arm64, Windows x64/arm64, and macOS arm64 artifacts - owner: agent
- [x] Auto-publish draft releases after successful release workflow run - owner: agent
- [x] Derive release binary version from the release tag during release builds - owner: agent
- [x] Update the Homebrew tap formula and automate tap updates from release workflow - owner: agent
- [x] Remove Go entrypoints, Go tests, and Go-specific build files after Rust verification passes - owner: agent

## Backlog

- [x] Expand `bb pr` with Bitbucket API-aligned read/write commands (`get`, `update`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comments`, `diff`, `statuses`, `activity`) - owner: agent
- [x] Document project DoD and post-DoD branch/commit/push/PR workflow in `AGENTS.md` - owner: agent
- [x] Implement `bb pr checkout` local Git workflow for same-repository pull requests - owner: agent
- [x] Configure deterministic local Rust checks with pinned toolchain and repo-managed Git hooks (`rust-toolchain.toml`, `.githooks`, CI alignment) - owner: agent
- [x] Align MVP contract behavior for `completion`, `issue` validation, and `pr checkout` guardrails - owner: agent
- [x] Broaden smoke coverage for `pipeline`, `issue`, and `wiki` command flows - owner: agent
- [x] Implement phase 2 agent-first extensions (`--describe`, `--dry-run`) - owner: agent
- [ ] Harden agent-specific invalid input handling (control characters, embedded query fragments, double-encoded identifiers) - owner: agent
