| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1400-the-path-spelling-hole-is-closed |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, sandbox, containment, salvage |

# The path-spelling hole is closed. This is the first fix in the chain that also closed the class

## What Nimbus landed

`52cdb8b` — the guard now compares **resolved paths** instead of strings.

`canon_path` canonicalises through `readlink -f`, which normalises symlinks, case, trailing
slashes, dots and doubled slashes **in one place** rather than as a list of special cases. And it
handles the case that a naive fix would miss: a path that does not exist yet — which is exactly
what a fresh install target is — walks up to the nearest existing ancestor and re-attaches the
remainder, so it is resolved rather than silently skipped.

`under_path` refuses when the target is **inside** the protected path, not only equal to it,
because `rm -rf` of a subdirectory of the real sandbox is still destruction.

And the failure mode is closed correctly: if canonicalisation fails, `CANON_FAILED=1` and the gate
refuses. A guard that cannot tell what it is protecting should block, not wave through.

## What I verified myself, entirely inside `/tmp`

I extracted the two helpers and drove them directly. Nothing touched `/Applications`.

| case | result |
|---|---|
| trailing slash | caught |
| `.` component | caught |
| doubled slash | caught |
| bare `.` | caught |
| `Contents/..` | caught |
| **lower-case variant** (macOS case-insensitive) | caught |
| an unresolvable path chain | returns non-zero → gate refuses (fail closed) |
| two genuinely different scratch paths | **allowed** — the redirect still works |
| a new file *under* the protected path | caught (sub-path, not just equality) |

Then I ran the full suite: **63 assertions, all 20 sections pass**, and the real `/Applications`
fingerprint is identical before and after the entire run at `0d00e85f3682fb8bdf52b9a2aceecdfb`.
The fixture now includes the gate-removing mutation run under combination 4 — the case that was
never tested — and it is contained.

## Why this one is different from the previous four

Every previous fix corrected **which strings** were compared. This one corrected **that strings
are what is being compared**. The verifier's framing, which Nimbus accepted and implemented:

> The guard compares a representation while the thing being protected is the referent.

The fixture also handles the one honest complication: the lower-case variant only holds on a
case-insensitive filesystem, so the assertion says so and **skips on a case-sensitive volume
rather than pretending the property is unconditional**.

## Salvage, measured before renaming

`check-salvage-fresh.sh` exited 1 for two reasons and I measured both before touching anything:

- **Nimbus's `20261108-1030` patch**: 80 of 84 added lines are verbatim in HEAD, and the 4 that
  are not are the diff's own file header and the mutation's `old =` string — the exact shape of
  committed-and-superseded work. Renamed to `.superseded.patch` on that evidence.
- **Vela has 69 uncommitted insertions** in `scripts/verify-docs.sh`, teaching the doc gate the
  knowledge convention. Captured as `vela-washing-pipeline-20261108-1400.patch` and verified both
  directions. Vela is mid-dispatch; I did not interrupt.

`check-salvage-fresh.sh` is green again, exit 0.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Both pending cards clickable.
- Sandbox: 95-byte fake, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched. Nothing I ran this hour wrote to either real path.

## Needs you

- **The sandbox rebuild** is now unblocked on my side: the guard is correct across every spelling
  and fails closed. It is still a 95-byte fake, and rebuilding it needs your authorisation. Once it
  exists, the six held-back deliverables can be offered to you.
- **Vela's doc-gate change** is in flight; I will verify it when it lands.
