# Delivery v2: Commit-pinned verification — Onyx runtime triple (schema 009)

[Flux -> Lyra] Verdict: **FAIL**

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v2 |
| status | delivered |
| author | flux |
| date | 2026-09-30 |
| version | v2 |
| to | lyra |
| verifies | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-v2.md` |
| supersedes | `…-delivery-v1.md` (HOLD, accepted) |
| target branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| **verified commit** | **`580bcf483e0ca9af293367e0602a802226ce1590`** |
| run location | `/Users/…/seatloom-worktrees/flux` (dedicated worktree) |
| pre_fix_commit | `d71f6597eb3179f51fd2ffa9fbb806c3e3c62e98` |
| fix_commit | `865f8209fca51d01a09d67affed235fcec705f0e` |
| verdict | **FAIL** |
| tags | flux, verification, commit-pinned, schema-009, runtime-triple, onyx, worktree, v2, fail |

---

## 0. Verdict — FAIL, with blame explicitly not on Onyx

The §3 scratch-database build you asked for found what the v1 live-database observation was hiding.
**`scripts/apply-db.sh` cannot build this merge candidate from scratch.** It fails at seed 001 with
exit code 3:

```
--- seed (001–004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql
ERROR:  column "default_harness" of relation "seats" does not exist
LINE 1: INSERT INTO seats (id, name, default_runtime, default_harnes...
                                                      ^
```

**Root cause — two canonical orders in the repo that cannot both be satisfied.**

- Seed 001 at `580bcf4` writes `seats.default_harness/_agent/_model` and
  `sessions.harness/agent/model`, so it **requires 009 to run before the seed**. Onyx states this
  itself, in `scripts/verify-postgres-baseline.sh` at `580bcf4`: *"009 is required here, not
  optional. Seed 001 now writes seats.default_harness/_agent/_model … applying the seed without 009
  fails with 'column … does not exist'."*
- `scripts/apply-db.sh` applies schema `001–005` → seed `001–004` → **all** migrations `≥006`
  after the seed, deliberately, because *"008 sets sessions.project_id NOT NULL but seed 001 has no
  project_id"*. `009` falls in the `≥006` bucket, so it lands after the seed that depends on it.

Per packet §3 this is a FAIL, and the environmental/defect test comes out on the defect side: this
is not about my machine. It is a property of the merge candidate.

**Control experiment — this is a regression the merge would introduce, not pre-existing breakage:**

| Build | Files | `apply-db.sh` exit |
|---|---|---|
| `track/infra-foundation` (pre-merge) | main checkout @ `865f820` | **0 — green** |
| Merge candidate | worktree @ `580bcf4` | **3 — ERROR** |

`apply-db.sh` describes itself as the CI-compatible path. Merging `580bcf4` as-is breaks it.

**Why this is not Onyx's fault, and I want this on the record.** `apply-db.sh` was added by
`8feea1b` ("ci: GitLab pipeline + ratchet verifiers + engineering standards"), and
`git merge-base --is-ancestor` confirms that commit is **not** an ancestor of Onyx's base
`9599136`. The script did not exist when Onyx worked. Onyx correctly registered `009` in the two
scripts that did exist (`bootstrap.sh`, `verify-postgres-baseline.sh`), both of which apply schema
before seed, where its ordering is right. This is a cross-packet collision created by `8feea1b`
landing between Onyx's base and this gate — exactly the class of defect a commit-pinned merge gate
exists to catch, and invisible to any amount of live-database observation.

**The migration content itself is sound.** Fixed ordering, everything passes from scratch (§3.2).

**Remedy, verified working (§3.1):** move `009` into the pre-seed group. Onyx's own comment
establishes it is safe there — *"009 is independent of 008 — it only touches columns created in
001."* One-line change to `apply-db.sh`'s ordering, owner's call whether that is Onyx, Nimbus or the
build-standards owner. I did not make it: it is outside both this packet's scope and the §15
authorisation, and it is a change to the canonical build contract.

---

## 1. Worktree identity gate (§14.2) — PASS

HOLD-1 is genuinely cleared. The watcher cannot reach the worktree.

```
$ git worktree list
/Users/…/seatloom                 d71f659 [track/infra-foundation]
/Users/…/seatloom-worktrees/flux  580bcf4 [packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1]

$ git -C /Users/…/seatloom-worktrees/flux rev-parse --abbrev-ref HEAD
packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1
$ git -C /Users/…/seatloom-worktrees/flux rev-parse HEAD
580bcf483e0ca9af293367e0602a802226ce1590     <- MATCHES target
$ git -C /Users/…/seatloom-worktrees/flux status --short
(empty — clean)
```

Clean at start **and** clean at end (§6), across the whole run. No `capabilities.json` churn: the
`pnpm tauri dev` watcher is bound to the main checkout only, as you said.

---

## 2. §15 bounded fix — D-5 container-runtime detection

`pre_fix_commit` `d71f659` → `fix_commit` `865f820`. One file:
`scripts/verify-postgres-baseline.sh`, +30/−9.

**Naming deviation, flagged:** the packet suggested `$CTR`; I used **`CONTAINER`** to match
`scripts/bootstrap.sh`, which already uses that name and is the implementation D-5 recommended
porting. Same precedence (`CONTAINER` override → docker → podman → fail closed) and same error text
as bootstrap.sh.

9 mechanical substitutions, nothing else: `docker compose`→`"$CONTAINER" compose` (2),
`docker exec`→`"$CONTAINER" exec` (4), `docker cp`→`"$CONTAINER" cp` (2),
`docker logs`→`"$CONTAINER" logs` (1). No change to what the script asserts, its step order, or its
exit semantics. Full diff is in `865f820`; the detection block:

```bash
if [ -n "${CONTAINER:-}" ]; then
  if ! command -v "$CONTAINER" >/dev/null 2>&1; then
    echo "ERROR: CONTAINER=$CONTAINER but that binary is not on PATH." >&2
    exit 1
  fi
elif command -v docker >/dev/null 2>&1; then
  CONTAINER="docker"
elif command -v podman >/dev/null 2>&1; then
  CONTAINER="podman"
else
  echo "ERROR: neither docker nor podman found on PATH." >&2
  echo "Install Docker Desktop OR Podman (https://podman.io/docs/installation)." >&2
  exit 1
fi
echo "container runtime: $CONTAINER"
```

### 2.1 Proof of detection — and I stopped where you said

I did **not** run the full script. Line 59 `compose down -v` destroys the volume the running app is
attached to. Proof used `bash -n` plus a probe truncated before Step 0 (`awk '/^echo "--- Step
0/{exit}'`), verified to contain **zero** `compose up`/`compose down` lines:

```
$ bash -n scripts/verify-postgres-baseline.sh
(exit 0)

$ bash /tmp/flux-detect-probe.sh                  -> container runtime: podman      exit 0
$ CONTAINER=podman bash /tmp/flux-detect-probe.sh -> container runtime: podman      exit 0
$ CONTAINER=docker bash /tmp/flux-detect-probe.sh -> ERROR: CONTAINER=docker but
                                                     that binary is not on PATH.    exit 1
```

Auto-detect resolves to podman; explicit podman honoured; a missing runtime fails closed.

### 2.2 The fix is NECESSARY BUT NOT SUFFICIENT — for Nimbus review

§2.6 will still not run after this fix, for a second reason nobody had found yet: **podman has no
compose provider on this machine.**

```
$ podman compose version
Error: looking up compose provider failed
	* exec: "docker-compose": executable file not found in $PATH
	* exec: "/opt/homebrew/bin/docker-compose": no such file or directory
$ command -v podman-compose  -> NOT FOUND
```

So lines 59–60 (`"$CONTAINER" compose down -v` / `up -d`) still fail. The complete D-5 remedy is the
`run`/`rm`/`volume rm` form `bootstrap.sh` already uses (`CONTAINER_NAME=seatloom-postgres`,
`VOLUME_NAME=seatloom-pgdata`, `POSTGRES_IMAGE=postgres:16-alpine`). I did not write it: that is
container **provisioning**, not detection, so it is outside the §15 authorisation — and it lives in
the destructive path this packet forbids me to execute, so I could not have tested it. Writing
untested code into the one path I am barred from running would be worse than leaving it. Recommend
a scoped packet.

This does not affect the verdict: §2.6 stays formally unproven and carried by you, as ruled.

---

## 3. §3 scratch-database build

### 3.0 Two environmental gaps I had to bridge — disclosed, both cleaned up

Neither is Onyx's defect; both are dispatch-planning gaps worth knowing for the Nimbus gate.

1. **`scripts/apply-db.sh` does not exist at `580bcf4`** (it arrived with `8feea1b`, after Onyx's
   base). I copied it in from `track/infra-foundation` @ `865f820`
   (sha256 `b7017fc8e167d7a97ee61b2306256e394a05fff78560ef992e77c29a5e23fe5f`) so it would resolve
   its `cd "$(dirname "$0")/.."` to the worktree and operate on the packet's schema/seed files.
   Removed afterwards; worktree is clean.
2. **There is no `psql` client on this host at all** — not on PATH, and no brew `libpq`/`postgresql`
   — yet `apply-db.sh` requires it. I used a temporary shim at `/tmp/flux-shim/psql` forwarding to
   `podman exec -i seatloom-postgres psql`, translating `-f <hostfile>` to stdin since the container
   cannot see host paths. Deleted afterwards.

   **Limitation, stated plainly:** this proves the SQL, the ordering and the row state. It does not
   prove `apply-db.sh`'s own host-`psql` invocation path, which remains unexercised on this seat.

All work used scratch databases. The live `seatloom` database was never written to.

### 3.1 Canonical-order build — FAIL (exit 3)

```
$ DATABASE_URL=…/seatloom_verify bash scripts/apply-db.sh
apply-db: target=postgresql://seatloom:seatloom@localhost:5432/seatloom_verify
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

Remedy verified: `001–005` **+ 009** pre-seed → seed `001–004` → `008` post-seed, from an empty
database, **exit 0**, every file clean:

```
-- schema/001_seatloom_core.sql … 005_prompt_and_channel_action_authority.sql
-- schema/009_runtime_triple.sql
-- infra/postgres/seed/001_real_collaboration_baseline.sql … 004_prompt_and_channel_action_seed.sql
-- schema/008_project_isolation.sql (post-seed)
=== corrected-order build exit: 0 ===
```

### 3.2 From-scratch query results — order-correctness, not a lucky outcome

Against the corrected from-scratch build. This is the v1 order-dependence argument answered:
`seats_agent_runtime_mismatch = 0` now holds on a database built from an empty state, so it is
order-correct rather than an artifact of `18a69fb` having landed before `009` on the live DB.

```
 sessions_missing_agent          -> 0
 sessions_agent_runtime_mismatch -> 0
 seats_agent_runtime_mismatch    -> 0

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

All five April rows have NULL `harness` **and** NULL `model` on a from-scratch build. No fabricated
attribution. §2.5 deprecation comments also present on the scratch DB, both naming the packet:

```
 seats.default_runtime -> DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. Sup…
 sessions.runtime      -> DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. Sup…
```

### 3.3 Idempotency — PASS, both transcripts

Re-applied `009` twice against the **populated** scratch database (the case that matters: the
backfill guards must no-op with rows present). Apply #1 is the build in §3.1.

```
=== 009 APPLY #2 ===                        === 009 APPLY #3 ===
BEGIN                                       BEGIN
ALTER TABLE                                 ALTER TABLE
NOTICE: column "default_harness" … skipping NOTICE: column "default_harness" … skipping
NOTICE: column "default_agent" … skipping   NOTICE: column "default_agent" … skipping
ALTER TABLE                                 ALTER TABLE
NOTICE: column "default_model" … skipping   NOTICE: column "default_model" … skipping
ALTER TABLE                                 ALTER TABLE
NOTICE: column "harness" … skipping         NOTICE: column "harness" … skipping
ALTER TABLE                                 ALTER TABLE
NOTICE: column "agent" … skipping           NOTICE: column "agent" … skipping
ALTER TABLE                                 ALTER TABLE
NOTICE: column "model" … skipping           NOTICE: column "model" … skipping
UPDATE 0                                    UPDATE 0
UPDATE 0                                    UPDATE 0
ALTER TABLE                                 ALTER TABLE
COMMENT x8                                  COMMENT x8
COMMIT                                      COMMIT
```

Both exit 0. **`UPDATE 0` on both backfills in both applies** — the `IS NULL` guards hold, so no
row is rewritten on a re-run. `diff` of the two transcripts sorted is empty (the raw diff showed
only stdout/stderr interleaving of `NOTICE` vs `ALTER TABLE`, not a semantic difference).

State after three applies is unchanged:

```
 sessions_missing_agent              -> 0
 sessions_agent_runtime_mismatch     -> 0
 seats_agent_runtime_mismatch        -> 0
 april_sessions_with_nonnull_harness -> 0
 seats_with_full_triple              -> 6
```

Idempotency (Onyx packet §7.5) is proven. This is the criterion v1 could not attempt.

---

## 4. Per-assertion table

| § | Assertion | Expected | Actual | Verdict |
|---|---|---|---|---|
| 1 | worktree HEAD == target | `580bcf4` | `580bcf483e0ca9…` | PASS |
| 1 | worktree clean, start and end | empty | empty both times | PASS |
| 2 | §15 fix is detection-only | mechanical | 9 substitutions + detection block, `865f820` | PASS |
| 2.1 | detection resolves, no destructive run | podman | podman; `CONTAINER=docker` exits 1; full script never run | PASS |
| 2.2 | fix makes §2.6 runnable | — | **NO** — podman has no compose provider | flagged, not blocking |
| **3.1** | **`apply-db.sh` builds from scratch** | **exit 0** | **exit 3, `default_harness` does not exist** | **FAIL** |
| 3.1 | control: same script on `track` | exit 0 | exit 0 | PASS (proves regression) |
| 3.1 | 009-pre-seed remedy builds | exit 0 | exit 0 | PASS |
| 3.2 | `sessions_missing_agent` from scratch | 0 | 0 | PASS |
| 3.2 | `sessions_agent_runtime_mismatch` from scratch | 0 | 0 | PASS |
| 3.2 | `seats_agent_runtime_mismatch` from scratch | 0 | 0 | PASS (now order-correct) |
| 3.2 | 6 seats carry the triple | 6 | 6 | PASS |
| 3.2 | April rows NULL harness and model | 5 NULL | 5 NULL | PASS |
| 3.2 | deprecated columns retained + commented | both | both | PASS |
| 3.3 | second apply is a clean no-op | no-op | `UPDATE 0` ×2, exit 0, ×2 applies | PASS |
| 2.6 | baseline verifier green | — | unproven, carried by Lyra | n/a by ruling |
| — | scratch DBs dropped | yes | all 3 dropped | PASS |
| — | live DB untouched | untouched | `seats=6 sessions=5`, never written | PASS |
| — | main checkout never branch-switched | on `track` | on `track` throughout | PASS |

Carried forward from v1 by §1 and **re-proved from scratch** in §3.2: §2.1 scope constraints, §2.3
queries, §2.4 row state, §2.5 retained columns, §2.7 artifacts. `18a69fb` ruled in packet §4; not
re-litigated.

---

## 5. What clears this FAIL

One decision, not a rework of Onyx's migration:

1. **Resolve the ordering contract.** Either move `009` to the pre-seed group in `apply-db.sh`
   (verified working in §3.1; safe per Onyx's "009 only touches columns created in 001"), or make
   the general rule explicit: *a migration that adds columns the seed writes must run pre-seed;
   only migrations that depend on seeded rows go post-seed.* `008` is post-seed because it needs
   rows; `009` is pre-seed because the seed needs its columns. Today's `≥006` bucket cannot express
   that distinction, so this will recur on the next column-adding migration.
2. **Re-run §3.1 after the fix.** Everything else in §3 already passes; the corrected build is
   proven green, so this should convert to PASS immediately.

Two items I am handing back rather than acting on:

- **`$CONTAINER compose` has no provider** (§2.2) — needs the `run`/`rm`/`volume rm` port before
  any §2.6 can ever be green here.
- **The Nimbus gate will need more than this worktree has** — your §6 gap is real, and add to it
  that there is **no `psql` on this host**, which will block any DB step that does not go through a
  container.

Re-dispatch as `…-v3` once the ordering call is made and I will re-run §3.1.

---

## 6. Closing state

```
$ git -C /Users/…/seatloom-worktrees/flux rev-parse --abbrev-ref HEAD
packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1
$ git -C /Users/…/seatloom-worktrees/flux rev-parse HEAD
580bcf483e0ca9af293367e0602a802226ce1590
$ git -C /Users/…/seatloom-worktrees/flux status --short
(empty — clean)

$ git -C /Users/…/seatloom rev-parse --abbrev-ref HEAD
track/infra-foundation

$ podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc \
    "SELECT datname FROM pg_database WHERE datname LIKE 'seatloom%'"
seatloom                                  <- scratch DBs gone, live DB only
$ podman exec -i seatloom-postgres psql -U seatloom -d seatloom -tAc "…"
seats=6 sessions=5                        <- live DB intact
```

**Main checkout was never branch-switched** — it stayed on `track/infra-foundation` for the whole
run; all packet-branch work happened in the worktree. All three scratch databases
(`seatloom_verify`, `seatloom_verify_track`, `seatloom_verify_pkt`) were dropped. The temporary
`apply-db.sh` copy, the `psql` shim and the detection probe were all removed. No stash anywhere.
Nothing was merged. The live database was never written to, and Mr. Zhang's running app was not
disturbed.

One note: `scripts/seat-dispatch.sh` appeared untracked in the main checkout during my run. Not
mine — I left it alone.

Per packet scope I did not check out `packet/nimbus/...` and did not verify the Nimbus packet.

---

*Flux · 2026-09-30 · FAIL · verified commit `580bcf483e0ca9af293367e0602a802226ce1590` ·
worktree run, §14.2 identity held throughout · §15 fix `865f820` · the migration content is sound;
the FAIL is a cross-packet ordering collision that breaks `apply-db.sh` on merge*
