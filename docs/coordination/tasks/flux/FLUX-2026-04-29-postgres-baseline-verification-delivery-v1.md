# PostgreSQL Baseline Verification Delivery v1

| Field | Value |
|---|---|
| packet | `FLUX-2026-04-29-postgres-baseline-verification-v1` |
| verdict | **HOLD** |
| date | 2026-04-29 |
| verifier | Flux (read-mostly verification) |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-29-infra-method-review-gate-v1.md` |

---

## 1. Verdict

**HOLD** — Implementation appears structurally correct and all Rust checks pass, but this verification seat lacks Docker, which is required to execute the primary end-to-end proof path (`scripts/verify-postgres-baseline.sh`). No implementation flaw was found. The blocker is environment-only.

---

## 2. Execution environment

| Item | Value |
|---|---|
| OS | macOS (darwin) |
| Rust toolchain | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` via `$HOME/.cargo/bin/cargo` |
| Docker | **not installed** — `docker: command not found` |
| Docker Compose | **not installed** — `docker compose: command not found` |
| psql (PostgreSQL client) | **not installed** — not checked (docker unavailable) |
| Working directory | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |

---

## 3. Command results matrix

### 3.1 Environment capture

| Command | Result |
|---|---|
| `docker --version` | **FAIL** — `command not found` |
| `docker compose version` | **FAIL** — `command not found` |
| `$HOME/.cargo/bin/cargo --version` | PASS — `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |

### 3.2 Rust-only sanity check

| Command | Result |
|---|---|
| `$HOME/.cargo/bin/cargo check` | PASS — Finished `dev` profile in 5.81s, 0 errors |
| `$HOME/.cargo/bin/cargo test -p seatloom-core` | PASS — 26 unit tests passed, 0 failed, 9 integration tests ignored (as designed) |

### 3.3 Primary end-to-end script

| Command | Result |
|---|---|
| `bash scripts/verify-postgres-baseline.sh` | **BLOCKED** — EXIT 127 at Step 1: `docker: command not found` |

Full stderr:

```
=== SeatLoom PostgreSQL Baseline Verification ===
repo root: /Users/jyxc-dz-0100609/Documents/GitHub/seatloom
database:  postgresql://seatloom:seatloom@localhost:5432/seatloom

--- Step 1: Start PostgreSQL via docker compose ---
/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/scripts/verify-postgres-baseline.sh: line 32: docker: command not found
EXIT_CODE=127
```

### 3.4 Fallback isolation commands

**Not executed.** The fallback path (`docker compose up -d`, `docker exec seatloom-postgres psql …`, `cargo test --include-ignored`) requires Docker which is unavailable. All file-level evidence below confirms the implementation is structurally complete and correct.

---

## 4. Coverage spot-check results (file-backed, deterministic)

### 4.1 Schema completeness

`infra/postgres/schema/001_seatloom_core.sql` defines all 9 required tables:

| # | Table | Required | Present |
|---|---|---|---|
| 1 | `projects` | ✓ | ✓ |
| 2 | `seats` | ✓ | ✓ |
| 3 | `project_role_bindings` | ✓ | ✓ |
| 4 | `seat_delegations` | ✓ | ✓ |
| 5 | `sessions` | ✓ | ✓ |
| 6 | `workitems` | ✓ | ✓ |
| 7 | `handoffs` | ✓ | ✓ |
| 8 | `artifacts` | ✓ | ✓ |
| 9 | `canonical_events` | ✓ | ✓ |
| 10 | `event_object_refs` | (M2M) | ✓ |

Plus 8 indexes for common query patterns.

### 4.2 Docker Compose service

`infra/postgres/docker-compose.yml` is well-formed:
- `postgres:16-alpine` image
- Container name `seatloom-postgres`
- Credentials: `seatloom / seatloom`
- Port `5432:5432`
- Named volume `seatloom-pgdata`
- Schema auto-mount via `./schema:/docker-entrypoint-initdb.d`
- Healthcheck using `pg_isready`
- No cloud or multi-node widening

### 4.3 Seeded object coverage (from seed SQL + source map)

| Family | Claimed | Evidence |
|---|---|---|
| Seats | 5 | `aegis`, `lyra`, `mira`, `nimbus`, `flux` — present in seed SQL lines 19-25 |
| Delegations | 1 | `del-flux-acting-mira-001` — Flux acting for Mira on WI-009, seed SQL lines 44-55 |
| Role Bindings | 5 | One per seat for `seatloom` project, seed SQL lines 31-38 |
| Sessions | 5 | One representative per seat (Apr 27-29) |
| WorkItems | 6 | wi-001, wi-004, wi-009, wi-scaffold, wi-storage, wi-hardening |
| Handoffs | 3 | aegis→nimbus, nimbus→lyra, lyra→flux |
| Artifacts | 14 | T1–T7 family coverage + active contract set (see below) |
| Canonical Events | 70+ | SessionStarted, WorkItemCreated/StatusChanged, Delegation Issued/Closed, ReviewVerdictIssued, HandoffSent/Accepted/Completed, ArtifactCreated |

### 4.4 Artifact family coverage (T1–T7 + active contract set)

| Family | Artifact ID | File | Subtype |
|---|---|---|---|
| T1 AuthorityDoc | ar-product-truth | `docs/PRODUCT_TRUTH.md` | prd |
| T1 AuthorityDoc | ar-prd-v05 | `docs/prd-v0.5.md` | prd |
| T1 AuthorityDoc | ar-interaction-v11 | `docs/interaction-spec-v1.1.md` | interaction_spec |
| T1 AuthorityDoc | ar-ux-v11 | `docs/ux-spec-v1.1.md` | ux_spec |
| T1 AuthorityDoc | ar-acceptance-v11 | `docs/acceptance-spec-v1.1.md` | acceptance_spec |
| T1 AuthorityDoc | ar-arch-decisions | `docs/architecture-decisions.md` | architecture_decisions |
| T1 AuthorityDoc | ar-arch-design | `docs/architecture-design.md` | architecture_design |
| T2 RoleProfile | ar-role-mira | `docs/coordination/roles/MIRA.md` | seat_role |
| T3 TaskPacket | ar-task-hardening | `…/NIMBUS-2026-04-29-foundation-hardening-v1.md` | task |
| T3 TaskPacket | ar-task-storage | `…/NIMBUS-2026-04-29-storage-ledger-foundation-v1.md` | task |
| T4 Review | ar-review-process-mapping | `…/2026-04-28-process-mapping-review.md` | process_mapping |
| T5 Acceptance | ar-acceptance-hardening | `…/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md` | acceptance_review |
| T6 DailyMemory | ar-memory-2026-04-29 | `docs/coordination/memory/2026-04-29.md` | daily_log |
| T7 GovernanceDoc | ar-governance-coordination-rules | `docs/coordination/COORDINATION_RULES.md` | coordination_rules |

All T1–T7 families represented. Active contract set present. All artifact records map to real repo files.

### 4.5 Integration tests (code review)

`crates/seatloom-core/tests/db_baseline_integration.rs` contains 9 `#[ignore]` tests covering exactly the spot-check criteria:

| Test | What it checks |
|---|---|
| `db_ping_succeeds` | DB connection + ping |
| `seats_non_empty_and_contain_all_five` | 5 seats including aegis/lyra/mira/nimbus/flux |
| `flux_mira_delegation_persisted` | `del-flux-acting-mira-001` exists with correct from/to/status/workitem |
| `all_six_workitems_seeded_as_done` | ≥6 workitems, 4 specific ones done |
| `artifacts_cover_t1_through_t7` | All 7 template families present |
| `active_contract_set_present` | All 7 active contract artifacts seeded |
| `timeline_events_descending_order` | Events sorted by occurred_at DESC |
| `handoffs_reference_valid_workitems` | ≥3 handoffs, each references an existing WI |
| `role_bindings_cover_all_seats` | ≥5 role bindings for `seatloom` project |

All tests are correctly structured. When Postgres is available, `cargo test -p seatloom-core -- --include-ignored` will run them against the live DB.

### 4.6 Provenance

`.seatloom/bootstrap/source-map.yaml` documents:
- Evidence sources: `MEMORY.md`, `process-mapping-review.md`, `memory/2026-04-29.md`
- All approximations explicitly marked (session timestamps, handoff timing, WI timestamps, seat `created_at`)
- Intentionally partial families listed with reasons (checkpoints, pipeline_runs, system-generated artifacts, comments)

---

## 5. Findings / blockers

### 5.1 Blocker (environment, not implementation)

| Finding | Severity | Classification |
|---|---|---|
| Docker is not installed on this verification seat. The primary script `scripts/verify-postgres-baseline.sh` cannot execute Step 1 (`docker compose up -d --wait`). | **BLOCKER** | Environment prerequisite missing — not an implementation flaw |

Per packet rules (§ Pass/Hold/Re-Scoped):
> Use **HOLD** if the implementation looks correct but the verification seat lacks the required docker capability or another environment-only prerequisite. In that case, separate environment blockers from implementation findings.

### 5.2 Implementation observations (non-blocking)

| Finding | Severity | Note |
|---|---|---|
| `ar-product-truth` uses subtype `prd` — the closest T1 match per source-map note | P3 (informational) | Documented in source-map.yaml line 99; acceptable approximation |
| Seed file is a single monolithic `INSERT` block (~320 lines) | P3 (informational) | Works correctly; could be split for maintainability but not required |
| `artifacts.storage_path` stores file paths on disk — dual-layer design confirmed | — | This is the intended architecture (metadata in Postgres, payloads on disk) |
| No `pgvector` extension or embedding columns | — | Deferred to P1 per delivery notes; correct for v1 scope |

**No hidden operational flaw found in docker-compose.yml, schema, seed strategy, or test code.**

---

## 6. Verification questions — explicit answers

1. **Does the repo-managed PostgreSQL service actually start on a verification-capable seat?**
   Cannot be proven on this seat. Docker is not installed. The `docker-compose.yml` is structurally correct (Postgres 16-alpine, healthcheck, volume mount, schema auto-init). Requires a docker-capable seat to execute.

2. **Does the baseline script run end to end without manual repair?**
   Cannot be proven on this seat. `scripts/verify-postgres-baseline.sh` fails at Step 1 with `docker: command not found` (EXIT 127). The script itself is well-formed (`set -euo pipefail`, correct step ordering, correct `DATABASE_URL`). Requires Docker to prove.

3. **Do ignored DB integration tests pass when explicitly included?**
   Cannot be proven on this seat (requires live Postgres). Code review confirms all 9 tests are correctly structured with `#[ignore]` and will execute against `SeatloomDb` when Postgres is reachable at `postgresql://seatloom:seatloom@localhost:5432/seatloom`.

4. **Does seeded data match Nimbus's claimed object-family coverage closely enough to accept the packet?**
   **Yes — by file-backed deterministic evidence.** The seed SQL (`001_real_collaboration_baseline.sql`) contains:
   - 5 seats (aegis, lyra, mira, nimbus, flux)
   - 1 delegation (Flux acting for Mira, WI-009)
   - 5 role bindings
   - 6 workitems
   - 3 handoffs
   - 14 artifacts covering T1–T7 + active contract set
   - 70+ canonical events
   All map to real repo evidence documented in `.seatloom/bootstrap/source-map.yaml`. No fabricated content found.

5. **Is the packet still infrastructure-only, with no business/UI widening?**
   **Yes.** Confirmed by reviewing Nimbus's delivery and the implementation files. Changes are limited to:
   - `infra/postgres/` (docker-compose, schema, seed)
   - `crates/seatloom-core/src/db/` (connection, models, repositories)
   - `crates/seatloom-core/Cargo.toml` (postgres dependencies)
   - `crates/seatloom-core/tests/db_baseline_integration.rs`
   - `scripts/verify-postgres-baseline.sh`
   - `.seatloom/bootstrap/source-map.yaml`
   No UI, route-engine, prompt-engine, or business workflow code was touched.

6. **Is there any hidden operational flaw in the docker-compose or seed strategy that should block acceptance?**
   **No.** Docker-compose uses standard Postgres 16-alpine, mounts schema for auto-init, uses a named volume for data persistence, and has a healthcheck. Seed uses `BEGIN; … ON CONFLICT DO NOTHING` for idempotency. Tests are properly `#[ignore]`-guarded. No flaws found.

---

## 7. Evidence file paths

```
.local/evidence/2026-04-29-postgres-baseline-verification/
├── docker-version.txt
├── docker-compose-version.txt
├── cargo-version.txt
├── cargo-check.txt
├── cargo-test-unit.txt
├── verify-postgres-baseline.txt
├── blocker-note.txt
```

(No fallback files — docker unavailable, so `docker-ps.txt`, `sql-*.txt`, `cargo-test-include-ignored.txt` were not generated.)

---

## 8. Recommended next action

**Lyra:** Route this packet to a docker-capable verification seat to complete the primary proof path.

Once Docker is available, run:

```bash
bash scripts/verify-postgres-baseline.sh
```

If that succeeds, the verdict upgrades to **PASS** and the baseline can be signed. If the script fails, use the fallback isolation commands in the packet (§4) to isolate the exact failing step.

**Nimbus:** No implementation changes requested. The infrastructure delivered is correct.
