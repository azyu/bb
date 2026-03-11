# bb

[![CI](https://github.com/azyu/bb-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/azyu/bb-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/azyu/bb-cli)](https://github.com/azyu/bb-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.MD) | [한국어](README.ko-kr.MD)

> `gh`에서 영감을 받은, 빠르고 에이전트 친화적인 Bitbucket Cloud CLI. Rust로 작성되었습니다.

## 특징

- repo, PR, pipeline, issue, wiki, raw API 호출을 `gh` 스타일 그룹으로 제공
- 로컬 Bitbucket `remote.origin.url`에서 repo context 자동 추론
- 사람용 text/table 출력과 자동화용 JSON 출력을 함께 지원
- bash, zsh, fish, PowerShell용 셸 자동완성 제공
- Linux, macOS, Windows용 릴리즈 바이너리 제공

## 설치

### Homebrew

```bash
brew install azyu/tap/bb
```

### 미리 빌드된 바이너리

[GitHub Releases](https://github.com/azyu/bb-cli/releases/latest)에서 최신 아카이브를 다운로드하세요.

| 플랫폼 | 에셋 |
|--------|------|
| Linux amd64 | `bb_0.x.y_linux_amd64.tar.gz` |
| Linux arm64 | `bb_0.x.y_linux_arm64.tar.gz` |
| macOS arm64 | `bb_0.x.y_macos_arm64.tar.gz` |
| Windows x64 | `bb_0.x.y_windows_x64.zip` |
| Windows arm64 | `bb_0.x.y_windows_arm64.zip` |

### 소스에서 빌드

Rust 1.93+가 필요합니다.

```bash
make install
```

## 빠른 시작

### 1. 인증

```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>"
```

> [!TIP]
> stdin이나 환경변수로도 토큰을 전달할 수 있습니다.
> ```bash
> echo "<token>" | bb auth login --username "<email>" --with-token
> BITBUCKET_USERNAME="<email>" BITBUCKET_TOKEN="<token>" bb auth login
> ```

### 2. 활성 프로필 확인

```bash
bb auth status
```

### 3. 자주 쓰는 명령 실행

Bitbucket 저장소 안에서는 `remote.origin.url`에서 `--workspace`와 `--repo`를 자동으로 추론할 수 있습니다.

```bash
bb pr list
bb pr comments 123
bb pr create --title "My PR" --source feature --destination main
bb pipeline list
bb pipeline get --uuid "{pipeline-uuid}"
```

### 4. 셸 자동완성 활성화

```bash
bb completion zsh
```

## 명령 개요

| 그룹 | 서브커맨드 |
|------|-----------|
| `bb auth` | `login`, `status`, `logout` |
| `bb repo` | `list` |
| `bb pr` | `list`, `get`, `create`, `update`, `merge`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comments`, `diff`, `statuses`, `activity` |
| `bb pipeline` | `list`, `get`, `steps`, `log`, `run` |
| `bb issue` | `list`, `create`, `update` |
| `bb wiki` | `list`, `get`, `put` |
| `bb api` | Bitbucket Cloud raw endpoint 호출 |
| `bb completion` | `bash`, `zsh`, `fish`, `powershell` |
| `bb version` | 빌드 메타데이터 출력 |

> [!NOTE]
> 기존 PR을 대상으로 하는 명령은 PR ID를 positional `<id>` 또는 `--id`로 받을 수 있습니다.
>
> 예: `bb pr comments 123`, `bb pr comments --id 123`
>
> 에이전트가 JSON 전체가 아니라 일부 필드만 필요하면 `--output json`과 `--json-fields`를 함께 사용하세요.

## 설정과 인증

설정 파일 경로 우선순위:

1. `BB_CONFIG_PATH`
2. `$XDG_CONFIG_HOME/bb/config.json`
3. `~/.config/bb/config.json`

인증 모드:

| 모드 | 방식 | 사용자명 필요 |
|------|------|---------------|
| Personal API token | Basic (`username:token`) | 예 |
| Access token | Bearer | 아니오 |

> [!CAUTION]
> 토큰은 로컬 설정 파일에 평문으로 저장됩니다. 파일 권한을 엄격하게 관리하세요.

토큰 생성과 권한 문서는 Atlassian 공식 문서를 참고하세요.

- [Create an API token](https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/)
- [API token permissions](https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/)

## 권한 가이드

권장 읽기 전용 기본 세트:

- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

변경 명령을 사용할 때만 `write:*` 권한을 추가하세요.

<details>
<summary>명령별 권한 매트릭스</summary>

| 명령 | 필요 권한 |
|------|-----------|
| `bb repo list` | `read:repository` |
| `bb pr list/get/comments/diff/statuses/activity` | `read:repository`, `read:pullrequest` |
| `bb pr create/update/merge/approve/unapprove/request-changes/remove-request-changes/decline/comment` | `read:repository`, `read:pullrequest`, `write:pullrequest` |
| `bb pipeline list/get/steps/log` | `read:repository`, `read:pipeline` |
| `bb pipeline run` | `read:repository`, `read:pipeline`, `write:pipeline` |
| `bb wiki list/get` | `read:repository`, `read:wiki` |
| `bb wiki put` | `read:repository`, `read:wiki`, `write:wiki` |
| `bb issue list` | `read:repository`, `read:issue` |
| `bb issue create/update` | `read:repository`, `read:issue`, `write:issue` |
| `bb api` | 엔드포인트에 따라 다름 |
| `bb completion`, `bb version` | 없음 |

</details>

## 개발자 문서

- [Technical spec](docs/SPEC.md)
- [Command contracts](docs/command-contracts.md)
