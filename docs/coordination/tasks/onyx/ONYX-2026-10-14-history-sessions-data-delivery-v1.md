# Delivery: migration 021 — the historical session corpus (38 projects / 183 sessions), and the live table that was not empty

[Onyx -> Nimbus / Mira / Flux / Aegis] `disp-history-sessions-data`: slice 1 of Lyra's
history-sessions-in-UI contract is landed on `packet/onyx/topic-opened-019` — 38 `projects` rows (one
per agent home path), the 183 corpus sessions with a message count each, 36,680 messages, zero rows in
`sessions`, zero new seats. **One thing in the brief was false and it changed the migration**: the live
`archived_sessions` table was not empty. It already held the same 271 manifest rows, so this migration
normalises 182 of them and inserts 1, instead of inserting 183 beside them and doubling every count.

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | implementation-delivery |
| id | ONYX-2026-10-14-history-sessions-data-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-10-14 |
| version | v1 |
| responds to | governed dispatch `disp-history-sessions-data`; dispatch events `ev-X3MqYbk4yOj_Ns3Dc6NNm` (issued), `ev-W3fvFlbqJsBk1cNIMCT5a` (delivered) |
| packet / contract | `docs/coordination/tasks/lyra/LYRA-2026-10-14-history-sessions-in-ui-contract-v1.md` on `packet/lyra/milestone-summary` @ `10504bc` — read via `git show packet/lyra/milestone-summary:<path>`; acceptance is its §6 and §8.2 slice 1 |
| base branch | `packet/onyx/topic-opened-019` @ `23c76ee` (schema state 001–020) |
| delivery branch | `packet/onyx/topic-opened-019` @ this commit |
| files changed | `infra/postgres/schema/021_history_sessions_corpus.sql` (new, 689 lines), `scripts/apply-db.sh` (+1 POST_SEED entry — its completeness guard refuses to build any database without it), `scripts/apply-schema-to-live.sh` (+021 to the apply list, +019/020 which were missing from it, +4 verification lines), this doc |
| rust / tsx touched | none — `cargo check -p seatloom-core -p seatloom-mcp-emit` clean. `seatloom-tauri` fails to build in this worktree for a pre-existing reason unrelated to this change (`tauri::generate_context!()` needs `ui/dist`, and the frontend has not been built here). No Rust reads `archived_sessions`, and `db::repositories::list_projects` / `get_project` select `projects` columns explicitly, so the two new columns cannot break hydration |
| live db touched | **none** — live was READ only (the census in §2 and §9). All writes were on scratch databases, dropped after |

---

## 0. What the dispatch asked for, and the one sentence in it that was not true

The brief: load the corpus into `archived_sessions` (not `sessions`), 38 `projects` rows, 183
`archived_sessions` rows, a message count per session summing to 36,680, the 8 repository-root sessions
as the `(root)` case with no guessed parent, idempotent, with a reconciliation query proving the six
numbers. All of that is delivered.

The false sentence is in the contract the brief rests on, §1.1: *"`archived_sessions` currently holds
zero rows — no seed writes it and no backfill script inserts into it."* That is true of a database built
by `scripts/apply-db.sh` and **false of the live one**. §2 is the measurement. It is the reason this
migration is an UPDATE-then-INSERT rather than the INSERT the brief describes, and it is escalated to
Aegis in §9 rather than quietly absorbed.

---

## 1. The corpus, measured (three sources, cross-checked row by row)

Nothing below is taken from the brief; every number was recomputed from the files.

| Source | What it is | What it supplied |
|---|---|---|
| `resume-contexts/SESSIONS-FLAT.csv` | 183 data rows | identity (`session_key`), home path, `first_ts`/`last_ts` |
| `resume-contexts/by-session/*.md` | 183 files | the `消息数` header line per session |
| `AgentHistory/_manifest.json` | 271 entries | the archive index 011 describes: `raw`/`src`/`bytes`/`lines`/`sha256`/`date` |

Cross-checks that passed for **all 183** rows: the by-session `home path` equals the CSV's home; the
by-session `来源文件` equals the CSV's file; the manifest's `cwd` equals the CSV's home; the manifest's
`raw` equals the CSV's file relativised under `AgentHistory/`; and the manifest's
`bytes`/`lines`/`sha256` were **re-computed from the file on disk** and agree exactly for the 169
file-backed rows.

| Measure | Value |
|---|---|
| sessions | 183 (codex 150, claude-code 18, cursor 14, stepcode 1; gemini 0) |
| distinct home paths = projects | 38 |
| messages | **36,680** (codex 25,927 + claude-code 10,753 + stepcode 0 + cursor 0) |
| `OfficeBench` | 49 sessions / 808 messages |
| `seatloom` | 14 sessions / 1,745 messages |
| cursor | 14 sessions, **0 with any timestamp** |
| sessions with a date | 169 (every non-cursor row) |
| indexed bytes | 1,294,569,345 |

### 1.1 Two measured corrections, both recorded rather than silently applied

**The flat CSV has a key defect.** `SESSIONS-FLAT.csv` row 19 (home `.../sf-code/brainpp`, codex) carries
`session_key` `019cd09f-d215-7142-a186-378692638b1e` — a copy of row 18's key. Its own `file`, the
manifest, and `by-session/sf-code__brainpp__codex__019cdb69-40b9.md` all name a different session,
`019cdb69-40b9-7873-8f21-d679c6a8a1ea`. Loading the CSV's key verbatim would violate
`PRIMARY KEY (harness, id)` and the migration would abort. The native id is taken from the raw file
name; three independent sources agree. **The CSV itself is a defect for Lyra/Aegis to fix at the
source** — this delivery does not touch `resume-contexts/`.

**cursor carries no timestamps, and its manifest date has no provenance.** Measured: cursor's
`chat_dump.json` contains only `prompts` (text + `commandType`) — there is no timestamp key anywhere in
it. The manifest's per-directory `date` is therefore not reproducible from the indexed artifact, so
`session_date` is set to `NULL` for the 14 cursor rows alongside `first_ts`/`last_ts`: the UI shows
时间未记录 and no date is invented. The dropped manifest dates are listed here so nothing is lost:
ThoughtOnly 2026-09-20 21:03, ai-insight 2026-04-02 18:22, benchmark 2026-02-14 14:05,
gnoloaixgnahz.github.io 2026-01-02 14:42, jobutils 2026-02-03 18:38,
private_knowledge_base/step-product/brainpp 2026-01-08 11:10, private_knowledge_base/freetalk
2025-11-20 13:57, private_knowledge_base 2026-02-02 18:45,
private_knowledge_base/step-product/mind/code/brainpp-main 2025-12-30 15:10 (both of its sessions),
seatloom 2026-06-24 12:02, sf-code/stepmind/brainpp 2026-02-04 15:15, sf-code/stepcloud 2026-03-02
10:34, tools 2026-01-09 17:00.

---

## 2. What the live database already held

Read-only census of `localhost:5432/seatloom` before this delivery (the same database the MCP ledger
serves — the event ids match):

| Table | Rows |
|---|---|
| `archived_sessions` | **271** |
| `projects` | 1 (`seatloom`) |
| `sessions` | 5 |
| `seats` | 7 |
| `dispatches` | 126 |
| `memories` | 135 (90 with `source_kind='archived_session'`) |

All 271 `archived_sessions` rows share one `archived_at` — `2026-10-08 08:15:27.089628+00` — i.e. a
single out-of-band load, and `ROADMAP.md` records it as intended state ("Archive: AgentHistory 271
sessions / 14GB ingested as `archived_sessions`"). Harness mix: codex 210, stepcode 20, claude-code 18,
cursor 13, gemini 9, opencode 1. 248 rows carry `sha256` (the 23 without are exactly the 13 cursor + 9
gemini + 1 opencode directory copies — 011's rule, already honoured by that load).

Matched to my 183 corpus sessions by `(harness, raw_path)`:

* **all 183 are already present**, in 182 distinct rows (the one cursor directory holding two sessions
  is one row);
* `session_date`, `bytes`, `lines`, `sha256`, `src_path` are **identical** for all 183;
* **31 rows carry a coarser `project` slug** than the home path — the manifest's 18-project grouping:
  `OfficeBench` for a session whose home is `.../OfficeBench/tasks` (17 rows),
  `ai-assistant/sf-package` for `.../ai-assistant/sf-package/storage` (5) and
  `.../brainpp-auto-test` (1), `sf-code/stepmind` for `.../sf-code/stepmind/brainpp` (2), `sf-code` for
  `.../sf-code/stepcloud` (1), `private_knowledge_base` for `.../step-product/brainpp` (1),
  `.../freetalk` (1) and `.../step-product/mind/code/brainpp-main` (2);
* **165 rows carry an id that is the raw file name**, not the session id: codex
  `rollout-<ts>-<uuid>`, cursor `<workspace hash>`, stepcode `<uuid>` where the corpus's native id is
  the wrapped `claude:<uuid>`.

**Why this rules out the brief's literal shape.** Inserting 183 rows beside the 271 would put the same
session in the table twice, and any project-grouped read would double: live already holds **66**
`OfficeBench` rows and **16** `seatloom` rows, so the operator's card step 2 ("点进去,应该列出 49 条会话")
would read **115** and step 4 ("应该有 14 条会话") would read **30**. That is not a rounding difference,
it is the exact failure the card exists to catch. So the migration normalises what exists and inserts
only what does not.

---

## 3. What `021_history_sessions_corpus.sql` does

Four guarded column additions, then four data statements:

| # | Statement | First apply (live-repro scratch) | Second apply |
|---|---|---|---|
| 1 | `projects.home_path`, `archived_sessions.message_count` / `first_ts` / `last_ts` — all `ADD COLUMN IF NOT EXISTS`, each with a `COMMENT` carrying its reasoning | 4 catalog writes | 0 (guards skip) |
| 2 | `INSERT INTO projects (id, name, created_at, home_path)` — 38 rows, `ON CONFLICT (id) DO NOTHING` | **37 rows** | 0 |
| 2b | `UPDATE projects SET home_path` for the seeded `seatloom` row, `WHERE home_path IS DISTINCT FROM ...` | **1 row** | 0 |
| 3 | `UPDATE archived_sessions ... FROM (VALUES ...)` — 182 rows: `id` → native session id, `project` → stripped home path, `+ message_count`/`first_ts`/`last_ts` | **182 rows** | 0 |
| 4 | `INSERT INTO archived_sessions ... VALUES` — 183 rows, `ON CONFLICT (harness, id) DO NOTHING` | **1 row** | 0 |

Net on a live-shaped database: 38 projects, 272 `archived_sessions` rows of which 183 are the corpus,
36,680 messages. On a **fresh** `apply-db.sh` database (§5.4) statement 3 matches nothing and statement
4 inserts all 183 — the migration is complete either way.

The `UPDATE`'s match key is `(harness, raw_path)` — stable across the re-key — plus
`id IN (prev_id, native_id)`, so it finds the row on the first apply by its previous id and is a no-op
on the second by its new id. Its trailing guard compares every column being written, which is what
makes the second apply a true zero rather than a silent rewrite.

`id` = `name` = the home path with `/Users/jyxc-dz-0100609/Documents/GitHub/` stripped. The basename is
**not** usable (`private_knowledge_base` is the basename of 4 distinct homes, `OfficeBench` of 2). The
bare repository root strips to the empty string and is carried as `(root)` — the exact slug
`v_archived_sessions_normalized`'s R5 escalates. `created_at` is the load date
(`2026-10-14T00:00:00Z`), not a session date: the corpus has no per-project creation date and 001 gives
the column no default. `parent_project_id` stays NULL for all 38 — no nesting, by instruction.

---

## 4. The three judgment calls (each with the alternative spelled out)

**4.1 The id re-key (164 of 182 updated rows).** 011 calls the column *"native session id (per
harness)"*. The live rows carry file names for 165 of the 183 corpus sessions. I re-keyed them to the
native session id because three independent things already use that convention: 011's own column
semantics, the corpus's identity everywhere (`SESSIONS-FLAT.csv`'s `session_key` and every
`by-session/*.md` filename), and the memory layer — 90 `memories` rows write `source_ref` as
`'<harness>:<id>'`, and the washing pipeline's own topics are named
`wash-sess-cx-019dce05` / `wash-sess-cx-019dda18` / `wash-sess-cx-019ddc3e` after the native codex
uuids. **Three of those codex references resolve against no row today** because the live ids are file
names; after 021 they resolve. No FK references `archived_sessions` (checked `pg_constraint`,
`pg_views`, `pg_trigger`), and the previous id is recoverable from `raw_path` by construction — it *is*
the raw file name — so the change is reversible. The zero-re-key alternative (keep the manifest ids,
correct only the 31 slugs) is a one-line change to statement 3 if Aegis prefers it.

**4.2 `session_date` → NULL for the 14 cursor rows.** See §1.1. The alternative (leave the manifest's
directory-level date) would let a read path surface a date for a harness that provably has none, which
the operator's card rejects outright ("打回:显示了一个具体日期 —— 那就是编的").

**4.3 A `(root)` projects row exists, and its 8 sessions are still unmapped.** The brief asks for 38
`projects` rows *and* for the 8 root sessions to land as the `(root)` case. Both hold: the row exists so
the 38 homes are all represented and carries the absolute `home_path`, while
`v_archived_sessions_normalized`'s R4/R5 `CASE` branch **precedes** R1, so those sessions are still
reported `unmapped` — verified in §5.2, not assumed. No parent was guessed for them.

---

## 5. Verification — everything measured on a scratch database

A scratch database was built in `apply-db.sh`'s exact order (001–005, 009 → seed 001–004 → 008,
010–020), then the live 2026-10-08 manifest load was reproduced into it from the live rows (271 rows,
271 distinct `raw_path`, 248 with `sha256`, identical `archived_at`), then 021 was applied. A second
scratch database was built the same way **without** the manifest load, to prove the fresh-database path.

### 5.1 The six numbers the operator's card checks

```
38 | 183 | 36680 | 49 | 14 | 14      -- projects | corpus sessions | messages | OfficeBench | seatloom | cursor
```

### 5.2 The root case and the normalization view

```
corpus '(root)' sessions ........................ 8     (13 across the whole table: 8 corpus + 5
                                                             inventory-only rows that already carried
                                                             the slug — the same 13 the 015 delivery
                                                             reported)
corpus rows by map_status ....................... mapped 175 | unmapped 8
whole table by map_status ....................... mapped 248 | unmapped 24
```

The 24 unmapped are the 13 `(root)` rows plus 11 inventory-only rows whose slugs resolve to nothing —
`ai-assistant/sf-package` (8), `(multi)`, `project_delivery`, `yx_work`. That is R4 at work: unmapped,
reported, never guessed. No project was created for them.

### 5.3 Zero contamination of the live machinery

```
sessions 5 | seats 6 | dispatches 0 (scratch)     -- unchanged by 021
archived_sessions 272                             -- 271 + the one inserted (2/2) cursor session
projects with parent_project_id NOT NULL ........ 0
seatloom project row: name 'SeatLoom' preserved, created_at 2026-04-27 preserved, home_path backfilled
```

### 5.4 Idempotency, with row counts

```
apply #1:  projects 37 | backfill 1 | update 182 | insert 1
apply #2:  projects  0 | backfill 0 | update   0 | insert 0
apply #3:  projects  0 | backfill 0 | update   0 | insert 0
```

A full-table fingerprint (`md5` over `id || harness || project || message_count || first_ts || last_ts`,
ordered) is byte-identical across applies 2 and 3. On the fresh database the same fingerprint is
identical after the second apply, and the six numbers are `38 | 183 | 36680 | 49 | 14 | 14` with
`cursor_dated = 0` and `root_sessions = 8`.

---

## 6. Hand-off notes for the next slices

**For Nimbus (slice 2, the read path):**

1. **Filter `message_count IS NOT NULL`.** The table holds 272 rows: 183 corpus + 89 inventory-only
   rows that have no extracted conversation. An unfiltered project-grouped count double-counts
   (`OfficeBench` 49 + 66 = 115).
2. **The date range comes from `first_ts`/`last_ts`, never from `session_date`.** For the 169 dated
   corpus rows `session_date` holds the manifest's local-time string (`'2026-03-06 11:30'`), retained
   verbatim from the 2026-10-08 load — a different representation, and not a range. For the 14 cursor
   corpus rows 021 set it to NULL. `first_ts`/`last_ts` are the UTC instants, and they are the only pair
   that can express both "when it ran" and "时间未记录".
3. **The 8 `(root)` sessions must be surfaced as an explicit 未归类(仓库根目录) group, not filtered
   out.** The view reports them `unmapped`; a read path that keeps only `map_status='mapped'` would
   make them vanish, which the card rejects as squarely as folding them into a neighbour.
4. Session ids are the native session ids (uuids for codex/claude-code, directory names for cursor,
   `claude:<uuid>` for the one stepcode row). The one stepcode id contains a colon, so split
   `'<harness>:<id>'` on the **first** colon only.
5. `by-session/<slug>__<harness>__<short-id>.md` on disk is the transcript source; the DB holds only
   the index, never a body.

**For Flux (slice 4):** the reconciliation queries are §5 of the migration file and §5 of this note.
The counts must be verified against `resume-contexts/`, not against the migration's own report — the
one place they differ by design is `SELECT count(*) FROM archived_sessions` (272, not 183).

**For Mira (slice 3):** nothing changes in this slice; the shapes Nimbus returns are unchanged.

---

## 7. The live apply — the one command, for 张先生

Live was not touched. The apply is a script run, auditable and repeatable:

```bash
bash scripts/apply-schema-to-live.sh
```

It applies missing migrations in number order and prints the verification. It now also closes the 020
gap (live has 015–019 but not 020's `fn_gate_is_gate_id`) — a deliberate side effect, since that script
exists to apply exactly what is missing; drop that one filename from its list if the timing is wrong.
Expected output after it runs: `历史会话语料(021)已加载 ✅183 条`, `消息总数 36680`, `项目数(应为 38) 38`,
`这 8 条仓库根会话未归类 ✅8 条`. The migration is idempotent, so a re-run is safe. **Until it runs, the
live database still has the old ids, the 31 coarse slugs and no message counts** — nothing in this
delivery claims otherwise.

---

## 8. Completion checklist

1. ✅ Committed to `packet/onyx/topic-opened-019` @ this commit (4 files; the commit message carries the
   dispatch id).
2. ✅ This delivery doc.
3. ✅ `ev-history-sessions-data-landed` — `ArtifactCreated`, subject `art-021-history-sessions-corpus`,
   actor `seat-onyx-001`, context `disp-history-sessions-data`, payload with branch, commit, files and
   the row counts. Deliberately an **announcement** id (`-landed`, no `verified` field): what it asserts
   is that the rows are on a branch at a commit, never that a read works.
4. ✅ Forward recording via `remember` — 4 records on `topic-aegis-fleet-operations`, source
   `ev-X3MqYbk4yOj_Ns3Dc6NNm`: 1 decision, 1 failure, 1 unresolved, 1 fact (§9).
5. ✅ `SupervisorMessage` to Aegis naming the one ruling this delivery needs (§9).
6. `complete_dispatch(disp-history-sessions-data)`.

---

## 9. Escalated to Aegis — one ruling needed, and it is cheap either way

**The ruling:** does the corpus own the `archived_sessions` ids, or do the manifest's file-name ids?
This delivery re-keyed 164 rows to the native session id (§4.1). The alternative — keep the manifest ids
and correct only the 31 project slugs — is one line in statement 3 of the migration and requires no
other change. Both produce the same six numbers; they differ only in what `id` means and in whether the
3 unresolvable `codex:<uuid>` memory references resolve.

**Recorded as unresolved, for Aegis:** one table now carries two id conventions — the 183 corpus rows
use native session ids, the 89 inventory-only rows keep file-name ids. That is the pre-existing
condition narrowed, not introduced, but closing it is the washing pipeline's own normalization decision
(Aegis lane), not this dispatch's.

**Also for Lyra/Aegis:** the `SESSIONS-FLAT.csv` row-19 key defect (§1.1). The migration works around it
and says so in its header; the file itself is on `track/infra-foundation` and is not mine to edit.
