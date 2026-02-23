---
name: bitbucket-cli-command-standards
description: Define and enforce command contracts for this Bitbucket CLI project. Use when creating or reviewing command names, flags, defaults, output modes, help text, and exit behavior across `bb` subcommands.
---

# Bitbucket CLI Command Standards

## Scope

Use this skill when changing CLI UX shape, including:
- New command groups or subcommands
- Flag naming and defaults
- Output formatting behavior
- Help text and examples
- Exit codes and error messaging

## Command Contract Rules

- Keep top-level command groups aligned with current plan: `auth`, `repo`, `pr`, `pipeline`, `issue`, `api`, `completion`.
- Use explicit flags for non-deterministic context (`--workspace`, `--repo`).
- Keep human output concise and provide JSON mode for automation.
- Avoid introducing flags/options not required by the current task.

## Review Workflow

1. Write command purpose in one sentence.
2. Define minimal required flags and optional flags.
3. Define output behavior in human mode and JSON mode.
4. Define failure behavior and expected exit semantics.
5. Verify naming consistency with existing command patterns.

## References

- Contract template: `references/command-contract-template.md`
- Project baseline: `docs/references.md`
