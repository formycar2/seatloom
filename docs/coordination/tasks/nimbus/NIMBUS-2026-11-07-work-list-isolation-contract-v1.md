# The work-list now obeys the isolation and honesty contract

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-work-list-isolation-contract-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | AEGIS-2026-11-02-1500-unified-todo-readmodel-contract, bf64d27 |
| supersedes | |
| tags | work-list, isolation, snapshot, provenance, contract, measured |

## What was wrong, and what each correction is

The first slice answered "what do I owe" but ignored six things the 11-02 contract
requires. Each is corrected, and each correction has a test that fails without it.

| # | the defect | the correction |
|---|---|---|
| I1 | queried all projects | `--project` is required; missing/blank/unknown → non-zero. No default project. |
| I2 | no actor context at all | identity from the two channels `cmd_accept_delivery` already uses (`~/.seatloom/operator`, then `SEATLOOM_OPERATOR_REF`). **No `--actor` parameter** — a CLI-passable identity is not a trusted context. Both channels silent → refuse. |
| I3 | five separate snapshots | one `BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY` transaction; all queries queued, then executed once |
| I4 | `limit 50` silently dropped rows | full count queried separately; output states "N in total, M shown" |
| I5 | every non-verification `done` was a human obligation | three tiers: a known product kind -> require acceptance; an **unknown kind -> a review candidate, "someone please look"**, still visible; `verification` -> excluded with the reason stated |
| I6 | `source+runtime` hardcoded | `source` only. A database read does not know whether the work reached the operator's application. |

## The two bugs I introduced while fixing these, both found by measurement

**The snapshot only covered the queries queued before it ran.** My first version
executed the transaction immediately after the `projects` query. The result: a
report saying "0 obligations" while `od-1` and `disp-ok` were both in the database.
The fix is structural — queuing and execution are separate phases, and the render
phase comes third. The test asserts on a row that genuinely has no card, because
asserting on `disp-nocard` (which gets a card earlier in the sequence) would have
passed vacuously.

**Deleting `<> 'verification'` brought verification dispatches back.** I removed
the whole filter to make room for the unknown-kind tier, and criterion 6 went red:
`disp-verif` appeared in the "unverified work" section. The correct shape is three
tiers, not two, and the `done`, `done_total` and `done_kinds` queries must all
apply the same exclusion — otherwise the stated total disagrees with the rows shown.

## Verified

| | result |
|---|---|
| first-round 8 criteria | all pass, unchanged |
| six new isolation criteria | all pass |
| determinism | 3/3 clean, and 1/1 under each mutation |
| mutation W-1 (Flux's acceptance-key finding) | still caught, exit 1 |
| mutation: project filter dropped | exit 1 |
| mutation: operator defaulted to `human:supervisor` | exit 1 |
| mutation: source failure swallowed | exit 1 |
| `cargo build` / `cargo test` | 0 errors / 367 passed |
| scratch database | dropped by trap, verified absent |

## One thing worth recording: overriding `$HOME` breaks podman

The test needs a controlled operator identity, so it writes a temporary
`$HOME/.seatloom/operator`. Podman derives its **socket path** from `$HOME`, so the
override broke every query — `psql` reported "Cannot connect to Podman … no such
file or directory", the command correctly reported each source as unreadable, and
the suite looked like criterion 2 was passing. Fixed with a wrapper that restores
the real `$HOME` for podman only. A test harness that changes the environment can
break the instrument, and the instrument's failure can look like the assertion
passing.

## Not done, and why

No live-DB writes, no migrations, no acceptance card, no deployment. This is a
diagnostic CLI, not the UI delivery — the contract's UI and notification sections
are a separate slice. Not merged to track. Flux gets a fresh verdict on a pinned
commit, per the dispatch's instruction to avoid measuring a changing tree.
