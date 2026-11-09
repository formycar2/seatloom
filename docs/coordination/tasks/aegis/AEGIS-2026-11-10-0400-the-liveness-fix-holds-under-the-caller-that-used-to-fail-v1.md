| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0400-the-liveness-fix-holds-under-the-caller-that-used-to-fail |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, liveness, verification, regression-window, db-flake |

# The liveness fix holds under the caller that used to fail, and changes nothing for the operator's shell

## What this hour verifies

Last hour I fixed `scripts/fleet-status.sh`'s liveness probe (`pgrep -P` → `pgrep -a -P`) and proved it
with a mutation. This hour I verified the two properties that decide whether the fix is actually safe,
rather than assuming them.

## Property 1: it works under the caller that used to fail

The defect only appeared when the caller descends from a seat agent. That is my situation, so the test
is exact:

| probe | pane `6704` (whose child is my ancestor) |
|---|---|
| old form, no `-a` | **empty** — the seat reads itself offline |
| new form, `-a` | `8518` — found |

And `fleet-status.sh`, run from here, now reports **6/6** including aegis. The instrument is correct from
inside a seat's own tree, which is the one context it was wrong in.

## Property 2: it changes nothing for the operator's shell

A fix that repairs one caller by breaking another is not a fix. Measured across all six mapped seats:

| session | old form | new form |
|---|---|---|
| Mira-UX/UED-seatloom | 1 | 1 |
| Nimbus-TechArchi-seatloom | 1 | 1 |
| Flux-Quality&Ops-seatloom | 1 | 1 |
| Onyx-data-seatloom | 1 | 1 |
| Lyra-po-seatloom | 1 | 1 |
| Vela-DataWash-seatloom | 1 | 1 |

**Identical.** For a caller with no seat agent among its ancestors, `-a` adds nothing and removes
nothing. So the number you have been reading for days was already right, and it stays right — the fix
only corrects the context where it was wrong.

## The condition I recorded last hour, re-measured

I noted that `podman exec` was failing between 1-in-20 and 1-in-2 times while the container itself was
stable. This hour: **10/10 successes**, and the full suite ran green. So it is intermittent rather than
degraded, and it is not currently happening.

That does not make last hour's note wrong, and it is worth saying why: **a green suite run during a flap
is not evidence of a green suite.** During a flap the checks correctly report exit 2 (`UNCHECKABLE`), and
a suite that happened to run between failures would report green while having examined less than it
claims. I waited for a stable window before claiming this hour's results, and I am recording that the
condition is intermittent so that a future green run is not read as proof the condition is gone.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0, plus `check-unclaimed-work` at 0.
- All four tests exit 0: `test-fleet-status-selfwake`, `test-fleet-status-liveness-probe`,
  `test-unclaimed-work`, `test-unclaimed-work-declarations`.
- Three pending cards awaiting you.
- Push: local and remote both at `5fe3429`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Three cards are waiting**, each separate work. Mira's is missing one statement you should read before
  accepting, and it is quoted in my 2026-11-10-0000 delivery note.
- **Option 1, 2 or 3 for the live-seats count.** My recommendation is 1, unchanged.
- **The sandbox decision unblocks five dispatches at once**, not one.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed.
