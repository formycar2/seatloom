# Standing-duty report: the operator has been absent from the product for 16.5 hours, all three invisibility causes re-verified, and the landing script re-proven end to end

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-operator-absent-16h-and-everything-reverified-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, blocking, report |

## The measurement that reframes the last several rounds

**His last action inside the SeatLoom product was 2026-10-30 08:41 — 16.5 hours ago.** He is still attached
to three tmux sessions (`Aegis-Supervisor`, `saih-self`, and a `Stepaicloud-seatloom` session belonging to a
different project of his), and the app process is still running. **He is present at the machine and absent
from this product.**

That changes how I should read the three pending cards. I have spent several rounds measuring why he has
not clicked and producing three findings — panel closed, project-scoped, stale build. **All three are true
and all three are still true.** But the proximate cause is simpler than any of them: **he is not in the app.**

## All three causes re-verified this round, not assumed

Read from his live persisted store, in the app's own WebKit localstorage:

```
seatloom.supervisor.open = false
activeProjectId           = project_delivery
```

and from the database:

```
acceptances where project_id = 'project_delivery'   ->  0
acceptances where project_id = 'seatloom'           ->  accepted 68, cancelled 9, pending 3
```

and from disk:

```
/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri   10-28 22:16
Mira's fix d2fea84                                         10-30 17:50
```

**So: the panel that lists his cards is closed, it would show him an empty list anyway because his active
project has never had an acceptance, and the app he would verify them with predates the fix by about 43
hours.** Every one of the three still holds, unchanged, and I re-measured each rather than carrying the
number forward.

## The standing-duty defect, re-measured and now at 106

```
heartbeat nudges in the waker log: 106
  seat-lyra-001: 106
  the other five seats: 0

dispatch nudges in the waker log: 72
  onyx 20, mira 19, nimbus 16, flux 12, vela 3, lyra 2
```

**The other five seats are being woken — 72 times — but only for dispatches. Not once for standing duty.**
The number is the whole case for `land-standing-duty-docs.sh`, and it has grown from 90 to 106 across the
rounds, which is itself the measurement: the mechanism works, it just only sees one seat.

## The landing script re-proven end to end, on the current track tip

I re-ran it in a scratch worktree at `052f6ff`, answering `no` at the prompt:

| Step | Result |
|---|---|
| 1 preconditions | branch is track, no uncommitted tracked changes, source is a descendant of the tip, all six files present on the source and **absent on track** |
| 2 extraction | all six `git hash-object` values match the source blobs |
| 3 grandfather | original **207 lines preserved byte for byte**, 5 entries + explanation appended |
| 4 docs gate | `verify-docs.sh` exit **0** |
| 5 waker resolution | **all seven seats FOUND** — compiled `heartbeat_doc_path()` standalone and ran it |
| 6 staged diff | 7 files, **569 insertions, 0 deletions**, nothing committed when answered `no` |

**So the one command that fixes the standing-duty defect is still runnable, still correct, and still
unrun.** Its preconditions have held across every round because I rebased the source branch when it fell
behind; it is currently **0 commits behind track**.

## What I am doing and not doing

**Not chasing him.** He is on another project. Three findings tell him the same thing and none of them
reaches a man who is not in the app; a fourth would be the same. The findings are recorded, the fixes are
one command and two lines each, and they are his to run.

**Not manufacturing work.** My heartbeat's hard rule is explicit: *do not create work to look busy; if there
is genuinely none, say so and say what you checked.* This round I re-verified rather than produced: the
operator's absence, the three invisibility causes, the standing-duty count, the landing script, and my own
branches. **All of it is current, and none of it needs a new artifact beyond this report.**

**What is genuinely open, in priority order, all his:**

1. `bash scripts/land-standing-duty-docs.sh` — one command, documents only, no restart, fixes a defect
   measured at 106 nudges for one seat and zero for five.
2. `bash scripts/prepare-verification-build.sh packet/mira/knowledge-panel` — so the three pending cards can
   actually be verified on a build that contains the work.
3. The pending-count badge and the acceptance scoping — so a card is visible without the panel open and
   without the right project selected.
4. The provenance lie, which a test currently enforces in both directions.
5. The PTY-forward half of the free-text message fix.

**None of those is mine to run or edit. All of them are measured, documented, and on both remotes.**
