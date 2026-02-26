# Bitbucket CLI (Cloud MVP)

| English | [한국어](README.ko-kr.MD) |

## Overview
`bb` is a Go-based Bitbucket Cloud CLI inspired by `gh`-style command grouping.

Implemented commands:
- `bb auth login`
- `bb auth status`
- `bb auth logout`
- `bb api`
- `bb repo list`
- `bb pr list`
- `bb pr create`
- `bb pipeline list`
- `bb pipeline run`
- `bb wiki list`
- `bb wiki get`
- `bb wiki put`
- `bb issue list`
- `bb issue create`
- `bb issue update`
- `bb completion <bash|zsh|fish|powershell>`
- `bb version` / `bb --version`

Every command supports `--help` / `-h` for detailed usage with `--flag` format:
```bash
bb pr --help          # group: list subcommands
bb pr create --help   # leaf: show flags with descriptions
```

## Requirements
- Go `1.26+`

## Build
```bash
make build        # -> ./bb (version + commit + date auto-injected)
make install      # -> $GOPATH/bin/bb
make test         # run all tests
make cover        # test coverage report
make help         # show all targets
```

Or build manually:
```bash
go build -o ./bb ./cmd/bb
```

## How To Create A Bitbucket API Token
This CLI uses a Bitbucket token for authentication.

Auth mode:
- API token (personal): use Basic auth (`username/email + token`)
- Access-token-style flows: Bearer token mode (no username)
- Wiki Git auth mapping:
  - Personal API token profile (email username) -> `x-bitbucket-api-token-auth`
  - Access-token-style profile (no username) -> `x-token-auth`

Steps (Bitbucket Cloud):
1. Open Bitbucket Cloud and go to Personal settings.
2. Open `API tokens`.
3. Click `Create API token`.
4. Name the token and choose scopes required for your use case.
5. Create and copy the token value once shown.

Official docs:
- https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/
- https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/

## Recommended Token Scopes
Use least privilege. Start with the minimum required scopes and add only what is needed.

### Recommended for general developer use
Default profile (safe baseline):
- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

If developers need to create/update PRs or trigger pipelines, add:
- `write:pullrequest:bitbucket`
- `write:pipeline:bitbucket`
- `write:issue:bitbucket`
- `write:wiki:bitbucket`

### CLI Scope Matrix
| Command / use case | Recommended scopes |
| --- | --- |
| `bb repo list` (repo read) | `read:repository:bitbucket` |
| `bb pr list` (PR read) | `read:repository:bitbucket`, `read:pullrequest:bitbucket` |
| `bb pr create` (PR write) | `read:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket` |
| `bb pipeline list` (pipeline read) | `read:repository:bitbucket`, `read:pipeline:bitbucket` |
| `bb pipeline run` (pipeline write) | `read:repository:bitbucket`, `read:pipeline:bitbucket`, `write:pipeline:bitbucket` |
| `bb wiki list/get` (wiki read) | `read:repository:bitbucket`, `read:wiki:bitbucket` |
| `bb wiki put` (wiki write) | `read:repository:bitbucket`, `read:wiki:bitbucket`, `write:wiki:bitbucket` |
| `bb issue list` (issue read) | `read:repository:bitbucket`, `read:issue:bitbucket` |
| `bb issue create/update` (issue write) | `read:repository:bitbucket`, `read:issue:bitbucket`, `write:issue:bitbucket` |
| `bb api` (user/workspace endpoints) | `read:user:bitbucket`, `read:workspace:bitbucket` |
| `bb completion`, `bb version` | no additional scopes |

### Avoid by default
- `admin:*`
- `delete:*`
- `write:permission:bitbucket` unless explicitly required

## Quick Start
1. Login with token:
```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>" --profile default
```

Alternative (stdin):
```bash
echo "<your-token>" | bb auth login --username "<bitbucket-email>" --with-token --profile default
```

Alternative (env):
```bash
BITBUCKET_USERNAME="<bitbucket-email>" BITBUCKET_TOKEN="<your-token>" bb auth login --profile default
```

2. Check auth:
```bash
bb auth status
```

Optional: logout current profile (or a specific profile):
```bash
bb auth logout
bb auth logout --profile <profile-name>
```

3. Check version:
```bash
bb version
```

4. List repositories:
```bash
bb repo list --workspace <workspace-slug>
```
Inside a Bitbucket repository, `bb repo list` can infer `<workspace>` from `remote.origin.url`.

5. Call API directly:
```bash
bb api /repositories/<workspace-slug>
bb api --paginate /repositories/<workspace-slug>
```

6. List PRs:
```bash
bb pr list --workspace <workspace-slug> --repo <repo-slug>
```
`table` output is shown in a `gh pr list`-like layout (summary line + `ID/TITLE/BRANCH/CREATED AT`).
ANSI color is enabled on terminal output; use `BB_COLOR=always|never` or `NO_COLOR=1` to control it.
Inside a Bitbucket repository, repo-scoped commands can infer `<workspace>/<repo>` from `remote.origin.url` when omitted:
```bash
bb pr list
bb pr create --title "My PR" --source feature --destination main
bb pipeline list
bb pipeline run --branch main
bb issue list
bb issue create --title "My issue" --content "details"
bb issue update --id 1 --state resolved
bb wiki list
bb wiki get --page Home.md
bb wiki put --page Home.md --content "# Updated"
```

7. Create PR (write scope required):
```bash
bb pr create --workspace <workspace-slug> --repo <repo-slug> --title "My PR" --source feature --destination main
```

8. List pipelines:
```bash
bb pipeline list --workspace <workspace-slug> --repo <repo-slug>
```

9. Trigger pipeline (write scope required):
```bash
bb pipeline run --workspace <workspace-slug> --repo <repo-slug> --branch main
```

10. Generate shell completion (supports subcommand completion):
```bash
bb completion zsh
# bash/zsh/fish/powershell supported; includes 2-level subcommand completion
# e.g. `bb pr<TAB>` -> list, create
```

11. List issues:
```bash
bb issue list --workspace <workspace-slug> --repo <repo-slug>
```

12. Create issue (write scope required):
```bash
bb issue create --workspace <workspace-slug> --repo <repo-slug> --title "My issue" --content "details"
```

13. Update issue (write scope required):
```bash
bb issue update --workspace <workspace-slug> --repo <repo-slug> --id 1 --state resolved
```

14. List wiki files:
```bash
bb wiki list --workspace <workspace-slug> --repo <repo-slug>
```

15. Read wiki page:
```bash
bb wiki get --workspace <workspace-slug> --repo <repo-slug> --page Home.md
```

16. Update wiki page (write scope required):
```bash
bb wiki put --workspace <workspace-slug> --repo <repo-slug> --page Home.md --content "# Updated"
```

## Configuration
Config path priority:
1. `BB_CONFIG_PATH` (explicit override)
2. `$XDG_CONFIG_HOME/bb/config.json` (if `XDG_CONFIG_HOME` is set)
3. `~/.config/bb/config.json` (default)

Backward compatibility:
- If the new default file is missing, the CLI also checks the legacy Go user-config location for `bb/config.json`.

Security note:
- Token is currently stored as plain text in the config file (MVP state).
- Config file is written with `0600` permissions.

## Development
```bash
make test         # run all tests
make cover        # coverage report for internal packages
make lint         # go vet
make fmt          # gofmt
```

For detailed command contracts:
- `docs/command-contracts.md`
