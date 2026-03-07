# TASKS

- [x] Re-baseline planning/docs for Rust migration (`PLAN.md`, `TASKS.md`, `docs/references.md`) - owner: agent
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
- [x] Remove Go entrypoints, Go tests, and Go-specific build files after Rust verification passes - owner: agent

## Backlog

- [ ] Evaluate phase 2 agent-first extensions (`--describe` or schema introspection, `--dry-run`, separate automation surface if needed) - owner: agent
