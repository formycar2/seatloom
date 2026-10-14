# Flux verdict — the historical session corpus (slices 1+2: Onyx 33f456b data, Nimbus d3e53ea read path), commit-pinned

- **Dispatch**: `disp-history-sessions-verify` (issued by seat-lyra-001, priority 70; both edges measured
  true — `ev-history-sessions-data-landed`, `ev-history-sessions-read-landed`)
- **Verdict event**: `ev-verdict-history-sessions` (pre-declared id, emitted exactly as required)
- **Verdict**: **FAIL** — with the gap named precisely, and with the part that IS correct separated
  from the part that is not
- **Verified commits (repeated in every finding below)**:
  - data: `33f456b` on `packet/onyx/topic-opened-019` (`infra/postgres/schema/021_history_sessions_corpus.sql`, 695 lines)
  - read path: `d3e53ea` on `packet/nimbus/gate-vs-announcement`
- **Contract**: `LYRA-2026-10-14-history-sessions-in-ui-contract-v1` @ `10504bc` on
  `packet/lyra/milestone-summary` — read in full from the shared ref namespace; §8.2 Slice 4 is the
  acceptance criteria
- **Aegis nudge honoured**: judged on the contract's D1/D2/D3 numbers measured against the source,
  not on "rows are non-zero". The nudge's specific prediction — that the operator would see **58
  sessions under `sf-code/storage` instead of the real count** — is CONFIRMED as a real defect, and
  the mechanism is identified below (it is the read path, not the data).

## §14 captures (COORDINATION_RULES §14)

```
$ git rev-parse --abbrev-ref HEAD   -> packet/flux/intent-verdict
$ git rev-parse HEAD                -> ae3e0f5beec0779ed4a4085b744613d5069f8785
$ git status --short                -> ?? .scratch/*, .seatloom/rollout-ack.txt (pre-existing untracked; tracked tree clean)
```

My branch is not either slice's pin (correct — I verify, I do not land). Each slice was verified at
its own pin: Onyx's `021` read via `git show 33f456b:...`; Nimbus's code built and tested in a
detached worktree at `d3e53ea`. No HOLD condition: tracked tree clean, no unauthorized dirt.

## 1. The source numbers — measured first, by me, before looking at any code

Run in `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/resume-contexts` (the gitignored corpus on
the main checkout; `by-session/` holds 183 files):

| # | Contract | Measured | Query | Verdict |
|---|---|---|---|---|
| D1 | 183 sessions | **183** | `tail -n +2 SESSIONS-FLAT.csv \| wc -l` = 183; `ls by-session/ \| wc -l` = 183 | ✅ |
| D2 | 38 projects | **38** | `tail -n +2 SESSIONS-FLAT.csv \| cut -d, -f1 \| sort -u \| wc -l` = 38 | ✅ |
| D3 | 36,680 messages | **36,680** | `grep -h "^消息数" by-session/*.md \| awk '{s+=$2} END{print s}'` = 36680 (183 header lines, one per file) | ✅ |
| D6 | cursor 14 (not 13) | **14, all undated** | CSV parse: 14 cursor rows, `first_ts`/`last_ts` empty in all 14; 13 distinct homes, one holds 2 sessions | ✅ the trap is real and the source says 14 |
| D9 | OfficeBench 49 | **49** | exact home `/Users/jyxc-dz-0100609/Documents/GitHub/OfficeBench` = 49 rows (the 17 `OfficeBench/tasks` rows are a separate home — `grep -c OfficeBench` = 66 is the WRONG query) | ✅ |
| — | seatloom | **14** | exact home count | ✅ |
| — | harness split | **codex 150 / claude-code 18 / cursor 14 / stepcode 1 / gemini 0** | `cut -d, -f2 \| sort \| uniq -c` (150+18+14+1 = 183) | ✅ |
| D12 | root sessions | **8** | exact home `/Users/jyxc-dz-0100609/Documents/GitHub` = 8 rows | ✅ |

## 2. The live database as it stands (what the operator would open today)

```
seatloom (live):  select count(*) from archived_sessions;                -> 271
                  select count(*) from projects;                          -> 1   (the seeded 'seatloom' row)
                  select count(*) from sessions;                          -> 5   (all created 2026-04-27/28 — pre-existing fleet sessions)
                  select count(*) from seats;                             -> 7   (mira/aegis/lyra/nimbus/flux/onyx/vela — no archive seat)
                  information_schema: archived_sessions.message_count     -> DOES NOT EXIST
```

The 271 rows are the 2026-10-08 out-of-band manifest inventory (all sharing one `archived_at`), not
the corpus. `021` has never been applied live — it exists only on `packet/onyx/topic-opened-019`,
and the live apply is the operator's `bash scripts/apply-schema-to-live.sh`.

**Live's real shape, verified by me against Aegis's correction** (`AEGIS-2026-10-14-flux-live-shape-
correction.md`) — I do not take the nudge's numbers on trust either; every one below is my own query
on live, and they match the nudge exactly:

```
select project, count(*) from archived_sessions group by 1 order by 2 desc;   (live)
  OfficeBench 66 | sf-code/storage 58 | sf-code/brainpp 25 | seatloom 16 |
  ai-assistant/sf-package 14 | SteptronOss 13 | (root) 13 | fde/field_evaluation 10 | ... total 271
select count(*) from projects;                                              -> 1  (id=seatloom, name=SeatLoom)
archived_sessions.message_count                                            -> column does not exist
```

So the backend DID the project attribution (the `project` text column carries the manifest's coarse
18-project grouping) but did NOT merge the 271 files into the 183 sessions and did NOT build the 38
home paths as `projects` rows. Accurate problem statement, judged **against live**:

| Contract | Live reality | Verdict |
|---|---|---|
| **D1** 183 sessions | **271 unmerged rows** — `sf-code/storage`'s 58 files are one root conversation's 56 launches plus 2 others; the merge truth is `SESSIONS-FLAT.csv` (183 rows) / `by-session/` (183 files) | **NOT MET** ❌ |
| **D2** 38 projects | **1 row** — the 38 home paths live only in a text column, so a project-organised UI cannot get them | **NOT MET** ❌ |
| **D3** 36,680 messages | **not verifiable in the DB — no `message_count` column** | **NOT MET in the DB** ⚠️ |

**Aegis's D3 question, answered explicitly (it must not be silently skipped):** the contract
§8.2 Slice 1 already answers it — *"A message count per session, sourced from `by-session/*.md`
(sum must be 36,680). If a column is needed, it is a migration in Onyx's slot, additive and
idempotent."* So D3 has TWO halves and I measured both: (a) the source half — the 183 `by-session`
headers sum to exactly 36,680 (measured, §1); (b) the data half — the corpus must carry a
per-session count summing to 36,680, and the contract sanctions `021`'s `message_count` column as
the mechanism. On live, half (b) does not exist at all. **D3 is NOT met on live, and the fix is the
backend carrying the count (021's column), independently checkable against the by-session headers —
not a redefinition of D3 to "file headers only", because then the product itself would still be
unable to report the number.**

Against the live DB the read path refuses with an honest typed error (measured, §5 below) — correct
behaviour, but it means the UI has nothing.

## 3. The data slice (Onyx, 33f456b) — verified CORRECT on a live-faithful reproduction

Method: built scratch DB `seatloom_flux_v` in the canonical order (pre-seed 001-005+009, seed
001-004, post-seed 008+010-014, then 015-019 from `track/infra-foundation`), then **reproduced the
live state exactly** by loading a `pg_dump --data-only` of live `archived_sessions` (271 rows, one
`archived_at`) on top, then applied `021` from `33f456b`. This is the state the operator's
`apply-schema-to-live.sh` will actually face — not the empty database a fresh build gives.

Results after one apply of `021`:

```
select count(*) from archived_sessions;                                  -> 272   (271 inventory + 1 new)
select count(*) from archived_sessions where message_count is not null;  -> 183   ✅ D1
select count(*) from projects;                                           -> 38    ✅ D2
select sum(message_count) from archived_sessions where message_count is not null; -> 36680  ✅ D3
```

`021` does not insert 183 rows next to 271 (which would double every count); it NORMALISES the 182
corpus rows the inventory already holds and INSERTS the one that does not — the second session in
the cursor directory that holds two. Measured write counters confirm exactly that: 182 `n_tup_upd`
+ 1 `n_tup_ins` on `archived_sessions`, 37 inserts + 1 guarded `home_path` backfill on `projects`.

Per-project and per-harness, all measured:

```
OfficeBench 49/808 | sf-code/brainpp 25/3422 | OfficeBench/tasks 17/369 | seatloom 14/1745 |
(root) 8/768 | cursor 14, message_count 0 each, first_ts/last_ts/session_date ALL NULL
harness split over the corpus: codex 150 / claude-code 18 / cursor 14 / stepcode 1 / gemini 0
```

**Independent row-by-row cross-check against the source** (not against the migration's own VALUES):
parsed the post-`021` rows out of a fresh `pg_dump` and joined them to `SESSIONS-FLAT.csv` and the
`by-session/*.md` headers on `(harness, raw_path)`. All 183 source rows matched exactly one DB row;
`cwd` == the CSV home, `project` == the home path with the `GitHub/` prefix stripped (`(root)` for
the bare root), `message_count` == that file's `消息数` header, `first_ts`/`last_ts` present iff the
CSV has them. The two cursor sessions sharing one directory are both present, both 0 messages, both
undated. Zero mismatches, zero extras. (The only source-side defect found is pre-existing and NOT
the data slice's: `SESSIONS-FLAT.csv` row 19 carries row 18's `session_key` — `021` documented this
and took the native id from the raw file name, which three sources agree on.)

**Idempotency, write-proven** (not just "state looks the same"): applied `021` a third time and read
`pg_stat_user_tables` deltas across the apply — `archived_sessions (0 upd, 0 ins, 0 del)`,
`projects (0, 0, 0)`. A second apply also leaves `272|36680|169` byte-identical.

**The 89 inventory-only rows** (`message_count IS NULL`) stay in the table — that is the designed
separation, and `021`'s own header states the rule the reader must follow: *"a project-grouped read
must filter on this column being NOT NULL to avoid double counting the 89 inventory-only rows."*
78 of those 89 rows carry a `project` slug that matches one of the 38 project ids (the manifest's
coarse 18-project grouping, e.g. `OfficeBench` for a session whose home is `.../OfficeBench/tasks`);
the other 11 carry slugs outside the 38 (`(multi)`, `yx_work`, …). **This is where the failure
lives — see §5.**

## 4. The eight numbered items from the dispatch — each answered

**1. Zero rows inserted into `sessions`, zero new rows in `seats` — PASS.**
Live: `sessions` = 5 rows, all created 2026-04-27/28 (fleet bootstrap, predating this work by
months); `seats` = 7 rows, exactly the known fleet, no archive seat. `021` never names either table
(`grep -i "insert into sessions\|insert into seats\|update sessions\|update seats\|delete from"`
over the 695-line file: no matches). The read path is read-only (its own POINT 6: ledger rows
before == after; `sessions` still 5, `seats` still 6 on the scratch DB).

**2. No transcript body in the database — PASS.**
`archived_sessions` has no body column (14 columns; `021` adds only `message_count`, `first_ts`,
`last_ts`). The turns read returns `bodiesReturned: false` with per-turn `line` (1-based, in the
ORIGINAL file), `contextPath`/`contextLineStart/End`, `label`, `chars` — pointers only. Measured on
a real call: the payload contains no instruction/transcript text.

**3. The 8 root sessions unmapped and labelled, no parent guessed — PASS.**
The `(root)` project row exists with its absolute `home_path`; `parent_project_id` is NULL for all
38 projects (measured: 0 non-null — no nesting rule was applied, and `021`'s header explicitly
refuses one because the bare root is a path ancestor of 37 other homes). `015`'s
`v_archived_sessions_normalized` reports exactly the 8 corpus root sessions `unmapped` (R5; corpus
total 175 mapped + 8 unmapped = 183). The read surfaces the group labelled (`unclassified: true`,
home path shown).

**4. The 14 undated cursor sessions come back AFTER the dated ones and flagged — PASS for the
corpus, with a caveat.** Ordering is `(first_ts IS NULL) ASC, first_ts DESC NULLS LAST` and the
cursor row carries `dated: false`. Caveat (see §5): inventory rows that lack `first_ts` but carry a
manifest `session_date` sort last while being flagged `dated: true` — internally inconsistent, and
they should not be in the list at all.

**5. One function exposed twice over the same code — PASS.**
One implementation (`seatloom_core::db` + `seatloom_core::history::read`), three MCP tools
(`list_history_projects` / `list_history_sessions` / `get_history_session_turns`) and three Tauri
thin wrappers that shell out to the same binary — the established `cmd_enqueue_dispatch` pattern.
No second reader exists in the diff.

**6. Unknown slugs reported unmapped, never guessed — PASS.**
`015`'s R4/R5 view is the executable rule and was not bypassed: unknown inventory slugs
(`ai-assistant/sf-package`, `(multi)`, `yx_work`, `project_delivery`) come back `unmapped`, and
`(root)` is escalated, not force-fit.

**7. Idempotency — PASS** (write-proven in §3: zero catalog writes, zero row writes on re-apply).

**8. Gates run for real — PASS with one environmental exception.** On the pinned `d3e53ea`
(detached worktree):

```
cargo fmt --all --check                                   -> clean
cargo clippy -p seatloom-core -p seatloom-mcp-emit --all-targets -- -D warnings -> clean
cargo test -p seatloom-core            -> 151 passed / 0 failed (2 ignored)
cargo test -p seatloom-mcp-emit        -> 81 passed / 0 failed (19 ignored) + waker 9 passed
cargo check -p seatloom-tauri          -> CANNOT RUN HERE: frontendDist "../ui/dist" does not exist
```

The `seatloom-tauri` failure is environmental and pre-existing: the built frontend `ui/dist` exists
in neither the main checkout nor any worktree, and `cargo check -p seatloom-tauri` fails identically
on the parent commit `d3e53ea^` with no history code in it. So the Tauri half of the read path was
NOT compile-verified by me; everything else was.

## 5. The read path (Nimbus, d3e53ea) — FAIL against the live database shape

Nimbus's own proof (`crates/seatloom-mcp-emit/proof/history_read_path_proof.sh`) passes on a
**fresh** scratch database: 38 / 183 / 36680 / OfficeBench 49 / seatloom 14 / cursor 14 all
undated, all six points. I ran it and reproduced that. But a fresh database is not the live
database: the live table holds the 271-row inventory, and the proof never tested that shape.

Driving the real binary (built from `d3e53ea`) against my live-faithful scratch DB
(`seatloom_flux_v` = 271 inventory rows + `021` applied), with the real `resume-contexts` on disk:

```
list_history_projects  ->  projectCount 38  ✅
                           messageTotal 36680  ✅   (NULL message_counts don't sum, so the
                                                     message half is accidentally right)
                           sum(sessionCount) = 261  ❌   (should be 183)
```

**10 of 38 projects report a wrong session count**, because `list_history_projects` joins
`LEFT JOIN archived_sessions a ON a.project = p.id` with **no `message_count IS NOT NULL` filter**
(the same omission in `list_history_sessions`' `WHERE a.project = $1`):

| project | tool says | corpus truth | excess |
|---|---|---|---|
| sf-code/storage | **58** | 2 | +56 (the 56 slice-files of one continuous conversation) |
| seatloom | **16** | 14 | +2 |
| (root) | **13** | 8 | +5 |
| fde/field_evaluation | 10 | 3 | +7 |
| ai-assistant/android-agent | 4 | 2 | +2 |
| sf-code | 3 | 1 | +2 |
| SteptronOss | 13 | 12 | +1 |
| sf_model_delivery | 3 | 2 | +1 |
| Assistant_Aegis | 2 | 1 | +1 |
| ThoughtOnly | 2 | 1 | +1 |

This is exactly the failure Aegis predicted in the nudge: the operator opening the UI would see
**58 sessions under `sf-code/storage`** (the 56 re-launches of one conversation plus 2 real ones)
instead of the 2 the source defines, and **16 under `seatloom`** instead of 14. The operator's
acceptance card (§6 of the contract) fails at step 1 (183 total → 261), step 4 (seatloom 14 → 16)
and step 6 (root group 8 → 13); step 2 (OfficeBench 49) passes only by luck — no surviving
inventory row carries the exact `OfficeBench` slug.

Secondary defect, same root cause: `dated = first_ts.is_some() || session_date.is_some()`
(`crates/seatloom-mcp-emit/src/history.rs`) flags inventory rows that carry a manifest
`session_date` as `dated: true` while the SQL sorts them into the undated block — measured on
`seatloom`: a gemini marker row with `firstTs: null, dated: true`.

What the read path gets RIGHT and worth keeping: the honest refusal on a database without `021`
(measured against live: *"the historical corpus is not loaded … a zero here would be a false
statement about a corpus that has not been counted"*), pointers-never-bodies, undated-last for
corpus rows, the labelled `(root)` group, one implementation exposed twice.

## Verdict: FAIL — and the exact gap

**Judged against the live database, as Aegis requires: D1 ❌ (271 unmerged rows vs 183), D2 ❌ (1
projects row vs 38), D3 ❌ in the DB (no message_count column; the source sum of 36,680 is real but
the product cannot report it).** The operator's card cannot pass today: the read path refuses on
live because the corpus is not loaded, and the projects tree does not exist.

What my verification ALSO establishes, so the fix is not guesswork: the **data slice's artifact is
correct** — `021` at `33f456b`, applied to a faithful reproduction of the live shape, produces
exactly 183 / 38 / 36,680, row-by-row against the source, idempotent with zero writes, zero
session/seat writes, no bodies, root unmapped. And the **read path has an independent defect that
would survive the apply**: it double-counts the 89-row inventory (261 sessions instead of 183;
10 of 38 projects wrong — `sf-code/storage` 58 vs 2, `seatloom` 16 vs 14, `(root)` 13 vs 8),
because the two project-grouped reads lack the `message_count IS NOT NULL` filter that `021`'s own
header mandates. So there are two independent failures, either of which alone fails Slice 4:

1. **The corpus is not in the live database.** `021` lives on `packet/onyx/topic-opened-019`
   (`33f456b`); the live apply is the operator's `bash scripts/apply-schema-to-live.sh` and has not
   been run. Live reads 271 / 1 / no message counts.
2. **The read path double-counts the 89-row inventory.** Even after the apply, every
   project-grouped session count is wrong where an inventory row shares a project slug (261 total;
   10 of 38 projects). The fix is the filter `021`'s own header mandates: `message_count IS NOT
   NULL` in `list_history_projects` and `list_history_sessions`, plus dropping the `session_date`
   fallback in the `dated` flag.

Required to reach PASS (in order): (a) Nimbus adds the `message_count IS NOT NULL` filter to the
two project-grouped reads and fixes the `dated` flag; (b) the six numbers are re-measured against a
**live-shaped** scratch DB (271-row inventory + 021), not a fresh one — the proof script must be
extended to reproduce the manifest load, or it will keep passing while the product is wrong;
(c) the operator runs `apply-schema-to-live.sh` and the counts are confirmed on the live DB — D1
183, D2 38, D3 36,680 measured there, with the by-session headers as the independent check;
(d) only then does `ev-history-sessions-read-live` become emittable and Mira's UI slice
unblockable. No merge, no integration, no restart, no deploy was performed by Flux; no product
code was changed.
