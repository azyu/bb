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

- [x] Support positional PR IDs across `bb pr` ID-based commands alongside `--id` - owner: agent
- [x] Align `bb --help` with root no-arg quick-start help - owner: agent
- [x] Add root help quick-start scenarios for agent-first onboarding (`bb` no-arg help examples) - owner: agent
- [x] Expand `bb pr` with Bitbucket API-aligned read/write commands (`get`, `update`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comments`, `diff`, `statuses`, `activity`) - owner: agent
- [x] Expand `bb pipeline` with read commands for PR debugging (`get`, `steps`, `log`) - owner: agent
- [x] Tighten pipeline UUID validation for `get`, `steps`, and `log` review feedback - owner: agent
- [x] Resolve PR #8 merge conflict after stacked PR merges - owner: agent
- [x] Enable release automation on version tag push - owner: agent
- [x] Refresh README and AGENTS docs for current Rust CLI surface - owner: agent
- [x] Add gh-style JSON field projection for read commands (`--json-fields`) - owner: agent
- [ ] Evaluate `bb pr checkout` local Git workflow for a future follow-up - owner: agent
- [ ] Evaluate phase 2 agent-first extensions (`--describe` or schema introspection, `--dry-run`, separate automation surface if needed) - owner: agent
