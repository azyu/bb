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

Do not invent status labels. Status lives in comments.

## Ownership and status

- Owner is the agent named in the most recent `Claimed by` comment. Assignee is a coarser signal: several assistants may share one authenticated `gh` account, so `@me` says which account holds the issue, not which agent.
- Claim with both, in this order, then re-read the comments to confirm yours is the newest claim:
  ```bash
  gh issue edit <number> -R azyu/bb-cli --add-assignee @me
  gh issue comment <number> -R azyu/bb-cli --body "Claimed by <agent-name>: <one-line plan>"
  gh issue view <number> -R azyu/bb-cli --comments
  ```
  If a competing claim landed after yours, release it and take another issue. The re-read catches the common interleaving; it is not a lock.
- Status is a comment, not a label and not a body edit:
  - on block: `Blocked by <agent-name>: <what is blocking> — <what would unblock it>`
  - on abandon: `Released by <agent-name>: <reason>`

## Closing comment

Closing takes two comments, because the commit or PR that finishes the work does not exist yet at the point where `AGENTS.md` requires the issue to be current.

**Before committing** — post the result, without a reference you cannot have yet:

```markdown
Work complete, pending commit.

Verified:
- `cargo test --manifest-path rust/Cargo.toml` — 99 passed
- <any other command actually run>

Not run: <command> — <reason>
```

**After the commit or PR exists** — post the reference:

```bash
gh issue comment <number> -R azyu/bb-cli --body "Done in <commit-sha or #pr-number>. Follow-ups: #<number>"
```

If there are no follow-ups, drop that sentence rather than writing `none`.

**Once the work has landed** — close it:

```bash
gh issue close <number> -R azyu/bb-cli --reason completed
```

Landed means the commit is on the branch, or the PR merged. Do not close at PR creation: the PR can still fail CI or come back from review, and the open `backlog` queue is the only signal that the work is unfinished.

State skipped checks explicitly. A closing comment that omits what was not run is wrong.
