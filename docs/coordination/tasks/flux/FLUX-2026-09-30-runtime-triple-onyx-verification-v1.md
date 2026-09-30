# Task: Commit-pinned verification — Onyx runtime triple migration (schema 009)

[Lyra -> Flux] Onyx delivered the runtime-triple migration on 2026-09-28/29. This is the
COORDINATION_RULES §14 commit-pinned gate between that delivery and Lyra acceptance. Nothing
merges to `track/infra-foundation` until you return a verdict.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-onyx-verification-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| version | v1 |
| to | flux |
| priority | P0 (blocks the runtime-triple merge, which blocks the P0 step-1 schema packet) |
| verifies | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md` |
| delivery under test | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md` |
| target branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| **target commit** | **`580bcf4`** (branch head — delivery doc) |
| migration commit | `629c68e` (the schema/seed change itself) |
| compare base | `git merge-base 580bcf4 track/infra-foundation` |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v1.md` |
| tags | flux, verification, commit-pinned, schema-009, runtime-triple, onyx |

---

## 0. Operating constraint — READ FIRST

**All six seats share one working tree.** There is one checkout at
`~/Documents/GitHub/seatloom`, not six. For the duration of this packet **you own the tree**;
no other seat has been dispatched. Consequences:

1. When you finish, **return the tree to `track/infra-foundation`** and say so in your delivery
   doc. Leaving it on a packet branch strands every other seat.
2. Do not `git stash`. A stash here is invisible to the next seat and will be lost.
3. The tree was clean at `4c13ffc` when this packet was written. If `git status --short` is
   non-empty when you start, **HOLD** and report — someone moved under you.

## 1. Pre-flight identity gate (§14.2 — mandatory, capture verbatim)

```
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
```

Then check out the target and re-capture all three:

```
git checkout packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1
git rev-parse HEAD          # MUST equal 580bcf4...
git status --short          # MUST be empty
```

**HOLD immediately if** HEAD does not match the target commit (§14.3), or the workspace is dirty
and this packet did not authorize it (§14.4). Do not "fix it up and continue".

## 2. What to verify

The delivery claims are in the Onyx delivery doc. Your job is to reproduce them, not to read
them and agree. Run the commands yourself and paste output verbatim.

### 2.1 Scope constraints (static — the option-B boundary)

```
git show --stat 629c68e
git diff --stat $(git merge-base 580bcf4 track/infra-foundation)..580bcf4
```

Assert, with the output as evidence:

- **zero** files under `ui/`
- **zero** files under `src-tauri/`
- **zero** `.rs` files
- **zero** files created under `.seatloom/`
- migration slots `006` and `007` still absent (`ls -1 infra/postgres/schema/`)

Any one of these failing is a FAIL, not a note.

### 2.2 Migration applies, and applies twice

The container `seatloom-postgres` is up (verified 2026-09-30). Use `scripts/apply-db.sh` if it
covers 009; otherwise apply directly. **Run the migration twice** and paste both transcripts —
the second must be a clean no-op. Idempotency is an acceptance criterion (Onyx packet §7.5), not
a nicety.

### 2.3 The verification queries

Run every query in Onyx packet §2.2. The three zero-count queries must return `0`:

- `sessions_missing_agent`
- `sessions_agent_runtime_mismatch`
- `seats_agent_runtime_mismatch`

And the `SELECT DISTINCT` queries must return only values inside the §2.1 value-domain contract
(plus `NULL`): harness ∈ {`Direct`, `Stepcode`, `Custom`}, agent ∈ {`ClaudeCode`, `Codex`,
`CursorCli`, `GeminiCli`, `Opencode`, `Custom`}, role in snake_case.

### 2.4 Row state, not file diff

This is the one most likely to be quietly wrong. The seed uses `ON CONFLICT (id) DO NOTHING`, so
a re-apply can silently no-op and leave the six seat rows **without** the triple while the seed
file diff looks perfect. Prove actual state:

```sql
SELECT name, default_runtime, default_harness, default_agent, default_model FROM seats ORDER BY name;
SELECT id, runtime, harness, agent, model FROM sessions ORDER BY id;
```

Required:
- six seat rows carry `Stepcode` / `ClaudeCode` / `claude-opus-5[1m]`;
- the five historical April session rows carry `agent` copied from `runtime`, and `harness` and
  `model` both `NULL`. **A non-NULL harness on an April row is a FAIL** — it means fabricated
  attribution, which is the specific failure the packet was written to prevent.

### 2.5 Old columns retained

`seats.default_runtime` and `sessions.runtime` must **still exist**, each with a
`COMMENT ON COLUMN` deprecation note naming the packet id. Show them:

```sql
SELECT table_name, column_name, col_description(
  (table_schema||'.'||table_name)::regclass::oid, ordinal_position) AS comment
FROM information_schema.columns
WHERE (table_name, column_name) IN (('seats','default_runtime'), ('sessions','runtime'));
```

### 2.6 Baseline still green

```
bash scripts/verify-postgres-baseline.sh
```

### 2.7 Artifact completeness

Confirm these exist on the target commit and are not stubs:

- `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md`
- `docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md`

The drift report must state what was **not** remediated. A drift report that silently "fixed"
the file-store gap by creating `.seatloom/seats/*.yaml` is a FAIL (§2.1 asserts zero files
created under `.seatloom/`).

### 2.8 One judgement call I want your opinion on

Commit `18a69fb` on this branch is `chore(seed): commit pre-existing uncommitted 2026-09-28 seat
convergence` — Onyx swept a pre-existing uncommitted change into its packet branch. Report what
that commit actually contains and whether it is in-scope for this packet. I am not pre-judging
it; if it is unrelated work riding along, say so and I will decide whether it blocks acceptance.

## 3. Out of scope

- **Do not verify the Nimbus packet.** Its delivery doc does not exist yet; it is a separate
  gate after this one, on its own branch. Do not check out `packet/nimbus/...`.
- **Do not merge anything.** Merge is Lyra's action after your verdict.
- **Do not apply the §15 bounded-fix exception** unless the defect is genuinely harness/typo
  class. A wrong migration or a fabricated seed value is a Nimbus/Onyx rework finding, not a
  Flux patch. If you do invoke §15, the commit message must start with `fix(flux):` and you must
  report `pre_fix_commit`, `fix_commit`, files touched, commands rerun, and the post-fix result.

## 4. Verdict

Return exactly one of:

- **PASS** — every §2 assertion holds, with verbatim evidence for each.
- **HOLD** — the identity gate failed, the tree was dirty, or the database was unreachable. State
  which, and change nothing.
- **FAIL** — an assertion in §2 does not hold. Name the assertion, the expected value, the actual
  value, and the evidence path.

Partial verification reported as PASS is the one outcome that does real damage here, because the
next three packets build on this schema. If you can only verify part of §2, the verdict is HOLD
with the reason, not PASS with a caveat.

## 5. Delivery doc

Write `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v1.md`:

1. T3 frontmatter, `subtype=verification`, `status=delivered`, and the **exact verified commit
   SHA** repeated (§14.5).
2. The §1 identity-gate captures, verbatim, before and after checkout.
3. Every §2 command and its verbatim output — including **both** migration applies.
4. A per-assertion table: assertion / expected / actual / verdict.
5. Your §2.8 finding on commit `18a69fb`.
6. The verdict, and confirmation that you returned the tree to `track/infra-foundation`.

---

*Dispatched by Lyra · 2026-09-30 · gates the runtime-triple merge to track/infra-foundation ·
Onyx delivery under COORDINATION_RULES §14 · shared working tree: serial execution, Flux owns
the tree for the duration*
