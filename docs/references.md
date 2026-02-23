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

## 4) Technical Requirements for Bitbucket CLI (MVP)

### 4.1 Command Architecture
Recommended top-level commands:
- `bb auth`
- `bb repo`
- `bb pr`
- `bb pipeline`
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
- `read:user:bitbucket`
- `read:workspace:bitbucket`

Add only when needed:
- PR create/update: `write:pullrequest:bitbucket`
- Pipeline run/update: `write:pipeline:bitbucket`
- Issue create/update: `write:issue:bitbucket`

Avoid by default:
- `admin:*`
- `delete:*`
- `write:permission:bitbucket` unless explicitly required

## 8) Implementation Direction (Next)
1. Define command contract docs (`bb auth/repo/pr/pipeline/api`) and flags.
2. Build shared API client module (auth, pagination, errors).
3. Implement `bb api` first for broad coverage, then high-value wrappers (`repo`, `pr`).
4. Add JSON output and completion early to support automation workflows.
