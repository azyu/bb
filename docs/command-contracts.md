# Bitbucket CLI Command Contracts (Cloud MVP)

This document is the contract baseline for `bb` command behavior.

## Global

- Target: Bitbucket Cloud REST API
- Profile source: config file (`BB_CONFIG_PATH` override supported)
- Auth: bearer token per profile
- Output policy:
  - Human output for operator use (`table` or concise text)
  - JSON output for automation where supported

## `bb auth`

### `bb auth login`
- Purpose: Save token/base URL into a named profile and set it active.
- Required inputs:
  - `--token` or `BITBUCKET_TOKEN` environment variable
- Optional flags:
  - `--profile` (default: `default`)
  - `--base-url` (default: `https://api.bitbucket.org/2.0`)
- Output:
  - Human: confirmation message with profile name
- Failure behavior:
  - Missing token -> non-zero exit with actionable message
  - Config write failure -> non-zero exit

### `bb auth status`
- Purpose: Show current/selected profile status without leaking secret values.
- Optional flags:
  - `--profile` (override active profile)
- Output:
  - Human only: profile name, base URL, token configured state
- Failure behavior:
  - No active profile -> non-zero exit with login guidance

## `bb api`

### `bb api [flags] <endpoint>`
- Purpose: Direct REST call escape hatch for unsupported wrappers.
- Optional flags:
  - `--method` (default: `GET`)
  - `--paginate` (follow `next` links and merge `values`)
  - `--profile`
  - `--q`, `--sort`, `--fields`
- Output:
  - JSON
- Failure behavior:
  - API error -> non-zero exit with status/body summary
  - Missing endpoint arg -> non-zero exit with usage

## `bb repo`

### `bb repo list`
- Purpose: List repositories in a workspace.
- Required flags:
  - `--workspace`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all` (follow pagination)
  - `--profile`
  - `--q`, `--sort`, `--fields`
- Output:
  - `table`: `SLUG`, `FULL_NAME`
  - `json`: array of repository objects
- Failure behavior:
  - Missing workspace -> non-zero exit
  - Unsupported output -> non-zero exit

## `bb pr` (stub)

Planned wrappers:
- `bb pr list`
- `bb pr create`

Current behavior:
- Command exists as a stub and returns non-zero.

## `bb pipeline` (stub)

Planned wrappers:
- `bb pipeline list`
- `bb pipeline run`

Current behavior:
- Command exists as a stub and returns non-zero.

## `bb issue` (stub)

Planned wrappers:
- `bb issue list`

Current behavior:
- Command exists as a stub and returns non-zero.

## `bb completion` (stub)

Planned wrappers:
- Shell completion generation for bash/zsh/fish/powershell

Current behavior:
- Command exists as a stub and returns non-zero.
