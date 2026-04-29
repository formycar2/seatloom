# Acceptance: Nimbus Real Collaboration PostgreSQL Baseline

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-29-nimbus-real-collaboration-postgres-baseline-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| target | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md` |
| verdict | HOLD |
| tags | acceptance, nimbus, postgres, persistence, infrastructure, seed-data, verification |

## Verdict

**HOLD**

Nimbus completed the bounded infrastructure implementation packet with strong structural evidence:

- PostgreSQL-backed schema, seed data, repositories, provenance, and verification script all exist;
- Rust compile / test / fmt / clippy gates pass;
- Flux confirmed the implementation is structurally correct and remained infrastructure-only.

However, Lyra is not closing this packet as `PASS` yet because the required end-to-end proof path is still environment-blocked:

- `bash scripts/verify-postgres-baseline.sh` cannot run on the available verification seats because Docker is unavailable.

This is an **environment-only hold**, not an implementation rejection.

## Scope Reviewed

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`
- `docs/coordination/tasks/lyra/LYRA-2026-04-29-infra-method-review-gate-v1.md`
- `infra/postgres/docker-compose.yml`
- `infra/postgres/schema/001_seatloom_core.sql`
- `infra/postgres/seed/001_real_collaboration_baseline.sql`
- `scripts/verify-postgres-baseline.sh`
- `.seatloom/bootstrap/source-map.yaml`
- `crates/seatloom-core/src/db/mod.rs`
- `crates/seatloom-core/src/db/connection.rs`
- `crates/seatloom-core/src/db/models.rs`
- `crates/seatloom-core/src/db/repositories.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `$HOME/.cargo/bin/cargo check`
- `$HOME/.cargo/bin/cargo test -p seatloom-core`
- `$HOME/.cargo/bin/cargo fmt --all --check`
- `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings`
- `bash scripts/verify-postgres-baseline.sh`

## Coverage Matrix

| Requirement slice | Evidence paths | Result | Notes |
|---|---|---|---|
| Structured collaboration truth is moved into a real database, not file-only authority | Nimbus delivery §1-§4; `infra/postgres/schema/001_seatloom_core.sql`; `crates/seatloom-core/src/db/repositories.rs` | PASS | PostgreSQL is the authoritative structured truth layer. |
| Real collaboration data is seeded, not fake demo content | `infra/postgres/seed/001_real_collaboration_baseline.sql`; `.seatloom/bootstrap/source-map.yaml`; Nimbus delivery §4-§6 | PASS | Seed coverage includes real seats, workitems, handoffs, delegation, artifacts, and 70+ events. |
| Artifact metadata uses `template + subtype` and keeps payloads on disk | schema `artifacts` table; source map; Nimbus delivery §5, §7 | PASS | Dual-layer design is explicit and bounded. |
| Rust-side DB repositories exist for later consumption | `crates/seatloom-core/src/db/repositories.rs`; Nimbus delivery §7 | PASS | List/get behavior exists for the required families. |
| Packet remains infrastructure-only | Nimbus delivery §1, §9; Flux verification §5-§6 | PASS | No UI, route-engine, or prompt/runtime widening. |
| Rust compile/test hygiene passes | Nimbus delivery §8; Flux verification §3.2 | PASS | `cargo check`, `cargo test -p seatloom-core`, `fmt`, `clippy` all pass. |
| Repo-managed PostgreSQL bootstrap path is proven end to end | `scripts/verify-postgres-baseline.sh`; Flux verification §3.3 | HOLD | Script exists and looks correct, but cannot be executed on current seats because `docker` is unavailable. |
| Ignored DB integration tests are proven on a live PostgreSQL instance | `crates/seatloom-core/tests/db_baseline_integration.rs`; Flux verification §4.5-§6 | HOLD | Tests are correctly structured but still unproven in a live DB environment. |
| Seeded coverage materially matches Nimbus's delivery claim | seed SQL; source map; Flux verification §4.3-§4.6 | PASS | Flux confirmed the claimed object-family coverage by deterministic file-backed evidence. |

## Findings

### Blocking finding

1. **Environment-only verification blocker**
   - `docker --version` and `docker compose version` both fail with `command not found` on the available verification seats.
   - `bash scripts/verify-postgres-baseline.sh` fails at Step 1 with exit `127` for the same reason.
   - This blocks final `PASS`, but it does **not** indicate a defect in Nimbus's implementation.

### Non-blocking notes

1. The chosen local orchestration method for this packet is `docker compose`.
2. If the team will not provide any docker-capable seat, a separate sponsor-reviewed infra packet will be needed to add a second PostgreSQL bootstrap/verification path.
3. That follow-up, if opened, must keep PostgreSQL as the storage authority and remain infrastructure-only.

## Required Fixes for Nimbus

None in this packet yet.

Nimbus does **not** receive a rework order from this acceptance review because no concrete implementation flaw has been proven.

## Go / No-Go Recommendation

- **Implementation quality:** **GO**
- **Close packet as final PASS now:** **NO-GO**
- **Keep packet in HOLD pending end-to-end PostgreSQL verification:** **GO**
- **Open business/backend expansion from this packet:** **NO-GO**

## Follow-up Action

1. Preferred path: run `bash scripts/verify-postgres-baseline.sh` on a docker-capable seat and then re-evaluate for `PASS`.
2. If no docker-capable seat will be available, do **not** silently switch methods. Open one separate sponsor-reviewed infra packet for an alternate local PostgreSQL bootstrap/verification path.

## Evidence Paths

- Nimbus packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
- Nimbus delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`
- Flux verification packet: `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-v1.md`
- Flux verification delivery: `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`
- Method review gate: `docs/coordination/tasks/lyra/LYRA-2026-04-29-infra-method-review-gate-v1.md`
- Compose file: `infra/postgres/docker-compose.yml`
- Schema: `infra/postgres/schema/001_seatloom_core.sql`
- Seed: `infra/postgres/seed/001_real_collaboration_baseline.sql`
- Verification script: `scripts/verify-postgres-baseline.sh`
- Provenance: `.seatloom/bootstrap/source-map.yaml`
