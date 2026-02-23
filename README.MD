# Bitbucket CLI (Cloud MVP)

| English | [한국어](README.ko-kr.MD) |

## Overview
`bb` is a Go-based Bitbucket Cloud CLI inspired by `gh`-style command grouping.

Implemented commands:
- `bb auth login`
- `bb auth status`
- `bb api`
- `bb repo list`
- `bb pr list`
- `bb pr create`
- `bb pipeline list`
- `bb pipeline run`
- `bb issue list`
- `bb issue create`
- `bb issue update`
- `bb completion <bash|zsh|fish|powershell>`
- `bb version` / `bb --version`

## Requirements
- Go `1.26+`

## Build
```bash
go build -o ./bb ./cmd/bb
```

Build with explicit version metadata (SemVer + git hash):
```bash
VERSION="0.0.1"
COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
BUILD_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
go build -ldflags "-X bitbucket-cli/internal/version.Version=${VERSION} -X bitbucket-cli/internal/version.Commit=${COMMIT} -X bitbucket-cli/internal/version.BuildDate=${BUILD_DATE}" -o ./bb ./cmd/bb
```

## Install (example: local user bin)
```bash
mkdir -p ~/.local/bin
go build -o ~/.local/bin/bb ./cmd/bb
chmod +x ~/.local/bin/bb
```

## How To Create A Bitbucket API Token
This CLI uses a Bitbucket token for authentication.

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
- `read:user:bitbucket`
- `read:workspace:bitbucket`

If developers need to create/update PRs or trigger pipelines, add:
- `write:pullrequest:bitbucket`
- `write:pipeline:bitbucket`

### Current CLI baseline
- `bb repo list`: `read:repository:bitbucket`
- `bb pr list`: `read:repository:bitbucket`, `read:pullrequest:bitbucket`
- `bb pr create`: `read:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`
- `bb pipeline list`: `read:repository:bitbucket`, `read:pipeline:bitbucket`
- `bb pipeline run`: `read:repository:bitbucket`, `read:pipeline:bitbucket`, `write:pipeline:bitbucket`
- `bb issue list`: `read:repository:bitbucket`, `read:issue:bitbucket`
- `bb issue create/update`: `read:repository:bitbucket`, `read:issue:bitbucket`, `write:issue:bitbucket`
- `bb api` against user/workspace endpoints: add `read:user:bitbucket`, `read:workspace:bitbucket`
- `bb completion` / `bb version`: no additional scopes

### Feature-based scope matrix
| Use case | Recommended scopes |
| --- | --- |
| Repo list/read | `read:repository:bitbucket` |
| PR read | `read:repository:bitbucket`, `read:pullrequest:bitbucket` |
| PR create/update | `read:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket` |
| Pipeline read | `read:repository:bitbucket`, `read:pipeline:bitbucket` |
| Pipeline run/update | `read:repository:bitbucket`, `read:pipeline:bitbucket`, `write:pipeline:bitbucket` |
| Issue read | `read:issue:bitbucket` |
| Issue create/update | `read:issue:bitbucket`, `write:issue:bitbucket` |

### Avoid by default
- `admin:*`
- `delete:*`
- `write:permission:bitbucket` unless explicitly required

## Quick Start
1. Login with token:
```bash
bb auth login --token "<your-token>" --profile default
```

Alternative (stdin):
```bash
echo "<your-token>" | bb auth login --with-token --profile default
```

Alternative (env):
```bash
BITBUCKET_TOKEN="<your-token>" bb auth login --profile default
```

2. Check auth:
```bash
bb auth status
```

3. Check version:
```bash
bb version
```

4. List repositories:
```bash
bb repo list --workspace <workspace-slug>
```

5. Call API directly:
```bash
bb api /repositories/<workspace-slug>
bb api --paginate /repositories/<workspace-slug>
```

6. List PRs:
```bash
bb pr list --workspace <workspace-slug> --repo <repo-slug>
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

10. Generate shell completion:
```bash
bb completion zsh
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

## Write Operation Safety
- Use a dedicated test repository for write operations (`bb pr create`, `bb pipeline run`, `bb issue create`, `bb issue update`) during development.
- Keep production repos on read-only tokens unless write access is explicitly needed.

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
Run tests:
```bash
go test ./...
```

Run coverage (internal packages):
```bash
go test -cover ./internal/...
```

For detailed command contracts:
- `docs/command-contracts.md`
