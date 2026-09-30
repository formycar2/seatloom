# Acceptance: Onyx runtime triple migration (schema 009)

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance |
| id | acceptance-2026-09-30-onyx-runtime-triple |
| status | **ACCEPTED** |
| author | lyra |
| date | 2026-09-30 |
| verdict | **ACCEPTED** — with one recorded open defect, independently confirmed pre-existing |
| packet | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md` |
| fix packet | `docs/coordination/tasks/onyx/ONYX-2026-09-30-apply-db-ordering-fix-v1.md` |
| **accepted commit** | **`d4f40b58edd075a33222745379f18c16e9c0138a`** |
| merge commit | `80be8b5` into `track/infra-foundation` |
| verification | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v3.md` (PASS) |
| tags | acceptance, onyx, schema-009, runtime-triple |

---

## 1. Verdict

**ACCEPTED** at `d4f40b5`, merged to `track/infra-foundation` as `80be8b5`.

`Runtime` is now three orthogonal columns in PostgreSQL — `harness` / `agent` / `model` on
`sessions`, `default_harness` / `default_agent` / `default_model` on `seats` — with the old
`runtime` / `default_runtime` columns retained and marked deprecated by `COMMENT ON COLUMN`.

## 2. Evidence basis

Three commit-pinned gates. Each of the first two caught something real; recording that, because the
value of the gate is the point.

| Gate | Target | Verdict | What it caught |
|---|---|---|---|
| v1 | `580bcf4` | HOLD | The shared working tree could not be pinned: `pnpm tauri dev` (pid 91929, 30+h) recompiled on checkout and **regenerated** `src-tauri/gen/schemas/capabilities.json`, overwriting the target commit's content. Also `verify-postgres-baseline.sh` unrunnable — hardcoded `docker`, only `podman` present (Onyx drift D-5). |
| v2 | `580bcf4` | FAIL | The merge candidate **broke `scripts/apply-db.sh`**. Seed 001 writes the columns `009` creates, so it needs `009` pre-seed; `apply-db.sh` bucketed all migrations `≥006` post-seed. Control: `track` exit 0, candidate exit 3. |
| v3 | `d4f40b5` | **PASS** | Fix verified from an empty database, `.sql` boundary proven untouched, and the blamelessness claim independently reproduced. |

### 2.1 What v3 proved

- `apply-db.sh` builds the candidate from an **empty** database at **exit 0**.
- `git diff --stat 580bcf4..d4f40b5 -- '*.sql'` is **empty** — Onyx's central constraint held. The
  fix was the apply-order contract only.
- From-scratch assertions all pass: the three zero-counts; `DISTINCT` domains inside the
  value-domain contract; six seat rows carrying `Stepcode` / `ClaudeCode` / `claude-opus-5[1m]`;
  and **`NULL` harness on all five April session rows** — the anti-fabrication check the packet was
  written to protect.

## 3. The cross-packet collision — recorded, with blame placed correctly

The v2 FAIL was **not an Onyx defect.** `scripts/apply-db.sh` arrived with `8feea1b`
("ci: GitLab pipeline + ratchet verifiers"). Verified independently:
`git merge-base --is-ancestor 8feea1b 9599136` returns **false** — the script did not exist when
Onyx worked. Onyx registered `009` correctly in the two scripts that did exist.

The remedy is the durable part: `apply-db.sh` now orders migrations **by rule, not by number** —
pre-seed if the seed writes columns the migration creates, post-seed if it constrains columns the
seed does not provide — with a justification comment per migration. The `001–005` / `≥006` bucket
was a heuristic that `ENGINEERING_STANDARDS.md` already called convention rather than structure,
and it has now demonstrably failed once.

## 4. Open defect accepted with this merge

**A second consecutive `apply-db.sh` run fails** — exit 3,
`null value in column "project_id" of relation "sessions" violates not-null constraint`. Seed 001
inserts `sessions` rows with no `project_id`; `008` makes it `NOT NULL`; `ON CONFLICT` cannot help
because `NOT NULL` is checked before conflict resolution.

Accepted because it is **pre-existing and not introduced by this merge**, established twice by
independent method — Onyx via `git archive` of `track` to `/tmp`, Flux via a live `track` checkout.
Both got exact parity: same exit code, same message, same relation, same failing row
(`ses-aegis-supervisor-001`). Flux further showed `009` applies cleanly and the failure comes
afterwards, at the seed.

CI is green today only because it always builds from an empty database. **"`apply-db.sh` is
idempotent" is a reasonable assumption and it is false.** Follow-up packet owed.

## 5. Carried by Lyra, not counted against the verdict

1. **`verify-postgres-baseline.sh` unproven.** Flux's D-5 fix (`865f820`) made runtime detection
   work but is necessary-not-sufficient: podman has no compose provider on this machine, so
   `compose down -v` / `up -d` still fail. Not run, deliberately — that line destroys the volume
   the live app is attached to.
2. **`apply-db.sh`'s host-`psql` path is unexercised.** There is no `psql` client on this host;
   both Onyx and Flux shimmed through `podman exec`. The SQL, ordering and row state are proven;
   the script's own invocation path is not.

## 6. Follow-ups owed

| # | Item | Owner |
|---|---|---|
| 1 | `project_id` on seed 001 session tuples, so `apply-db.sh` is genuinely idempotent (§4) | Onyx |
| 2 | `verify-postgres-baseline.sh` to the `run`/`rm`/`volume rm` form `bootstrap.sh` uses (D-5 remainder) | Nimbus + Flux |
| 3 | Host `psql` client, or an explicitly supported containerised path | infrastructure — Mr. Zhang |
| 4 | When the destructive baseline reset may run, given the live app is attached | Mr. Zhang |
| 5 | Proper ordered-migration runner + separate idempotent seed | Onyx (already tracked) |
| 6 | `seat-dispatch.sh` misattributes every dispatch as "AEGIS dispatch" | Aegis |

## 7. Corrections against Lyra, recorded

1. **Packet §1.1 fact table was wrong (drift D-1).** It claimed six seats all `ClaudeCode` and a
   `data_engineer` binding "measured at base commit `9599136`". Committed state has **five** seats
   (`lyra`/`mira` = `GeminiCli`, `flux` = `Custom`) and no `data_engineer`. I read a dirty working
   tree. Conclusions were unaffected — the seeded values are evidenced by the 2026-09-28
   convergence addendum, not by that table.
2. **Branch-ref recovery went one commit short.** Onyx could not advance its own branch ref (Flux's
   worktree held it) and correctly flagged rather than forced it. I moved the ref to `cfdd647`,
   dropping Onyx's delivery-doc commit; caught it in the reflog and fast-forwarded to `d4f40b5`.
   Nothing lost, nothing rewritten. Cause: I acted on a snapshot taken when the monitor fired
   instead of re-reading state at the moment of acting.
3. **v3 packet predicted 3 commits in `580bcf4..d4f40b5`; there are 42**, because `ef4af6b` merges
   `track` into the packet branch. Flux verified the ancestry was clean regardless.

`18a69fb` is accepted as a **named exception**: not idle ride-along work, since the `009` seed
section sets the triple for six seats and the `onyx` row exists only because of it.

---

*Lyra · 2026-09-30 · accepted `d4f40b5`, merged `80be8b5` · Flux PASS after HOLD and FAIL, both of
which caught real defects · Nimbus runtime-triple refactor still outstanding, delivery doc owed*
