# Drift Report: Runtime Triple Migration — file store vs PostgreSQL

| Field | Value |
|---|---|
| template | T4 |
| subtype | drift_report |
| id | 2026-09-28-onyx-runtime-triple-drift-report-v1 |
| status | delivered |
| author | onyx |
| date | 2026-09-28 |
| packet_ref | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md` §2.5 |
| depends_on | `docs/coordination/reviews/2026-09-28-aegis-multi-harness-session-ingestion-architecture-v1.md` §8 item 2; `docs/architecture-decisions.md` AD-011 |
| base commit | `9599136d615db0c4cdb15811ccaca5532820870d` |
| delivery branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| tags | onyx, drift-report, runtime-triple, ad-011, seat-registry, schema-008 |

---

## 0. Scope and posture

This is a **report, not a fix** (packet §2.5, §3). Seven findings are recorded below. **None was
remediated**, except where explicitly noted as a hygiene action in D-1. Section 4 states exactly
what was left alone and why.

Two of the seven (D-3, D-5) were discovered while executing the packet's own §5 verification plan
and are **pre-existing at base commit**, independent of this packet's changes. They are recorded
here because they block or distort the verification path, not because the migration caused them.

---

## 1. §2.5 comparison table — file store vs PostgreSQL

Measured on the delivery branch, against the database built by this packet's §5 run.

| Object | File path | Files on disk | PG table | PG rows | Verdict |
|---|---|---|---|---|---|
| `SeatIdentity` | `.seatloom/seats/<name>/identity.yaml` | **0** (parent dir absent) | `seats` | **6** | 6 in PG, absent from disk |
| `ProjectRoleBind` | `.seatloom/seats/<name>/role-bindings/<project>.yaml` | **0** (parent dir absent) | `project_role_bindings` | **6** | 6 in PG, absent from disk |
| `SeatDelegation` | `.seatloom/delegations/<id>.yaml` | **0** (parent dir absent) | `seat_delegations` | **1** | 1 in PG, absent from disk |

Per-direction breakdown required by packet §2.5:

- **In PG but absent from disk:** all 13 rows (6 seats, 6 role bindings, 1 delegation).
- **On disk but absent from PG:** none — there are zero files of any of the three kinds.
- **Present in both but disagreeing:** none — the intersection is empty, so no field-level
  comparison is possible. **This is the reason no value-level diff appears in this report.**

Evidence:

```
$ ls -a .seatloom/
.
..
bootstrap
seat-sessions.tsv
transcripts

$ for d in .seatloom/seats .seatloom/delegations; do [ -e "$d" ] && echo "EXISTS: $d" || echo "ABSENT: $d"; done
ABSENT: .seatloom/seats
ABSENT: .seatloom/delegations

$ find .seatloom -name '*.yaml' -o -name '*.yml'
.seatloom/bootstrap/source-map.yaml
(count: 1)

$ podman exec seatloom-postgres psql -U seatloom -d seatloom \
    -c "SELECT (SELECT count(*) FROM seats) AS seats, (SELECT count(*) FROM project_role_bindings) AS role_bindings, (SELECT count(*) FROM seat_delegations) AS delegations;"
 seats | role_bindings | delegations
-------+---------------+-------------
     6 |             6 |           1
(1 row)
```

The single YAML under `.seatloom/` is `bootstrap/source-map.yaml`, which is the seed *evidence
basis* (ONYX.md "Primary Inputs"), not a `SeatIdentity` / `ProjectRoleBind` / `SeatDelegation`
artifact. It is unrelated to this comparison.

---

## 2. Findings

### D-1 — Packet §1.1 "facts measured at base commit" were measured against the uncommitted working tree — **HIGH**

Two rows of the packet's own fact table are false at the commit the packet pins.

| Packet §1.1 claim | Actual at base commit `9599136` |
|---|---|
| "Seed seat rows `:25-32` — six seats, all `default_runtime = 'ClaudeCode'`" | **five** seats; `default_runtime` is **mixed**: aegis `ClaudeCode`, lyra `GeminiCli`, mira `GeminiCli`, nimbus `ClaudeCode`, flux `Custom`. No `onyx` row. |
| "Seed role values `:40-45` — `supervisor`, `product_owner`, `designer`, `architect`, `verifier`, `data_engineer`" | **five** bindings; **no `data_engineer`**. |

The six-seat / all-`ClaudeCode` state — i.e. Addendum decisions 3 and 5 — existed only as an
**uncommitted working-tree modification** when this packet began.

Evidence:

```
$ git show 9599136:infra/postgres/seed/001_real_collaboration_baseline.sql | sed -n '19,25p'
INSERT INTO seats (id, name, default_runtime, capability_tags, status, created_at) VALUES
  ('seat-aegis-001', 'aegis',  'ClaudeCode',  ARRAY[...], 'active', '2026-04-24T00:00:00Z'),
  ('seat-lyra-001',  'lyra',   'GeminiCli',   ARRAY[...], 'active', '2026-04-24T00:00:00Z'),
  ('seat-mira-001',  'mira',   'GeminiCli',   ARRAY[...], 'active', '2026-04-24T00:00:00Z'),
  ('seat-nimbus-001','nimbus', 'ClaudeCode',  ARRAY[...], 'active', '2026-04-24T00:00:00Z'),
  ('seat-flux-001',  'flux',   'Custom',      ARRAY[...], 'active', '2026-04-24T00:00:00Z')
ON CONFLICT (id) DO NOTHING;

$ git show 9599136:infra/postgres/seed/001_real_collaboration_baseline.sql \
    | grep -c -i -E 'onyx|data_engineer'
0
```

**Disagreeing values:** `lyra`/`mira` `GeminiCli` and `flux` `Custom` (committed) vs `ClaudeCode`
(working tree); seat count 5 vs 6; role-binding count 5 vs 6.

**Why it matters.** Addendum decision 5 ("`default_runtime` realigned to `ClaudeCode` for all six
seats in the PostgreSQL seed") was recorded as *done* in durable memory while the change lived only
in an uncommitted working tree. Any clone of `9599136` — including any commit-pinned Flux verify —
would not have it. Durable memory asserted a state the repository did not contain.

**Hygiene action taken (the one exception to "report, not fix"):** that pre-existing change was
committed on its own as `18a69fb`, *before* any of this packet's work, with a message attributing
it as not authored by this packet. This keeps the packet's own delivery diff auditable rather than
silently absorbing someone else's change. No value was altered in the process — see the verbatim
`diff` in the delivery doc §7.

**Recommended owner:** Lyra — correct the §1.1 fact table, and confirm whether Addendum decision 5
should be considered landed as of `18a69fb`.

---

### D-2 — Headline finding: the file-backed seat store has **no production writer at all** — **HIGH**

The packet's expected headline finding (`.seatloom/seats/` and `.seatloom/delegations/` do not
exist) is **confirmed**. The packet asks a sharper question: *is the emptiness itself intended, or
is some code path supposed to be writing those files and silently not doing so?*

**Answer, with evidence: no code path is supposed to be writing them, because nothing outside the
module's own tests ever constructs a `SeatRegistry`.** This is not a wired writer failing silently;
the write path is unreachable in production.

`SeatRegistry` defines three writers:

- `save_identity` — `crates/seatloom-core/src/storage/seat_registry.rs:26`
- `save_role_binding` — `:35`
- `save_delegation` — `:59`

Every call site of all three is inside the module's own test module, which begins at
`seat_registry.rs:176` (`#[cfg(test)] mod tests`, file is 510 lines):

```
$ rg -n 'save_identity|save_role_binding|save_delegation' crates/ src-tauri/ src-cli
crates/seatloom-core/src/storage/seat_registry.rs:26:    pub fn save_identity(...)          <- definition
crates/seatloom-core/src/storage/seat_registry.rs:35:    pub fn save_role_binding(          <- definition
crates/seatloom-core/src/storage/seat_registry.rs:59:    pub fn save_delegation(...)        <- definition
crates/seatloom-core/src/storage/seat_registry.rs:252:            .save_identity(&identity)      <- test (>176)
crates/seatloom-core/src/storage/seat_registry.rs:283:            .save_role_binding("nimbus", ...)  <- test
crates/seatloom-core/src/storage/seat_registry.rs:319:            .save_delegation(&delegation)  <- test
crates/seatloom-core/src/storage/seat_registry.rs:402:            .save_delegation(&invalid)     <- test
crates/seatloom-core/src/storage/seat_registry.rs:446:        registry.save_identity(&mira)      <- test
crates/seatloom-core/src/storage/seat_registry.rs:447:        registry.save_identity(&flux)      <- test
crates/seatloom-core/src/storage/seat_registry.rs:499:        registry.save_delegation(&d1)      <- test
crates/seatloom-core/src/storage/seat_registry.rs:500:        registry.save_delegation(&d2)      <- test

$ rg -n 'SeatRegistry::new|SeatRegistry' crates/ src-tauri/ src-cli \
    | grep -v '^crates/seatloom-core/src/storage/seat_registry.rs'
(no output)
```

The module is exported (`crates/seatloom-core/src/storage/mod.rs:5`, `pub mod seat_registry;`) but
never instantiated outside its own tests. The readers are defensively written for this state:
`list_seat_identities` returns an empty `Vec` when the directory is absent
(`seat_registry.rs:75-79`), as does `list_delegations` (`:104-108`) — so the emptiness is silent by
design and produces no error anywhere.

**Is the emptiness intended?** Partly determinable from the code, partly not, and the distinction
matters:

- **Determinable:** the *absence of files* is fully consistent with the code as written. Under
  AD-011 PostgreSQL is canonical and `.seatloom/` is a cache/export, and an unpopulated cache is a
  legitimate state. Nothing is broken, and no error is being swallowed.
- **Not determinable from the code:** whether `SeatRegistry` is (a) deliberately dormant
  infrastructure awaiting a future export packet, or (b) an abandoned pre-AD-011 storage path that
  should be deleted. Both are consistent with what is on disk. **I cannot determine intent from
  the code** — there is no TODO, no `#[allow(dead_code)]`, no doc comment stating either
  disposition, and no packet reference in the module.

The practical consequence either way: the round-trip test
`seat_identity_round_trips` (`seat_registry.rs:239`) is the *only* thing that ever exercises the
typed YAML path, so that path's correctness is asserted exclusively against synthetic fixtures and
never against a real seat.

**Recommended owner:** Nimbus (owns `crates/seatloom-core/src/storage/`) — decide between wiring
the export path to a real writer or formally declaring the seat store PG-only and retiring the
module. Lyra to arbitrate if that is a product-scope question rather than an architecture one.

---

### D-3 — Schema 008 and seed 001 are mutually inapplicable; `bootstrap.sh` cannot succeed on a fresh volume — **P0, pre-existing**

`008_project_isolation.sql` adds `sessions.project_id` with `DEFAULT 'seatloom'`, backfills, then
**drops the default and sets `NOT NULL`** (`008_project_isolation.sql`, step 3:
`ALTER TABLE sessions ALTER COLUMN project_id DROP DEFAULT;` /
`ALTER COLUMN project_id SET NOT NULL;`).

`seed/001_real_collaboration_baseline.sql` inserts session rows **without** a `project_id` column.
After 008 has run, that insert therefore violates the not-null constraint:

```
$ podman exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom -f /tmp/seed001_base.sql
psql:/tmp/seed001_base.sql:68: ERROR:  null value in column "project_id" of relation "sessions" violates not-null constraint
DETAIL:  Failing row contains (ses-aegis-supervisor-001, seat-aegis-001, ClaudeCode, null,
         /Users/jyxc-dz-0100609/Documents/GitHub/seatloom, main, completed, null, null, null,
         2026-04-27 08:00:00+00, 2026-04-29 23:59:00+00, null).

$ podman exec seatloom-postgres psql -U seatloom -d seatloom \
    -c "SELECT column_name, is_nullable, column_default FROM information_schema.columns WHERE table_name='sessions' AND column_name='project_id';"
 column_name | is_nullable | column_default
-------------+-------------+----------------
 project_id  | NO          |
(1 row)
```

`scripts/bootstrap.sh` applies schema **including 008** (lines 131-139) and *then* applies seed 001
(lines 155-176), under `set -euo pipefail` with `ON_ERROR_STOP=1`. On a fresh volume that ordering
hits exactly the error above, so **the documented one-command bring-up fails on a clean machine.**

**Disagreeing values:** `sessions.project_id` is `NOT NULL` with no default (post-008) vs absent
from every seed 001 session tuple.

Verified working orderings (both exercised during this packet's §5 run):

- schema 001-005 → seed 001 → 008 → 009 ✔ (008's `DEFAULT 'seatloom'` backfills the seeded rows)
- schema 001-005 → 009 → seed 001 ✔ (the path `verify-postgres-baseline.sh` now takes)

**Scope note:** I did **not** execute `bootstrap.sh` end-to-end to confirm the failure in situ,
because its step 4 runs a full `cargo` build plus a ~271-document reconcile. The claim rests on the
reproduced constraint violation above plus the script's line-pinned ordering, not on an observed
`bootstrap.sh` exit code. Flagged as such rather than overstated.

**Recommended owner:** Onyx follow-up packet (add `project_id` to seed 001's session tuples, the
minimal fix) coordinated with Nimbus (owns `bootstrap.sh` ordering). **Not fixed here** — adding
`project_id` to seed 001 is outside this packet's §2.3 scope, which is confined to the runtime
triple.

---

### D-4 — The two schema-application authorities disagree: `verify-postgres-baseline.sh` omits 008 — **MEDIUM, pre-existing**

| Script | Schema files applied |
|---|---|
| `scripts/bootstrap.sh:131-139` | 001, 002, 003, 004, 005, **008**, (009 added by this packet) |
| `scripts/verify-postgres-baseline.sh:76-81` | 001, 002, 003, 004, 005 — **no 008** |

`verify-postgres-baseline.sh:24-26` claims to be "the sole authority for schema application", yet
it never applies 008 — so the database Flux verifies against **lacks `project_id` entirely**, while
the database `bootstrap.sh` builds has it. Two different shapes are both called "the baseline".

This is not merely an oversight: per D-3, adding 008 to the verifier would make its seed step fail.
The omission is what keeps the verifier green, which makes D-4 and D-3 the same defect seen from
two ends.

Consequence worth flagging for Flux: `verify-postgres-baseline.sh` step 7 runs
`cargo test -p seatloom-core -- --include-ignored`, and `db/repositories.rs` reads `project_id` off
session rows (`row_to_session`, `r.get(12)`). Whether those integration tests actually pass against
a 001-005-only database **could not be determined on this seat** (see D-5 — the verifier cannot run
here at all), so I am reporting the inconsistency, not its test-level consequence.

**Recommended owner:** Flux (owns the verifier's contract) with Nimbus — reconcile the two lists
once D-3 is fixed. **Not fixed here:** I added only `009` to the verifier, which is required for
coherence (see §3 below); I deliberately did **not** add `008`, because that change is untestable
on this seat and would alter the verifier's behaviour beyond this packet.

---

### D-5 — `verify-postgres-baseline.sh` hardcodes `docker`; it cannot run on the Onyx seat — **HIGH, pre-existing, blocks packet §7.12**

`verify-postgres-baseline.sh` invokes `docker` unconditionally (`:59-60` `docker compose down -v` /
`docker compose up -d`, and `docker cp` / `docker exec` throughout), unlike `bootstrap.sh`, which
auto-detects the runtime (`bootstrap.sh:44-52`) and even starts the podman VM (`:57-71`).

This seat has podman only, and **no compose provider of any kind**:

```
$ command -v docker
(no output)

$ podman compose version
Error: looking up compose provider failed
6 errors occurred:
	* exec: "docker-compose": executable file not found in $PATH
	* exec: "/Users/jyxc-dz-0100609/.docker/cli-plugins/docker-compose": no such file or directory
	* exec: "/opt/homebrew/bin/docker-compose": no such file or directory

$ ls /Applications | grep -i docker
no Docker.app
```

**Proved pre-existing:** this was run at base commit `9599136`, before any edit by this packet:

```
--- Step 1: Teardown stale volume + start clean container ---
(destructive reset — ensures deterministic schema + seed state)
scripts/verify-postgres-baseline.sh: line 59: docker: command not found
scripts/verify-postgres-baseline.sh: line 60: docker: command not found
```

Exit code `127` (command not found). Step 0 — the pure-Rust seed-consistency gate — passes; the
script dies the moment it needs a container.

**Recommended owner:** Flux / Nimbus — port the runtime detection from `bootstrap.sh:44-71` into
the verifier, and replace `docker compose up -d` / `down -v` with the `run`/`rm`/`volume rm` form
`bootstrap.sh` already uses. **Not fixed here:** rewriting Flux's verification harness is outside
this packet and would change the meaning of a PASS.

---

### D-6 — `infra/README.md` seat count is stale — **LOW**

`infra/README.md` "What just happened" table states `seats | 5 | seed 001 (aegis, lyra, mira,
nimbus, flux)`. The seed now carries **six** seats including `onyx`. Documentation-only drift.

**Recommended owner:** Onyx, folded into the D-3 follow-up packet. Not fixed here (out of §2.3
scope).

---

### D-7 — `.seatloom/seat-sessions.tsv` appeared mid-session — **INFO**

At the start of this packet `ls -a .seatloom/` returned exactly `bootstrap` and `transcripts`
(matching packet §1.1). A later `ls` in the same work session also shows an untracked
`seat-sessions.tsv`. It was created by seat tooling during the session, not by this packet, and is
not one of the three objects in the §2.5 comparison. Recorded only so the differing `ls` outputs in
the delivery doc are not mistaken for an inconsistency.

**Recommended owner:** none required. Noted for accuracy.

---

## 3. What this packet changed outside its stated deliverables, and why

Disclosed explicitly rather than left for a reviewer to find in the diff:

1. **`scripts/verify-postgres-baseline.sh`** — added `009_runtime_triple.sql` to the schema list.
   **Required, not optional:** seed 001 now writes the six triple columns, so applying the seed
   against a database without 009 fails with `column ... does not exist`. Without this the packet
   would have broken the verifier outright. 009 is independent of 008 (it touches only columns
   created in 001), so adding it alone is safe. A comment at the edit site records D-3/D-4/D-5.
2. **`scripts/bootstrap.sh`** — added `009_runtime_triple.sql` after `008`, for the same reason.

Neither script's runtime detection, ordering, or destructive semantics were altered. No `.rs`,
no `ui/`, no `src-tauri/`, and no file under `.seatloom/` was created or modified.

---

## 4. Explicitly NOT remediated

| Finding | Not done | Reason |
|---|---|---|
| D-1 | §1.1 fact table not corrected in the packet | Lyra owns the packet text; a seat does not edit its own dispatched packet. The pre-existing seed change was committed separately (`18a69fb`) purely for diff auditability. |
| D-2 | No YAML written to `.seatloom/seats/` or `.seatloom/delegations/`; `SeatRegistry` untouched | Packet §2.5 and §3 forbid it, and AD-011 makes an empty cache legitimate. Writing files to make the two stores "agree" would fabricate a cache that nothing maintains. `crates/**` is Nimbus's in the paired packet. |
| D-3 | `project_id` not added to seed 001 session rows; `bootstrap.sh` ordering unchanged | Outside §2.3 (runtime triple only). Fixing it here would mean a semantic change to historical seed rows under a packet that forbids exactly that. |
| D-4 | `008` not added to the verifier | Untestable on this seat (D-5) and would change verifier behaviour beyond this packet. |
| D-5 | Verifier not ported to runtime-agnostic container detection | Flux owns the verification harness; changing it changes what a PASS means. |
| D-6 | `infra/README.md` seat count left stale | Documentation-only; belongs with the D-3 follow-up. |

---

## 5. Cross-reference

- Delivery doc: `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md`
- Packet: `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md`
- Paired packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md`

---

*Onyx · 2026-09-28 · drift report for ONYX-2026-09-28-runtime-triple-migration-v1 §2.5 ·
Aegis review §8 item 2*
