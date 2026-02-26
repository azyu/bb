# Bitbucket CLI (Cloud MVP)

영문 문서: `README.MD`

## 개요
`bb`는 `gh` 스타일 명령 구성을 참고한 Go 기반 Bitbucket Cloud CLI입니다.

현재 구현된 명령:
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

모든 명령은 `--help` / `-h`로 상세 사용법을 확인할 수 있습니다 (`--flag` 형식):
```bash
bb pr --help          # 그룹: 서브커맨드 목록 표시
bb pr create --help   # 리프: 플래그와 설명 표시
```

## 요구사항
- Go `1.26+`

## 빌드
```bash
make build        # -> ./bb (version + commit + date 자동 주입)
make install      # -> $GOPATH/bin/bb
make test         # 전체 테스트 실행
make cover        # 테스트 커버리지 리포트
make help         # 모든 타겟 표시
```

또는 수동 빌드:
```bash
go build -o ./bb ./cmd/bb
```

## Bitbucket API Token 발급 방법
이 CLI는 인증에 Bitbucket 토큰을 사용합니다.

인증 모드:
- API token(개인 토큰): Basic 인증(`username/email + token`) 사용
- Access token 계열: Bearer 토큰 모드(사용자명 없이 토큰만)
- Wiki Git 인증 사용자 매핑:
  - Personal API token 프로파일(이메일 username) -> `x-bitbucket-api-token-auth`
  - Access-token 계열 프로파일(username 비움) -> `x-token-auth`

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
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

PR 생성/수정 또는 Pipeline 실행이 필요하면 추가:
- `write:pullrequest:bitbucket`
- `write:pipeline:bitbucket`
- `write:issue:bitbucket`
- `write:wiki:bitbucket`

### CLI Scope 매트릭스
| 명령/사용 목적 | 권장 scope |
| --- | --- |
| `bb repo list` (저장소 조회) | `read:repository:bitbucket` |
| `bb pr list` (PR 조회) | `read:repository:bitbucket`, `read:pullrequest:bitbucket` |
| `bb pr create` (PR 생성/수정) | `read:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket` |
| `bb pipeline list` (Pipeline 조회) | `read:repository:bitbucket`, `read:pipeline:bitbucket` |
| `bb pipeline run` (Pipeline 실행/수정) | `read:repository:bitbucket`, `read:pipeline:bitbucket`, `write:pipeline:bitbucket` |
| `bb wiki list/get` (Wiki 조회) | `read:repository:bitbucket`, `read:wiki:bitbucket` |
| `bb wiki put` (Wiki 수정) | `read:repository:bitbucket`, `read:wiki:bitbucket`, `write:wiki:bitbucket` |
| `bb issue list` (이슈 조회) | `read:repository:bitbucket`, `read:issue:bitbucket` |
| `bb issue create/update` (이슈 생성/수정) | `read:repository:bitbucket`, `read:issue:bitbucket`, `write:issue:bitbucket` |
| `bb api` (user/workspace 엔드포인트) | `read:user:bitbucket`, `read:workspace:bitbucket` |
| `bb completion`, `bb version` | 추가 scope 불필요 |

### 기본적으로 피할 권한
- `admin:*`
- `delete:*`
- 명시적 필요가 없는 `write:permission:bitbucket`

## 빠른 시작
1. 토큰으로 로그인:
```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>" --profile default
```

대안 (stdin):
```bash
echo "<your-token>" | bb auth login --username "<bitbucket-email>" --with-token --profile default
```

대안 (환경변수):
```bash
BITBUCKET_USERNAME="<bitbucket-email>" BITBUCKET_TOKEN="<your-token>" bb auth login --profile default
```

2. 인증 상태 확인:
```bash
bb auth status
```

선택: 현재 프로파일(또는 특정 프로파일) 로그아웃:
```bash
bb auth logout
bb auth logout --profile <profile-name>
```

3. 버전 확인:
```bash
bb version
```

4. 저장소 목록 조회:
```bash
bb repo list --workspace <workspace-slug>
```
Bitbucket 저장소 디렉토리에서 실행하면 `bb repo list`는 `remote.origin.url`에서 `<workspace>`를 자동 추론할 수 있습니다.

5. API 직접 호출:
```bash
bb api /repositories/<workspace-slug>
bb api --paginate /repositories/<workspace-slug>
```

6. PR 목록 조회:
```bash
bb pr list --workspace <workspace-slug> --repo <repo-slug>
```
`table` 출력은 `gh pr list`와 유사한 형태(요약 라인 + `ID/TITLE/BRANCH/CREATED AT`)로 표시됩니다.
터미널 출력에서는 ANSI 색상이 적용되며 `BB_COLOR=always|never` 또는 `NO_COLOR=1`로 제어할 수 있습니다.
Bitbucket 저장소 디렉토리에서 실행하면 repo 대상 명령은 `remote.origin.url`에서 `<workspace>/<repo>`를 자동 추론할 수 있습니다:
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

10. 셸 자동완성 스크립트 생성 (서브커맨드 자동완성 지원):
```bash
bb completion zsh
# bash/zsh/fish/powershell 지원; 2단계 서브커맨드 자동완성 포함
# 예: `bb pr<TAB>` -> list, create
```

11. 이슈 목록 조회:
```bash
bb issue list --workspace <workspace-slug> --repo <repo-slug>
```

12. 이슈 생성 (쓰기 scope 필요):
```bash
bb issue create --workspace <workspace-slug> --repo <repo-slug> --title "My issue" --content "details"
```

13. 이슈 수정 (쓰기 scope 필요):
```bash
bb issue update --workspace <workspace-slug> --repo <repo-slug> --id 1 --state resolved
```

14. Wiki 파일 목록 조회:
```bash
bb wiki list --workspace <workspace-slug> --repo <repo-slug>
```

15. Wiki 페이지 읽기:
```bash
bb wiki get --workspace <workspace-slug> --repo <repo-slug> --page Home.md
```

16. Wiki 페이지 수정 (쓰기 scope 필요):
```bash
bb wiki put --workspace <workspace-slug> --repo <repo-slug> --page Home.md --content "# Updated"
```

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
```bash
make test         # 전체 테스트 실행
make cover        # 내부 패키지 커버리지 리포트
make lint         # go vet
make fmt          # gofmt
```

명령 계약 상세 문서:
- `docs/command-contracts.md`
