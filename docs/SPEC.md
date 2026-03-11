# Technical Specification

## Purpose
- Rebuild `bb` as a Rust-first Bitbucket Cloud CLI.
- Keep the public binary name `bb`.
- Phase 1 established the Cloud MVP command set; current follow-up work may extend that surface without changing the Cloud-only target.
- Treat the CLI as friendly to both humans and automation/agents, with predictable machine-readable behavior.

## Agent-First Design Notes
- Keep a human-friendly CLI surface, but make the runtime predictable enough for AI agents to call safely.
- Phase 1 already treats these as first-class requirements:
  - machine-readable JSON output for automation-facing paths
  - JSON error envelopes when `--output json` is selected
  - strict early validation before network or Git write operations
  - non-interactive auth via flags, env vars, and config
  - `bb api` plus `q`, `sort`, and `fields` passthrough as the raw automation escape hatch
- Phase 1 does not try to make every mutating command raw-JSON-first. The raw payload path for advanced agents is `bb api`, while typed subcommands remain the default UX for common flows.
- The CLI should preserve one source of truth for both human and agent surfaces:
  - `bb-cli` owns parsing and process behavior
  - `bb-core` owns validation, execution, and rendering contracts
- Follow-up work, inspired by Justin Poehnelt's March 4, 2026 article on agent-oriented CLIs:
  - add `bb schema` or `--describe` style runtime introspection
  - add `--dry-run` for mutating commands
  - harden more agent-specific invalid inputs such as control characters, path traversal, embedded query fragments, and double-encoded identifiers
  - consider NDJSON/streaming output for very large paginated responses
  - expose future MCP/extension surfaces by reusing `bb-core`, not by forking behavior into a second implementation

## Architecture
- Rust workspace layout:
  - `rust/bb-cli`: thin binary wrapper
  - `rust/bb-core`: runtime, config, Bitbucket client, rendering, command handlers
- `bb-cli` delegates execution to `bb-core`.
- The phase 1 rewrite is synchronous/blocking.

## Current Command Surface
- `bb auth login`
- `bb auth status`
- `bb auth logout`
- `bb api`
- `bb repo list`
- `bb pr list`
- `bb pr create`
- `bb pr merge`
- `bb pr get`
- `bb pr update`
- `bb pr approve`
- `bb pr unapprove`
- `bb pr request-changes`
- `bb pr remove-request-changes`
- `bb pr decline`
- `bb pr comment`
- `bb pr comments`
- `bb pr diff`
- `bb pr statuses`
- `bb pr activity`
- `bb pipeline list`
- `bb pipeline get`
- `bb pipeline steps`
- `bb pipeline log`
- `bb pipeline run`
- `bb issue list`
- `bb issue create`
- `bb issue update`
- `bb wiki list`
- `bb wiki get`
- `bb wiki put`
- `bb completion <bash|zsh|fish|powershell>`
- `bb version`
- `bb --version`

Still out of scope:
- local Git checkout helpers such as `bb pr checkout`
- extra PR wrappers not backed by a clear Bitbucket Cloud REST operation title

## Config and Auth
- Default config path:
  - `BB_CONFIG_PATH` if set
  - else `XDG_CONFIG_HOME/bb/config.json`
  - else `~/.config/bb/config.json`
- Config precedence:
  - CLI args
  - environment variables
  - config file
  - defaults
- Profile model:
  - active profile name
  - map of named profiles
  - each profile stores `base_url`, `token`, optional `username`
- Default REST base URL: `https://api.bitbucket.org/2.0`
- REST auth mode:
  - Basic auth if profile username is non-empty
  - Bearer token otherwise
- `auth login` token precedence:
  - `--token <value>`
  - `--with-token` or bare `--token` from stdin
  - `BITBUCKET_TOKEN`
  - else error
- `auth login` username precedence:
  - `--username`
  - `BITBUCKET_USERNAME`

## Repo Inference
- Repo-scoped commands may infer missing `--workspace` and `--repo` from local Git `remote.origin.url`.
- Supported remotes:
  - `https://bitbucket.org/<workspace>/<repo>.git`
  - `git@bitbucket.org:<workspace>/<repo>.git`
- Explicit flags always win over inferred values.
- Non-Bitbucket remotes must not infer values.

## Output and Errors
- Success data goes to stdout.
- Text-mode errors go to stderr with non-zero exit status.
- Commands that support machine-readable output emit JSON to stdout.
- Commands with `--output json` must emit JSON error envelopes to stdout on failure.
- Selected read commands may accept `--json-fields <comma-separated-fields>` to project full JSON output down to a smaller object/array shape; this is client-side output shaping and requires `--output json`.
- Running `bb` with no arguments or top-level `--help` prints the same root help plus a short quick-start block for common agent-first flows.
- Supported output modes:
  - list commands: `table|json`
  - write/detail commands: `text|json`
  - wiki get: `text|json`
  - `bb api`: JSON only
- PR-specific conventions:
  - `bb pr get`, `bb pr update`, `bb pr approve`, `bb pr unapprove`, `bb pr request-changes`, `bb pr remove-request-changes`, `bb pr decline`, `bb pr comment`: `text|json`
  - `bb pr comments`, `bb pr statuses`, `bb pr activity`: `table|json`
  - `bb pr diff`: `text|json`, where JSON wraps the raw diff payload in an object
- Pipeline-specific conventions:
  - `bb pipeline get`, `bb pipeline log`, `bb pipeline run`: `text|json`
  - `bb pipeline list`, `bb pipeline steps`: `table|json`
  - `bb pipeline log`: JSON wraps the raw log payload in an object with pipeline and step UUID metadata
- `bb pr list` text output keeps:
  - summary line
  - columns `ID`, `TITLE`, `BRANCH`, `CREATED AT`
  - relative timestamps
  - color controls via `BB_COLOR`, `NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`
- PR commands that operate on an existing pull request accept the pull request ID as positional `<id>` or `--id`.
- `--fields` remains the Bitbucket API query passthrough, while `--json-fields` is a local output projection feature.

## Agent-Oriented CLI Rules
- Prefer predictable structured output over prose for automation-facing commands.
- Keep runtime validation strict for all user/agent-provided inputs.
- Reject invalid or ambiguous command inputs early, before making network or git write operations.
- When a CLI command is a thin wrapper over a Bitbucket Cloud REST operation, prefer the Bitbucket API operation/resource naming in the CLI contract (`get`, `update`, `request-changes`, `remove-request-changes`) instead of local synonyms such as `view` or `edit`.
- Keep the active contract in repository documents instead of relying on stale prompt context:
  - `docs/SPEC.md` for implementation rules
  - `docs/references.md` for research and external references
  - `AGENTS.md` for agent workflow
- Reuse raw API objects for JSON output instead of re-parsing formatted text.
- Preserve `q`, `sort`, and `fields` passthrough to let automation reduce response size explicitly.
- Current phase 1 alignment:
  - human-facing commands keep concise text/table output
  - automation-facing commands use stable `json` output modes and JSON error envelopes
  - parser-level conflicts and invalid combinations must fail before network or git write operations
- Root help should surface a minimal "copy and adapt" onboarding path for agents with no prior repo knowledge, centered on auth plus common PR flows such as `pr create` and `pr comments`.
- Future extensions, motivated by agent-oriented CLI design:
  - command/schema introspection for discovery (`bb <command> --describe` or equivalent)
  - `--dry-run` or validation-only modes for write commands
  - optional MCP exposure or a separate automation surface if human CLI ergonomics and agent ergonomics diverge materially

## Bitbucket Client Rules
- Follow server-provided pagination using `next`.
- Support relative API paths and absolute URLs.
- Support query params `q`, `sort`, `fields` where applicable.
- Surface API failures with status code and short response body context.

## Wiki Rules
- Wiki commands use Git over HTTPS, not REST.
- Remote URL includes only the auth username, never the token.
- Provide the token via `GIT_ASKPASS`.
- Wiki auth username mapping:
  - empty profile username -> `x-token-auth`
  - email-like username -> `x-bitbucket-api-token-auth`
  - any other username -> unchanged
- If API host is `api.bitbucket.org`, wiki host normalizes to `bitbucket.org`.
- `wiki put` supports either `--content` or `--file`, not both.

## Build and Release
- Cargo is the primary build surface.
- CI and release workflows build Rust artifacts named `bb`.
- Release workflow runs on version tag pushes matching `v*.*.*` and also supports manual `workflow_dispatch` with an explicit release tag.
- Release workflow publishes:
  - `linux_amd64` as `.tar.gz`
  - `linux_arm64` as `.tar.gz`
  - `macos_arm64` as `.tar.gz`
  - `windows_x64` as `.zip`
  - `windows_arm64` as `.zip`
- Release workflow also uploads `checksums.txt` covering all published archives.
- Release workflow auto-publishes the GitHub Release after assets upload, even if the tag already had a draft release.
- Release builds derive the binary semantic version from the release tag (for example `v0.1.0` -> `0.1.0`) via build-time version injection.
- If `HOMEBREW_TAP_TOKEN` is configured, release workflow also updates the `azyu/homebrew-tap` formula to the released version and checksums.
- Go build/test/release paths are removed after Rust verification passes.
