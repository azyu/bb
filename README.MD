# Bitbucket CLI (Cloud MVP)

## English

### Overview
`bb` is a Go-based Bitbucket Cloud CLI inspired by `gh`-style command grouping.

Current implemented commands:
- `bb auth login`
- `bb auth status`
- `bb api`
- `bb repo list`

Current stub commands (planned):
- `bb pr`
- `bb pipeline`
- `bb issue`
- `bb completion`

### Requirements
- Go `1.26+`

### Build
```bash
go build -o ./bb ./cmd/bb
```

### Install (example: local user bin)
```bash
mkdir -p ~/.local/bin
go build -o ~/.local/bin/bb ./cmd/bb
chmod +x ~/.local/bin/bb
```

### Quick Start
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

### Configuration
Config path priority:
1. `BB_CONFIG_PATH` (explicit override)
2. `$XDG_CONFIG_HOME/bb/config.json` (if `XDG_CONFIG_HOME` is set)
3. `~/.config/bb/config.json` (default)

Backward compatibility:
- If the new default file is missing, the CLI also checks the legacy Go user-config location for `bb/config.json`.

Security note:
- Token is currently stored as plain text in the config file (MVP state).
- Config file is written with `0600` permissions.

### Development
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

---

## 한국어

### 개요
`bb`는 `gh` 스타일의 명령 구조를 참고해 만든 Go 기반 Bitbucket Cloud CLI입니다.

현재 구현된 명령:
- `bb auth login`
- `bb auth status`
- `bb api`
- `bb repo list`

현재 스텁(예정) 명령:
- `bb pr`
- `bb pipeline`
- `bb issue`
- `bb completion`

### 요구사항
- Go `1.26+`

### 빌드
```bash
go build -o ./bb ./cmd/bb
```

### 설치 (예: 사용자 로컬 bin)
```bash
mkdir -p ~/.local/bin
go build -o ~/.local/bin/bb ./cmd/bb
chmod +x ~/.local/bin/bb
```

### 빠른 시작
1. 토큰으로 로그인:
```bash
bb auth login --token "<your-token>" --profile default
```

대안 (stdin):
```bash
echo "<your-token>" | bb auth login --with-token --profile default
```

대안 (환경변수):
```bash
BITBUCKET_TOKEN="<your-token>" bb auth login --profile default
```

2. 인증 상태 확인:
```bash
bb auth status
```

3. 저장소 목록 조회:
```bash
bb repo list --workspace <workspace-slug>
```

4. API 직접 호출:
```bash
bb api /repositories/<workspace-slug>
bb api --paginate /repositories/<workspace-slug>
```

### 설정 파일 위치
우선순위:
1. `BB_CONFIG_PATH` (명시적 override)
2. `$XDG_CONFIG_HOME/bb/config.json` (`XDG_CONFIG_HOME` 설정 시)
3. `~/.config/bb/config.json` (기본값)

호환성:
- 새 기본 경로 파일이 없으면, 레거시 Go user-config 경로의 `bb/config.json`도 함께 확인합니다.

보안 참고:
- 현재 토큰은 설정 파일에 평문으로 저장됩니다(MVP 단계).
- 설정 파일 권한은 `0600`으로 저장됩니다.

### 개발
테스트 실행:
```bash
go test ./...
```

커버리지(내부 패키지) 확인:
```bash
go test -cover ./internal/...
```

명령 계약 상세 문서:
- `docs/command-contracts.md`
