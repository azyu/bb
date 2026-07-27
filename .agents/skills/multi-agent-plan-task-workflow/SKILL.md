---
name: multi-agent-plan-task-workflow
description: Run collaborative work with multiple agents in this repository using `.context/STEERING.md` and GitHub Issues on `azyu/bb-cli` as the source of truth. Use when claiming tasks, handing off work, updating execution status, and closing completed work.
---

# Multi-Agent STEERING/Issue Workflow

## Startup Protocol

Always do this before implementation:
1. Read `.context/STEERING.md`.
2. Determine which issue you are working on:
   - If an issue number was given, use it.
   - If you already hold an open claim on an issue, resume that one.
   - Otherwise pick one from the queue that no agent has claimed:
     ```bash
     gh issue list -R azyu/bb-cli --state open --label backlog
     ```
3. Read it in full, including comments, so you see existing claims:
   ```bash
   gh issue view <number> -R azyu/bb-cli --comments
   ```
4. Claim it before touching code, naming yourself in the comment:
   ```bash
   gh issue edit <number> -R azyu/bb-cli --add-assignee @me
   gh issue comment <number> -R azyu/bb-cli --body "Claimed by <agent-name>: <one-line plan>"
   ```

Do not start work that has no issue. Open one first.

## Execution Rules

- Keep `.context/STEERING.md` for phase-level changes and success criteria updates.
- Keep GitHub Issues for concrete tasks, ownership, status, and blockers.
- Ownership is the most recent `Claimed by <agent-name>` comment, not the assignee. Several assistants can share one authenticated `gh` account, so `@me` identifies the account, not the agent. Do not work on an issue another agent has claimed and not released.
- To release a claim you are abandoning, comment `Released by <agent-name>: <reason>` so the next agent can take it.
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
