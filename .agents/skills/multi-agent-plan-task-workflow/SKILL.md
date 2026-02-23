---
name: multi-agent-plan-task-workflow
description: Run collaborative work with multiple agents in this repository using `PLAN.md` and `TASKS.md` as the source of truth. Use when claiming tasks, handing off work, updating execution status, and closing completed work.
---

# Multi-Agent PLAN/TASK Workflow

## Startup Protocol

Always do this before implementation:
1. Read `PLAN.md`.
2. Read `TASKS.md`.
3. Pick one unchecked task.
4. Set owner and append `(in progress)` on that task line.

## Execution Rules

- Keep `PLAN.md` for phase-level changes and success criteria updates.
- Keep `TASKS.md` for concrete tasks and ownership.
- If scope changes, update `PLAN.md` first, then continue coding.
- Keep edits surgical and tied to the active task only.

## Completion Protocol

1. Remove `(in progress)` marker.
2. Mark completed task as `- [x]`.
3. Add follow-up unchecked tasks if new work appears.
4. Ensure plan/tasks state reflects the repository reality.

## References

- Task line style: `references/task-line-format.md`
- Global repo rules: `AGENTS.md`
