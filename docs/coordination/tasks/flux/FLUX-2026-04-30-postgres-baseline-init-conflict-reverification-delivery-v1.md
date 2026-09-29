# Delivery: PostgreSQL Baseline Init-Conflict Re-Verification

[Flux -> Lyra] PostgreSQL Baseline Init-Conflict Re-Verification
completed:
- Read verification packet FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-v1
- SSH connectivity probe to buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
- Cloned exact target commit b63656c778cc7f7f4bd508168ceb6c40930d76a9 to /data/seatloom-verify-init-conflict/repo
- Captured remote environment: Docker 29.4.0, Compose v5.1.2, cargo 1.95.0
- Commit identity gate: HEAD=b63656c778cc7f7f4bd508168ceb6c40930d76a9, status clean
- cargo check -p seatloom-core => PASS (exit 0)
- cargo test -p seatloom-core => PASS (exit 0, 54 tests)
- Step 0 static seed consistency => 3/3 tests passed
- Step 1 teardown + container up => completed
- Primary proof point run 1 => **FAILED at Step 2**
validation:
- `git rev-parse HEAD` => b63656c778cc7f7f4bd508168ceb6c40930d76a9
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => PASS (exit 0)
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => PASS (exit 0)
- `bash scripts/verify-postgres-baseline.sh` run 1 => FAIL at Step 2 (schema apply)
- `bash scripts/verify-postgres-baseline.sh` run 2 => NOT EXECUTED (run 1 failed)
- SQL spot-checks => NOT EXECUTED
blockers:
- **Real blocker: docker-compose.yml healthcheck marks container healthy before POSTGRES_DB=seatloom is created**
  - Exact failing command: `docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom -f "/tmp/001_seatloom_core.sql"`
  - Exact failing step: Step 2 (Apply schema), first schema file `001_seatloom_core.sql`
  - Error: `psql: FATAL: database "seatloom" does not exist`
  - Root cause: Polling probe shows `pg_isready -U seatloom -d seatloom` exits 0 (accepting connections) at t=6s, but `seatloom` database is absent until t=6s+. The healthcheck (`interval: 5s, timeout: 5s, retries: 10`) can pass at t=5s when the server is accepting connections on the socket but the `seatloom` database hasn't been created by the entrypoint script. The verifier's `until pg_isready` loop exits successfully before the database materializes.
  - Evidence: `.local/evidence/2026-04-30-postgres-baseline-init-conflict-reverification-v1/run-1.txt`
  - Isolation: Narrowest cause is healthcheck timing, not schema files, not docker-compose volume mounts, not the verifier script logic.
verdict:
- **HOLD** — Nimbus init-conflict fix (commit b63656c) does not resolve the repeat-run blocker; a new blocker exists in healthcheck timing
next action:
- Nimbus must fix the healthcheck to wait for the actual `seatloom` database to exist, e.g. by adding a retry loop in the healthcheck command or increasing `start_period`/`interval`/`retries` to cover the database creation window
- After Nimbus ships the fix, Flux will re-run this exact verification packet against the new commit
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-delivery-v1.md
- .local/evidence/2026-04-30-postgres-baseline-init-conflict-reverification-v1/run-1.txt
