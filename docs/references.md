# Bitbucket CLI Reference Research

## 1) Scope and Assumptions
- Target platform: **Bitbucket Cloud** (based on Atlassian Cloud REST docs)
- Goal: Build a CLI with a UX similar to `gh` and `tea`
- Initial phase focus: **MVP command set + API client foundation**

## 2) Benchmark CLIs

### GitHub CLI (`gh`)
- Official site: https://cli.github.com/
- Manual index: https://cli.github.com/manual/gh

Observed structural patterns to reuse:
- Command groups are clearly separated (`auth`, `repo`, `pr`, `api`, `completion`, ...)
- API escape hatch command exists (`gh api`) for unsupported or advanced flows
- Consistent auth UX (`auth login`, token env vars, profile-like behavior)
- Shell completion as a first-class feature
- Human-readable output plus machine-readable output pathways

Useful docs:
- `gh auth login`: https://cli.github.com/manual/gh_auth_login
- `gh api`: https://cli.github.com/manual/gh_api
- `gh completion`: https://cli.github.com/manual/gh_completion
- environment variables: https://cli.github.com/manual/gh_help_environment
- extension model: https://cli.github.com/manual/gh_extension

### Gitea CLI (`tea`)
- Project: https://gitea.com/gitea/tea/
- CLI docs mirror: https://git.nroo.de/mirrors/tea/src/branch/main/docs/CLI.md

Observed structural patterns to reuse:
- Multi-login/profile approach
- Local Git context integration for auto target repo inference
- Global flags reused across commands (`--login`, `--repo`, output-related flags)
- Practical command naming and predictable hierarchy

## 3) Bitbucket Cloud API Reference
- REST entry: https://developer.atlassian.com/cloud/bitbucket/rest/
- Intro: https://developer.atlassian.com/cloud/bitbucket/rest/intro/

Core API groups for CLI MVP:
- Workspaces: https://developer.atlassian.com/cloud/bitbucket/rest/api-group-workspaces/
- Repositories: https://developer.atlassian.com/cloud/bitbucket/rest/api-group-repositories/
- Pull Requests: https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/
- Pipelines: https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pipelines/
- Issues: https://developer.atlassian.com/cloud/bitbucket/rest/api-group-issue-tracker/
- Wiki (git-based operations): https://support.atlassian.com/bitbucket-cloud/docs/set-up-and-use-wiki-in-bitbucket-cloud/
- Wiki clone/update reference: https://support.atlassian.com/bitbucket-cloud/docs/view-and-configure-a-repositorys-wiki/

## 4) Technical Requirements for Bitbucket CLI (MVP)

### 4.1 Command Architecture
Recommended top-level commands:
- `bb auth`
- `bb repo`
- `bb pr`
- `bb pipeline`
- `bb wiki`
- `bb issue`
- `bb api`
- `bb completion`

Rationale:
- Mirrors successful `gh` mental model
- Keeps direct API command (`bb api`) for rapid coverage
- Allows incremental feature growth without breaking CLI shape

### 4.2 Authentication Layer
Support in design:
- API token-based auth
- Workspace/Project/Repository access token compatibility
- OAuth 2.0 support path

Design note:
- Avoid designing around deprecated auth paths as default behavior.
- API token usage should support Basic auth (`username/email + token`).
- Keep Bearer token mode for token types that require it.
- For wiki Git operations, auth user may differ from REST auth user:
  - Personal API token profiles (REST uses email) should use `x-bitbucket-api-token-auth` for wiki Git.
  - Access-token-style profiles should use `x-token-auth` for wiki Git.

### 4.3 API Client Behavior
Required client capabilities:
- Pagination support (`values`, `next`, `pagelen` model)
- Follow server-provided `next` link, not manual page URL construction
- Common query parameters support (`q`, `sort`, `fields`)
- Structured error handling for API failures and rate/permission issues

### 4.4 Local Git Context Mapping
Required behavior:
- Infer `{workspace}/{repo_slug}` from local Git remote
- Allow explicit override with flags (e.g., `--workspace`, `--repo`)
- Keep non-interactive scripts deterministic via explicit flags

### 4.5 Output and UX
MVP output modes:
- Human mode (table/concise text)
- JSON mode for automation

Operational UX:
- Global `--verbose` / `--debug`
- Shell completion generation
- Stable exit codes for CI usage

## 5) Suggested MVP Endpoint Mapping
- `GET /user/workspaces`
- `GET /repositories/{workspace}`
- `GET /repositories/{workspace}/{repo_slug}/pullrequests`
- `POST /repositories/{workspace}/{repo_slug}/pullrequests`
- `GET /repositories/{workspace}/{repo_slug}/pipelines`
- `POST /repositories/{workspace}/{repo_slug}/pipelines`
- `GET /repositories/{workspace}/{repo_slug}/issues`
- Wiki operations via Git remote:
  - `https://bitbucket.org/{workspace}/{repo_slug}.git/wiki`

## 6) Risks and Boundaries
- Bitbucket Cloud and Bitbucket Data Center APIs differ significantly.
- To avoid scope explosion, keep first release **Cloud-only**.
- If Data Center support is needed later, split transport/auth/config logic by backend type.

## 7) Token Scope Strategy (Bitbucket Cloud)

Principle:
- Use least-privilege scopes and separate read-only/write tokens when possible.

General developer preset (recommended):
- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

Add only when needed:
- PR create/update: `write:pullrequest:bitbucket`
- Pipeline run/update: `write:pipeline:bitbucket`
- Issue create/update: `write:issue:bitbucket`
- Wiki create/update: `write:wiki:bitbucket`
- During development, run write-scope flows against a dedicated test repository/workspace first.

Avoid by default:
- `admin:*`
- `delete:*`
- `write:permission:bitbucket` unless explicitly required

## 8) Implementation Status (2026-02-23)

Implemented:
- Shared API client with token auth and pagination (`next` traversal)
- Optional Basic auth mode via profile username (`bb auth login --username` / `BITBUCKET_USERNAME`)
- `bb auth login`, `bb auth status`, `bb auth logout`
- `bb api`
- `bb repo list`
- `bb pr list`, `bb pr create`
- `bb pr list` supports local Git `origin` inference for Bitbucket remotes when `--workspace/--repo` are omitted
- `bb pipeline list`, `bb pipeline run`
- `bb wiki list`, `bb wiki get`, `bb wiki put` (git-based wiki repository operations)
- `bb issue list`, `bb issue create`, `bb issue update`
- `bb completion <bash|zsh|fish|powershell>`
- `bb version` / `bb --version` and root help version display

Remaining wrappers:
- None in current MVP command set

## 9) Implementation Direction (Next)
1. Improve ergonomics (global debug flag, optional local Git remote inference).
2. Add stronger secret handling for git-based wiki auth flows (avoid credential exposure in process args).
3. Harden auth storage strategy beyond plaintext config for post-MVP.

## 10) Versioning Strategy

- Adopt SemVer as the canonical release version.
- Attach short git hash as build metadata for traceability.
  - Example format: `0.0.1+abc1234`
- Expose version information via:
  - `bb version`
  - `bb --version`
  - root help output when running `bb` with no args
- Build-time injection fields:
  - `bitbucket-cli/internal/version.Version`
  - `bitbucket-cli/internal/version.Commit`
  - `bitbucket-cli/internal/version.BuildDate`
