---
template: T5
subtype: product_contract
doc_id: LYRA-2026-10-14-history-sessions-in-ui-contract-v1
title: Product contract — historical sessions in the SeatLoom UI, organized by project (38 projects / 183 sessions)
status: active
author: lyra
doc_date: 2026-10-14
version: v1
depends_on: AEGIS-2026-10-14-history-sessions-in-ui-contract.md (dispatch payload), docs/prd-v0.5.md, infra/postgres/schema/011_agent_history_archive.sql, infra/postgres/schema/015_knowledge_layer.sql
tags: product-contract, history-sessions, archive, projects, decomposition
---

# Product contract — historical sessions in the SeatLoom UI, by project

**Owner:** Lyra (PO). **Purpose:** satisfy PRD v0.5's two hard rules before implementation:
a feature may not enter implementation unless it exists in all three layers (user story,
interaction flow, screen/UI surface definition), and may not stay in scope unless the pain,
the cost of omission, and the behavior change are written down.

The referenced packet path in the dispatch payload
(`docs/coordination/tasks/aegis/AEGIS-2026-10-14-history-sessions-in-ui-contract.md`) does not
exist in any worktree or on any branch as of this writing; the contract was written from the
dispatch payload plus the measurements in §1.

---

## 1. Verified current state (measured, not reported)

Everything below was measured from `resume-contexts/` on `track/infra-foundation` before this
contract was written. **One number in the brief is wrong** and is corrected here; it is
flagged in §8 because the operator has already corrected a count in this dataset once.

| # | Claim in the brief | Measurement | Verdict |
|---|---|---|---|
| D1 | 183 sessions | `SESSIONS-FLAT.csv` has 184 lines = 183 data rows; `by-session/` holds 183 files | ✅ |
| D2 | 38 projects (home paths) | 38 distinct values in the `home` column | ✅ |
| D3 | 36,680 messages | Sum of `消息数` across all 183 `by-session/*.md` headers = 36,680 | ✅ |
| D4 | codex 150 | 150 rows | ✅ |
| D5 | claude-code 18 | 18 rows | ✅ |
| D6 | **cursor 13** | **14 rows.** `CURSOR-HOMES.csv` has 13 *directories*; one directory (`private_knowledge_base__fbb8b86a…`, home `…/private_knowledge_base/step-product/mind/code/brainpp-main`) contains **2** sessions — the CSV rows are literally labelled `(1/2) 12-02→12-03 quotagroup 前段` and `(2/2) 12-16 起 quotagroup 后段(隔 13 天)`. The brief's own parenthetical ("其中 1 个目录含 2 条会话") contradicts its own count. **183 = 150 + 18 + 14 + 1.** | ❌ corrected to 14 |
| D7 | stepcode 1 | 1 row, `wrapped=claude:104f1244-…` | ✅ |
| D8 | gemini 0 | 0 rows | ✅ |
| D9 | OfficeBench's 49 sessions are one ops line sliced into many short command sessions | 49 rows have home `/Users/jyxc-dz-0100609/Documents/GitHub/OfficeBench`, all codex, all `slices=1`; message counts are 3–20 with the mode at 4–5 | ✅ explanation confirmed |
| D10 | 70 codex sub-agents not counted separately | Consistent with `CODEX-PARENT-CHILD.csv` / `by-root-session/` (151 files vs 150 codex roots) | ✅ |
| D11 | *(not in the brief)* | **All 14 cursor sessions have empty `first_ts` and `last_ts`.** Cursor's `chat_dump.json` carries no timestamps. A "when" column cannot be filled for them from the source. | ⚠️ new finding |
| D12 | *(not in the brief)* | **The bare home `/Users/jyxc-dz-0100609/Documents/GitHub` holds 8 codex sessions and is a path-prefix ancestor of 37 of the other 37 home paths.** Four more paths are also ancestors of others: `…/OfficeBench` (1 child), `…/ai-insight` (2), `…/private_knowledge_base` (3), `…/sf-code` (7). The 38 home paths are **not a clean two-level tree**. | ⚠️ new finding |

### 1.1 The schema already has a designed home for this data

This is the most consequential finding, and it changes the recommended model.

- **`archived_sessions`** (migration `011_agent_history_archive.sql`) exists and was designed
  for exactly this corpus: harness-agnostic
  (`codex | claude-code | stepcode | cursor | gemini | opencode`), `PRIMARY KEY (harness, id)`,
  free-text `project` slug, `cwd`, `session_date`, `bytes`, `lines`, `sha256`, `raw_path`,
  `src_path`. Its header states the design intent explicitly: **"NO FK to seats (archived
  sessions have no seat) and NO FK to projects (the corpus spans ~21 projects not in the
  projects table)"**, and **"Never store transcript bodies (may contain secrets)"**.
- **`v_archived_sessions_normalized`** (migration `015_knowledge_layer.sql`) already
  implements an executable slug→tree normalization with rules R1–R5, including **R4: unknown
  slug → unmapped, reported, never guessed** and **R5: `(root)` → unmapped + ESCALATED to
  Aegis (13 sessions; strategy decision requested, not assumed)**.
- **`projects.parent_project_id`** was added by 015, so a tree is expressible.
- **`archived_sessions` currently holds zero rows** — no seed writes it and no backfill script
  inserts into it. It is a designed, empty home.
- By contrast, **`sessions`** requires `seat_id NOT NULL REFERENCES seats(id)` and (post-008)
  `project_id NOT NULL`. `seats` is referenced by **6 tables** — `project_role_bindings`,
  `seat_delegations`, `sessions`, `workitems`, `handoff_receipts`, `dispatches` — so seats is
  load-bearing for the live dispatch mechanism, not a scratch table.

### 1.2 The recommended data model, and why it is not the one in the brief

The brief proposes: 38 `projects` rows, 183 `sessions` rows whose `seat_id` points at one
archive seat, and no change to seat semantics.

**Recommendation: use `archived_sessions` for the corpus and `projects` for the tree. Do not
write 183 rows into `sessions`.**

| | Brief's model (`sessions`) | Recommended (`archived_sessions` + `projects`) |
|---|---|---|
| Fake seat needed | **Yes** — `seat_id` is `NOT NULL` | **No** — the table was designed with no seat FK |
| Blast radius | A new seat row is visible in the Seat Registry and in dispatch targeting (`dispatches.target_seat_id`); the waker maps by `.seatloom/dispatch-seats.tsv` and logs unmapped seats | None outside the new read path |
| Transcript bodies | `sessions` has no body column, so bodies would still live on disk — fine either way | Same, and 011 already mandates it |
| Slug normalization | Would have to be built | Already exists (`v_archived_sessions_normalized`), already refuses to guess |
| Existing UI read path | `cmd_list_sessions_for_project` works unchanged | Needs one new read command |

The cost of the recommendation is one new read command. The cost of the brief's model is a
synthetic seat in a table six other tables depend on, plus 183 rows in a table whose NOT NULL
columns force a fiction ("this session belongs to seat-archive-001") that the operator will
eventually see in a dropdown.

**This is a recommendation, not a ruling.** It is architecture-adjacent, so Aegis can
override it; the override and its reason should be recorded before the data slice starts,
because the two models produce different migrations.

### 1.3 The project identity question, stated plainly

Mr. Zhang's words: "project 是之前各 agent harness 启动的各 session 的 home path,每一个
homepath 下面有多个 session".

A project id/name must be derived from an absolute path. Two candidate derivations, both with
a measurable cost:

- **Basename** (`OfficeBench`, `storage`, `private_knowledge_base`) collides:
  `private_knowledge_base` is the basename of 4 distinct home paths, `OfficeBench` of 2.
- **Path with the `/Users/jyxc-dz-0100609/Documents/GitHub/` prefix stripped**
  (`OfficeBench`, `sf-code/storage`, `private_knowledge_base/step-product/brainpp`) is
  unique, and its `a/b` shape is exactly what `v_archived_sessions_normalized`'s R2 already
  handles.

**Recommendation: the stripped path.** Display name = the stripped path; the full absolute
path stays on the row so nothing is lost. The bare `/Users/…/GitHub` home strips to the empty
string, which is precisely the `(root)` case that R5 already escalates — so those 8 sessions
should be shown as an explicitly-labelled "未归类(仓库根目录)" group rather than silently
folded into a neighbouring project. **Guessing a parent for them is forbidden.**

---

## 2. Layer 1 — User story

**As** Mr. Zhang,
**I want to** open SeatLoom and see the projects I have actually worked on — one per home
path where an agent harness was started — and under each one the sessions that ran there,
with the conversation inside each session,
**so that** my own history is inside the product instead of in 183 files on disk that only a
script can read.

**Who is stuck today, and how.** The corpus exists and has been extracted: 183 sessions,
36,680 messages, every message carrying the line number it came from in the original file.
It is reachable only by reading `resume-contexts/` in a repository checkout. Nothing in the
product surfaces it. The operator asked for it on 2026-10-14 in his own words, and corrected
the description twice while asking — which is what happens when the thing you are describing
is not in front of you.

---

## 3. Layer 2 — Interaction flow

### 3.1 Main path

1. The user opens the project list and sees every project that has at least one session,
   with the session count and the total message count.
2. The user picks a project and sees its sessions: which harness, when it ran, how many
   messages, and how long it ran.
3. The user picks a session and reads the conversation, in order, with each turn showing the
   line number it came from in the original file on disk.
4. The user can go back to the project list, or to another project, at any point.

### 3.2 Empty and missing states — every one must say why, and what is next

| Condition | Required behavior |
|---|---|
| A project with no sessions | Show the project with 0 and state why (for example: this home path only ever launched a harness that produced no conversation, such as a gemini marker) and what the user can do next |
| A session whose harness produced no conversation (gemini: 0 rows by design) | Say so in one line; do not render an empty transcript frame |
| A session with no timestamps (all 14 cursor sessions) | Show "时间未记录" and explain that this harness does not record timestamps. **Do not invent a date, and do not sort them as if they were the oldest.** |
| The 8 sessions at the repository root | An explicit "未归类(仓库根目录)" group, with the reason stated. Not folded into a neighbour. |
| A message whose original file is no longer on disk | Show the turn with its line reference and state that the source file is missing, rather than hiding the turn |

### 3.3 Failure paths

- **The read command fails:** show the measured error text and name what is missing. Never a
  spinner forever, never a silent empty list.
- **A count disagrees with the source:** if the product reports a number that differs from
  `SESSIONS-FLAT.csv`, that is a defect, not a rounding difference. See §6.
- **The corpus is not loaded yet:** say "历史会话还没加载" and name the next step. Do not
  render an empty project list that looks like "you have no history" — that is the false
  statement this project has already paid for once.

### 3.4 What this flow deliberately does not do

- It does not guess a project for a session whose home path does not resolve.
- It does not merge two sessions into one because they look similar.
- It does not re-order messages by anything other than their order in the source file.
- It does not put transcript bodies in the database (011's rule; the bodies may contain
  secrets and the originals are already on disk).

---

## 4. Layer 3 — Screen definition

**Inherit the existing skeleton. No re-layout.** The operator has required this explicitly
and the interface checks are file-granular with an already-red baseline; the deliverable is a
measured zero-new-issues result.

| Surface | Definition |
|---|---|
| **Entry** | The existing project switcher and the existing `sessions` tab. No new top-level navigation. |
| **Project list** | A project row per home path that has sessions: display name (stripped path), session count, message count. Ordered by session count descending, matching `SESSIONS-FLAT.csv`'s natural order so the numbers are checkable. The repository-root group appears last, explicitly labelled. |
| **Session list (per project)** | One row per session: harness, date range (or "时间未记录"), message count. Sorted by start time descending, with the undated ones after the dated ones and labelled — never silently first or last. |
| **Session detail** | The conversation in source order. Each turn shows the original file's line number. The source file path is shown and is openable. Progressive disclosure is allowed but the default view must already be readable — no "click to reveal everything". |
| **Empty / missing states** | Exactly the table in §3.2. |
| **What is NOT built** | No new window, no new route, no search-over-all-projects in this slice, no editing, no deletion, no re-ingestion trigger. |

---

## 5. The three must-answers (PRD v0.5 rule 2)

1. **What pain it solves.** The operator's own history — 183 sessions and 36,680 messages
   across 38 projects, already extracted and line-anchored — is invisible inside the product.
   He has to ask an agent to read files for him, which is the same "pull me" loop the rest of
   this project keeps trying to remove. This slice makes his history a first-class object he
   can browse himself.
2. **What happens if it is omitted.** The knowledge layer's whole premise is that distilled
   memory is grounded in real sessions with line-level evidence. Without a way to see the
   sessions, the operator cannot check any of that grounding himself — he can only be told it
   is true. The corpus also stays un-queryable inside the product, so every future feature
   that wants "what did we do on project X" rebuilds its own access path.
3. **How user behavior changes.** The operator opens the product and browses his own history
   by project, instead of asking an agent to go read 183 files. The first thing he can verify
   without trusting anyone is whether the sessions shown are the sessions he actually had.

---

## 6. Acceptance criteria — in plain language, for the operator

Written to Aegis's plain-language rule (`disp-acceptance-cards-plain-language-v2`): every
sentence in Chinese, no English jargon, no self-invented abbreviations, no dispatch titles,
and each step says what to do → what to see → what passes → what gets rejected.

**这张卡验什么**: SeatLoom 里能按项目看到你自己的历史会话,一个项目就是一个你启动过 agent
的目录,目录下面是他真实开过的会话,点进去能读到当时的对话。

**怎么验(照着做)**:

1. 打开 SeatLoom,找到项目列表。**看**:应该出现一批项目,名字就是你以前启动 agent 的那些
   目录名,每一个后面标着"几个会话、多少条消息"。**算对**:项目数应该是 38 个;全部会话数
   应该是 183 个;全部消息数应该是 36680 条。**打回**:数字不对 —— 差一条都不行,这不是约数。
2. 找到会话最多的那个项目(应该是 `OfficeBench`,49 个会话)。**看**:点进去,应该列出 49 条
   会话,每一条能看出是用什么工具跑的、什么时候、有几条消息。**算对**:49 条,一条不多一条
   不少。**打回**:数量不对,或者列表是空的但项目名还在。
3. 点开其中任意一条 `OfficeBench` 会话。**看**:应该能读到当时的对话,一条一条按原来的顺序。
   **算对**:读得到内容,而且能看到每条内容出自原始文件的第几行。**打回**:内容是空的、顺序是乱的,
   或者看不到行号。
4. 找到 `seatloom` 这个项目(应该有 14 条会话)。**看**:点进去,这 14 条应该就是你在这个
   仓库里真实干过的活。**算对**:条数对得上,点开任意一条能读到当时的对话。**打回**:条数对不上,
   或者读不到内容。
5. **关键一步 —— 找一个用 cursor 工具跑的项目**(比如 `ThoughtOnly`)。**看**:那条会话的
   时间应该显示成"时间未记录",并且有一句说明:这个工具不记录时间。**算对**:说了没记录,
   没有编一个日期出来。**打回**:显示了一个具体日期 —— 那就是编的,这个工具根本没有时间戳。
6. **关键一步 —— 看列表最下面**。**看**:应该有一组明确标着"未归类(仓库根目录)"的会话,
   并且说明了为什么单独放。**算对**:有这一组,有说明。**打回**:这 8 条被并进了别的项目里,
   或者干脆不见了 —— 两种都是猜。
7. 找一个没有任何会话的项目(如果列表里有这样的项目)。**看**:它应该说清为什么是空的、
   下一步能做什么。**算对**:有说明。**打回**:只有一个空框,什么都不说。
8. 回到项目列表,随便换到另一个项目再换回来。**看**:次数和条数应该和第一次看到的一致。
   **算对**:两次一致。**打回**:数字变了。

**这张卡不验什么**: 不验界面好不好看,不验能不能搜索全部项目,不验能不能改或删会话
(这些这次都不做)。

---

## 7. Explicitly out of scope, with reasons

- **Search across all projects.** Useful, but not what was asked, and it needs its own
  contract. A second slice.
- **Editing, deleting, or re-ingesting sessions.** The corpus is a read-only archive; 011
  already forbids storing bodies, and mutation here has no requested use case.
- **Re-running the extraction.** The extraction script exists and has been run. This slice
  reads its output; it does not rebuild it.
- **Nesting the 38 projects into a tree.** `parent_project_id` exists and the normalization
  view already expresses R2, but the home paths are not a clean tree (D12) and any nesting
  rule is a guess about the operator's mental model. Flat, with the ancestor overlap shown
  honestly, until he asks for a tree.
- **Touching `sessions`, `seats`, or `dispatches`.** See §1.2.
- **Any re-layout of the existing interface skeleton.** Forbidden by the operator.

---

## 8. Decomposition into dispatchable slices

Dependency edges are **event ids**, never prose. `blocked` is not an error and not a stall.
Unlike the previous contract, Aegis has not pre-issued downstream dispatches here, so the
slices below are new.

| Slice | Seat | Dispatch | Edges (all `dep_kind=event`) |
|---|---|---|---|
| 0. This contract | Lyra | `disp-history-sessions-in-ui-contract` | — → emits `ev-history-sessions-in-ui-contract-landed` |
| 1. Data | Onyx | **`disp-history-sessions-data`** (new) | `ev-history-sessions-in-ui-contract-landed` |
| 2. Read path | Nimbus | **`disp-history-sessions-read`** (new) | `ev-history-sessions-data-landed` |
| 3. UI | Mira | **`disp-history-sessions-ui`** (new) | `ev-history-sessions-read-live` |
| 4. Verification | Flux | **`disp-history-sessions-verify`** (new) | `ev-history-sessions-data-landed` **and** `ev-history-sessions-read-landed` |

**Why these edges.** Slice 2 reads rows that do not exist yet, and a read path verified
against an empty table proves nothing — so it waits for the data. Slice 3 *calls* the read
path, so it needs the measured `-live` gate, not the announcement. Slice 4 verifies both the
data and the code, so it waits for both announcements.

### 8.1 The event contract — exact ids and required payloads

| Event id | Emitter | Type | Required payload |
|---|---|---|---|
| `ev-history-sessions-in-ui-contract-landed` | Lyra | `ArtifactCreated` | `contract`: this file's path + commit. Announcement; no `verified` field. |
| `ev-history-sessions-data-landed` | Onyx | `ArtifactCreated` | `branch`, `commit`, `files`, row counts. Announcement; no `verified` field. |
| `ev-history-sessions-read-landed` | Nimbus | `ArtifactCreated` | `branch`, `commit`, `files`. Announcement; no `verified` field. |
| `ev-history-sessions-read-live` | Nimbus | `ArtifactCreated` | **`verified`** (mandatory): which call was made, exactly what it returned, on which database, when. A gate; only after a real call succeeded against the live database. |
| `ev-verdict-history-sessions` | Flux | `ReviewVerdictIssued` | Pre-declared when slice 4 is enqueued; Flux must emit exactly this id. |

### 8.2 Acceptance criteria, per slice

**Slice 1 (Onyx) — the data.**
- 38 `projects` rows for the 38 home paths; display name = stripped path, full path retained.
- 183 `archived_sessions` rows, `PRIMARY KEY (harness, id)` respected; `harness` from the
  closed set; `cwd` = the absolute home path; `project` = the stripped-path slug;
  `raw_path`/`src_path` from the CSV; `sha256` where the source is a file (NULL for the
  cursor/gemini directory copies, as 011 specifies).
- A message count per session, sourced from `by-session/*.md` (sum must be 36,680). If a
  column is needed, it is a migration in Onyx's slot, additive and idempotent.
- **Zero row inserts into `sessions` and zero new rows into `seats`.** If the brief's model is
  taken instead, that is an Aegis override recorded before this slice starts.
- The 8 repository-root sessions must land as `(root)`-equivalent and must **not** be assigned
  a parent by guesswork.
- Idempotent: a second apply writes zero rows.
- Reconciliation query that proves the loaded counts equal the CSV's, pasted in the delivery.

**Slice 2 (Nimbus) — the read path.**
- One repository function, exposed twice over the **same** code: an MCP tool and a Tauri
  command (the established thin-wrapper pattern). No second implementation.
- Returns: projects with session + message counts; sessions for a project (harness, date
  range or explicit "not recorded", message count); a session's turns in source order with
  their original line numbers.
- Undated sessions are returned last and flagged, never silently interleaved.
- No transcript bodies in the database; the read returns pointers.
- Emits `ev-history-sessions-read-landed`, then `ev-history-sessions-read-live` after a real
  measured call.

**Slice 3 (Mira) — the UI.**
- Project list, per-project session list, session detail, per §4, inside the existing
  skeleton and the existing `sessions` tab.
- Every §3.2 state implemented with the required explanation.
- **Zero new interface-check issues, measured.** Files already red stay exactly as red.
- Blocked until the read path is live; built against the contract's shapes in the meantime if
  Mira chooses, but not wired to anything unmeasured.

**Slice 4 (Flux) — verification, commit-pinned.**
- §14 captures before starting; `HOLD` on mismatch or unauthorized dirt.
- **The counts are verified against the source, not against the code's own report**: 38
  projects, 183 sessions, 36,680 messages, 49 for `OfficeBench`, 14 for `seatloom`, 14 cursor
  sessions all undated. Any discrepancy is a fail, including off-by-one.
- The 8 root sessions are unmapped-and-labelled, not folded into a neighbour.
- No transcript body was written to the database.
- No row was inserted into `sessions`; no new seat exists.
- Gates run for real, output pasted.

**Slice 5 (the operator's card).** After slice 3, the acceptance card is requested with §6 as
its acceptance plan verbatim. The card merges only on the operator's Accept click; no seat
merges.

---

## 9. The two things I am flagging rather than deciding

1. **The cursor count.** The brief says 13; the data says 14, and the brief's own
   parenthetical explains why. The operator has already corrected a count in this dataset
   once ("183 条真实会话" → "183 个会话,每一个里面有很多条消息"), so I have used 14
   throughout and put the arithmetic in §6 so it is checkable at a glance:
   **150 + 18 + 14 + 1 = 183**.
2. **The data model.** §1.2 recommends `archived_sessions` + `projects` over the brief's
   `sessions`-with-an-archive-seat. This is architecture-adjacent and Aegis's to rule on; the
   two models produce different migrations, so the override (if any) must be recorded before
   slice 1 starts rather than discovered halfway through it.
