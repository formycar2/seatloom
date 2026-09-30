# Delivery: Commit-pinned verification — Onyx runtime triple migration (schema 009)

[Flux -> Lyra] Verdict: **HOLD**

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-09-30 |
| to | lyra |
| verifies | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-v1.md` |
| delivery under test | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md` |
| target branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| **verified commit** | **`580bcf483e0ca9af293367e0602a802226ce1590`** (§14.5) |
| migration commit | `629c68e0d64f62f818de472ed9735c65a720132b` |
| compare base | `9599136d615db0c4cdb15811ccaca5532820870d` |
| verdict | **HOLD** |
| tags | flux, verification, commit-pinned, schema-009, runtime-triple, onyx, hold |

---

## 0. Verdict summary — why HOLD and not PASS

Every static assertion in §2.1, §2.5, §2.7 holds, and the live database state satisfies §2.3 and
§2.4 including the anti-fabrication check that the packet was written to protect. **No defect was
found in Onyx's work.** The HOLD is not about the migration; it is about two conditions that make
a PASS unsupportable under §14:

**HOLD-1 — §1 identity gate: workspace dirty, and actively re-dirtying itself.**
A `pnpm tauri dev` watcher has been running against this shared working tree for 30+ hours
(pid 91929, elapsed `01-06:29:38`). My `git checkout` of the target branch changed `src-tauri/`
sources, the watcher recompiled (`target/debug/seatloom-tauri` pid 41742, elapsed `01:35` — started
~95s after checkout), and the build **regenerated** `src-tauri/gen/schemas/capabilities.json`,
overwriting the target commit's content. The tree cannot be held pinned at `580bcf4` while that
process runs. Per §1 I did not fix it up and continue.

**HOLD-2 — §2.2 and §2.6 are not verifiable on this seat.**
§2.6 (`scripts/verify-postgres-baseline.sh`) cannot run: `docker` is not installed on this machine
and the script hardcodes it. This is **pre-existing and already self-reported by Onyx as drift
finding D-5**, proved at base commit `9599136` before any edit by the packet. §2.2 (apply the
migration twice, both transcripts) was **not attempted**: writing to the shared live database while
the tree integrity is compromised and a live app process is attached to that same database
contradicts the §4 HOLD instruction to "change nothing".

Per §4: *"If you can only verify part of §2, the verdict is HOLD with the reason, not PASS with a
caveat."*

**The distinction that matters for acceptance:** the §2.3/§2.4/§2.5 evidence below is
**observational** — it reads the database Onyx applied 43 hours ago. It is not a reproduction by
me. In particular `seats_agent_runtime_mismatch = 0` is **order-dependent**: it holds because seed
convergence (`18a69fb`) landed before `009` backfilled `default_agent` from `default_runtime`.
Observing the live database cannot distinguish "the migration is idempotent and order-correct" from
"it happened to work once, 43 hours ago." Proving the former is exactly what §2.2 and §2.6 exist to
do, and both are blocked.

---

## 1. Identity gate captures (§14.2, verbatim)

### 1.1 Before checkout

```
$ git rev-parse --abbrev-ref HEAD
track/infra-foundation

$ git rev-parse HEAD
e17170b992fd5b72b5d369cb5767112feb8f04df

$ git status --short
(empty — clean)
```

Note: HEAD is `e17170b`, two commits after the `4c13ffc` stated in the packet §0.3 — `ad82153`
(tree-clearing commit) and `e17170b` (this packet's own dispatch). Tree was clean, so no HOLD at
this point.

### 1.2 After checkout of the target

```
$ git checkout packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1
Switched to branch 'packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1'

$ git rev-parse HEAD
580bcf483e0ca9af293367e0602a802226ce1590     <- MATCHES target 580bcf4

$ git status --short
?? .gemini/
?? .seatloom/seat-sessions.tsv
?? .seatloom/transcripts/
```

`git status --short` is **not** empty. Two distinct causes, separated below.

### 1.3 Cause A — the three `??` entries are a branch-ignore artifact, not dirt (benign)

`track/infra-foundation` carries `.gitignore` rules that `580bcf4` predates:

```
$ git diff 580bcf4 track/infra-foundation -- .gitignore
+# Local harness / runtime state — machine-specific, never committed.
+# .seatloom/bootstrap/ stays tracked; only live runtime artifacts are ignored.
+.gemini/
+.seatloom/transcripts/
+.seatloom/seat-sessions.tsv
...
$ git show 580bcf4:.gitignore | grep -Ei 'gemini|seatloom'
(no match)
```

These are pre-existing machine-local runtime artifacts (`.gemini/settings.json`, 11
`.seatloom/transcripts/*.raw.log`, `.seatloom/seat-sessions.tsv`; mtimes 2026-05-11 and
2026-09-28). None is tracked on either branch; none appears in the packet diff. The track branch we
are merging *into* explicitly declares them "never committed". **This alone would not justify a
HOLD** — it does not contaminate the pinned tree.

Relevant to §2.1: `.seatloom/seats/` **does not exist on disk** (`ls: .seatloom/seats/: No such
file or directory`), so the D-2 failure mode the packet §2.7 warns about has not occurred.

### 1.4 Cause B — a live build watcher modified a tracked file mid-gate (this is HOLD-1)

My §1.2 capture showed no modified tracked files. Minutes later the same command showed:

```
$ git status --porcelain --untracked-files=all
 M src-tauri/gen/schemas/capabilities.json
?? .gemini/settings.json
?? .seatloom/seat-sessions.tsv
?? .seatloom/transcripts/ses-18ad8e3c3aa72360.raw.log
   ... (11 transcript logs total)

$ git diff -- src-tauri/gen/schemas/capabilities.json
-{}
\ No newline at end of file
+{"default":{"identifier":"default","description":"Default capabilities for SeatLoom main window","local":true,"windows":["main","supervisor"],"permissions":["core:default","core:event:allow-listen","core:event:allow-emit","core:window:allow-create","core:window:allow-close","core:window:allow-set-focus"]}}

$ ls -lT src-tauri/gen/schemas/capabilities.json
-rw-r--r--@ 1 ... 307 Sep 30 17:02:32 2026      <- written AFTER my checkout
$ date
Wed Sep 30 17:03:15 CST 2026
```

Attribution:

```
$ ps -Ao pid,etime,command | grep -Ei 'tauri|vite'
91929 01-06:29:38 node /opt/homebrew/bin/pnpm tauri dev
91961 01-06:29:38 node .../@tauri-apps/cli/tauri.js dev
92059 01-06:29:37 node .../ui/node_modules/.bin/../vite/bin/vite.js
41742       01:35 /Users/.../target/debug/seatloom-tauri     <- rebuilt on my checkout
```

The generated file is a Tauri build product. Checking out `580bcf4` reverted it to `{}`; the live
watcher rebuilt and regenerated it to the supervisor-window content. This is §14.4 — a dirty
workspace this packet did not authorize — with the aggravating property that it **re-dirties
automatically on every branch switch**, so serial seat execution over a shared tree is not actually
isolated while a dev watcher runs.

I did **not** kill the watcher (not my process, and the packet does not authorize it), did **not**
revert-and-continue, and did **not** stash (§0.2).

---

## 2. Evidence per §2

### 2.1 Scope constraints — ALL HOLD

```
$ git show --stat 629c68e
 ...26-09-28-onyx-runtime-triple-drift-report-v1.md | 376 +++++++++++++++++++++
 infra/postgres/schema/009_runtime_triple.sql       | 176 ++++++++++
 .../seed/001_real_collaboration_baseline.sql       | 109 +++++-
 scripts/bootstrap.sh                               |   3 +-
 scripts/verify-postgres-baseline.sh                |  14 +-
 5 files changed, 661 insertions(+), 17 deletions(-)

$ git merge-base 580bcf4 track/infra-foundation
9599136d615db0c4cdb15811ccaca5532820870d

$ git diff --stat 9599136..580bcf4
 ...26-09-28-onyx-runtime-triple-drift-report-v1.md | 376 +++++++++++
 ...6-09-28-runtime-triple-migration-delivery-v1.md | 732 +++++++++++++++++++++
 .../ONYX-2026-09-28-runtime-triple-migration-v1.md | 394 +++++++++++
 infra/postgres/schema/009_runtime_triple.sql       | 176 +++++
 .../seed/001_real_collaboration_baseline.sql       | 117 +++-
 scripts/bootstrap.sh                               |   3 +-
 scripts/verify-postgres-baseline.sh                |  14 +-
 7 files changed, 1795 insertions(+), 17 deletions(-)

$ git diff --name-only 9599136..580bcf4 | grep -E '^ui/|^src-tauri/|\.rs$'
(zero matches — all three assertions hold)

$ git diff --name-only 9599136..580bcf4 | grep -E '^\.seatloom/'
(zero matches — assertion holds)

$ ls -1 infra/postgres/schema/
001_seatloom_core.sql
002_document_authority.sql
003_write_ingest_reconcile.sql
004_operational_review_and_continuity.sql
005_prompt_and_channel_action_authority.sql
008_project_isolation.sql
009_runtime_triple.sql
                                    <- 006 and 007 absent; 009 used
```

### 2.2 Migration applies, and applies twice — **NOT VERIFIED**

Not attempted. See HOLD-2. No migration transcript exists in this delivery, and therefore the
idempotency acceptance criterion (Onyx packet §7.5) is **unproven by Flux**. Onyx's own claim of
idempotency (`ADD COLUMN IF NOT EXISTS`, `IS NULL`-guarded backfills, `ON CONFLICT ... DO UPDATE`
scoped to new columns) is visible in the migration source and is plausible on reading, but reading
is not what §2 asks for.

### 2.3 Verification queries — hold, as observation

Database reachable via podman (`docker` absent):

```
$ podman exec seatloom-postgres psql -U seatloom -d seatloom -tAc "select 'ok', current_database(), version()"
ok|seatloom|PostgreSQL 16.14 on aarch64-unknown-linux-musl, compiled by gcc (Alpine 15.2.0) 15.2.0, 64-bit
```

```
 sessions_missing_agent
------------------------
                      0
 sessions_agent_runtime_mismatch
---------------------------------
                               0
 seats_agent_runtime_mismatch
------------------------------
                            0
```

All three zero-counts return `0`.

```
   agent                 harness            default_agent
------------          ---------          ---------------
 ClaudeCode                              ClaudeCode
 Custom               (NULL only)        (1 row)
 GeminiCli            (1 row)
(3 rows)

 default_harness |   default_model            role
-----------------+-------------------   ---------------
 Stepcode        | claude-opus-5[1m]     architect
(1 row)                                  data_engineer
                                         designer
                                         product_owner
                                         supervisor
                                         verifier
                                        (6 rows)
```

Domain conformance: sessions `agent` ⊂ {ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Custom};
sessions `harness` is NULL only; seats `default_harness` = `Stepcode`, `default_model` =
`claude-opus-5[1m]`; all six roles snake_case and parseable. **All within the §2.1 value-domain
contract.**

### 2.4 Row state — holds, including the critical anti-fabrication check

```
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

Six seat rows carry the full triple. All five historical April session rows carry `agent` copied
from `runtime` with `harness` **and** `model` both NULL. **No April row has a non-NULL harness** —
the specific fabricated-attribution failure this packet was written to prevent has not occurred.

### 2.5 Old columns retained with deprecation comments — holds

```
 table_name |   column_name   | comment
------------+-----------------+--------------------------------------------------------------
 seats      | default_runtime | DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1.
                                Superseded by default_harness / default_agent / default_model.
                                Retained, not dropped: column retirement is deferred to a
                                follow-up packet after Flux confirms no reader references this
                                column. Read default_agent instead.
 sessions   | runtime         | DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1.
                                Superseded by harness / agent / model. Retained, not dropped:
                                column retirement is deferred to a follow-up packet after Flux
                                confirms no reader references this column. Read agent instead.
(2 rows)
```

Both columns exist, both comments name the packet id.

### 2.6 Baseline still green — **BLOCKED (pre-existing, Onyx drift D-5)**

```
$ command -v docker
docker: NOT FOUND
```

`scripts/verify-postgres-baseline.sh` hardcodes `docker compose` / `docker exec` / `docker cp`.
Only `podman` is available on this seat. Onyx already reported this as **D-5**, proved at base
commit `9599136` with exit code 127 — *"scripts/verify-postgres-baseline.sh: line 59: docker:
command not found"* — i.e. before the packet touched anything. Onyx correctly declined to fix it
(§4 D-5: "Flux owns the verification harness; changing it changes what a PASS means").

I did not port the verifier under the §15 bounded-fix exception: this is not harness/typo class,
it rewrites the meaning of a PASS, and it needs coordination with Nimbus who owns `bootstrap.sh`
ordering.

### 2.7 Artifact completeness — holds

```
$ wc -l docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md \
        docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md
     732 .../ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md
     376 .../2026-09-28-onyx-runtime-triple-drift-report-v1.md
```

Neither is a stub. The drift report has an explicit `## 4. Explicitly NOT remediated` section
tabulating six findings (D-1..D-6) with reasons — it states what was **not** fixed, as required.
It did **not** create `.seatloom/seats/*.yaml` (zero `.seatloom/` paths in the packet diff; the
directory does not exist on disk), so the §2.7 FAIL condition is not met.

---

## 3. Per-assertion table

| § | Assertion | Expected | Actual | Verdict |
|---|---|---|---|---|
| 1 | HEAD == target commit | `580bcf4` | `580bcf483e0ca9af293367e0602a802226ce1590` | PASS |
| 1 | `git status --short` empty after checkout | empty | 3 untracked (ignore-rule artifact) + 1 tracked file modified by live watcher | **FAIL -> HOLD-1** |
| 2.1 | zero files under `ui/` | 0 | 0 | PASS |
| 2.1 | zero files under `src-tauri/` | 0 | 0 | PASS |
| 2.1 | zero `.rs` files | 0 | 0 | PASS |
| 2.1 | zero files created under `.seatloom/` | 0 | 0 | PASS |
| 2.1 | slots 006 / 007 absent | absent | absent (001-005, 008, 009) | PASS |
| 2.2 | migration applies | proven | not attempted | **NOT VERIFIED -> HOLD-2** |
| 2.2 | second apply is clean no-op | proven | not attempted | **NOT VERIFIED -> HOLD-2** |
| 2.3 | `sessions_missing_agent` | 0 | 0 | PASS (observed) |
| 2.3 | `sessions_agent_runtime_mismatch` | 0 | 0 | PASS (observed) |
| 2.3 | `seats_agent_runtime_mismatch` | 0 | 0 | PASS (observed, order-dependent) |
| 2.3 | value domains conform | in-contract | in-contract | PASS (observed) |
| 2.4 | 6 seats `Stepcode`/`ClaudeCode`/`claude-opus-5[1m]` | 6 rows | 6 rows | PASS (observed) |
| 2.4 | April sessions: agent=runtime, harness NULL, model NULL | 5 rows | 5 rows | PASS (observed) |
| 2.4 | no April row has non-NULL harness | none | none | PASS (observed) |
| 2.5 | `seats.default_runtime` retained + commented | yes | yes | PASS |
| 2.5 | `sessions.runtime` retained + commented | yes | yes | PASS |
| 2.6 | `verify-postgres-baseline.sh` green | green | unrunnable, `docker` absent (D-5, pre-existing) | **BLOCKED -> HOLD-2** |
| 2.7 | delivery doc exists, not a stub | yes | 732 lines | PASS |
| 2.7 | drift report exists, states non-remediation | yes | 376 lines, §4 table D-1..D-6 | PASS |
| 2.7 | no `.seatloom/seats/*.yaml` fabricated | none | none | PASS |

---

## 4. §2.8 finding — commit `18a69fb`

```
$ git show --stat 18a69fb
chore(seed): commit pre-existing uncommitted 2026-09-28 seat convergence
 infra/postgres/seed/001_real_collaboration_baseline.sql | 16 ++++++++++++----
 1 file changed, 12 insertions(+), 4 deletions(-)
```

**What it actually contains** (one file, seed 001 only):

- `default_runtime` realigned to `ClaudeCode` for `lyra`, `mira`, `flux` (was `GeminiCli`,
  `GeminiCli`, `Custom`);
- a sixth seat row `seat-onyx-001` / `onyx` added, with `data-modeling, postgres,
  schema-migration, data-quality` capability tags and `created_at 2026-09-28`;
- a sixth `project_role_bindings` row: `onyx` / `data_engineer`.

Nothing else. No schema change, no session rows touched.

**Is it in-scope?** My assessment: **not authored by this packet, but load-bearing for it — and
honestly disclosed.** Three points for your decision:

1. **It is not idle ride-along work.** The `009` seed section sets the triple for *six* seats. The
   `onyx` row only exists because of `18a69fb`. Revert `18a69fb` and the packet's own §2.3
   deliverable no longer matches the seed it ships. These two commits are not separable in
   practice, even though they are separable in authorship.
2. **Onyx handled the disclosure correctly.** It was committed separately with a message stating it
   was pre-existing at `9599136` and not authored by the packet, and it is self-reported as drift
   finding **D-1**, which also corrects the record: the packet §1.1 "measured at base commit" facts
   were actually measured against a dirty working tree — `9599136` has **five** seats with mixed
   `default_runtime` and five role bindings with no `data_engineer`. That is the seat catching and
   reporting an error in its own dispatched packet, which is the behaviour the rules want.
3. **The one thing I would not let pass silently:** it makes `seats_agent_runtime_mismatch = 0`
   order-dependent, as flagged in §0. The live database cannot distinguish a correct idempotent
   migration from a lucky one-time ordering. This is a second, independent reason the §2.2 /§2.6
   proofs matter rather than being a formality.

**My recommendation:** `18a69fb` should not block acceptance on scope grounds. It should be
accepted explicitly, as a named exception, with your acknowledgement that packet §1.1's base-commit
fact table is wrong per D-1 — not accepted by silence.

---

## 5. What would clear this HOLD

1. **Quiesce the tree.** Stop `pnpm tauri dev` (pid 91929) for the duration of the gate, or give
   Flux a dedicated checkout. A shared working tree with a live build watcher cannot satisfy §14.2
   for any packet that touches `src-tauri/`, so this will recur on the Nimbus gate, which touches
   `crates/**` and is more exposed to it than this one was.
2. **Decide the verifier-portability question,** since it blocks §2.6 for every future DB packet:
   either install/alias `docker`, or dispatch the D-5 port of `verify-postgres-baseline.sh` to
   podman-or-docker detection (Flux + Nimbus, per Onyx's recommendation). Until then no Flux
   delivery can honestly report §2.6 green on this seat.
3. **Re-run §2.2 and §2.6 on a quiesced tree.** With those two transcripts and the DB reset the
   verifier performs, the remaining assertions are already proven above and the verdict should
   convert to PASS.

Re-dispatch as `...-v2` and I will run it; nothing in §2.1/§2.3/§2.4/§2.5/§2.7 needs redoing
except as the verifier's own destructive reset re-proves it from scratch.

---

## 6. Tree returned (§0.1)

```
$ git checkout track/infra-foundation
error: Your local changes to the following files would be overwritten by checkout:
	src-tauri/gen/schemas/capabilities.json
Aborting
```

The watcher-generated file blocked the switch. I proved the discard was lossless before making it —
the on-disk content is byte-identical to what `track/infra-foundation` already has committed:

```
on-disk : cc4097f8e30676021b85c350361cb83abedd499e2391c4c6f5ce7fbc47658a72
track   : cc4097f8e30676021b85c350361cb83abedd499e2391c4c6f5ce7fbc47658a72
580bcf4 : 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a   ("{}")
```

Then:

```
$ git checkout -- src-tauri/gen/schemas/capabilities.json
$ git checkout track/infra-foundation
Switched to branch 'track/infra-foundation'
$ git rev-parse --abbrev-ref HEAD
track/infra-foundation
$ git rev-parse HEAD
e17170b992fd5b72b5d369cb5767112feb8f04df
$ git status --short
(empty — clean)
```

**Tree returned to `track/infra-foundation` @ `e17170b`, clean. No stash was created. No commit was
made to the packet branch. Nothing was merged. The database was not written to.** The watcher will
regenerate `capabilities.json` to the track-branch content on its next rebuild, which leaves the
tree clean.

Per packet §3 I did not check out `packet/nimbus/...` and did not verify the Nimbus packet.

---

*Flux · 2026-09-30 · HOLD · verified commit `580bcf483e0ca9af293367e0602a802226ce1590` ·
COORDINATION_RULES §14 commit-pinned gate · no defect found in Onyx's migration; HOLD is on gate
integrity and two unverifiable assertions*
