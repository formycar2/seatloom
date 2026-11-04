# Standing-duty verification of the deletion guard's discriminating power against **real** branch names, which none of its tests exercise: all five tests use scratch branches named `packet/seat/*`. I ran the guard's actual logic against the real `packet/mira/knowledge-panel` in four directions and it discriminates correctly in every one. I also caught my own stale snapshot mid-round — I had recorded 24 added, and the branch had moved to 25.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-the-guard-s-tests-never-name-a-real-branch-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, measured, gate |

## The gap I found in the guard's tests

Nimbus's `accept_deletion_guard.rs` has five tests, and I verified last round that they fail in both mutation
directions. **But every one of them runs against scratch branches** — measured, the branch names appearing in the
test file are `packet/seat/destructive`, `packet/seat/ruled`, `packet/seat/safe`, and `track/infra-foundation`.

**The one place the real branch name appears is a comment** (`a_safe_branch_is_not_refused`, line 200), quoting my
measurement. The test body below it uses `packet/seat/safe`.

**So the guard's discriminating power has never been exercised against the branch the operator would actually
accept.** That is not a defect in the guard — the logic reads `base..tree` and cannot see a name — but *"the logic
is name-independent"* is exactly the kind of claim I should measure rather than assume, and nobody had.

## Four directions, measured against the real branch

**The real branch today:** `packet/mira/knowledge-panel` @ `854e74c`, level with origin.

| direction | measured |
|---|---|
| the real branch, as it is | `merge-tree` → `73b6e937`; `diff --name-status` → **25 A / 17 M / 0 D** — the guard lets it through |
| a branch that deletes a tracked file, **named with the real branch name** | `D src/keep.rs` — the guard catches it |
| a branch that deletes the one **ruled** file | raw diff shows `D ui/src/app-v2/AppV2.tsx`; after the exemption filter, **0 remaining** — the guard lets it through |
| a branch that deletes a ruled file **and** another | both appear in the raw diff; after the filter, the other one remains — the guard catches it |

**So the guard discriminates correctly in all four directions, with real names, including the exemption path.** The
name-independence holds, and now it is measured rather than assumed.

## The exemption list, and why it is the strongest part

`DELETION_EXEMPTIONS` in `git/ops.rs:51` has **exactly one entry** — `ui/src/app-v2/AppV2.tsx`, ruled by
`human:supervisor`, with a reason naming the dispatch and the evidence that it was already unreachable on the base.

**And it is audited by the compiler, not only by a test someone may delete** — the comment says so and the
`const _: () = { ... }` block proves it: an entry with no path, no ruling, or no reason fails the build. *A bare path
would pass every test and still be a hole in six months.*

**That is the design answer to the failure I have been recording all week** — a check that cannot fail. Here the
audit is a compile error, which cannot be skipped by deleting a test.

## My own stale snapshot, caught mid-round

**I first measured 24 added. The branch had moved. Re-measured at the current tip: 25 added / 17 modified / 0
deleted.** The difference is one file, and it changed nothing about the conclusion — but I had quoted the old
number in a document two rounds ago and in Nimbus's test comment.

**This is the snapshot-as-state failure, mine, one round after I recorded it as a recurring family.** The
measurement was correct when taken and wrong when read. **What saved me was re-running rather than recalling** — and
the honest record is that the number in the earlier document is now stale, not that it was wrong when written.

## What this means for the operator

**Nothing changes for the two pending cards.** The real branch merges with 0 deletions, so the guard would let both
through — measured, at today's tip. And the guard is still not on `track` and not in the sandbox, unchanged from
last round.

**What this adds is confidence that when it does arrive, it works on the branches that matter**, not only on
synthetic ones.

## What I did not do

I did not edit any code or test. I did not add the real-branch case to his test file — that is his test and adding
it is his change to make; I am recording that it is missing rather than quietly filling it in. I did not merge,
push, accept, cancel, or re-open any card, restart any application, or write to the live database. My measurements
were reads plus three scratch simulations: `merge-tree --write-tree` and `diff --name-status` on the real branches
and in three scratch repositories, reading `accept_deletion_guard.rs` for its branch names and `git/ops.rs:51-70`
for the exemption list and its compile-time audit. The ledger check is exit 0, `seats` is 10, `canonical_events` is
26.
