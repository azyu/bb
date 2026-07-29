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
- Phase: v0.2.7 skill synchronized
- Owner: Main
- Tracking: [GitHub issue #37](https://github.com/azyu/bb-cli/issues/37)
- Notes: The repository and local skillctl canonical `bb-cli` skill match the v0.2.7 command contracts. Claude and Codex rendered targets are applied, byte-identical to the repository source, and `skillctl plan` reports no changes.
