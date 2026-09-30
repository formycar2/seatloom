# Delivery v3: Commit-pinned re-verification — Onyx runtime triple after the apply-db ordering fix

[Flux -> Lyra] Verdict: **PASS**, with one recorded open defect (pre-existing, independently confirmed)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v3 |
| status | delivered |
| author | flux |
| date | 2026-09-30 |
| to | lyra |
| verifies | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-v3.md` |
| supersedes | `…-delivery-v2.md` (FAIL, accepted and addressed) |
| onyx delivery under test | `docs/coordination/tasks/onyx/ONYX-2026-09-30-apply-db-ordering-fix-delivery-v1.md` |
| target branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| **verified commit** | **`d4f40b58edd075a33222745379f18c16e9c0138a`** |
| previous target | `580bcf4` (FAILed) |
| verdict | **PASS** |
| tags | flux, verification, commit-pinned, runtime-triple, onyx, v3, pass |

---

## 0. Verdict

**PASS.** The v2 FAIL is fixed: `apply-db.sh` builds the merge candidate from an empty database at
exit 0, and every assertion re-proves from scratch. Onyx's `.sql`-untouched boundary holds. And
§3 — the crux — **reproduces independently**: the failing second run is pre-existing on
`track/infra-foundation` with no `009` anywhere, so it is not a regression this merge introduces.

Two deviations from the packet's own predictions, neither of them a defect, both detailed below:

- **§1 ancestry**: the packet expected exactly 3 commits in `580bcf4..d4f40b5`; there are **42**,
  because `ef4af6b` is a merge of `track` into the packet branch. Onyx disclosed this in its
  delivery §1. Ancestry is clean, nothing rewritten, and I verified the candidate contains nothing
  beyond track plus the runtime-triple work. **Not a HOLD on you.**
- **§3 control**: I ran mine differently from Onyx (live `track` checkout rather than
  `git archive` to `/tmp`) and reached the same conclusion by an independent path, which is the
  point of not inheriting the claim.

Carried by you, not counted against this verdict: `verify-postgres-baseline.sh` remains unrunnable
(podman has no compose provider); I did not run it.

---

## 1. Identity and ancestry

```
$ git worktree list
/Users/…/seatloom                 abb14dd [track/infra-foundation]
/Users/…/seatloom-worktrees/flux  d4f40b5 [packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1]

$ git -C …/seatloom-worktrees/flux rev-parse --abbrev-ref HEAD
packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1
$ git -C …/seatloom-worktrees/flux rev-parse HEAD
d4f40b58edd075a33222745379f18c16e9c0138a     <- MATCHES target
$ git -C …/seatloom-worktrees/flux status --short
(empty — clean)
```

Clean at start and at end. §14.2 identity holds.

### 1.1 Ancestry — clean, but not the shape the packet predicted

```
$ git merge-base --is-ancestor 580bcf4 d4f40b5 && echo true
true

$ git rev-list --count 580bcf4..d4f40b5
42                                    <- packet §1 expected 3

$ git log -1 --format='%h parents: %p' ef4af6b
ef4af6b parents: 580bcf4 3bf9017      <- a MERGE, and this is why

$ git log -1 --format=%p d4f40b5  -> cfdd647
$ git log -1 --format=%p cfdd647  -> ef4af6b
```

The chain `580bcf4 → ef4af6b → cfdd647 → d4f40b5` is exactly as you described, each a
single-parent step except `ef4af6b`, which is the declared merge of `track` @ `3bf9017`. The 42
commits are track's own history arriving through that merge, not unexplained work. **No commit was
rewritten and nothing was lost** — `580bcf4` is still an ancestor. Your `cfdd647`/reflog note is
consistent with what is on disk: `cfdd647` is the fix commit and `d4f40b5` its delivery doc, so
stopping at `cfdd647` would indeed have dropped only that doc.

Your instruction was that unclean ancestry is a HOLD on you. It is clean; the packet's "expect
exactly 3" was simply written without accounting for the merge Onyx performed and disclosed. No
HOLD.

### 1.2 The `.sql` boundary — PROVEN, not accepted

```
$ git diff --stat 580bcf4..d4f40b5 -- '*.sql'
(empty)
```

**No `.sql` file changed.** Onyx's central constraint holds.

### 1.3 What Onyx actually authored, and what the merge candidate really is

Since the merge widened the range, I checked the candidate against the track tip it merged rather
than trusting the range alone.

```
$ git show --stat cfdd647   -> scripts/apply-db.sh | 138 +++-   (1 file, +114/−24)
$ git show --stat d4f40b5   -> ONYX-…-ordering-fix-delivery-v1.md | 495 +++   (1 file)

$ git diff --stat 3bf9017..d4f40b5 -- '*.rs' 'ui/' 'src-tauri/'
(empty)                     <- zero code drift versus track

$ git diff --stat 3bf9017..d4f40b5
 …onyx-runtime-triple-drift-report-v1.md            | 376 +++
 …runtime-triple-migration-delivery-v1.md           | 732 +++
 …ONYX-2026-09-28-runtime-triple-migration-v1.md    | 394 +++
 …ONYX-2026-09-30-apply-db-ordering-fix-delivery…  | 495 +++
 infra/postgres/schema/009_runtime_triple.sql       | 176 +++
 infra/postgres/seed/001_real_collaboration_…sql    | 117 ++-
 scripts/apply-db.sh                                | 138 ++-
 scripts/bootstrap.sh                               |   3 +-
 scripts/verify-postgres-baseline.sh                |  14 +-
 9 files changed, 2404 insertions(+), 41 deletions(-)
```

The merge candidate over track is **exactly** the runtime-triple work plus the ordering fix:
the 009 migration, the seed realignment, the three script registrations, four Onyx documents.
Zero `.rs`, zero `ui/`, zero `src-tauri/`. The v1 §2.1 scope assertions therefore still hold at
`d4f40b5`, now measured against the tip it will merge into.

Onyx touched one file today, as it claimed.

---

## 2. Re-run of what failed in v2 — PASS

Same tooling caveats as v2, restated: there is **no `psql` client on this host**, so I used a
temporary shim at `/tmp/flux-shim/psql` forwarding to `podman exec -i seatloom-postgres psql`,
translating `-f <hostfile>` to stdin. Removed afterwards. **`apply-db.sh`'s own host-`psql`
invocation path therefore remains unexercised on this seat** — unchanged from v2, and it will
block the Nimbus gate. `scripts/apply-db.sh` now exists at the target, so unlike v2 no copy-in was
needed.

### 2.1 Scratch build from empty — exit 0

```
$ DATABASE_URL=…/seatloom_verify_v3 bash scripts/apply-db.sh
apply-db: target=postgresql://seatloom:seatloom@localhost:5432/seatloom_verify_v3
--- pre-seed schema (seed writes columns these create) ---
  infra/postgres/schema/001_seatloom_core.sql
      why: creates projects/seats/sessions/… — every table the seed inserts into
  … 002 / 003 / 004 / 005 …
  infra/postgres/schema/009_runtime_triple.sql
      why: ADDS seats.default_harness/_agent/_model and sessions.harness/agent/model, which seed
           001 WRITES — must exist before the seed runs. Safe here: it only touches columns
           created in 001 and constrains nothing the seed omits (its sole SET NOT NULL is on
           sessions.agent, which the seed supplies).
--- seed (001-004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql … 004_prompt_and_channel_action_seed.sql
--- post-seed schema (constrains columns the seed does not provide) ---
  infra/postgres/schema/008_project_isolation.sql
      why: backfills project_id … then SET NOT NULL. Seed 001 supplies no project_id …
apply-db: done.

RUN 1 exit: 0
```

The rule-based ordering is the remedy I verified in v2 §3.1, now implemented with a per-migration
justification printed at apply time. The v2 FAIL is cleared.

### 2.2 Assertions in that from-scratch build — all PASS

```
 sessions_missing_agent          -> 0
 sessions_agent_runtime_mismatch -> 0
 seats_agent_runtime_mismatch    -> 0
 april_sessions_nonnull_harness  -> 0

  name  | default_runtime | default_harness | default_agent |   default_model
--------+-----------------+-----------------+---------------+-------------------
 aegis  | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 flux   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 lyra   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 mira   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 nimbus | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 onyx   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
(6 rows)

            id            |  runtime   | harness |   agent    | model
--------------------------+------------+---------+------------+-------
 ses-aegis-supervisor-001 | ClaudeCode |         | ClaudeCode |
 ses-flux-verify-001      | Custom     |         | Custom     |
 ses-lyra-coord-001       | GeminiCli  |         | GeminiCli  |
 ses-mira-ui-001          | GeminiCli  |         | GeminiCli  |
 ses-nimbus-infra-001     | ClaudeCode |         | ClaudeCode |
(5 rows)
```

All five April rows: NULL `harness`, NULL `model`. The one that must not slip has not slipped.

Value domains, all inside the §2.1 contract:

```
sessions.agent           -> ClaudeCode, Custom, GeminiCli
sessions.harness         -> NULL only
seats.default_agent      -> ClaudeCode
seats.default_harness/_model -> Stepcode | claude-opus-5[1m]
project_role_bindings.role -> architect, data_engineer, designer, product_owner, supervisor, verifier
```

---

## 3. The crux — Onyx's blamelessness claim, independently reproduced

I did not inherit the claim. I ran my own control against the **live `track` checkout** (Onyx used
`git archive` to `/tmp`), so the two results are independent in method as well as in operator.

Control validity first — `track` really is a no-`009` world:

```
$ grep -q 'pre-seed schema' scripts/apply-db.sh   -> absent (old bucket version, not Onyx's fix)
$ ls infra/postgres/schema/ | grep -c 009          -> 0
$ grep -c default_harness infra/postgres/seed/001_real_collaboration_baseline.sql -> 0
```

### 3.1 Result — exact parity

| | run 1 | run 2 | error on run 2 | failing row |
|---|---|---|---|---|
| **Control — `track` @ `abb14dd`, no `009` anywhere** | **exit 0** | **exit 3** | `null value in column "project_id" of relation "sessions" violates not-null constraint` | `ses-aegis-supervisor-001` |
| **Candidate — `d4f40b5`, with `009`** | **exit 0** | **exit 3** | `null value in column "project_id" of relation "sessions" violates not-null constraint` | `ses-aegis-supervisor-001` |

Same exit code, same message, same relation, same failing row id, and in both cases the failure is
at `infra/postgres/seed/001_real_collaboration_baseline.sql`.

### 3.2 And `009` is provably not implicated

In the candidate's own failing run, `009` applied cleanly and the failure came afterwards:

```
/tmp/flux-v3-run2.txt:
  line  99:  infra/postgres/schema/009_runtime_triple.sql      <- applied, no error
  line 107: --- seed (001-004) ---
  line 109: ERROR:  null value in column "project_id" …        <- fails here, after 009 succeeded
```

The only difference between the two `DETAIL` tuples is that the candidate's failing row carries
three extra trailing columns — the ones `009` adds — which is what you would expect and is not the
cause.

**Mechanism** (for the follow-up packet): seed 001 inserts `sessions` rows with no `project_id`.
Run 1's `008` backfills from those rows and then sets `project_id NOT NULL`. On run 2 the seed
re-inserts the same rows against the now-NOT NULL column and is rejected — `ON CONFLICT (id) DO
NOTHING` does not save it, because the proposed row is constraint-checked before conflict
arbitration. Nothing in that chain involves `009`. This is drift finding **D-3** surfacing on the
CI path, exactly as Onyx said.

### 3.3 My independent conclusion

**Confirmed: pre-existing, not a regression introduced by this merge.** Per your §3 decision rule
this is *yes = parity*, so: PASS with a recorded open defect. Onyx's claim was accurate.

One caveat I want stated rather than buried, because it bounds what I proved: the control used
track's own bucket-based `apply-db.sh` and the candidate used Onyx's rule-based one, since that is
what each branch actually ships. So this proves **`track` today and the merge candidate behave
identically on run 1 and fail identically on run 2**. It does not isolate script-version from
`.sql` as variables — it does not need to, because §3.2 shows the failure mechanism never touches
`009` or the ordering change.

**Open defect for your follow-up packet:** `apply-db.sh` is not re-runnable against an
already-seeded database. It needs a `.sql` change (give seed 001's session rows a `project_id`, or
make the seed's insert tolerate the constraint), which this packet forbade. It is unrelated to the
runtime triple and equally broken on `track` today.

---

## 4. Per-assertion table

| § | Assertion | Expected | Actual | Verdict |
|---|---|---|---|---|
| 1 | worktree HEAD == target | `d4f40b5` | `d4f40b58edd075a…` | PASS |
| 1 | worktree clean, start and end | empty | empty both | PASS |
| 1 | `580bcf4` ancestor of `d4f40b5` | true | true | PASS |
| 1 | commits in range | 3 | 42 (merge `ef4af6b` pulls in track) | deviation, benign, disclosed |
| 1 | no commit rewritten / nothing lost | clean | clean chain, `580bcf4` still ancestor | PASS |
| 1 | **no `.sql` changed `580bcf4..d4f40b5`** | **empty** | **empty** | **PASS** |
| 1.3 | Onyx authored only `apply-db.sh` (+ its doc) | 1 file | `cfdd647` = 1 file | PASS |
| 1.3 | no `.rs` / `ui/` / `src-tauri/` drift vs track | empty | empty | PASS |
| 2.1 | `apply-db.sh` from empty DB | exit 0 | exit 0 | PASS |
| 2.2 | `sessions_missing_agent` | 0 | 0 | PASS |
| 2.2 | `sessions_agent_runtime_mismatch` | 0 | 0 | PASS |
| 2.2 | `seats_agent_runtime_mismatch` | 0 | 0 | PASS |
| 2.2 | six seats `Stepcode`/`ClaudeCode`/`claude-opus-5[1m]` | 6 | 6 | PASS |
| 2.2 | NULL harness on all five April rows | 5 | 5 | PASS |
| 2.2 | DISTINCT domains in contract | in-contract | in-contract | PASS |
| **3** | **control: track also 0 then 3, same `project_id` error** | **parity** | **exact parity** | **PASS — claim reproduced** |
| 3.2 | `009` not implicated in run-2 failure | not implicated | applied clean, failure later at seed 001 | PASS |
| — | scratch DBs dropped | yes | both dropped | PASS |
| — | live DB untouched | untouched | `seats=6 sessions=5` | PASS |
| — | main checkout never switched | on `track` | on `track` throughout | PASS |
| 4 | `verify-postgres-baseline.sh` | — | not run, carried by Lyra | n/a by ruling |

---

## 5. Recommendation

**Merge.** The candidate is at exact behavioural parity with `track` on the first-run path, builds
from scratch, and the one failing path is demonstrably older than this work.

Three things to carry forward, none blocking:

1. **Issue the D-3 follow-up** — `apply-db.sh` not re-runnable against a seeded database. Needs a
   `.sql` change. Equally broken on `track` today, so it is not urgent for this merge, but it means
   no CI job can run the seed step twice.
2. **`verify-postgres-baseline.sh` still cannot run here** — podman has no compose provider. Needs
   the `run`/`rm`/`volume rm` port (v2 §2.2). Until then §2.6 can never be green on this seat.
3. **The Nimbus gate needs environment work first** — your v2 §6 gap (`ui/node_modules`, `ui/dist`,
   `target/` all absent from the worktree) plus **no `psql` client on this host at all**, which
   blocks any DB step that does not route through a container.

---

## 6. Closing state

```
$ git -C …/seatloom-worktrees/flux rev-parse HEAD
d4f40b58edd075a33222745379f18c16e9c0138a
$ git -C …/seatloom-worktrees/flux status --short
(empty — clean)

$ git -C …/seatloom rev-parse --abbrev-ref HEAD
track/infra-foundation

$ podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc \
    "SELECT datname FROM pg_database WHERE datname LIKE 'seatloom%'"
seatloom                          <- both scratch DBs gone
$ … -d seatloom -tAc "…"
seats=6 sessions=5                <- live DB intact
```

Scratch databases `seatloom_verify_v3` and `seatloom_ctl_track` were dropped. The `psql` shim was
removed. The **main checkout was never branch-switched** — it stayed on `track/infra-foundation`
for the whole run, and the worktree is still on the packet branch at the target commit. No stash
anywhere, nothing merged, the live database never written to, and
`scripts/verify-postgres-baseline.sh` never run. Mr. Zhang's running app was not disturbed.

Per scope I did not check out `packet/nimbus/...` and did not verify the Nimbus packet.

---

*Flux · 2026-09-30 · PASS · verified commit `d4f40b58edd075a33222745379f18c16e9c0138a` ·
worktree run, §14.2 identity held · `.sql` boundary proven empty · blamelessness claim
independently reproduced by a live-`track` control, not inherited*
