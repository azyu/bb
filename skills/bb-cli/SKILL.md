---
name: bb-cli
version: 1.0.0
description: "Bitbucket Cloud: inspect and operate repos, pull requests, pipelines, issues, wiki pages, and raw REST endpoints through the local bb CLI."
metadata:
  openclaw:
    category: "developer-tools"
    requires:
      bins: ["bb"]
    cliHelp: "bb --help"
---

# bb CLI

```bash
bb <command> <subcommand> [flags]
```

## Agent Usage Rules

- Prefer `--output json` for automation.
- Use `--json-fields` only on commands that explicitly support it.
- Outside a cloned Bitbucket repo, pass `--workspace` and `--repo` explicitly.
- Before any write operation, inspect the exact subcommand help in the current session and use only documented flags.
- Existing-PR commands accept positional `ID` or `--id`; passing both is an error.
- For `bb pr comments`, `ID`/`--id` always mean the pull request ID. Use `--comment-id` to target a single comment; never pass a comment ID via `--id`.
- For write operations, do not guess IDs, branch names, or target repos. Resolve them first.
- `bb pr create` uses `--description` and `--destination`; do not substitute `--body` or `--dest`.
- Use `bb api` when the wrapped command surface does not cover the operation you need.
- `bb api` is JSON-only.
- Wiki commands use the repo's wiki Git remote, not a REST endpoint.
- Runtime failures in JSON mode return JSON error envelopes; parse/help failures stay text.

## Command Groups

### auth

- `login` - Save a profile and set it active.
- `status` - Show current profile status without leaking token values.
- `logout` - Remove a saved profile.

### repo

- `list` - List repositories in a workspace.

### pr

- `list`, `get`, `create`, `update`, `merge`
- `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`
- `comment`, `comments`, `diff`, `statuses`, `activity`

### pipeline

- `list`, `get`, `run`, `steps`, `log`

### issue

- `list`, `create`, `update`

### wiki

- `list`, `get`, `put`

### api

- Raw Bitbucket Cloud REST calls with JSON output.

## Discovering Commands

Before calling a subcommand, inspect it:

```bash
# Root command surface
bb --help

# Group help
bb pr --help
bb pipeline --help

# Exact flags and positional arguments
bb pr create --help
bb pr get --help
bb pr comments --help
bb pipeline log --help
bb wiki put --help
```

## Common Calls

```bash
# Auth
bb auth status
bb auth login --token "$BITBUCKET_TOKEN" --username you@example.com

# Read operations
bb repo list --workspace acme --output json
bb pr list --workspace acme --repo widgets --state OPEN --output json
bb pr get 123 --workspace acme --repo widgets --output json
bb pr comments 123 --workspace acme --repo widgets --output json
bb pr comments 123 --comment-id 456 --workspace acme --repo widgets --output json
bb pipeline list --workspace acme --repo widgets --output json
bb pipeline get --workspace acme --repo widgets --uuid "{pipeline-uuid}" --output json
bb pipeline log --workspace acme --repo widgets --uuid "{pipeline-uuid}"
bb issue list --workspace acme --repo widgets --output json
bb wiki get --workspace acme --repo widgets --page Home.md

# Write operations
bb pr create --workspace acme --repo widgets --title "Add widget support" --source feature/widgets --destination main
bb pr create --workspace acme --repo widgets --title "Add widget support" --source feature/widgets --destination main --description "$(cat ./pr-body.md)"
bb issue create --workspace acme --repo widgets --title "Broken widget" --kind bug --priority major --output json
bb wiki put --workspace acme --repo widgets --page Home.md --file ./docs/home.md

# Escape hatch
bb api repositories/acme/widgets/pullrequests --paginate
```

## GitHub CLI Compatibility

Subcommand aliases accepted: `view`→`get`, `edit`→`update`, `close`→`decline`, `checks`→`statuses`.

Flag names differ — `bb pr create` uses `--description` (not `--body`) and `--destination` (not `--base`/`--dest`). When unsure, run `<command> --help`.

## References

- Command contracts: `docs/command-contracts.md`
- Shared implementation rules: `docs/spec.md`
