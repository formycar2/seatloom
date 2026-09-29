# Acceptance: Nimbus PostgreSQL Baseline Verifier Hardening

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-30-nimbus-postgres-baseline-verifier-hardening-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| target | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, nimbus, flux, postgres, verifier, docker, repeat-run, infrastructure |

## Verdict

**PASS**

Lyra accepts the PostgreSQL baseline verifier hardening as scope-complete and verified.

The repeat-run verifier gate is now closed on the exact remotely verified commit:
- branch: `track/infra-foundation`
- final verified commit: `b9bb7340416e3729f30dc751a4bb1f41ee726520`
- verification seat: sponsor-provided remote workspace (`buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com`)
- proof path: `bash scripts/verify-postgres-baseline.sh` run twice consecutively with no manual cleanup between runs

The two intermediate HOLDs were real and useful:
1. duplicate schema authority between Docker init and explicit script apply,
2. readiness probe succeeding before the `seatloom` database was actually queryable.

Nimbus fixed both without widening scope into schema semantics, product logic, or UI work. Flux then re-ran the exact commit-pinned remote proof and returned PASS.

## Scope Reviewed

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-delivery-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-delivery-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-delivery-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-database-readiness-gate-reverification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-database-readiness-gate-reverification-delivery-v1.md`
- `docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md`
- `docs/infra/ssh-tunnel-workspace.md`
- `scripts/verify-postgres-baseline.sh`
- `infra/postgres/docker-compose.yml`

## Coverage Matrix

| Requirement slice | Evidence paths | Result | Notes |
|---|---|---|---|
| Verifier self-resets the Docker volume and is safe to rerun on the same seat | `scripts/verify-postgres-baseline.sh`, Flux final delivery | PASS | Flux proved two consecutive successful runs without manual cleanup. |
| Schema application has one explicit authority path | Nimbus init-conflict fix delivery, `infra/postgres/docker-compose.yml`, `scripts/verify-postgres-baseline.sh` | PASS | Hidden `/docker-entrypoint-initdb.d` schema auto-apply path was removed. |
| Readiness means the `seatloom` database is truly queryable, not merely socket-open | Nimbus database-readiness delivery, current verifier script, current healthcheck | PASS | Both layers now use a `psql ... SELECT 1` probe against `seatloom`. |
| Commit-pinned remote verification reproduces the exact verifier behavior | Flux final delivery, SSH workspace note | PASS | Exact commit `b9bb734...` was fetched, checked out cleanly, and verified remotely. |
| Repeat-run proof remains infra-only | Nimbus deliveries, Flux deliveries | PASS | No schema semantics change, no seed widening, no UI/business logic touched. |
| Seed/reconcile/results remain materially healthy after verifier hardening | Flux SQL spot-checks | PASS | `reconcile_runs`, `document_versions`, `checkpoints`, `handoff_receipts`, `pipeline_runs`, `review_threads`, and `review_comments` are all non-zero. |

## Findings

### Resolved blockers

1. **Duplicate schema-apply path is closed**
   - Prior issue: PostgreSQL container init auto-applied schema while the verifier also reapplied schema in Step 2.
   - Fix: schema mount removed from `docker-compose.yml`; verifier now copies schema files explicitly and applies them itself.
   - Result: schema authority is explicit and singular.

2. **False-positive readiness gate is closed**
   - Prior issue: bare `pg_isready` succeeded before the `seatloom` database existed.
   - Fix: both the verifier and Docker healthcheck now wait for `psql -U seatloom -d seatloom -c "SELECT 1" -q` to succeed.
   - Result: the verifier no longer advances to schema apply before the target database is real and queryable.

### Non-blocking notes

1. The sponsor remote workspace is now the accepted Docker-capable proof seat for this verifier path.
2. The verifier contract is now strong enough to serve as the stable gate for future schema packets, including `schema/005` and later infrastructure-only expansions.
3. This acceptance closes verifier hardening only. It does not by itself close the broader runtime-authority gap identified in `docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md`.

## Required Fixes for Nimbus

None for this packet.

## Go / No-Go Recommendation

- **Close PostgreSQL baseline verifier hardening as final PASS now:** **GO**
- **Resume the paused prompt + channel-action authority packet:** **GO**
- **Widen into UI/business/runtime automation from this packet:** **NO-GO**
- **Treat file-only truth as the primary persistence authority again:** **NO-GO**

## Evidence Paths

- Hardening delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`
- Init-conflict fix delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md`
- Database-readiness fix delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-delivery-v1.md`
- Final Flux PASS delivery: `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-database-readiness-gate-reverification-delivery-v1.md`
- Verifier script: `scripts/verify-postgres-baseline.sh`
- Docker Compose config: `infra/postgres/docker-compose.yml`
- SSH workspace operator note: `docs/infra/ssh-tunnel-workspace.md`
