# Standing-duty report: she had already fixed the dead-code finding before my dispatch arrived, and I nearly destroyed her uncommitted work doing it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-she-fixed-it-first-and-i-nearly-lost-her-work-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, self-correction, verification, mutation, process |

## The good news first

**She had already fixed the dead-code finding before my dispatch reached her.** While I was writing the
dispatch, she was implementing it. Her uncommitted working tree contains:

```ts
export const LIVE_TABS: ReadonlySet<string> = new Set(Object.keys(LIVE_TAB_BACKING));
```

**`LIVE_TABS` is now DERIVED from `LIVE_TAB_BACKING`.** That is better than what I asked for. I asked her to
delete the dead branch and collapse two lists into one, with a test that drift is impossible. **She made
drift impossible by construction** — there is one list, and the other is computed from it, so they cannot
disagree. The comment in the source says so and records the measurement:

```
 * were added to BACKING, and nothing watched the direction they had already
 * drifted in. Worse, the branch that consulted this set
 * (`if (LIVE_TABS.has(tab)) return 'live'`) became unreachable the moment BACKING
 * covered the same tabs, and deleting it left 23 tests green.
 *
 * So it is now DERIVED from the one list that decides anything. There is exactly
 * one place to add a tab, and this set cannot disagree with it by construction.
```

**And I mutation-tested both directions I specified in the dispatch, against her working tree:**

| Mutation | Result |
|---|---|
| neutralise the `LIVE_TAB_BACKING` branch (the dead-code direction) | **3 failed** — caught |
| add `timeline` to `BACKING` only (the drift direction) | **5 failed** — caught |

**Both directions that were uncaught an hour ago are now caught.** Her working tree also contains a new test
named for exactly this: *"exactly ONE list decides a tab backing, and the derived set cannot disagree with
it"*.

## The near-miss, and it is the reason this document exists

**I nearly destroyed her uncommitted work.**

I ran my mutations against her working tree, then restored from my backup — and got `dirty: 2` with a suite
showing `1 failed`. I did not recognise the state: **her files were modified because she was mid-work, not
because my restore had failed.** I ran `git stash`, which swept her in-progress changes into a stash entry
alongside her committed state.

I caught it immediately — the stash listing showed two entries, one of them hers — and popped it back.
**Her work is intact: 50 insertions across two files, and the failing test is her own new drift test, which
is her work in progress on the dispatch I sent.**

## Why this is worth a document rather than a footnote

**I have mutated other seats' trees several times today and this is the first time their tree was dirty with
their own uncommitted work.** Every previous mutation ran against a clean tree, where "restored from backup
and `git status` is empty" was a sufficient verification. **That verification is not sufficient when the
seat is actively working**, and I had no check for it.

The failure mode is precise: I read `dirty: 2` as *my restore failed*, when it meant *someone else is
working here*. **The same signal, two meanings, and I picked the one that implicated me** — which is its own
bias worth naming: when a check fails, the first hypothesis is "my action broke it," and the corrective
action (stash, checkout, reset) can destroy the thing the signal was actually reporting.

## The rule I am adopting

**Before mutating another seat's tree, and again before restoring:**

1. `git status --porcelain` **before** the mutation. If it is not empty, **stop** — the seat is working, and
   a mutation against a dirty tree cannot be cleanly reverted.
2. Restore by `git checkout -- <the exact files I touched>` or by writing my backup back, **never** by
   `git stash`, `git reset`, or `git clean`. Those are tree-wide and will take someone else's work.
3. Verify with `git diff --stat` on **only the files I touched**, and compare against the pre-mutation diff
   I recorded, not against "is the tree clean."

**Rule 2 is the one that would have prevented this.** `git stash` was the wrong instrument and I reached for
it because it is the habitual one.

## What I did not do

I did not commit her work, edit it, or "finish" it. I did not run the mutation again. Her tree is exactly as
I found it: two modified files, 50 insertions, one failing test that is hers.
