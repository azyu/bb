# Bitbucket CLI Command Contracts (Cloud MVP)

This document is the contract baseline for `bb` command behavior.

## Global

- Target: Bitbucket Cloud REST API + wiki Git repository flow
- Profile source: config file (`BB_CONFIG_PATH` override supported)
- Auth: per-profile token with optional Basic auth username (`--username` / `BITBUCKET_USERNAME`) and Bearer fallback
- Versioning: SemVer + short git hash build metadata (e.g. `0.0.1+abc1234`)
- Repo context inference: for repo-scoped commands, `--workspace`/`--repo` can be inferred from local Bitbucket `remote.origin.url` (`https://bitbucket.org/<workspace>/<repo>.git` or `git@bitbucket.org:<workspace>/<repo>.git`) when omitted
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
  - `--username` (Bitbucket username/email; when set, uses Basic auth)
  - `--base-url` (default: `https://api.bitbucket.org/2.0`)
  - `--with-token` (read token from stdin)
- Optional env:
  - `BITBUCKET_USERNAME` (same as `--username`)
- Auth behavior notes:
  - Personal API token: set `--username` to Atlassian account email (Basic auth for REST API).
  - Access-token-style profile: omit `--username` (Bearer mode for REST API).
  - Wiki Git operations map auth user automatically:
    - email-based profile -> `x-bitbucket-api-token-auth`
    - username-empty profile -> `x-token-auth`
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
  - Human only: profile name, base URL, auth mode, token configured state
- Failure behavior:
  - No active profile -> non-zero exit with login guidance

### `bb auth logout`
- Purpose: Remove a saved profile credential and clear/switch active profile.
- Optional flags:
  - `--profile` (remove a specific profile; default removes current profile)
- Output:
  - Human: removed profile name; prints new active profile when one remains
- Failure behavior:
  - Not logged in and no profile selected -> non-zero exit with login guidance
  - Unknown profile -> non-zero exit with profile-not-found message
  - Config write failure -> non-zero exit

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
  - `--workspace` unless it can be inferred from local Bitbucket `remote.origin.url`
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
  - `--workspace`, `--repo` unless both can be inferred from local Git `remote.origin.url` pointing to Bitbucket (`https://bitbucket.org/<workspace>/<repo>.git` or `git@bitbucket.org:<workspace>/<repo>.git`)
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
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
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
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
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
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
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
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
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

### `bb issue create`
- Purpose: Create an issue for a repository.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - `--title`
- Optional flags:
  - `--content` (mapped to `content.raw`)
  - `--state`
  - `--kind` (`bug|enhancement|proposal|task`)
  - `--priority` (`trivial|minor|major|critical|blocker`)
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: created issue summary and URL when provided by API
  - `json`: created issue object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb issue update`
- Purpose: Update selected fields of an existing issue.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - `--id`
- Optional flags (at least one required):
  - `--title`
  - `--content` (mapped to `content.raw`)
  - `--state`
  - `--kind`
  - `--priority`
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: updated issue summary and URL when provided by API
  - `json`: updated issue object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - No update field provided -> non-zero exit
  - Unsupported output -> non-zero exit

## `bb wiki`

Implementation note:
- Bitbucket Cloud wiki operations are handled through the wiki Git repository (`.../<repo>.git/wiki`) instead of REST wiki endpoints.

### `bb wiki list`
- Purpose: List wiki files/pages.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
- Optional flags:
  - `--profile`
  - `--output` (`table` default, `json`)
- Output:
  - `table`: `PATH`, `SIZE`
  - `json`: array of wiki file objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Clone/auth failure -> non-zero exit

### `bb wiki get`
- Purpose: Read a wiki page/file content.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - `--page`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: raw file content
  - `json`: `{page, content}`
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Invalid page path -> non-zero exit
  - Page not found -> non-zero exit

### `bb wiki put`
- Purpose: Create or update a wiki page/file and push the change.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - `--page`
  - one of `--content` or `--file`
- Optional flags:
  - `--message` (git commit message)
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: update/no-change summary
  - `json`: `{page, status}`
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Invalid page path -> non-zero exit
  - Both `--content` and `--file` set -> non-zero exit
  - Git commit/push failure -> non-zero exit

## `bb completion`

### `bb completion <bash|zsh|fish|powershell>`
- Purpose: Print shell completion script to stdout.
- Output:
  - Raw completion script for the selected shell
- Failure behavior:
  - Wrong argument count -> non-zero exit with usage
  - Unsupported shell -> non-zero exit
