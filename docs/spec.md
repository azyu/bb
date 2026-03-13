# Technical Specification

## Purpose
- Build and maintain `bb` as a Rust-first Bitbucket Cloud CLI.
- Keep the public binary name `bb`.
- Keep the runtime predictable for both humans and automation.

## Documentation Boundaries
- `docs/spec.md` is the source of truth for implementation invariants and shared technical rules.
- `docs/command-contracts.md` is the single source of truth for command surface, flags, output modes, help text, and failure behavior.
- `.context/STEERING.md` and `.context/TASKS.md` hold roadmap, backlog, and follow-up work. Do not record speculative or future command ideas in this file.

## Architecture
- Rust workspace layout:
  - `rust/bb-cli`: CLI parsing and process behavior
  - `rust/bb-core`: validation, execution, rendering, config, Bitbucket client, and command handlers
- `bb-cli` delegates request execution to `bb-core`.
- The current implementation is synchronous/blocking.

## Supported Scope
- Bitbucket Cloud only.
- Supported top-level command groups are `auth`, `api`, `repo`, `pr`, `pipeline`, `issue`, `wiki`, `completion`, and `version`.
- Local Git workflow helpers such as `bb pr checkout` remain out of scope.
- Detailed subcommand contracts live only in `docs/command-contracts.md`.

## Config and Auth
- Config file path resolution:
  - `BB_CONFIG_PATH` if set
  - else `XDG_CONFIG_HOME/bb/config.json`
  - else `~/.config/bb/config.json`
- Config model:
  - active profile name
  - map of named profiles
  - each profile stores `base_url`, `token`, and optional `username`
- Default REST base URL: `https://api.bitbucket.org/2.0`
- REST auth mode:
  - Basic auth when profile `username` is non-empty
  - Bearer token otherwise
- `auth login` input precedence:
  - token: `--token <value>`, bare `--token`/`--with-token` from stdin, then `BITBUCKET_TOKEN`
  - username: `--username`, then `BITBUCKET_USERNAME`
  - base URL: `--base-url`, else default
- Environment-variable precedence is limited to config-path resolution and `auth login` input resolution; general command execution does not have a global env-over-config override layer.

## Repo Inference
- Repo-scoped commands may infer missing `--workspace` and `--repo` from local Git `remote.origin.url`.
- Only Bitbucket remotes must be inferred; non-Bitbucket remotes must not.
- Explicit CLI flags always win over inferred values.

## Output and Errors
- Success data goes to stdout.
- Text-mode runtime errors go to stderr with non-zero exit status.
- Commands that support machine-readable output emit JSON to stdout.
- Runtime failures for JSON-capable commands emit JSON error envelopes when that command is running in JSON mode.
- CLI parse/help errors are emitted by `bb-cli` before runtime dispatch and therefore remain clap-rendered text output.
- `bb api` is JSON-only.
- `--fields` is Bitbucket API query passthrough.
- `--json-fields` is local client-side JSON projection, requires `--output json`, and uses command-specific allowlists defined by the implementation and described in `docs/command-contracts.md`.
- `bb`, top-level `bb --help`, and bare `bb help` print the same root help with the quick-start block.

## Agent-Oriented CLI Rules
- Prefer predictable structured output over prose for automation-facing commands.
- Reject invalid or ambiguous inputs before network or Git write operations when possible.
- Reuse raw API objects for JSON output instead of re-parsing formatted text.
- Preserve `q`, `sort`, and `fields` passthrough where the Bitbucket API supports them.
- Prefer Bitbucket API-aligned naming (`get`, `update`, `request-changes`, `remove-request-changes`). GitHub CLI aliases accepted: `view`→`get`, `edit`→`update`, `close`→`decline`, `checks`→`statuses`.

## Bitbucket Client Rules
- Follow server-provided pagination via `next`.
- Support both relative API paths and absolute URLs.
- Support `q`, `sort`, and `fields` query params where applicable.
- Surface API failures with HTTP status and short response-body context.

## Wiki Rules
- Wiki commands use the wiki Git repository over HTTPS, not REST endpoints.
- Remote URLs include only the auth username, never the token.
- Provide the token to Git through `GIT_ASKPASS`.
- Wiki auth username mapping:
  - empty profile username -> `x-token-auth`
  - email-like username -> `x-bitbucket-api-token-auth`
  - any other username -> unchanged
- If the API host is `api.bitbucket.org`, normalize the wiki host to `bitbucket.org`.
- Wiki page paths must reject absolute paths and parent-directory traversal.
- `wiki put` accepts either `--content` or `--file`, not both.

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
- Release workflow uploads `checksums.txt` for the published archives.
- Release workflow auto-publishes the GitHub Release after asset upload.
- Release builds derive the binary semantic version from the release tag via build-time version injection.
- If `HOMEBREW_TAP_TOKEN` is configured, release workflow updates the `azyu/homebrew-tap` formula to the released version and checksums.
- Go build/test/release paths have been removed.
