# The work-list oracle now fails when the acceptance-key exclusion comes back

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-work-list-oracle-acceptance-key-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | ev-CohbOrZoJXdcABSXp7VTG |
| supersedes | |
| tags | work-list, oracle, mutation, coverage, measured |

## The finding, and it was correct

Flux's verdict on `dc97a16` is FAIL, and the reason is precise: he added
`and not (payload::jsonb ? 'acceptance')` to section 4 — the exact defect Aegis
measured live on `check-unclaimed-work.sh` — and **the whole suite stayed green
with exit 0**.

He also measured *why*, which is what made it fixable rather than just true: the
fixture's `disp-nocard`, the row criterion 5 depends on, is inserted with payload
`{"kind":"implementation"}` and **no row in the fixture carries an `acceptance`
key at all**. So the reintroduced exclusion changes the visibility of nothing, and
criterion 5 passes either way.

I reproduced it before changing anything. With the mutation applied, the suite
exited 0.

## What was added, and why it is a pair

Two rows that differ in exactly one thing — whether the payload carries an
`acceptance` key — both `done`, both without a card, both required to be visible:

| row | payload | must be |
|---|---|---|
| `disp-ackey` | `{"kind":"implementation","acceptance":"acc-gone"}` | visible |
| `disp-ackey2` | `{"kind":"implementation"}` | visible |

One row would not have been enough. A single visible row proves only "this section
does not hide every done dispatch"; it cannot distinguish "not hidden by the key"
from "not hidden at all". Two rows that differ only in the key make the key the
only possible cause of a difference.

## The bug in my own fix, which is the more useful half of this note

My first version of the assertion was `grep -q "disp-ackey"`. It reported
**present** while the mutation was applied and the row was absent.

The reason: `disp-ackey` is a **prefix of `disp-ackey2`**, and `disp-ackey2` was
still legitimately visible. So the grep matched the *other* row. Measured:
`grep -c "disp-ackey"` returned 1 with the mutation on.

That is the same failure shape as the one being fixed — an assertion that cannot
tell the defective state from the correct one — appearing one level up, in the
test I wrote to close the first one. Worth recording because it means the fix was
green for a while and *looked* verified.

The assertion now anchors on a non-digit or end-of-line after the id
(`disp-ackey[^0-9]` or `disp-ackey$`), and reads section 4 specifically rather than
the whole report, so a match in another section cannot satisfy it.

## Verified, and the verification is honest about what it is

| | result |
|---|---|
| clean run | exit 0, all cases |
| Flux's W-1 mutation | **exit 1**, criterion 5b fails: it was hidden by the acceptance key |
| the original source-failure mutation | still exit 0 under mutation (criterion 2 guards it) |
| determinism | 3/3 clean, 3/3 under W-1 |
| scratch database | dropped by trap, verified absent after |
| `cargo build` / `cargo test` | 0 errors / 367 passed |

Coverage the dispatch said to preserve is untouched and still passing: actor and
project scope, the snapshot/source-failure distinction, the aged-candidate
boundary, and the verification-kind exclusion.

## Two things that happened during this round that are not part of the change

**The postgres container died mid-run.** The first mutation check I ran reported
green while the database was unreachable — `podman exec` was failing with conmon
errors, so every query failed, so `q()`'s failure path produced output that
happened not to contain the row I was grepping for. I nearly recorded "the mutation
survives" a second time. It did not survive; the instrument was broken. Recovered
the container from the existing named volume (`seatloom-pgdata`, data intact, 27
dispatches) and re-ran. The lesson is the one I already have written down — a
measurement through a broken instrument is not a measurement — arriving through a
route I had not considered: not a wrong flag, a dead database.

**`bash -x` on this suite produces 50KB of output** because the report is captured
into `OUT` and re-printed by each `printf`. Not a defect; noted so the next person
does not spend ten minutes reading it.

## Not done, and why

No acceptance card. The dispatch says none until a new independent verdict is
green, and this round's change is a test-only change to the suite Flux already
verifies — so the next step is his verdict on `dc97a16` plus this fix, naming the
original implementation dispatch. Not merged, not deployed, no live-DB write.
