# Delivery: PostgreSQL Baseline Init-Conflict Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-v1` |
| tags | nimbus, infrastructure, postgres, docker, verifier, repeat-run, init-conflict |
| owner | Nimbus |

## 1. Flux Blocker Restated

Flux verified commit `24c820b81de8f8a479e9d229f5c106ed440eb675` on the remote Docker-capable workspace and returned HOLD:

- `cargo check -p seatloom-core` → PASS
- `cargo test -p seatloom-core` → PASS
- `bash scripts/verify-postgres-baseline.sh` first run → FAIL at schema apply

Root cause narrowed by Flux:
- `infra/postgres/docker-compose.yml` mounted `./schema` into `/docker-entrypoint-initdb.d`.
- On a fresh volume (`docker compose down -v && docker compose up -d`), PostgreSQL init auto-applied the schema files during first container boot.
- `scripts/verify-postgres-baseline.sh` Step 2 then attempted to apply the same schema files again via `docker exec psql -f "/docker-entrypoint-initdb.d/$schema_file"`.
- The double-apply path was structurally broken: even with `CREATE TABLE IF NOT EXISTS`, the script relied on files inside `/docker-entrypoint-initdb.d/` which only exist via the volume mount that was producing the hidden auto-apply.
- This made the repeat-run hardening claim invalid.

## 2. Resolution Chosen

**Preferred resolution applied: make the verifier script the sole schema-apply authority.**

Changes:

1. **`infra/postgres/docker-compose.yml`**: Removed the `./schema:/docker-entrypoint-initdb.d` volume mount. PostgreSQL now starts with an empty data directory. No schema is applied during container init. The container is purely a clean Postgres instance.

2. **`scripts/verify-postgres-baseline.sh`** Step 2: Changed schema application from referencing `/docker-entrypoint-initdb.d/$schema_file` (which relied on the volume mount) to `docker cp "$INFRA_DIR/schema/$schema_file" "seatloom-postgres:/tmp/$schema_file"` followed by `docker exec psql -f "/tmp/$schema_file"`. This is the same pattern already used for seed files. The script now owns schema application end-to-end.

The proof sequence is unchanged: schema → seed → reconcile → ingest helper → DB tests. Schema application is now explicit, visible, and not duplicated anywhere.

## 3. Files Changed

- `infra/postgres/docker-compose.yml` — removed schema volume mount
- `scripts/verify-postgres-baseline.sh` — Step 2 now uses docker cp + /tmp/ for schema files; updated header comment
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md` — this file

No schema SQL, seed SQL, Rust code, or UI code was modified.

## 4. Validation Commands and Results

```
$HOME/.cargo/bin/cargo check -p seatloom-core
=> Finished `dev` profile (PASS)

$HOME/.cargo/bin/cargo test -p seatloom-core
=> 54 unit tests + 3 seed-consistency tests: all pass (PASS)

bash -n scripts/verify-postgres-baseline.sh
=> SYNTAX OK (PASS)

bash scripts/verify-postgres-baseline.sh
=> Docker unavailable on this seat — see section 5.
```

## 5. Docker Availability Note

Docker is not installed on this seat (darwin, no Docker daemon). The end-to-end `bash scripts/verify-postgres-baseline.sh` (and consecutive-run proof) cannot be executed locally.

The fix is structurally correct by inspection:
- The hidden auto-init path is gone (no schema volume mount).
- Step 2 now copies schema files explicitly with `docker cp`, identical to the seed pattern that already works.
- The `pg_isready` readiness loop from the previous hardening commit remains in place.

Remaining proof must be done by Flux on the remote Docker-capable seat against the exact commit hash below.

## 6. Exact Branch and Commit

- Branch: `track/infra-foundation`
- Commit: see git log after push (Flux should verify by exact hash)

## 7. Status of the Broader Prompt/Channel Packet

The prompt/channel-authority packet (`NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-v1`) was interrupted at the schema-design and model-stub stage. No committed changes were made from that packet before this interrupt. That packet resumes after Flux accepts this fix.

In-progress state at interrupt:
- Schema 005 SQL file drafted (not committed).
- Seed 004 SQL file drafted (not committed).
- Rust model structs drafted in `models.rs` (not committed).
- source-map.yaml updated (not committed).

All of that work remains intact in the working tree and will be completed in the next bounded packet.
