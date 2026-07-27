# GitHub Issue Format

Issues live on `azyu/bb-cli`. This repository uses no issue templates, so follow this shape by hand.

## Title

Imperative, no prefix, no issue-type tag. The labels carry that information.

- `Add bb pr diffstat and diff --name-only`
- `Evaluate phase 2 agent-first extensions`

Avoid: `[FEATURE] diffstat`, `bb pr diffstat?`, `fix stuff`.

## Body

Three sections, in this order. Keep each short.

```markdown
## Goal
What should be true when this is closed. One or two sentences.

## Context
Why now. Link the code, spec section, PR, or issue that prompted it.

## Done when
- Observable criteria, not activities.
- For behavior changes, name the test that must pass.
```

For an evaluation issue (no implementation committed yet), replace `Done when` with `Outcome`, stating which decision closes it and where the decision gets recorded.

## Labels

- `backlog` — actionable, in the queue. Every open work issue gets this.
- `archive` — historical record, not actionable. Closed on creation.
- `bug`, `enhancement`, `documentation` — kind of work. Combine with `backlog`.

Do not invent status labels. Status lives in assignee plus comments.

## Ownership and status

- Owner = assignee. Claim with `gh issue edit <number> --add-assignee @me`.
- Status is a comment, not a label and not a body edit:
  - on claim: `Starting: <one-line plan>`
  - on block: `Blocked: <what is blocking> — <what would unblock it>`

## Closing comment

Before `gh issue close <number> --reason completed`, comment:

```markdown
Done in <commit-sha or #pr-number>.

Verified:
- `cargo test --manifest-path rust/Cargo.toml` — 99 passed
- <any other command actually run>

Not run: <command> — <reason>

Follow-ups: #<number>, #<number>
```

State skipped checks explicitly. A closing comment that omits what was not run is wrong.
