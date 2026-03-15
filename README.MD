# bb

[![CI](https://github.com/azyu/bb-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/azyu/bb-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/azyu/bb-cli)](https://github.com/azyu/bb-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.MD) | [한국어](README.ko-kr.MD)

> A fast, agent-friendly Bitbucket Cloud CLI inspired by `gh`, built in Rust.

## Features

- `gh`-style command groups for repos, PRs, pipelines, issues, wiki, and raw API calls
- Repo inference from local Bitbucket `remote.origin.url`
- Human-friendly text/table output plus stable JSON output for automation
- Shell completion for bash, zsh, fish, and PowerShell
- Cross-platform release binaries for Linux, macOS, and Windows

## Installation

### Homebrew

```bash
brew install azyu/tap/bb
```

### Prebuilt binaries

Download the latest archive from [GitHub Releases](https://github.com/azyu/bb-cli/releases/latest).

| Platform | Asset |
|----------|-------|
| Linux amd64 | `bb_0.x.y_linux_amd64.tar.gz` |
| Linux arm64 | `bb_0.x.y_linux_arm64.tar.gz` |
| macOS arm64 | `bb_0.x.y_macos_arm64.tar.gz` |
| Windows x64 | `bb_0.x.y_windows_x64.zip` |
| Windows arm64 | `bb_0.x.y_windows_arm64.zip` |

### From source

Requires Rust 1.93+.

```bash
make install
```

## Quick Start

### 1. Authenticate

```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>"
```

> [!TIP]
> You can also read the token from stdin or the environment.
> ```bash
> echo "<token>" | bb auth login --username "<email>" --with-token
> BITBUCKET_USERNAME="<email>" BITBUCKET_TOKEN="<token>" bb auth login
> ```

### 2. Verify the active profile

```bash
bb auth status
```

### 3. Run common commands

Inside a Bitbucket repository, `bb` can infer `--workspace` and `--repo` from `remote.origin.url`.

```bash
bb pr list
bb pr comments 123
bb pr comments 123 --comment-id 456 --output json
bb pr create --title "My PR" --source feature --destination main
bb pipeline list
bb pipeline get --uuid "{pipeline-uuid}"
```

### 4. Enable shell completion

```bash
bb completion zsh
```

## Command Overview

| Group | Subcommands |
|-------|-------------|
| `bb auth` | `login`, `status`, `logout` |
| `bb repo` | `list` |
| `bb pr` | `list`, `get`, `create`, `update`, `merge`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comments`, `diff`, `statuses`, `activity` |
| `bb pipeline` | `list`, `get`, `steps`, `log`, `run` |
| `bb issue` | `list`, `create`, `update` |
| `bb wiki` | `list`, `get`, `put` |
| `bb api` | raw Bitbucket Cloud endpoint access |
| `bb completion` | `bash`, `zsh`, `fish`, `powershell` |
| `bb version` | build metadata |

> [!NOTE]
> Existing-PR commands accept the pull request ID as positional `<id>` or `--id`.
>
> Example: `bb pr comments 123` and `bb pr comments --id 123` are both valid.
>
> For `bb pr comments`, `--comment-id` selects a single comment within that pull request. Do not pass a comment ID via `--id`.
>
> When an agent only needs part of a JSON payload, combine `--output json` with `--json-fields`.

## Configuration and Auth

Config path priority:

1. `BB_CONFIG_PATH`
2. `$XDG_CONFIG_HOME/bb/config.json`
3. `~/.config/bb/config.json`

Auth modes:

| Mode | Scheme | Username required |
|------|--------|-------------------|
| Personal API token | Basic (`username:token`) | Yes |
| Access token | Bearer | No |

> [!CAUTION]
> Tokens are stored in plain text in the local config file. Keep file permissions tight.

For token creation and permission details, see the Atlassian docs:

- [Create an API token](https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/)
- [API token permissions](https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/)

## Scope Guide

Recommended read-only baseline:

- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

Add `write:*` scopes only for mutating commands.

<details>
<summary>Command scope matrix</summary>

| Command | Required scopes |
|---------|-----------------|
| `bb repo list` | `read:repository` |
| `bb pr list/get/comments/diff/statuses/activity` | `read:repository`, `read:pullrequest` |
| `bb pr create/update/merge/approve/unapprove/request-changes/remove-request-changes/decline/comment` | `read:repository`, `read:pullrequest`, `write:pullrequest` |
| `bb pipeline list/get/steps/log` | `read:repository`, `read:pipeline` |
| `bb pipeline run` | `read:repository`, `read:pipeline`, `write:pipeline` |
| `bb wiki list/get` | `read:repository`, `read:wiki` |
| `bb wiki put` | `read:repository`, `read:wiki`, `write:wiki` |
| `bb issue list` | `read:repository`, `read:issue` |
| `bb issue create/update` | `read:repository`, `read:issue`, `write:issue` |
| `bb api` | varies by endpoint |
| `bb completion`, `bb version` | none |

</details>

## Developer Docs

- [Technical spec](docs/spec.md)
- [Command contracts](docs/command-contracts.md)
