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
- Phase: post-MVP CLI usability follow-ups
- Owner: Main
- Linear: [AZYU-5](https://linear.app/azyu/issue/AZYU-5/bb-cli-usability-follow-ups-from-2026-07-26-feedback)
- Notes: Linear is the execution-task source of truth. Current focus is [AZYU-12](https://linear.app/azyu/issue/AZYU-12/show-tty-only-progress-for-all-pagination), reporting `--all` pagination progress only when stderr is an interactive terminal.
