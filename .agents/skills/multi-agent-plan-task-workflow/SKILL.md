---
name: multi-agent-plan-task-workflow
description: Run collaborative work with multiple agents in this repository using `.context/STEERING.md` and GitHub Issues on `azyu/bb-cli` as the source of truth. Use when claiming tasks, handing off work, updating execution status, and closing completed work.
---

# Multi-Agent STEERING/Issue Workflow

## Startup Protocol

Always do this before implementation:
1. Read `.context/STEERING.md`.
2. List the actionable queue:
   ```bash
   gh issue list -R azyu/bb-cli --state open --label backlog
   ```
3. Pick one unassigned issue and read it in full:
   ```bash
   gh issue view <number> -R azyu/bb-cli --comments
   ```
4. Claim it before touching code:
   ```bash
   gh issue edit <number> -R azyu/bb-cli --add-assignee @me
   gh issue comment <number> -R azyu/bb-cli --body "Starting: <one-line plan>"
   ```

Do not start work that has no issue. Open one first.

## Execution Rules

- Keep `.context/STEERING.md` for phase-level changes and success criteria updates.
- Keep GitHub Issues for concrete tasks, ownership, status, and blockers.
- An issue's assignee is its owner. Do not work on an issue assigned to someone else.
- Record status changes and blockers as issue comments, not as edits to the issue body. The body stays the statement of work.
- If scope changes, update `.context/STEERING.md` first, then continue coding.
- Keep edits surgical and tied to the active issue only.

## Completion Protocol

1. Comment the verification commands actually run and their results:
   ```bash
   gh issue comment <number> -R azyu/bb-cli --body-file <notes>.md
   ```
2. Close the issue:
   ```bash
   gh issue close <number> -R azyu/bb-cli --reason completed
   ```
3. If new work appeared, open follow-up issues with the `backlog` label and cross-link them by number in both directions. GitHub has no parent/child issues in this repository, so the cross-links are the only trail.
4. Ensure `.context/STEERING.md` and the issue state reflect repository reality.

## References

- Issue format: `references/issue-format.md`
- Global repo rules: `AGENTS.md`
