# Bitbucket CLI (Cloud MVP)

영문 문서: `README.MD`

## 개요
`bb`는 `gh` 스타일 명령 구성을 참고한 Go 기반 Bitbucket Cloud CLI입니다.

현재 구현된 명령:
- `bb auth login`
- `bb auth status`
- `bb api`
- `bb repo list`

예정(스텁) 명령:
- `bb pr`
- `bb pipeline`
- `bb issue`
- `bb completion`

## 요구사항
- Go `1.26+`

## 빌드
```bash
go build -o ./bb ./cmd/bb
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

3. 저장소 목록 조회:
```bash
bb repo list --workspace <workspace-slug>
```

4. API 직접 호출:
```bash
bb api /repositories/<workspace-slug>
bb api --paginate /repositories/<workspace-slug>
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
