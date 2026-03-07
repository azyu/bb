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

## 2.1) Agent-Oriented CLI Reference

Reference:
- Justin Poehnelt, "Improving Command-Line Programs for LLMs": https://justin.poehnelt.com/posts/llm-friendly-cli-programs/
- Korean summary/discussion: https://news.hada.io/topic?id=27246

Observed patterns to reuse:
- Harden input handling so conflicting or ambiguous flags fail before side effects.
- Keep a machine-readable surface explicit instead of forcing agents to scrape prose.
- Support flexible formatting per client (`text`/`table` for humans, `json` for automation).
- Treat introspection and dry-run support as valuable follow-up features for discovery and safe execution.

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
- `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}`
- `PUT /repositories/{workspace}/{repo_slug}/pullrequests/{id}`
- `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/merge`
- `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/approve`
- `DELETE /repositories/{workspace}/{repo_slug}/pullrequests/{id}/approve`
- `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/request-changes`
- `DELETE /repositories/{workspace}/{repo_slug}/pullrequests/{id}/request-changes`
- `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/decline`
- `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/comments`
- `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/comments`
- `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/diff`
- `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/statuses`
- `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/activity`
- `GET /repositories/{workspace}/{repo_slug}/pipelines`
- `POST /repositories/{workspace}/{repo_slug}/pipelines`
- `GET /repositories/{workspace}/{repo_slug}/issues`
- Wiki operations via Git remote:
  - `https://bitbucket.org/{workspace}/{repo_slug}.git/wiki`

Command naming rule for wrapper commands:
- When a CLI subcommand is a thin wrapper over a Bitbucket Cloud REST operation, prefer the Bitbucket API operation/resource name in the CLI (`get`, `update`, `request-changes`, `remove-request-changes`) instead of local synonyms like `view` or `edit`.

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

## 8) Implementation Status (2026-03-08)

Current repository state:
- The Rust rewrite is now the only implementation in this repository.
- The legacy Go source tree and Go build surface have been removed after Rust verification.

Rust migration decisions:
- Toolchain target: Rust
- Workspace shape: `bb-cli` + `bb-core`
- Public binary name remains `bb`
- Phase 1 established the MVP command set, and current follow-up work expands only the PR area:
  - `bb auth login|status|logout`
  - `bb api`
  - `bb repo list`
  - `bb pr list|create|merge|get|update|approve|unapprove|request-changes|remove-request-changes|decline|comment|comments|diff|statuses|activity`
  - `bb pipeline list|run`
  - `bb issue list|create|update`
  - `bb wiki list|get|put`
  - `bb completion`
  - `bb version`
- Command naming for direct PR wrappers follows Bitbucket Cloud API titles (`get`, `update`, `request-changes`, `remove-request-changes`) instead of older wrapper synonyms (`view`, `edit`).
- `bb pr checkout` remains out of scope because it requires local Git workflow design beyond a direct REST wrapper.
- Go config/runtime compatibility is intentionally dropped for the Rust rewrite.

Behavior preserved by the Rust rewrite:
- Shared API client behavior with token auth and `next`-link pagination
- Optional Basic auth mode via profile username (`bb auth login --username` / `BITBUCKET_USERNAME`)
- Repo-scoped local Git `origin` inference for Bitbucket remotes
- `bb pr list` table output shape and color controls (`BB_COLOR` / `NO_COLOR`)
- `bb completion <bash|zsh|fish|powershell>`
- `bb version` / `bb --version` and root help version display

Current agent-oriented alignment:
- The Rust MVP keeps JSON success/error contracts for automation-facing commands.
- Command parsing rejects invalid combinations before network or git write operations.
- Structured passthrough parameters (`q`, `sort`, `fields`) are preserved for precise automation.
- Schema introspection and dry-run support remain explicit phase 2 candidates rather than implicit scope creep in MVP.

## 9) Implementation Direction (Next)
1. Harden release packaging across additional OS/arch targets if distribution expands beyond local/personal use.
2. Evaluate agent-first extensions such as schema introspection, dry-run support, or a separate automation surface.
3. Keep the Cloud MVP contract stable while adding any post-MVP commands.

## 10) Versioning Strategy

- Adopt SemVer as the canonical release version.
- Attach short git hash as build metadata for traceability.
  - Example format: `0.0.1+abc1234`
- Expose version information via:
  - `bb version`
  - `bb --version`
  - root help output when running `bb` with no args
- Build-time injection inputs:
  - Cargo package version
  - `BB_BUILD_COMMIT`
  - `BB_BUILD_DATE`
