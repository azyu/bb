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
- Phase: complete
- Owner: agent
- Notes: Phase 1 Rust migration is complete. The Rust 2-crate workspace (`bb-cli`, `bb-core`) is now the only implementation, CI/release workflows target Cargo, and the legacy Go source tree has been removed. The PR surface has been expanded with Bitbucket Cloud API-aligned command names (`get`, `update`, `request-changes`, `remove-request-changes`, etc.) while keeping the repo Cloud-only and the architecture unchanged. Local Git helpers such as `bb pr checkout` remain explicit follow-up work.
