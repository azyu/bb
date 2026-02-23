# Bitbucket CLI (Cloud MVP)

| English | [한국어](README.ko-kr.MD) |

## Overview
`bb` is a Go-based Bitbucket Cloud CLI inspired by `gh`-style command grouping.

Implemented commands:
- `bb auth login`
- `bb auth status`
- `bb api`
- `bb repo list`

Planned (stub) commands:
- `bb pr`
- `bb pipeline`
- `bb issue`
- `bb completion`

## Requirements
- Go `1.26+`

## Build
```bash
go build -o ./bb ./cmd/bb
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

3. List repositories:
```bash
bb repo list --workspace <workspace-slug>
```

4. Call API directly:
```bash
bb api /repositories/<workspace-slug>
bb api --paginate /repositories/<workspace-slug>
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
