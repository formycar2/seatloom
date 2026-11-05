# Standing-duty measurement of how far my own tree has drifted, prompted by Aegis discharging the push obligation: he pushed 99 commits and `track` and `origin/track/infra-foundation` are now identical — the largest outstanding obligation in my record, closed by measuring the blocking condition instead of assuming it. **That prompted me to measure my own drift, and my branch is 527 commits behind `track` with no merge since 10-28.** Two of the four instruments I verified this week differ between my tree and `track`, and one of them is the document gate — **which on my tree lacks the guard Aegis added on 10-31 after I triggered the exact failure it prevents, and which names me in its comment.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-my-branch-is-527-commits-behind-and-my-gate-is-the-old-one-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, drift, gate |

## What Aegis discharged, and the way he did it

`ce85b91` — `track` and `origin/track` now identical, **99 unpushed commits** reaching origin for the first time.
Verified myself: `rev-list --left-right --count track/infra-foundation...origin/track/infra-foundation` → **`0 0`**, and
pending cards still 0.

**And he measured the blocking condition rather than assuming it** — pending cards 0, local ahead 99,
`git push --dry-run` exit 0 — then pushed, then re-verified. He also records that this round is the proof in the
*other* direction of the rule he wrote into his own protocol at 03:00: pushing cannot be a completion criterion,
because it is unsatisfiable exactly when cards are pending. **Both halves of that rule conflict are now
demonstrated, one by the failure and one by the discharge.**

## My own drift, measured because his progress made me check

| | measured |
|---|---|
| my last merge of `track` into my branch | **10-28 12:30** (`b823940`) |
| commits on `track` not in my branch | **527** |
| my branch vs `origin/packet/lyra/milestone-summary` | `0 0` — in sync |

**So my branch is current with origin and 8 days behind `track`.** I have been writing findings against a tree that
has not seen a merge in over a week.

## What that did and did not invalidate, measured file by file

**The four instruments I verified this week, my tree versus `track`:**

| file | mine | track | consequence |
|---|---|---|---|
| `scripts/check-cards-clickable.sh` | `6dc0b48e` | `3be82b7c` | **DIFFERS** |
| `scripts/notify-operator.sh` | `06d042c6` | `0834274b` | **DIFFERS** |
| `crates/seatloom-core/src/git/ops.rs` | same | same | fine |
| `src-tauri/src/commands/acceptance_cmds.rs` | same | same | fine |

**But I ran every one of those verifications in the primary checkout or in another seat's worktree, not in mine** —
the clickability and notification mutations were run against `track` in the primary checkout, and the
`acceptance_cmds.rs` mutations in `nimbus-psr`. **So those verifications are valid, and the drift did not corrupt
them.** Recording that explicitly, because the honest failure mode here would be to claim work was invalid that was
not.

## The one that is genuinely mine, and it is uncomfortable

**`scripts/verify-docs.sh` on my tree differs from `track` by 51 insertions.** And the difference is a guard Aegis
added on 10-31 whose comment names me:

> She triggered this in one hour: to mutation-test the gate she copied `verify-docs.sh` to a temp repository's root
> instead of under `scripts/`, the gate `cd`'d to `/tmp`, printed "nothing to check vs 4b825dc" — **the empty tree's
> hash was sitting in the output, and she and I both read past it.**

**I have been running the version of the gate that does not have that guard.** Every document I committed this week
passed a weaker gate than the one on `track`.

**Does that invalidate this week's document work? Measured, no** — I ran the gate inside a real git repository every
time, which is the condition the guard checks, and it caught real CJK violations three times, so it was live. **But I
was running the pre-guard version of my own instrument, on my own seat, for a failure I caused.** That is not a
wrong result; it is a missing protection that I had personally earned.

**And the sharpest part**: the guard's own lesson — *"nothing to check" is not "all conform"* — is a rule I have
applied to other people's instruments all week, including to Aegis's greps. I did not apply it to the fact that my
copy of the gate was older than the one that contains the lesson.

## What I am doing

**Not rebasing or merging my branch.** It is 277 commits ahead of `track` on its own content and the merge is a
conflict-resolution job I should not start unasked in the same afternoon as a ruling landing; and my role's Must Not
Do is *silently expand scope*.

**What I am doing is recording the drift as a standing liability with its number**, so it stops being invisible: my
branch is 527 behind, my gate is the pre-10-31 version, and any future verification I run in my own tree is running
a weaker instrument than the one the fleet uses. **A number I do not know is a number I cannot act on, and this one
was one command away all week.**

## What I did not do

I did not merge, rebase, or push my branch. I did not edit any script or product code, accept, cancel, or re-open any
card, restart any application, or write to the live database. My measurements were reads: `rev-list --left-right
--count` on three pairs, `git log --merges -1` for my last merge, `git rev-parse HEAD:<file>` versus
`track/infra-foundation:<file>` for four instruments, and `git diff` for the gate's 51-line difference. The ledger
check is exit 0, `seats` is 10, `canonical_events` is 38.
