# PLAN

## Objective
- Rebuild `bb` as a Rust-first Bitbucket Cloud CLI with a `gh`-like structure.
- Keep the public binary name `bb`.
- Complete phase 1 with Rust MVP parity for the documented command set only.

## Phases
1. Rust migration reset (scope, plan/task tracker, docs baseline).
2. Rust workspace bootstrap (`bb-cli` + `bb-core`) and shared foundations.
3. MVP command port (`auth`, `api`, `repo`, `pr`, `pipeline`, `issue`, `wiki`, `completion`, `version`).
4. Rust-only validation, release workflow conversion, and Go removal.

## Success Criteria
- Rust workspace builds and tests cleanly with Cargo.
- The documented Cloud MVP commands are implemented in Rust and verified.
- Config precedence, auth modes, repo inference, pagination, and output modes match the documented contract.
- CI and release workflows build Rust artifacts named `bb`.
- Go entrypoints and Go-only workflows are removed after Rust verification passes.

## Current Phase
- Phase: v0.2.8 released
- Owner: Main
- Tracking: [GitHub PR #44](https://github.com/azyu/bb-cli/pull/44)
- Notes: Release v0.2.8 was published from commit `efbde3c`. All five platform archives and `checksums.txt` are available, published checksums pass, the macOS ARM64 artifact reports `0.2.8+efbde3c`, and the Homebrew formula is synchronized with matching sha256 digests. 131 workspace tests and the formatting check pass. The release contents are the `bb api --input -` stdin fix (#40), the CLI/runtime module refactor (#38), the Actions Node 24 migration (#42), and the agent-facing docs sync (#44).
