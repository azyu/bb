# Bitbucket CLI Command Contracts (Cloud MVP)

This document is the contract baseline for `bb` command behavior.

## Global

- Target: Bitbucket Cloud REST API
- Profile source: config file (`BB_CONFIG_PATH` override supported)
- Auth: bearer token per profile
- Versioning: SemVer + short git hash build metadata (e.g. `0.0.1+abc1234`)
- Output policy:
  - Human output for operator use (`table` or concise text)
  - JSON output for automation where supported

## `bb auth`

### `bb auth login`
- Purpose: Save token/base URL into a named profile and set it active.
- Required inputs:
  - `--token <value>` or `--with-token` or `BITBUCKET_TOKEN` environment variable
- Optional flags:
  - `--profile` (default: `default`)
  - `--base-url` (default: `https://api.bitbucket.org/2.0`)
  - `--with-token` (read token from stdin)
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

## `bb version`

### `bb version` / `bb --version`
- Purpose: Show build metadata for traceability.
- Output:
  - `bb version <semver+short-hash>`
  - `commit: <short-hash|unknown>`
  - `built: <RFC3339 timestamp|unknown>`
- Behavior note:
  - Running `bb` with no args also prints the current version in help output.

## `bb pr`

### `bb pr list`
- Purpose: List pull requests for a repository.
- Required flags:
  - `--workspace`
  - `--repo`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all` (follow pagination)
  - `--profile`
  - `--state` (`OPEN|MERGED|DECLINED`)
  - `--q`, `--sort`, `--fields`
- Output:
  - `table`: `ID`, `STATE`, `SOURCE`, `DEST`, `TITLE`
  - `json`: array of pull request objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr create`
- Purpose: Create a pull request for a repository.
- Required flags:
  - `--workspace`
  - `--repo`
  - `--title`
  - `--source`
  - `--destination`
- Optional flags:
  - `--description`
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: created PR summary and URL when provided by API
  - `json`: created pull request object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

## `bb pipeline`

### `bb pipeline list`
- Purpose: List pipelines for a repository.
- Required flags:
  - `--workspace`
  - `--repo`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all` (follow pagination)
  - `--profile`
  - `--sort`, `--fields`
- Output:
  - `table`: `UUID`, `STATE`, `REF`
  - `json`: array of pipeline objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pipeline run`
- Purpose: Trigger a pipeline by branch reference.
- Required flags:
  - `--workspace`
  - `--repo`
  - `--branch`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: triggered pipeline summary (`UUID`, state, ref)
  - `json`: triggered pipeline object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

## `bb issue`

### `bb issue list`
- Purpose: List issues for a repository.
- Required flags:
  - `--workspace`
  - `--repo`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all` (follow pagination)
  - `--profile`
  - `--q`, `--sort`, `--fields`
- Output:
  - `table`: `ID`, `STATE`, `KIND`, `PRIORITY`, `TITLE`
  - `json`: array of issue objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

## `bb completion`

### `bb completion <bash|zsh|fish|powershell>`
- Purpose: Print shell completion script to stdout.
- Output:
  - Raw completion script for the selected shell
- Failure behavior:
  - Wrong argument count -> non-zero exit with usage
  - Unsupported shell -> non-zero exit
