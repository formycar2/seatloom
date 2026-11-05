# Standing-duty verification of the practice I committed to last round — that probes go in my own worktree, not the primary checkout — and the two things the verification found: **it works.** A headerless file staged in my own index is refused by `track`'s gate, exit 1, in my worktree, with the primary checkout untouched. **But the same run exposed that my `.gitlab/docs-grandfather.txt` is a strict superset of `track`'s — 55 paths wider — including `docs/coordination/ROADMAP.md`, which is the file that made the docs gate red on `track` on 11-02.** My branch is carrying grandfather entries the fleet's list does not have, so the same document is deferred debt on my branch and live debt on `track`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-my-grandfather-list-is-55-paths-wider-than-track-s-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, gate, debt |

## What I set out to verify

Last round I recorded that a probe needing to be in an index must go in my own worktree, not the primary checkout,
because I had staged a file in Aegis's index and he traced it as a ghost. **A commitment I do not test is an
intention.** So this round I ran the probe in my own worktree and measured both the mechanism and the side effects.

## The practice works, measured

**Staged in my worktree:** `LYRA-2026-11-06-probe-in-my-own-worktree-v1.md`, `# No header at all`.

**Driven with `track`'s gate, from my worktree:**

```
EXIT=1
```

**And the primary checkout's `git status --porcelain` was untouched throughout** — the file never existed there.
Removed afterwards from my index and worktree; `git status` clean.

**So the corrected practice is real, not stated.** The probe mechanism works from any worktree, which is what I claimed
and had not checked.

## What the same run exposed, which I was not looking for

**Running `track`'s gate against my branch reported four violations**, and my first instinct was that they were the
reverse-range artefact I hit last night. They are not:

| file | first added |
|---|---|
| `docs/coordination/knowledge/i_pushed_to_track_after_cards_opened_and_broke_the_operators_accept.md` | 10-28 |
| `docs/coordination/ROADMAP.md` | 09-30 |
| `docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md` | 10-16 |
| `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` | 10-08 |

**All four are old, none is mine, and all four are in my grandfather list.** Measured with `comm` on the non-comment
lines:

| | |
|---|---|
| paths in my list, not in `track`'s | **55** |
| paths in `track`'s list, not in mine | **0** |

**So my list is a strict superset, 55 paths wider.** And the gate reads whichever list is present in the tree it runs
against — so on my branch those 55 are deferred debt, and on `track` they are live debt.

**`ROADMAP.md` is the concrete case.** On 11-02 I recorded that the docs gate was red on `track` because
`BUDGET_POOL_MODEL_MAP.md` needed a §2 header, and that the only working option was to grandfather it. `ROADMAP.md` is
a different file failing a different rule (§10, CJK), and my branch has it grandfathered while `track` does not.

## Why this matters, and it is not a filing detail

**The grandfather list is the mechanism that lets a gate be both strict and usable.** A path in it is debt the fleet has
decided to defer. **A path in my copy and not in the fleet's is a deferral decision I am effectively making alone** —
and one that my branch will carry into any merge, silently converting 55 live violations into deferred ones.

**That is the same failure family as the `.zhs.md` exemption I recorded yesterday**: a rule and an instrument that
disagree about what they cover, where the instrument is what runs. **Here I am on the wrong side of it** — my
instrument is more permissive than the fleet's, for 55 paths, and I did not know.

**I am not claiming my entries are wrong.** They may be correct deferrals that were never landed. **But they are
decisions, and a decision I have made without knowing I made it is the thing I have been recording all week.**

## What I am doing

**Recording it with its number, and not merging until it is resolved** — because merging my list would land 55
unilateral deferrals on `track`.

**And I am not editing either list.** `track`'s is the fleet's instrument, and adding 55 paths to it is a decision for
the operator, not a seat tidying its own branch.

## What I did not do

I did not edit either grandfather list, the gate, or any script. I did not commit the probe, and removed it from my
index and worktree afterwards. I did not merge, push, accept, cancel, or re-open any card, restart any application, or
write to the live database. My measurements were reads plus one probe with cleanup: staging the probe, `git write-tree`
and `git commit-tree`, the gate run from my worktree, `git log --diff-filter=A` for the four files, `comm` on the
non-comment lines of both lists, and `git status` for restoration. The ledger check is exit 0, `seats` is 10, and
`canonical_events` is **46**.
