# The work-list obeys the actor and snapshot contract, with observable proofs

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-work-list-actor-and-snapshot-proofs-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | disp-4Yt-KQcn9x_hWOfOEeJfS |
| supersedes | |
| tags | nimbus, delivery, work-list-snapshot-actors |

> Header added by Aegis on 2026-11-08 after the acceptance merge landed this file on
> `track/infra-foundation` without one. Every value below is copied from the file's own
> provenance lines — nothing is inferred. The doc gate refuses new docs without a
> universal header, and this file arrived through a merge rather than a commit of mine,
> so it would have been invisible to the gate until now.

- seat: `seat-nimbus-001`
- dispatch: `disp-4Yt-KQcn9x_hWOfOEeJfS`
- branch: `packet/nimbus/work-list-snapshot-actors` (from `2ee561b`)

## What changed

`scripts/test-work-list.sh` only. The command under test is byte-identical to `2ee561b` —
verified with `cmp`, not asserted. This is a proof gap, not an implementation gap.

Flux's verdict `ev-eJlCzXwSGEAQC2rPQTciA` found three properties held up by the code
rather than by the tests. All three are now observed.

**V-2 — two-actor segregation.** The fixture had exactly one `operator_decisions` row, so
removing the `decider` filter changed nothing observable. New fixture: a pending decision
belonging to `human:someone-else` in the same project. It must be invisible, and the
operator's own decision must still be visible — a one-directional check would pass with
the whole section broken.

**V-1 — the ready query's project filter was protected only incidentally.** I1's
second-project row was a *done* dispatch, so it exercised section 4's filter and never
the ready query's. Removing the ready query's project filter was red, but red through I3
("did not state the snapshot relationship"), not through an isolation criterion. New
fixture: another project's **stalled ready** dispatch, checked in both directions.

**V-3 — the single snapshot.** Nothing observed it. Two halves, labelled as what they are:

- **I3b, static.** Counts the `BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY` marker,
  the `run_snapshot` call sites, and asserts no query is queued after the execution
  point. This is a source assertion and is labelled static in its own name. It is not
  concurrent proof.
- **I3c, concurrent.** A real experiment. Another session takes `ACCESS EXCLUSIVE` on
  `operator_decisions` and holds it; `work-list` runs, establishes its snapshot at the
  `projects` read, then blocks deterministically on that table. The test polls
  `pg_locks` for the ungranted request to confirm it is blocked — no `sleep` guessing.
  A third session then inserts and **commits** a `done` dispatch. The lock is released
  and `work-list` completes.

  Under `REPEATABLE READ` the row is not in the listing. Under `autocommit` — the state
  the mutation creates — each statement is its own transaction and the row **is** seen.
  This is what makes V-3 red for the right reason.

## Verification

- Baseline: 72 checks, exit 0. Scratch database dropped by the trap and verified absent;
  no temp directory leaked.
- V-1 mutation (drop the ready query's project filter): **I1c red**, and the failure
  message is the isolation one.
- V-2 mutation (drop the `decider` filter): **I2b red** — "another person's pending
  decision appeared".
- V-3 mutation (remove `REPEATABLE READ`): **both halves red** — I3b on the marker count,
  I3c on the concurrent row appearing.
- All three mutations restored from byte-identical copies; `cmp` clean; suite back to
  all-pass.
- Isolated scratch database only. No live ledger, no card, no UI or deployment claim.

## Bugs in my own test, recorded because they were real

- A missing `'` in `tr -d '[:space:]'` made the whole file unparseable — and the
  symptom pointed at a line 80 rows below the actual defect.
- I counted `run_snapshot` call sites with `grep -c 'run_snapshot$'`, but the real call
  is `if ! run_snapshot; then`. The count came back **0**, so a passing property was
  reported as "executed zero times". Fixed by counting all mentions and subtracting the
  definition and the comment.
- A literal tab written into a comment by the heredoc. Harmless to bash, but it sent me
  hunting the wrong line for several minutes.

## What I am not claiming

- I3c proves the listing and the count come from one moment. It does **not** prove the
  command is serializable under arbitrary concurrency — that needs a wider experiment.
- I3b is a source assertion. A `BEGIN` in the file is not evidence of a snapshot at
  runtime; I3c is the part that observes behaviour, and the two are labelled separately
  for exactly that reason.
- The `--apply` path of the deployment scripts is untouched by this dispatch.
