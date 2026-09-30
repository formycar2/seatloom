# Task: Fix `apply-db.sh` ordering so the runtime-triple merge candidate builds from scratch

[Lyra -> Onyx] Flux's commit-pinned gate returned **FAIL** on your runtime-triple delivery. **The
migration content is sound and the failure is not your fault** — it is a cross-packet collision
created by a script that did not exist when you worked. This packet fixes the collision so the
merge can land. Read Flux's delivery first; it is unusually precise and already contains the
verified remedy.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | ONYX-2026-09-30-apply-db-ordering-fix-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | onyx |
| priority | P0 (blocks the runtime-triple merge, which blocks P0 step 1) |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v2.md` (FAIL) |
| base branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` @ `580bcf4` |
| merge in | `track/infra-foundation` @ `281beaa` (or later) |
| delivery branch | same packet branch — extend it, do not open a new one |
| delivery path | `docs/coordination/tasks/onyx/ONYX-2026-09-30-apply-db-ordering-fix-delivery-v1.md` |
| tags | onyx, apply-db, migration-ordering, cross-packet-collision, p0 |

---

## 0. Operating constraints

- **Work in the main checkout** at `~/Documents/GitHub/seatloom`. The worktree at
  `seatloom-worktrees/flux` belongs to Flux; do not touch it.
- Serial execution: you own the tree for this packet. If `git status --short` is non-empty when you
  start, stop and report.
- Return the tree to `track/infra-foundation` when you finish.
- This packet file lives on `track/infra-foundation`. Once you check out the packet branch it will
  vanish from the tree — re-read it with
  `git show track/infra-foundation:docs/coordination/tasks/onyx/ONYX-2026-09-30-apply-db-ordering-fix-v1.md`.

## 1. What Flux found

`scripts/apply-db.sh` cannot build your merge candidate from scratch — exit 3:

```
--- seed (001–004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql
ERROR:  column "default_harness" of relation "seats" does not exist
```

Two canonical orders in the repo that cannot both be satisfied:

- **Seed 001 at `580bcf4` requires `009` BEFORE the seed** — it writes
  `seats.default_harness/_agent/_model` and `sessions.harness/agent/model`. You stated this
  yourself in `verify-postgres-baseline.sh`: *"009 is required here, not optional."*
- **`apply-db.sh` applies `009` AFTER the seed** — it buckets all migrations `≥006` post-seed,
  deliberately, because *"008 sets sessions.project_id NOT NULL but seed 001 has no project_id."*

Control experiment (Flux): `track/infra-foundation` builds green (exit 0); the merge candidate
exits 3. **Merging as-is introduces the regression.**

**Why this is not your defect, for the record:** `apply-db.sh` arrived with `8feea1b`
("ci: GitLab pipeline + ratchet verifiers"). I confirmed independently that `8feea1b` is **not** an
ancestor of your base `9599136` — `git merge-base --is-ancestor 8feea1b 9599136` returns false. The
script did not exist when you worked. You correctly registered `009` in the two scripts that did
exist (`bootstrap.sh`, `verify-postgres-baseline.sh`), both of which apply schema before seed,
where your ordering is right.

## 2. Scope

### 2.1 Bring the branch up to date

Merge `track/infra-foundation` (≥ `281beaa`) into your packet branch. This brings in `apply-db.sh`
and Flux's D-5 container-detection fix (`865f820`). Extend the existing packet branch — do not open
a new one — so the merge candidate is self-consistent and Flux can verify one SHA.

### 2.2 Fix the ordering

Flux verified this remedy from an empty database, exit 0, every file clean:

```
schema 001–005  +  schema 009      (pre-seed)
seed   001–004                     (seed)
schema 008                         (post-seed)
```

Your own comment establishes `009` is safe pre-seed: *"009 is independent of 008 — it only touches
columns created in 001."*

**Do not hardcode a bare `009` into the pre-seed list and move on.** The real finding is that
"`001–005` pre-seed, `≥006` post-seed" is a **bucket heuristic that does not hold**, and
`ENGINEERING_STANDARDS.md` already records it as *"convention, not enforced structure"* with a
proper migration runner tracked as an Onyx follow-up. Encode the actual rule and make it legible:

- a migration belongs **pre-seed** if the seed writes columns it creates (→ `009`);
- a migration belongs **post-seed** if it constrains columns the seed does not provide (→ `008`,
  which sets `sessions.project_id NOT NULL`).

Express that per-migration with a comment giving the reason for each placement, so the next
migration is classified by rule rather than by number. A short explicit ordered list with a
one-line justification per entry is fine and is preferable to a clever derivation.

### 2.3 Prove it from scratch

Rebuild from an **empty scratch database**, exactly as Flux did. Do not write to the live
`seatloom` database — Mr. Zhang's `pnpm tauri dev` app (pid 91929) is attached to it and SeatLoom
hard-panics without Postgres.

```
CREATE DATABASE seatloom_verify_onyx;   -- drop first if present
DATABASE_URL=<…/seatloom_verify_onyx> bash scripts/apply-db.sh     # must exit 0
```

Then in that scratch database re-run the §2.2 verification queries from your original packet:
the three zero-counts, the `DISTINCT` domain checks, and the row-state `SELECT`s — the six seat
rows carrying `Stepcode`/`ClaudeCode`/`claude-opus-5[1m]`, and **`NULL` harness on all five April
session rows**. Drop the scratch DB afterwards and say so.

**Note on tooling, from Flux:** there is no `psql` client on this host — not on PATH, no brew
`libpq`/`postgresql` — yet `apply-db.sh` requires one. Flux bridged it with a temporary shim
forwarding to `podman exec -i seatloom-postgres psql`. You will hit the same wall. Use the same
approach, disclose it, and clean it up. **State plainly in the delivery that `apply-db.sh`'s own
host-`psql` invocation path remains unexercised on this seat** — do not let a shimmed run read as
proof of the real path. Installing a host `psql` is a separate infrastructure question I am raising
with Mr. Zhang; do not install it yourself.

## 3. Out of scope

- **Do not change any `.sql` file.** The migration content passed. This packet is the apply-order
  contract only. If you believe the SQL must change, stop and tell me — that would reopen the gate.
- **Do not fix the podman-compose gap.** Flux found that `verify-postgres-baseline.sh` still cannot
  run because podman has no compose provider on this machine; the remedy is the `run`/`rm`/
  `volume rm` form `bootstrap.sh` already uses. That is a separate scoped packet, and it lives in
  the destructive path. Not yours today.
- **Do not run the full `verify-postgres-baseline.sh`.** Line 59 `compose down -v` destroys the
  volume the live app is attached to.
- **Do not edit `.rs` files** or anything under `ui/` or `src-tauri/`.
- **Do not build the proper migration runner.** This incident is evidence for it; it stays a
  tracked follow-up, not today's scope.

## 4. Done-definition

1. `track/infra-foundation` (≥ `281beaa`) merged into the packet branch; merge candidate is one
   verifiable SHA.
2. `apply-db.sh` orders `001–005` + `009` pre-seed, seed `001–004`, `008` post-seed.
3. Placement is justified **per migration by rule**, in a comment — not a bare number appended to a
   list.
4. `bash scripts/apply-db.sh` against an empty scratch database exits **0**, transcript verbatim.
5. All §2.3 verification queries pass in that scratch build, output verbatim; **`NULL` harness on
   all five April session rows**.
6. Scratch database dropped; shim (if used) removed; tree clean and returned to
   `track/infra-foundation`.
7. Delivery doc states the unexercised host-`psql` path limitation explicitly.
8. Zero `.sql`, `.rs`, `ui/`, `src-tauri/` files changed — `git show --stat` proves it.
9. Flux re-verify at the new merge-candidate SHA returns PASS.

## 5. Evidence expectations

Base/delivery branch and commit SHAs named; verbatim transcripts, not summaries; the failing
"before" and passing "after" both shown, so the fix is demonstrated rather than asserted; explicit
statement of anything not done and why.

---

*Dispatched by Lyra · 2026-09-30 · Flux gate FAIL at `580bcf4` · cross-packet collision from
`8feea1b`, not an Onyx defect · fixes the apply-order contract so the merge candidate builds from
scratch*
