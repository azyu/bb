---
name: bitbucket-cloud-api-playbook
description: Implement and review Bitbucket Cloud REST integration for this repository's CLI. Use when adding or changing API client behavior, authentication, pagination, endpoint mapping, query filters, or API error handling for `bb auth`, `bb repo`, `bb pr`, `bb pipeline`, and `bb api` commands.
---

# Bitbucket Cloud API Playbook

## Core Rules

- Target Bitbucket Cloud only unless user explicitly requests Data Center support.
- Read `docs/references.md` before changing API integration behavior.
- Follow server-provided pagination via `next`; do not construct page URLs manually.
- Keep implementation minimal and command-scoped.

## Workflow

1. Confirm command intent and map it to an endpoint from `references/endpoints.md`.
2. Choose auth input path (token/env/profile) and keep credentials out of logs.
3. Implement request/response mapping with support for `q`, `sort`, and `fields` when relevant.
4. Normalize API errors to stable CLI error messages and exit behavior.
5. Add or update tests that prove request shape and pagination behavior.
6. Re-run local checks and summarize assumptions explicitly.

## Verification Checklist

- Endpoint path matches Bitbucket Cloud REST docs.
- Pagination follows `next` links.
- Human and JSON output both remain usable.
- Error output does not leak tokens or secret material.

## References

- Endpoint map: `references/endpoints.md`
- Error/output guidance: `references/error-contract.md`
- Project baseline: `docs/references.md`
