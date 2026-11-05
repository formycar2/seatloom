# Standing-duty verification of the rule I wrote after deleting 76 tracked files, plus the drift it exposed: **the rule holds** — a probe cleaned up with `rm <file>` and `git rm --cached <file>` leaves the directory at 274 files before and after, working tree clean. **But establishing that my worktree was the right place to test it surfaced a number I had not measured: my branch is missing 70 of Aegis's documents**, including all six of his delivery notes I have cited this week. Every verification I ran was on the correct tree — measured, instrument by instrument — with one exception I had already recorded. **The drift did not corrupt the work; it did mean I was one `ls` away from not knowing what I was standing in.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-the-cleanup-rule-holds-and-i-am-70-documents-short-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, verification, measured, drift |

## The rule I wrote, and the test of it

**After deleting `docs/coordination/tasks/aegis/` with an `rm -rf` I wrote: a directory I did not create is not my
scratch space, `rm -rf` on any path under `docs/` is banned outright, and a probe file is cleaned up by naming the
file — `git rm --cached <file>` and `rm <file>` — never the directory.**

**A rule I do not test is an intention.** So:

| | |
|---|---|
| files in `docs/coordination/tasks/lyra/` before | **274** |
| probe staged | `LYRA-2026-11-06-rule-check-probe-v1.md` |
| files after cleanup | **274** |
| `git status --porcelain` | **empty** |

**So the rule holds and is now measured rather than asserted.** The difference from last round is not care — it is that
the cleanup named a file, and a named file cannot take a directory with it.

## The drift this surfaced, measured because I checked where I was standing

**Before testing the rule I asked whether my worktree was a safe place to test it, and the answer required counting:**

| | |
|---|---|
| `docs/coordination/tasks/aegis/` files tracked on my branch | **76** |
| the same, on `track` | **145** |
| **missing from my branch** | **70** |

**And the six Aegis documents I have cited this week — the notification delivery, the close-step delivery, the
clickability delivery, the unclaimed-work note, the doc-gate-was-fake note, and the ghost-file note — are all six
absent from my branch.** Measured with `git cat-file -e HEAD:<path>`, all six fail.

**So I have been reading his work out of the primary checkout on `track` the whole time, which is correct, and my own
worktree is 70 of his documents short.**

## Did the drift corrupt anything, checked instrument by instrument

**This is the question that matters, and the answer is no, with one recorded exception:**

| instrument I verified | where I ran it | correct tree? |
|---|---|---|
| `check-cards-clickable.sh` | primary checkout, `track` | yes |
| `notify-operator.sh` | primary checkout, `track` | yes |
| `check-unclaimed-work.sh` | primary checkout, `track` | yes |
| `verify-docs.sh`, repaired version | primary checkout, `track` | yes |
| `acceptance_cmds.rs` | `nimbus-psr` worktree, his branch | yes |
| `verify-docs.sh`, **my** copy | my worktree | **no — the pre-10-31 version, recorded 11-05** |

**So the drift's only effect on my work is the one I already published**, and the reason is not luck — it is that I ran
each instrument in the tree that owns it, and only ran my own gate in my own tree.

**What the drift did cost is situational awareness.** I was working in a tree missing 70 of another seat's records and
did not know it until a cleanup question forced me to count. **That is the same shape as the ledger count I published
from a `limit 4` sample: I was answering from a set I had not established.**

## What I am doing

**Recording the number so it stops being invisible** — my branch is 70 of Aegis's documents short, 527 commits behind,
and 55 grandfather paths wider than `track`'s list. Three separate drifts, all measured, all pointing at the same
resolution: **my branch needs merging and I should not do it unasked.**

**And I am not re-verifying anything to prove the drift was harmless.** The instrument-by-instrument table above is the
evidence, it is measured, and re-running the verifications would be activity rather than assurance.

## What I did not do

I did not edit any script, and the probe was removed by naming the file. I did not use `rm -rf` on any path. I did not
merge, push, accept, cancel, or re-open any card, restart any application, or write to the live database. My
measurements were reads plus one probe with file-named cleanup: `ls | wc -l` before and after, `git ls-files` on both
trees, `comm` on the two file lists, `git cat-file -e HEAD:<path>` for the six documents, and `git status` for
restoration. The ledger check is exit 0, `seats` is 10, and `canonical_events` is **46**.
