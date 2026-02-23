# Bitbucket CLI (Cloud MVP)

영문 문서: `README.MD`

## 개요
`bb`는 `gh` 스타일 명령 구성을 참고한 Go 기반 Bitbucket Cloud CLI입니다.

현재 구현된 명령:
- `bb auth login`
- `bb auth status`
- `bb api`
- `bb repo list`
- `bb pr list`
- `bb pr create`
- `bb pipeline list`
- `bb pipeline run`
- `bb issue list`
- `bb completion <bash|zsh|fish|powershell>`
- `bb version` / `bb --version`

예정 하위 명령:
- `bb issue create`

## 요구사항
- Go `1.26+`

## 빌드
```bash
go build -o ./bb ./cmd/bb
```

버전 메타데이터(SemVer + git hash) 포함 빌드:
```bash
VERSION="0.0.1"
COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
BUILD_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
go build -ldflags "-X bitbucket-cli/internal/version.Version=${VERSION} -X bitbucket-cli/internal/version.Commit=${COMMIT} -X bitbucket-cli/internal/version.BuildDate=${BUILD_DATE}" -o ./bb ./cmd/bb
```

## 설치 (예: 사용자 로컬 bin)
```bash
mkdir -p ~/.local/bin
go build -o ~/.local/bin/bb ./cmd/bb
chmod +x ~/.local/bin/bb
```

## Bitbucket API Token 발급 방법
이 CLI는 인증에 Bitbucket 토큰을 사용합니다.

Bitbucket Cloud 기준 단계:
1. Bitbucket Cloud에서 Personal settings로 이동합니다.
2. `API tokens` 메뉴를 엽니다.
3. `Create API token`을 클릭합니다.
4. 토큰 이름을 입력하고 필요한 권한(scope)을 선택합니다.
5. 생성 직후 표시되는 토큰 값을 복사합니다.

공식 문서:
- https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/
- https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/

## 권장 Token Scope
최소 권한 원칙으로 시작하고, 필요한 기능에 맞춰 scope를 추가하세요.

### 일반 개발자 용도 권장 세트
기본 프로파일(안전한 읽기 중심):
- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

PR 생성/수정 또는 Pipeline 실행이 필요하면 추가:
- `write:pullrequest:bitbucket`
- `write:pipeline:bitbucket`

### 현재 CLI 기준
- `bb repo list`: `read:repository:bitbucket`
- `bb pr list`: `read:repository:bitbucket`, `read:pullrequest:bitbucket`
- `bb pr create`: `read:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`
- `bb pipeline list`: `read:repository:bitbucket`, `read:pipeline:bitbucket`
- `bb pipeline run`: `read:repository:bitbucket`, `read:pipeline:bitbucket`, `write:pipeline:bitbucket`
- `bb issue list`: `read:repository:bitbucket`, `read:issue:bitbucket`
- `bb api`로 user/workspace 엔드포인트를 호출할 경우: `read:user:bitbucket`, `read:workspace:bitbucket` 추가
- `bb completion` / `bb version`: 추가 scope 불필요

### 기능별 권장 scope 매트릭스
| 사용 목적 | 권장 scope |
| --- | --- |
| 저장소 조회 | `read:repository:bitbucket` |
| PR 조회 | `read:repository:bitbucket`, `read:pullrequest:bitbucket` |
| PR 생성/수정 | `read:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket` |
| Pipeline 조회 | `read:repository:bitbucket`, `read:pipeline:bitbucket` |
| Pipeline 실행/수정 | `read:repository:bitbucket`, `read:pipeline:bitbucket`, `write:pipeline:bitbucket` |
| 이슈 조회 | `read:issue:bitbucket` |
| 이슈 생성/수정 | `read:issue:bitbucket`, `write:issue:bitbucket` |

### 기본적으로 피할 권한
- `admin:*`
- `delete:*`
- 명시적 필요가 없는 `write:permission:bitbucket`

## 빠른 시작
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

3. 버전 확인:
```bash
bb version
```

4. 저장소 목록 조회:
```bash
bb repo list --workspace <workspace-slug>
```

5. API 직접 호출:
```bash
bb api /repositories/<workspace-slug>
bb api --paginate /repositories/<workspace-slug>
```

6. PR 목록 조회:
```bash
bb pr list --workspace <workspace-slug> --repo <repo-slug>
```

7. PR 생성 (쓰기 scope 필요):
```bash
bb pr create --workspace <workspace-slug> --repo <repo-slug> --title "My PR" --source feature --destination main
```

8. Pipeline 목록 조회:
```bash
bb pipeline list --workspace <workspace-slug> --repo <repo-slug>
```

9. Pipeline 실행 (쓰기 scope 필요):
```bash
bb pipeline run --workspace <workspace-slug> --repo <repo-slug> --branch main
```

10. 셸 자동완성 스크립트 생성:
```bash
bb completion zsh
```

11. 이슈 목록 조회:
```bash
bb issue list --workspace <workspace-slug> --repo <repo-slug>
```

## 쓰기 작업 안전 가이드
- 개발 중 `bb pr create`, `bb pipeline run` 같은 쓰기 명령은 테스트용 저장소에서 먼저 실행하세요.
- 운영 저장소에는 필요 시점 전까지 읽기 전용 토큰 사용을 권장합니다.

## 설정 파일 위치
우선순위:
1. `BB_CONFIG_PATH` (명시적 override)
2. `$XDG_CONFIG_HOME/bb/config.json` (`XDG_CONFIG_HOME` 설정 시)
3. `~/.config/bb/config.json` (기본값)

호환성:
- 새 기본 경로 파일이 없으면, 레거시 Go user-config 경로의 `bb/config.json`도 함께 확인합니다.

보안 참고:
- 현재 토큰은 설정 파일에 평문으로 저장됩니다(MVP 단계).
- 설정 파일 권한은 `0600`으로 저장됩니다.

## 개발
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
