# Bitbucket CLI Command Contracts (Cloud MVP)

This document is the contract baseline for `bb` command behavior.

## Global

- Target: Bitbucket Cloud REST API + wiki Git repository flow
- Profile source: config file (`BB_CONFIG_PATH` override supported)
- Auth: per-profile token with optional Basic auth username (`--username` / `BITBUCKET_USERNAME`) and Bearer fallback
- Versioning: SemVer + short git hash build metadata (e.g. `0.0.1+abc1234`)
- Repo context inference: for repo-scoped commands, `--workspace`/`--repo` can be inferred from local Bitbucket `remote.origin.url` (`https://bitbucket.org/<workspace>/<repo>.git` or `git@bitbucket.org:<workspace>/<repo>.git`) when omitted
- Root help behavior: `bb` and top-level `bb --help` print the same top-level help with a short quick-start block for auth and common PR flows (`pr create`, `pr comments`) plus a note about `--output json`
- `bb help` is a root-help alias and prints the same output as `bb`/`bb --help`
- Existing-PR commands under `bb pr` accept the pull request ID as positional `<id>` or `--id`; passing both in one invocation is an error
- Selected read commands support `--json-fields <comma-separated-fields>` as a client-side JSON projection helper; it only works with `--output json`
- Output policy:
  - Human output for operator use (`table` or concise text)
  - JSON output for automation where supported
- Error policy:
  - Runtime failures for JSON-capable commands emit JSON error envelopes when JSON mode is selected
  - CLI parse/help failures are emitted before runtime dispatch and remain clap-rendered text

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
  - `--json-fields` (requires `--output json`)
- Output:
  - `table`: `SLUG`, `FULL_NAME`
  - `json`: array of repository objects
- Failure behavior:
  - Missing workspace -> non-zero exit
  - Unsupported output -> non-zero exit

## `bb version`

### `bb version` / `bb --version` / `bb -v`
- Purpose: Show build metadata for traceability.
- Output:
  - `bb version <semver+short-hash>`
  - `commit: <short-hash|unknown>`
  - `built: <RFC3339 timestamp|unknown>`
- Behavior note:
  - Running `bb` with no args also prints the current version in help output.

## `bb pr`

Naming rule: prefer Bitbucket API-aligned names (`get`, `update`, `request-changes`, `remove-request-changes`). GitHub CLI aliases accepted: `view`→`get`, `edit`→`update`, `close`→`decline`, `checks`→`statuses`.

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
  - `--json-fields` (requires `--output json`)
- Output:
  - `table`: summary line (`Showing <n> [of <total>] <state> pull requests in <workspace>/<repo>`) and columns `ID`, `TITLE`, `BRANCH`, `CREATED AT` (relative time)
    - ANSI color is enabled for terminal output; control with `BB_COLOR=always|never` or `NO_COLOR=1`
  - `json`: array of pull request objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Invalid `--state` value -> non-zero exit
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
  - `--close-branch` (delete source branch after merge)
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: created PR summary and URL when provided by API
  - `json`: created pull request object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr merge`
- Purpose: Merge a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--message` (merge commit message)
  - `--strategy` (`merge_commit|squash|fast_forward`)
  - `--close-branch` (delete source branch after merge)
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: merged PR summary and URL when provided by API
  - `json`: merged pull request object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Invalid `--strategy` value -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr get`
- Alias: `bb pr view`
- Purpose: Get a pull request by ID.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--profile`
  - `--fields`
  - `--output` (`text` default, `json`)
  - `--json-fields` (requires `--output json`)
- Output:
  - `text`: PR summary (`ID`, `STATE`, `TITLE`, source/destination branch, optional author/description/URL)
  - `json`: raw pull request object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr update`
- Alias: `bb pr edit`
- Purpose: Update selected pull request fields.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags (at least one required):
  - `--title`
  - `--description`
  - `--source`
  - `--destination`
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: updated PR summary and URL when provided by API
  - `json`: updated pull request object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - No update field provided -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr approve`
- Purpose: Approve a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: approval confirmation
  - `json`: participant object returned by Bitbucket
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr unapprove`
- Purpose: Remove an approval from a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: approval removal confirmation
  - `json`: synthetic success envelope (`id`, `action`, `ok`)
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr request-changes`
- Purpose: Request changes on a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: change-request confirmation
  - `json`: participant object returned by Bitbucket
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr remove-request-changes`
- Purpose: Remove a change request from a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: change-request removal confirmation
  - `json`: synthetic success envelope (`id`, `action`, `ok`)
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr decline`
- Alias: `bb pr close`
- Purpose: Decline a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: declined PR summary and URL when provided by API
  - `json`: declined pull request object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr comment`
- Purpose: Create a comment on a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
  - `--content`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: created comment summary and URL when provided by API
  - `json`: created comment object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Missing `--content` -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr comments`
- Purpose: List pull request comments.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all`
  - `--profile`
  - `--q`, `--sort`, `--fields`
  - `--json-fields` (requires `--output json`)
- Output:
  - `table`: `ID`, `AUTHOR`, `CREATED AT`, `CONTENT`
  - `json`: array of pull request comment objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr diff`
- Purpose: Get the diff for a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: raw diff payload
  - `json`: object with a single `diff` string field
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr statuses`
- Alias: `bb pr checks`
- Purpose: List commit statuses for a pull request.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all`
  - `--profile`
  - `--q`, `--sort`, `--fields`
  - `--json-fields` (requires `--output json`)
- Output:
  - `table`: `KEY`, `STATE`, `NAME`, `UPDATED AT`
  - `json`: array of status objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pr activity`
- Purpose: List pull request activity.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - pull request ID via positional `<id>` or `--id`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all`
  - `--profile`
  - `--q`, `--sort`, `--fields`
  - `--json-fields` (requires `--output json`)
- Output:
  - `table`: `TYPE`, `USER`, `CREATED AT`, `DETAIL`
  - `json`: array of activity objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Non-numeric pull request ID value -> non-zero exit
  - Passing both `<id>` and `--id` -> non-zero exit
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
  - `--json-fields` (requires `--output json`)
- Output:
  - `table`: `UUID`, `STATE`, `REF`
  - `json`: array of pipeline objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pipeline get`
- Purpose: Get a pipeline by UUID.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - `--uuid`
- Optional flags:
  - `--profile`
  - `--fields`
  - `--output` (`text` default, `json`)
  - `--json-fields` (requires `--output json`)
- Output:
  - `text`: pipeline summary (`UUID`, state, ref, optional build number and URL)
  - `json`: raw pipeline object
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Invalid `--uuid` value -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pipeline steps`
- Purpose: List steps for a pipeline.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - `--uuid`
- Optional flags:
  - `--output` (`table` default, `json`)
  - `--all`
  - `--profile`
  - `--sort`, `--fields`
  - `--json-fields` (requires `--output json`)
- Output:
  - `table`: `UUID`, `STATE`, `NAME`
  - `json`: array of pipeline step objects
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Invalid `--uuid` value -> non-zero exit
  - Unsupported output -> non-zero exit

### `bb pipeline log`
- Purpose: Get the raw log for a pipeline step.
- Required flags:
  - `--workspace`, `--repo` unless both can be inferred from local Bitbucket `remote.origin.url`
  - `--uuid`
  - `--step`
- Optional flags:
  - `--profile`
  - `--output` (`text` default, `json`)
- Output:
  - `text`: raw pipeline step log payload
  - `json`: object with `pipeline_uuid`, `step_uuid`, and `log`
- Failure behavior:
  - Missing required flags -> non-zero exit
  - Invalid `--uuid` or `--step` value -> non-zero exit
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

### `bb completion [bash|zsh|fish|powershell]`
- Purpose: Print shell completion script to stdout.
- Output:
  - No shell arg: completion usage text
  - Shell arg present: raw completion script for the selected shell
- Failure behavior:
  - Unsupported shell -> non-zero exit
