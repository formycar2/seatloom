# Unified work list: the first slice

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task-packet |
| id | NIMBUS-2026-11-07-unified-work-list-slice-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | scripts/work-list.sh, scripts/test-work-list.sh |
| supersedes | |
| tags | work-list, operator-obligations, unified-work-list, acceptance, stall-rule |

## What was built

`scripts/work-list.sh` — one command that prints the operator's actionable
obligations from the three existing tables. Each row names its source table, id,
subject, owner, age, action, provenance, and whether it is unresolved.

Sources, and what each contributes:

- `operator_decisions` where `status='pending'` — owner is the decider, action is
  "rule on it".
- `acceptances` where `status='pending'` — owner is the card's seat, action is
  "click Accept or Reject".
- `dispatches` where `status='ready' AND issued_at < now() - 30 minutes` — owner is
  the target seat, action is "nothing; this is the seat's work", marked as a stall.

The dispatch inclusion rule is deliberately the STALL rule, not the ready rule.
A ready dispatch is seat work. `notify-operator.sh:166-168` already gets this
right — it fires only past 30 minutes and says "this is a system problem, not your
work". This command reuses that boundary, so a ready dispatch is never rendered
as something the operator owes.

A fourth section covers the row the existing check loses: a `done` dispatch with
no card at all. That means work shipped and the operator was never asked to look
at it. Measured on live, that section found 8 such dispatches, the oldest 1878
minutes old.

## What it is not

Not the read model (no new table, no snapshot service, no pagination — that is the
11-02 contract's sections 2 and 3, the next slice), not a UI (a CLI is a
diagnostic slice, not a substitute for eventual existing-UI integration), and not
a re-classifier.

## The eight-case oracle, all verified

`scripts/test-work-list.sh` covers all eight against a scratch database. It never
touches the live one — the scratch is created and dropped inside a trap, and
verified absent afterwards.

1. All three sources reachable with live data — exit 0, rows per source.
2. A source query forced to fail — exit non-zero, output names the table.
3. A ready dispatch aged under 30 minutes — absent.
4. A ready dispatch aged over 30 minutes — present, marked stall, action says it
   is not the operator's work.
5. A `done` dispatch with no card — present, marked unresolved.
6. The same with `kind=verification` — absent, and the output says why.
7. Two pending obligations for one dispatch — both rows present, distinct ids.
8. Zero rows across all three — exit 0, "0 obligations", and a line stating all
   three sources were read.

## Three bugs found by running it, all of which made a read failure look like health

The dispatch flagged case 2 as the one most likely to be skipped. It was also the
one that exposed the most, because the first version reported "0 obligations" and
exited 0 with a renamed table — the exact shape the dispatch warned about.

- `local sql out rc` followed by `rc=$?` captured `local`'s exit status, not the
  command substitution's. `rc` was always 0.
- `out="$(q "$sql")"` is itself a command substitution, so `$?` was the
  substitution's status, and `q` returned 0 either way.
- The most hidden one: `X="$(require ...)"` put `fail()`'s `exit 1` inside a
  subshell, so the main script continued and printed the failure line as data.
  Measured output: "❌ cannot read dispatches" immediately followed by
  "result: 9 obligations" and exit 0 — a report that lied.

The fix removes command substitution from the failure path entirely: `q()` writes
into globals, `require()` calls `fail()` in the main shell, so `exit 1` really
ends the command. All three traps are documented at the code they occurred in.

## The mutation

Assertion 9 makes the dry-run... rather, the mutation makes `q()`'s `return 1`
into `return 0`, which is exactly what `notify-operator.sh`'s `q()` does. The
suite then exits 0 on a source failure — so case 2 is guarding something real,
not an empty room. The mutation's own application is verified (the file must
actually change), so a stale anchor cannot masquerade as a passing mutation.

## Provenance

Every row carries `source+runtime` today. The dispatch notes these can differ —
the deletion guard is source-true and runtime-false. The column exists so the
difference is visible rather than assumed; nothing in this slice computes it yet,
and that is recorded here rather than left as an implied guarantee.

## Gates

- `cargo build --workspace` — 0 errors
- `cargo test --workspace` — 367 passed, 0 failed
- `scripts/test-work-list.sh` — exit 0, all eight cases plus the mutation,
  determinism 3/3

Read-only: no live-DB writes; the scratch database is dropped in a trap and
verified absent after the run.

Not merged, not deployed, no card opened, no live-DB writes.
