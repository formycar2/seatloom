| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1300-the-verdict-existed-after-all-and-a-masked-rule |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, doc-gate, acceptance, masked-rule, structural-finding |

# The verdict existed after all, the branch was the real blocker, and a masked rule was found underneath

## Vela reported a blocker that was already gone

Vela's note said `request_acceptance` needs a passing Flux verdict naming their dispatch, and none
existed, so their work had no route to the mainline. They were right at the time they wrote it — and
they were also right not to fabricate one, because a seat emitting a verdict about its own work is
exactly what that guard prevents.

I queried the ledger rather than accepting or dismissing the claim. **The verdict now exists.**
`ev-oO1C34dPuLklTsJRUETgj`, issued `2026-11-09 04:52 UTC`, `verdict=pass`,
`dispatch_id=disp-MlhRWjAlGcaPUj2BqMpOr`, `branch=packet/vela/washing-pipeline`, `commit=4927d34`. It
arrived **after** Vela's note, so the note was accurate when written and is out of date now.

I checked the whole population, not just the one row: 35 `ReviewVerdictIssued` events over 21 distinct
dispatches, and `disp-MlhRWjAlGcaPUj2BqMpOr` is the only one targeting Vela. The guard the code
actually enforces is `WHERE id=$1 AND target_seat_id=$2 AND status='done'` — that dispatch is done
and it is theirs, so the verdict satisfies it.

## What was really blocking, and it was never the verdict

The branch is **22 commits behind** `track/infra-foundation`. `request_acceptance` enforces
containment — the delivery branch must contain the base — so the branch has to be synced before a
card can be opened. That is Vela's work in their worktree, and it is the reason the card cannot exist
this minute.

So the seat's report named the wrong blocker, and it was not carelessness: the blocker it named was
real an hour earlier. This is worth saying plainly, because a report that goes stale in one direction
gets refuted, and a report that goes stale in the other gets believed. **A reported blocker has a
timestamp, like every other fact.**

I dispatched `disp-eBkpELwKY_geVRG0Y8xgI` at priority 96 with the sequence: sync the branch, re-run
the test and read 8/8 on the new tip, then open the card naming `ev-oO1C34dPuLklTsJRUETgj`. The
dispatch repeats Flux's correction of the numbers — **26 unblocked, 37 still failing (26 CJK + 11
missing frontmatter keys), header 63 → 37, not 63 → 0** — because an operator who accepts believing
the exemption empties the directory has accepted on a false premise.

## The finding underneath, which is worth more than the card

`check_file` returns early on a header failure, with a comment saying it skips the deeper checks.
**So for as long as the header was broken, the CJK rule never ran on those 26 files at all.** The
exemption did not create a CJK problem; it made an existing one visible for the first time.

That inverts the reading of the last two hours. It would be easy to file this as "the exemption
unblocked 26 files and revealed 26 failures". What actually happened is: **the 63-file header failure
was masking a 26-file CJK failure, and every instrument that asked whether the knowledge directory was
clean got an answer computed without ever looking at those files.**

A control that fails early hides everything behind it. The 63 I reported last hour was not "63 header
failures" — it was "at least 63 failures, of which the count stopped at the first rule each file hit".

## Two disclosures I verified rather than took on trust

- Flux disclosed that it appended a newline to 90 files in Vela's worktree by mistake, restored them
  immediately, and recorded it. I checked: `git status --porcelain` in Vela's worktree is **empty**.
  The disclosure is accurate and the damage was undone.
- Vela disclosed that `scripts/.lyra-neut.sh` is untracked in its worktree, and that another seat is
  running the same neutralise-the-gate measurement in that tree. Two seats mutating one worktree is
  how a stray file gets committed by accident. I have not touched it — it is Vela's tree to clean.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Vela's branch is clean and the gate change is intact (`KNOWLEDGE_DIR` appears 3 times, test file
  present), and its suite is 8/8.
- Push: local and remote both at `6a99882`, confirmed. No new commits this hour, so nothing to push.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **The 26 CJK files and the 11 incomplete-frontmatter files.** Now measured separately, which they
  never were before. The 26 cannot simply be renamed — 29 inbound references, 24 of them in the
  knowledge index. The 11 are a different thing again.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
- **Which remote is the source of truth.** gitlab is 935+ commits behind and I have not pushed there.
