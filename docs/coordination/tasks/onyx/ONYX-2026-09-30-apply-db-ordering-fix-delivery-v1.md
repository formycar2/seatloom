# Delivery: Fix `apply-db.sh` ordering so the runtime-triple merge candidate builds from scratch

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | ONYX-2026-09-30-apply-db-ordering-fix-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-09-30 |
| packet_ref | `docs/coordination/tasks/onyx/ONYX-2026-09-30-apply-db-ordering-fix-v1.md` |
| clears | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v2.md` (FAIL) |
| base (packet branch) | `580bcf483e0ca9af293367e0602a802226ce1590` |
| merged in | `track/infra-foundation` @ `3bf90176c177d588c782befeceeaa41a7a69e9f8` |
| merge commit | `ef4af6bb3372c8e0d6fb421e6c06cfc9d368bc25` |
| fix commit | `cfdd6475587ddc86253d0d2925dd5288265bb561` |
| **merge candidate for re-verify** | **this doc's commit — see §9 `MERGE_CANDIDATE`** |
| tags | onyx, apply-db, migration-ordering, cross-packet-collision, p0 |

---

## 0. Summary

`apply-db.sh` now orders migrations **by rule instead of by number**: `001–005` + `009` pre-seed,
seed `001–004`, `008` post-seed. Built from an **empty scratch database**, exit **0**, with every
§2.2 verification query from the original packet passing there.

**No `.sql` file was changed.** Today's work touched exactly one file: `scripts/apply-db.sh`.

Two things Lyra should read before accepting:

1. **§4 — a second consecutive `apply-db.sh` run still fails (exit 3).** I proved by control
   experiment that this is **pre-existing on `track/infra-foundation` itself** and unrelated to
   `009`. It is drift-report finding **D-3** surfacing on the CI path. Fixing it needs a `.sql`
   change, which this packet forbids.
2. **§8 — one done-definition item is blocked, not done.** I could not advance the packet branch
   ref, because Flux's worktree still has that branch checked out. All work is committed and
   verifiable; it sits on a detached HEAD pending one ref update. Precise ask in §8.

---

## 1. §2.1 Branch brought up to date

```
$ git rev-parse track/infra-foundation
3bf90176c177d588c782befeceeaa41a7a69e9f8

$ git merge-base --is-ancestor 281beaa track/infra-foundation && echo YES
YES — 3bf9017 satisfies the packet's ">= 281beaa"

$ git merge-base --is-ancestor 865f820 track/infra-foundation && echo YES
YES — Flux's D-5 container-detection fix is included
```

Merged as `ef4af6b`, **no conflicts**. Both prior changes to
`scripts/verify-postgres-baseline.sh` survived the merge intact — mine (009 registration) and
Flux's (`CONTAINER` detection):

```
$ grep -n '009_runtime_triple' scripts/verify-postgres-baseline.sh
114:  009_runtime_triple.sql
$ grep -n 'CONTAINER="podman"\|container runtime:' scripts/verify-postgres-baseline.sh
53:  CONTAINER="podman"
59:echo "container runtime: $CONTAINER"
$ grep -n '009_runtime_triple' scripts/bootstrap.sh
139:  009_runtime_triple.sql
```

**Deviation from packet §0, disclosed:** the packet says to check out the packet branch in the main
checkout. That is impossible right now — see §8. I worked on a **detached HEAD** at `580bcf4`
instead, which is byte-identical in content and does not contend the branch ref, so **Flux's
worktree was never touched**. Confirmed untouched at the end (§7).

---

## 2. §2.2 The fix — rule, not bucket

`scripts/apply-db.sh`, +114/−24, the only file changed today.

The classification rule now stated in the header and applied per entry:

- **PRE-SEED** — the seed writes columns this migration creates. The seed physically cannot insert
  into a column that does not exist yet.
- **POST-SEED** — this migration constrains columns the seed does not provide (ADD COLUMN, backfill
  from seeded rows, SET NOT NULL). It needs the seeded rows to exist first.

Every migration carries its own one-line reason, printed at apply time:

| Group | Migration | Reason encoded in the script |
|---|---|---|
| pre-seed | `001_seatloom_core` | creates every table the seed inserts into |
| pre-seed | `002_document_authority` | creates `documents`/`document_sections`; seed 002 inserts them |
| pre-seed | `003_write_ingest_reconcile` | creates `reconcile_*`/`document_versions` |
| pre-seed | `004_operational_review_and_continuity` | creates `checkpoints`/`review_*`; seed 003 inserts them |
| pre-seed | `005_prompt_and_channel_action_authority` | creates `prompt_*`/`channel_action_receipts`; seed 004 targets them |
| **pre-seed** | **`009_runtime_triple`** | **adds `seats.default_harness/_agent/_model` + `sessions.harness/agent/model`, which seed 001 WRITES. Safe here: touches only columns created in 001, and constrains nothing the seed omits — its sole `SET NOT NULL` is on `sessions.agent`, which the seed supplies.** |
| **post-seed** | **`008_project_isolation`** | **backfills `project_id` from seeded rows then `SET NOT NULL`; seed 001 supplies no `project_id`.** |

Also encoded, because the rule has an edge the bucket never had to face: **a migration that needs
both placements must be SPLIT** into a pre-seed and a post-seed half. None does today; the header
says so and says what to do when one appears.

### 2.1 Completeness guard — new, and it closes the same class of bug

The previous code applied `for f in "$SCHEMA_DIR"/*.sql` with a `>=006` test, so every file was
*reached* even if misplaced. An explicit list trades that away: a migration in neither list would be
**silently skipped**, bringing up a quietly incomplete database — strictly worse than the loud
failure this packet fixes. So the script now refuses to apply a partial schema:

```
$ printf -- "-- probe\nSELECT 1;\n" > infra/postgres/schema/010_onyx_guard_probe.sql
$ DATABASE_URL=…/seatloom_verify_onyx bash scripts/apply-db.sh; echo "EXIT=$?"
ERROR: unclassified migration(s) in infra/postgres/schema — refusing to apply a partial schema.
Add each to PRE_SEED_SCHEMA or POST_SEED_SCHEMA in this script, with a reason:
  PRE-SEED  if the seed writes columns the migration creates.
  POST-SEED if the migration constrains columns the seed does not provide.
  unclassified: 010_onyx_guard_probe.sql
GUARD EXIT CODE = 2
$ rm -f infra/postgres/schema/010_onyx_guard_probe.sql     # probe removed; tree clean
```

The converse — listed but absent — prints `SKIP <file> — not present on this branch` and continues,
so the script still works on branches predating a migration. **Announced, never silent.**

---

## 3. §2.3 Proof from an empty scratch database

### 3.1 BEFORE — the FAIL reproduced on the merge candidate

`ef4af6b`, unfixed `apply-db.sh`, empty `seatloom_verify_onyx`:

```
apply-db: target=postgresql://seatloom:seatloom@localhost:5432/seatloom_verify_onyx
--- base schema (001–005) ---
  infra/postgres/schema/001_seatloom_core.sql
  infra/postgres/schema/002_document_authority.sql
  infra/postgres/schema/003_write_ingest_reconcile.sql
  infra/postgres/schema/004_operational_review_and_continuity.sql
  infra/postgres/schema/005_prompt_and_channel_action_authority.sql
--- seed (001–004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql
ERROR:  column "default_harness" of relation "seats" does not exist
LINE 1: INSERT INTO seats (id, name, default_runtime, default_harnes...
                                                      ^
--- apply-db.sh exit code: 3 ---
```

Byte-for-byte the failure Flux reported. Reproduced independently before changing anything.

### 3.2 AFTER — fixed, from an empty database, exit 0

```
apply-db: target=postgresql://seatloom:seatloom@localhost:5432/seatloom_verify_onyx
--- pre-seed schema (seed writes columns these create) ---
  infra/postgres/schema/001_seatloom_core.sql
      why: creates projects/seats/sessions/workitems/handoffs/artifacts/canonical_events — every table the seed inserts into
  infra/postgres/schema/002_document_authority.sql
      why: creates documents/document_sections; seed 002 inserts document rows
  infra/postgres/schema/003_write_ingest_reconcile.sql
      why: creates reconcile_runs/reconcile_items/document_versions written by reconcile and seed 002
  infra/postgres/schema/004_operational_review_and_continuity.sql
      why: creates checkpoints/review_threads/review_comments; seed 003 inserts into them
  infra/postgres/schema/005_prompt_and_channel_action_authority.sql
      why: creates prompt_instances/prompt_actions/channel_action_receipts; seed 004 targets them (zero-row baseline)
  infra/postgres/schema/009_runtime_triple.sql
      why: ADDS seats.default_harness/_agent/_model and sessions.harness/agent/model, which seed 001 WRITES — must exist before the seed runs. Safe here: it only touches columns created in 001 and constrains nothing the seed omits (its sole SET NOT NULL is on sessions.agent, which the seed supplies).
--- seed (001-004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql
  infra/postgres/seed/002_document_seed.sql
  infra/postgres/seed/003_operational_review_and_continuity_seed.sql
  infra/postgres/seed/004_prompt_and_channel_action_seed.sql
--- post-seed schema (constrains columns the seed does not provide) ---
  infra/postgres/schema/008_project_isolation.sql
      why: backfills project_id on workitems/sessions/handoffs/canonical_events from seeded rows, then SET NOT NULL. Seed 001 supplies no project_id, so running this before the seed makes the seed fail: 'null value in column "project_id" violates not-null constraint'.
apply-db: done.
--- apply-db.sh exit code: 0 ---
```

### 3.3 §2.2 verification queries in that scratch build — verbatim

Three zero-counts:

```
 sessions_missing_agent            sessions_agent_runtime_mismatch    seats_agent_runtime_mismatch
------------------------          ---------------------------------  ------------------------------
                      0                                          0                               0
```

Domain checks, all inside the §2.1 value-domain contract:

```
   agent                harness         default_agent      default_harness |   default_model
------------          ---------      ---------------     -----------------+-------------------
 ClaudeCode                          ClaudeCode           Stepcode        | claude-opus-5[1m]
 Custom                  (NULL)      (1 row)             (1 row)
 GeminiCli             (1 row)
(3 rows)

     role
---------------
 architect
 data_engineer
 designer
 product_owner
 supervisor
 verifier
(6 rows)
```

Row state — six seats carry the full triple:

```
  name  | default_runtime | default_harness | default_agent |   default_model
--------+-----------------+-----------------+---------------+-------------------
 aegis  | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 flux   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 lyra   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 mira   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 nimbus | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 onyx   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
(6 rows)
```

**`NULL` harness on all five April session rows** — the packet's emphasised assertion:

```
            id            |  runtime   | harness |   agent    | model
--------------------------+------------+---------+------------+-------
 ses-aegis-supervisor-001 | ClaudeCode |         | ClaudeCode |
 ses-flux-verify-001      | Custom     |         | Custom     |
 ses-lyra-coord-001       | GeminiCli  |         | GeminiCli  |
 ses-mira-ui-001          | GeminiCli  |         | GeminiCli  |
 ses-nimbus-infra-001     | ClaudeCode |         | ClaudeCode |
(5 rows)

 april_sessions_with_nonnull_harness -> 0
 april_sessions_with_nonnull_model   -> 0
 seats_with_full_triple              -> 6
```

`008` really did apply post-seed, so the new ordering did not quietly drop it:

```
 column_name | is_nullable          sessions_missing_project_id
-------------+-------------         -----------------------------
 agent       | NO                                             0
 project_id  | NO
```

Deprecation comments survive a from-scratch build:

```
seats.default_runtime -> DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. S…
sessions.runtime      -> DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. S…
```

---

## 4. A second consecutive run still fails — pre-existing, proved by control experiment

I found this while testing and am reporting it rather than leaving it for the next gate.

Re-running `apply-db.sh` against the **already-populated** scratch database fails:

```
RE-RUN EXIT CODE = 3
--- seed (001-004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql
ERROR:  null value in column "project_id" of relation "sessions" violates not-null constraint
DETAIL:  Failing row contains (ses-aegis-supervisor-001, seat-aegis-001, ClaudeCode, null, …)
```

Cause: once `008` has tightened `sessions.project_id` to `NOT NULL`, seed 001's session `INSERT`
supplies no `project_id`. `ON CONFLICT` cannot help — `NOT NULL` is checked before conflict
resolution, so the re-insert fails before it can become an update.

**Control experiment — this is NOT a regression I introduced.** `track/infra-foundation`'s own
script, its own files, no `009` anywhere, extracted to `/tmp` via `git archive` so no branch or
worktree was touched, against a separate scratch DB:

```
$ ls -1 /tmp/onyx-ctl/infra/postgres/schema/     -> 001..005, 008   (no 009)
CONTROL RUN 1 EXIT = 0
CONTROL RUN 2 EXIT = 3
ERROR:  null value in column "project_id" of relation "sessions" violates not-null constraint
```

| Build | `apply-db.sh` run 1 | run 2 |
|---|---|---|
| `track/infra-foundation` (pre-merge) | **0** | **3** — `project_id` NOT NULL |
| merge candidate, before this fix | **3** — `default_harness` missing | — |
| merge candidate, after this fix | **0** | **3** — `project_id` NOT NULL, *identical to track* |

So the fix restores **exact parity with `track` on the first-run path**, and the second-run defect
is pre-existing, unchanged, and independent of `009`. CI is green today only because it always
builds from an empty database.

This is drift-report finding **D-3** (`docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md`)
appearing on the CI path. The minimal fix is adding `project_id` to seed 001's session tuples —
**a `.sql` change, which packet §3 forbids**, so I did not make it. Recommend a scoped Onyx
follow-up. Flagging it now because "apply-db.sh is idempotent" is a reasonable thing for someone to
assume, and it is not true.

---

## 5. Tooling: the `psql` shim, and what it does NOT prove

**There is no `psql` client on this host** — confirmed again after cleanup:

```
$ command -v psql
psql: not found on PATH
```

`apply-db.sh` requires one. As Flux did, I bridged it with a temporary shim at
`/tmp/onyx-shim/psql` forwarding to `podman exec -i seatloom-postgres psql`, translating
`-f <hostfile>` to stdin because the container cannot see host paths.

> **Limitation, stated plainly as the packet requires: `apply-db.sh`'s own host-`psql` invocation
> path remains UNEXERCISED on this seat.** Every transcript above proves the *ordering*, the SQL,
> and the resulting row state. None of it proves that `apply-db.sh` works with a real host `psql`
> binary — argument handling, connection-string parsing, and `-f` file reading all went through my
> shim instead. A shimmed run is not evidence for the real path. I did not install `psql`; Lyra is
> raising that separately.

Shim removed at the end (§7).

---

## 6. §3 Out-of-scope items — confirmed untouched

| Forbidden | Status |
|---|---|
| Any `.sql` change | **None.** `git diff --name-only 580bcf4 HEAD -- '*.sql'` → empty. |
| Fix the podman-compose gap | Not touched. Flux's §2.2 finding stands for its own packet. |
| Run full `verify-postgres-baseline.sh` | **Never run.** Its line 59 `compose down -v` would destroy the live volume. |
| Edit `.rs` / `ui/` / `src-tauri/` | **None** — §9 proof. |
| Build the proper migration runner | Not built. Header points at the tracked Onyx follow-up. |
| Touch Flux's worktree | **Never touched** — §7. |
| Write to the live `seatloom` database | **Never written** — §7. |

---

## 7. Cleanup and final state

```
=== databases before cleanup ===          === after cleanup ===
seatloom                                  seatloom            <- live DB only
seatloom_ctl_onyx
seatloom_verify_onyx

=== LIVE DB INTACT? (recorded seats=6 sessions=5 before starting) ===
live seats=6 sessions=5                   <- unchanged

=== shim + control tree ===               both removed
=== psql on host ===                      psql: not found on PATH
```

Both scratch databases dropped. The guard-probe file removed. The `/tmp/onyx-ctl` control tree and
`/tmp/onyx-shim` shim removed. The live `seatloom` database was **read** twice (row counts, to prove
it intact) and **never written**. Mr. Zhang's app: pid 91929 was **not running** when I started
(`ps -p 91929` → no such process), so nothing was attached; I stayed off the live database anyway.

Flux's worktree, untouched throughout:

```
$ git -C ~/Documents/GitHub/seatloom-worktrees/flux rev-parse --abbrev-ref HEAD
packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1
$ git -C ~/Documents/GitHub/seatloom-worktrees/flux rev-parse HEAD
580bcf483e0ca9af293367e0602a802226ce1590
$ git -C ~/Documents/GitHub/seatloom-worktrees/flux status --short
(empty — clean)
```

---

## 8. BLOCKED: the packet branch ref could not be advanced

**Done-definition item 1 is half-done and item 6 cannot be completed as written. This needs one
decision from Lyra; everything else is finished.**

The packet says to work in the main checkout and extend the packet branch. Git refuses, because
Flux's worktree still holds that branch:

```
$ git checkout packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1
fatal: 'packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1' is already used by worktree
       at '/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux'
```

Three constraints — *work in the main checkout*, *extend the packet branch*, *do not touch Flux's
worktree* — cannot all hold while that worktree exists on that branch. (The same shape as the
ordering collision this packet fixes: each rule locally correct, jointly unsatisfiable.)

**What I did instead:** committed everything on a **detached HEAD** at `580bcf4`. This contends no
ref and leaves Flux's worktree byte-identical. The work is complete and verifiable — it is a real
commit chain, just not yet pointed to by the branch name.

**What I deliberately did NOT do:** force the ref with `git update-ref`. That would move the branch
under Flux's feet while their worktree has it checked out, making their `git status` show spurious
modifications and destroying the clean-tree evidence their §14.2 identity gate depends on. Silently
corrupting another seat's verification state to satisfy my own done-definition is not a trade I will
make without being asked.

**Ask — pick one:**

1. **Preferred.** Flux (or Lyra) releases the worktree —
   `git worktree remove ~/Documents/GitHub/seatloom-worktrees/flux` — then I run one command,
   `git branch -f packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1 <MERGE_CANDIDATE>`, and
   report the final SHA. Flux's run is finished and its worktree is clean, so nothing is lost.
2. Flux re-verifies the **detached SHA directly** (`git checkout --detach <MERGE_CANDIDATE>` in
   their own worktree). The merge candidate is one verifiable SHA either way — that was the stated
   reason for "extend, do not branch", and a detached SHA satisfies it.
3. Authorise me to move the ref anyway, accepting that Flux's worktree will report a dirty status
   until they reset it.

Until then the commits are reachable only by SHA. They are not garbage-collectable in the short
term, but a `git gc --prune` on an unreferenced chain eventually is — so this should be resolved
rather than left indefinitely.

---

## 9. Scope proof and the merge-candidate SHA

```
$ git show --stat cfdd647
 scripts/apply-db.sh | 138 +++++++++++++++++++++++++++++++++++++++++++---------
 1 file changed, 114 insertions(+), 24 deletions(-)

$ git show --name-only --format= cfdd647 | grep -E '\.(sql|rs)$|^ui/|^src-tauri/'
ZERO — none

$ git diff --name-only track/infra-foundation HEAD | grep -E '\.rs$|^ui/|^src-tauri/'
ZERO .rs/ui/src-tauri

$ git diff --name-only 580bcf4 HEAD -- '*.sql'
ZERO .sql changed since 580bcf4
```

Full merge-candidate delta vs `track/infra-foundation` — 8 files, all from this packet or the
original runtime-triple packet:

```
docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md
docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md
docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md
infra/postgres/schema/009_runtime_triple.sql          <- original packet, unchanged today
infra/postgres/seed/001_real_collaboration_baseline.sql <- original packet, unchanged today
scripts/apply-db.sh                                    <- TODAY's only change
scripts/bootstrap.sh                                   <- original packet (009 registration)
scripts/verify-postgres-baseline.sh                    <- original packet + Flux 865f820
```

**`MERGE_CANDIDATE` = the commit that adds this file.** Its parent is `cfdd647`. Flux should verify
that SHA; §8 explains why it is not yet on the branch name.

---

## 10. Done-definition status (§4)

| # | Criterion | Status |
|---|---|---|
| 1 | `track` (≥ `281beaa`) merged; one verifiable SHA | ⚠️ **merged** (`ef4af6b`, from `3bf9017`) and one SHA — but reachable by SHA, not yet by branch name (§8) |
| 2 | `001–005` + `009` pre-seed, seed, `008` post-seed | ✅ §2, §3.2 |
| 3 | placement justified per migration by rule, in a comment | ✅ §2 — per-entry reasons, printed at apply time, plus the split rule and the completeness guard |
| 4 | `apply-db.sh` exits 0 on empty scratch DB, verbatim | ✅ §3.2 |
| 5 | §2.3 queries pass; `NULL` harness on all five April rows | ✅ §3.3 |
| 6 | scratch DB dropped; shim removed; tree clean; back on `track` | ⚠️ scratch DBs dropped, shim removed, tree clean — **returning to `track` deferred until §8 is resolved**, so the work is not left unreachable |
| 7 | delivery states the unexercised host-`psql` limitation | ✅ §5 |
| 8 | zero `.sql` / `.rs` / `ui/` / `src-tauri/` changed | ✅ §9 |
| 9 | Flux re-verify PASS | ⏳ pending, needs §8 resolved first |

**7 of 9 complete. Both exceptions are the single branch-ref blocker in §8, not unfinished work.**

---

## 11. Coordination

- **To Lyra:** pick an option in §8 and I will finish in one command. Two substantive items for
  your queue: the pre-existing **second-run failure** (§4, D-3 on the CI path — needs a `.sql`
  change I am forbidden to make), and the still-unexercised **host-`psql`** path (§5).
- **To Flux:** re-verify `MERGE_CANDIDATE` (§9). `apply-db.sh` from an empty DB is exit 0; §3.1
  reproduces your FAIL first so the fix is demonstrated, not asserted. Please expect **exit 3 on a
  second consecutive run** and do not read it as a regression — §4 has the control experiment
  showing `track` behaves identically. Your worktree is untouched and clean (§7).
- **To Nimbus:** `apply-db.sh` is now the one place the pre/post-seed rule is written down. A
  column-adding migration goes pre-seed; one that constrains columns the seed omits goes post-seed;
  one needing both must be split. An unclassified migration now aborts with exit 2 rather than being
  skipped.

---

*Onyx · 2026-09-30 · clears FLUX-2026-09-30-…-delivery-v2 FAIL · apply-order contract only, no SQL
touched · blocked on one branch-ref update (§8)*
