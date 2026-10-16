# Product contract — joining one historical session to the work it produced, and why the key already exists twice

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-16-session-work-bridge-contract-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-16 |
| version | v1 |
| to | onyx, nimbus, vela, mira, flux |
| depends_on | LYRA-2026-10-16-next-round-plan-v1.md, docs/coordination/reviews/2026-09-28-aegis-multi-harness-session-ingestion-architecture-v1.md, docs/coordination/reviews/2026-09-30-aegis-object-model-and-traceability-v1.md, infra/postgres/schema/011_agent_history_archive.sql, infra/postgres/schema/015_knowledge_layer.sql, infra/postgres/schema/018_memory_hardening.sql, infra/postgres/schema/021_history_sessions_corpus.sql, tools/wash/README.md |
| tags | product-contract, session-work-bridge, attribution, provenance, decomposition |

## 1. The judgment call: should this key be produced by the washing pipeline?

**It already is — twice — and that is the finding that reshapes this contract.**

I expected to design a key. What I found instead is that the washing pipeline has been writing
one since 2026-10-11, and that a migration was written *specifically to make it resolve*:

| What | Where | Measured |
|---|---|---|
| 90 of the 151 live memories carry `source_kind='archived_session'` with `source_ref` = `'<harness>:<native session id>'` | `query_ledger`, `MemoryCreated` payloads | 84 `claude-code`, 6 `codex`, across **10 distinct sessions** (30/15/12/9/8/7/3/3/2/1 records) |
| `archived_sessions` is `PRIMARY KEY (harness, id)` and 021 re-keyed `id` to the native session id **for exactly this reason** | `021_history_sessions_corpus.sql`, header | the migration says so in as many words: *"the memory layer already writes source_ref as '<harness>:<native id>' … 3 of which resolve against NO row today because the live ids are file names"* — and it checked `pg_constraint`/`pg_views`/`pg_trigger` for any FK before re-keying |
| The migration's own match key is `(harness, raw_path)` | `021` §3 | *"The match key is (harness, raw_path) — stable across the re-key"* |

So the honest answer to Aegis's question is **not** "yes, the pipeline should produce it" and
**not** "no, it should not". It is:

> **The bridge does not need to be built. It needs to be pinned, made measurable, and stopped
> from being quietly broken again — and the ledger half of it must be declared empty rather
> than filled.**

Three consequences, each measured, that decide the whole contract:

**1. The key already broke once, silently, and nothing detected it.** Before 021's re-key, 3 of
6 codex `source_ref` values resolved against no row at all. Nothing in the product reported
that. A memory with an unresolvable `source_ref` is an orphan: the session it came from looks
like it produced nothing. The contract's load-bearing requirement is therefore not a new key
but a **drift measurement** — a number the product can show, so the next re-key cannot break the
join in silence again.

**2. The ledger half is empty by construction, not by accident.** Measured: the earliest event
in the entire ledger is `2026-10-09T16:53:28 SeatDispatchIssued`, and **zero of ~600 events
carry a `session` ref**. The corpus's sessions end 2026-10-08 at the latest. The capture spine
did not exist when any of the 183 sessions ran. So "the work this session produced, as recorded
in the ledger" is an empty set for every historical session — and the product must say *why*,
with the reason, rather than showing an empty list that reads as "nothing happened".

**3. Therefore the worst possible outcome is a contract that promises "link the session to its
work" without saying which work.** Someone will reasonably read that as "put events in the
ledger for these sessions", i.e. re-extract 183 transcripts into `canonical_events`. That would
fill the ledger with events nobody captured, wearing the costume of captured ones, in the one
product whose entire thesis is *"the record, not the harness, holds the truth."* This contract
forbids it explicitly (§5, red line 1).

---

## 2. Verified current state

| # | Claim | Measurement | Verdict |
|---|---|---|---|
| B1 | The washing pipeline writes a session key | 90/151 memories carry `source_kind='archived_session'`, `source_ref='<harness>:<native id>'` | ✅ it exists |
| B2 | The key resolves against `archived_sessions` | 021 re-keyed `id` to the native session id so it would; no measurement has been taken since | ⚠️ **unmeasured** — this contract requires it measured, not assumed |
| B3 | *(not in the brief)* | **`memories.l2_file_path` is a second, stronger key.** `s5_writein.py` normalises it to the `~`-rooted corpus path; `archived_sessions.raw_path` is that same path relative to the AgentHistory root. Both sides declare the path; neither derives it. | new finding — see §3 |
| B4 | *(not in the brief)* | **The topic ids are derived by truncation and must never be used as keys.** `wash-sess-cc-621ae382` is `sid[:8]`; `wash-seg-cc-01565715-t000` likewise. `tools/wash/s5_writein.py:296-309` builds them by string formatting. | new finding — §4 anti-pattern 1 |
| B5 | The ledger holds nothing for a historical session | earliest event 2026-10-09T16:53:28; 0 events with a `session` ref | ✅ empty, and §5 explains why that is correct |
| B6 | The washing pipeline's coverage | 16 manifest sessions (operator-fixed scope, seatloom project only); 10 of them produced memories | ⚠️ 173 of 183 corpus sessions have no distilled records at all |
| B7 | `remember`'s own contract for this field | the tool's description: *"source_ref: Polymorphic provenance: '<harness>:<id>' for archived_session … **No FK — recorded, never guessed**"* | the design already says this is a recorded provenance string, not a foreign key |
| B8 | *(not in the brief)* | `archived_sessions.sha256` is populated for file-backed rows and indexed (`idx_archived_sessions_sha`). A memory's `l2_body_digest` is a **line-range** digest, not a file digest — so sha256 does **not** join a memory to a session. | recorded so nobody tries it |
| B9 | Attribution design authority | `2026-09-28-aegis-multi-harness-session-ingestion-architecture-v1.md` §2: the cross-harness join is `(project cwd, wall-clock window, git branch)`, *"approximate — the design must treat attribution as **derived and revisable**, never as ground truth baked into the object at ingest time"*, stored *"as a separate, revisable table with a confidence field, never merged destructively"* | the authority for §4 Tier B |

---

## 3. The key: three candidates, two recorded, one that must never be used

Tier A keys are **recorded facts**: someone wrote them at a known time, and both sides can be
re-read and compared. Tier B is **derived attribution**: useful, approximate, and revisable.
The distinction is the whole contract, because a Tier B guess presented as a Tier A fact is
exactly the failure this project has already paid for.

### Tier A — recorded keys (facts)

**A1 (primary). The declared path pair.**

```
memories.l2_file_path   '~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-….jsonl'
        strip the corpus-root prefix  ↓
archived_sessions.raw_path           'raw/claude-code/b0100cff-….jsonl'
```

An exact string match after **one** documented normalization. Why this is primary:

- **Both sides declare the path; neither derives it.** `s5_writein.py`'s own comment records why
  it normalises rather than leaving the bare relative form: *"a bare `raw/...` path would look
  repo-relative while resolving to nothing here, i.e. an unresolvable pointer — which red line 2
  forbids."* The migration and the write-in were both written against the same declared path.
- **It is immune to id re-keying.** This is the decisive property. A1's sibling key broke once
  because 165 rows carried the raw *file name* as the id; 021 had to re-key 182 rows to repair
  it. A path does not change when an id convention changes. `021` §3 already uses
  `(harness, raw_path)` as its own match key for precisely this reason, and says so.
- It needs no parsing: no splitting, no truncation, no prefix arithmetic beyond one strip.

**A2 (secondary). `memories.source_ref` ↔ `archived_sessions (harness, id)`.**

Split `source_ref` on the **first** colon: `harness` = before, `id` = after. Why secondary and
not primary: it is exact and already written by 90 memories, but it is a free-text column with
no constraint, and it has already failed once. It must therefore be **measured on every read**,
not assumed — and the measurement is part of the product, not a one-off script.

**A3 (never a key). The topic id.** `wash-sess-cc-621ae382` and `wash-seg-cc-01565715-t000` are
built by `f"wash-sess-{ab}-{short}"` with `short = sid[:8]`. Truncating to 8 hex characters is a
collision risk, and a topic is a *segmentation* — a revisable label — not an identity. Joining a
topic to a session by parsing its id is a guess. §4 anti-pattern 1.

### Tier B — derived attribution (revisable, confidence-carrying)

Everything that would answer "what did this session *do*" beyond its own distilled records:
the git commits made while it ran, the workitem it belonged to, the seat that ran it. The
authority is the 2026-09-28 review's `(cwd, wall-clock window, git branch)` triple, which is
explicitly **approximate** — many sessions share a repo and overlap in time. Rules:

- Stored **separately** from the session row, never merged into it.
- Carries `method`, `confidence`, and `evidence` — always shows its working.
- **Revisable**: an override is a new row, never an edit of the old one.
- **Never promoted to Tier A by familiarity.** A Tier B link that has been right nine times is
  still a Tier B link, and the UI must still label it as derived.

### What must be answered "no record", with the reason

| Question | Honest answer today | Why it is not a gap to be closed by guessing |
|---|---|---|
| What ledger events did this session produce? | **None — the capture spine did not exist when it ran.** The earliest ledger event is 2026-10-09; every corpus session predates it. | The record is absent because nothing was recording. Filling it would be fabrication, not capture. |
| What distilled records came out of it? | The 90 memories whose `source_ref`/`l2_file_path` resolve to it — or **"none"**, which is a different fact from "unresolvable". | A session with no distilled records is a real state (the washing pipeline's scope is 16 sessions; most sessions contribute no qualifying record). |
| What commits did it produce? | **Not recorded.** Tier B derivation is a later slice with its own contract. | Time-window matching across sessions in one repo collides. |
| Which seat ran it? | Not recorded. The corpus predates `external_ref` (G4, step 3) and seats. | — |

---

## 4. Who writes the key, when, and what happens when it is written wrong

### 4.1 The producers, and the one spelling they must share

The key is written by **two producers at two different times**, and the contract's job is to pin
one spelling between them:

| Side | Written by | When | Spelling |
|---|---|---|---|
| The session | migration `021` (§3 UPDATE + §4 INSERT) | at corpus load | `raw_path` = path relative to the AgentHistory root; `id` = the harness's native session id (021 re-keyed 182 rows to this) |
| The memory | `tools/wash/s5_writein.py` → `remember` | at write-in | `source_ref` = `'<harness>:<native id>'`; `l2_file_path` = the `~`-rooted corpus path |

**Not at ingest, not backfilled, not both-ways-written.** The session side is written once by the
migration; the memory side is written once by the write-in; and the two agree because 021 was
written *knowing* what the write-in had already written. That is the whole mechanism, and it is
already correct. What is missing is that nobody has written it down as *the* mechanism, so the
next person to touch either side has no way to know they are coupled.

### 4.2 When it is written wrong

`source_ref` is free text with no FK and no CHECK (B7: *"No FK — recorded, never guessed"*). A
misspelled harness or a stale id produces an unresolvable reference, and the failure is silent:
the session looks like it produced nothing. Four requirements:

1. **Validate exhaustively at write-in, never by sample.** `s5_writein.py` already does this for
   L2 digests — its README records that a 5-record random sample reported 3 bad anchors as
   clean, and the driver now checks *every* pointer. The session key gets the same treatment:
   every record in the write set must carry a `source_ref` **and** an `l2_file_path` that both
   resolve, or the manifest is refused.
2. **Measure drift continuously, and show the number.** A read that reports *"N memories carry a
   session reference that resolves against no row"* turns the next silent break into a visible
   one. This is the single most important requirement in the contract.
3. **The UI must distinguish three states, because the operator's next action differs:**
   *no distilled records* (the session genuinely produced none — nothing to do),
   *records exist but do not resolve* (something is broken — report it), and
   *records exist and resolve* (show them). Collapsing the first two into one empty list is how
   the last break went unnoticed.
4. **Repair by forget + remember, never by editing the row.** `015` already fixes this rule for
   content: *"a content correction is forget + remember, not an in-place edit (no revision
   machinery)"* and rows are never hard-deleted. A wrong `source_ref` is repaired the same way —
   a tombstone stays, the corrected memory is a new row, and the ledger shows both. An in-place
   UPDATE of `source_ref` would rewrite history with no event.

---

## 5. Red lines

1. **No event is ever written for a historical session.** No backfill, no re-extraction into
   `canonical_events`, no "reconstructing" what a session did from its transcript. The 2026-10-14
   contract already excludes re-running the extraction; this extends it to the ledger. A
   fabricated event is worse than a missing one.
2. **No joining by derived filename or by topic id.** The history read module's rule stands and
   is quoted here because it is the same rule: *"A derived filename would be a guess; an index
   keyed on a declared path is a fact."*
3. **No Tier B link is ever presented as a recorded one.** Derived attribution is labelled as
   derived, carries its evidence, and stays revisable.
4. **No transcript bodies in the database.** 011's rule, unchanged. The bridge carries pointers
   and line ranges only.
5. **No touching the live database, no gate events, no merging** — Aegis's boundaries for this
   dispatch, restated so no slice has to guess.

---

## 6. Layer 1 — User story

**Who:** Mr. Zhang, using SeatLoom, looking at one of his own past sessions.
**What he wants:** to know what came out of that session — not to re-read it, but to see what it
produced and be able to trust that answer.
**How he is stuck today:** the session is readable (Mira's screens) and its distilled records are
somewhere in the knowledge panel, but nothing connects them. The join exists in the data and is
invisible in the product. He cannot tell "this session produced nothing" from "the link is
broken", and he has no way to know which.

## 7. Layer 2 — Interaction flow

### 7.1 Main path
Open the session → see, on the same screen: the distilled records that came out of it, each with
its kind, its confidence, and the line range it came from in the original file → click a record
→ see its full text and the exact lines it cites → the line numbers are checkable against the
original file on disk.

### 7.2 Every empty and missing state, with its reason
- **A session with no distilled records.** Say so, and say why in one line: this session was
  outside the washing scope, or it contributed no qualifying record. Do not render an empty list
  that reads as "nothing happened".
- **A session whose records do not resolve.** Say so, name the count, and name what is missing
  (the session reference or the file pointer). This state must be visually distinct from the one
  above.
- **A record whose source file is no longer on disk.** Show the record with its line reference
  and say the file is missing. Do not hide the record.
- **A read that fails.** Show the measured error text and name what is missing. No
  forever-spinner, no silent empty list.
- **The corpus not loaded.** Say the history corpus is not loaded yet, and name the next
  step. The exact wording belongs to the operator-facing card.

### 7.3 What this flow deliberately does not do
It does not show ledger events for a historical session (§5 red line 1). It does not show git
commits (Tier B, later slice). It does not nest or re-segment anything.

## 8. Layer 3 — Screen definition

One addition to the **existing** session detail screen Mira is building — a section, not a new
surface, not a re-layout:

- A header line stating the join's measured health for this session: how many records resolve,
  how many do not.
- The record list: title, kind, confidence, and the line range rendered as "source file
  lines N–M". The exact wording belongs to the operator-facing card.
- Each record opens to its full text with the cited lines shown, and the source file path shown
  and openable.
- Every §7.2 state rendered with its reason.

## 9. Acceptance criteria

The operator-facing acceptance card is a **separate artifact**, written to the plain-language rule
(all Chinese, no self-invented abbreviations, every step says what to do → what to see → what
passes → what gets rejected): `docs/coordination/acceptance/2026-10-16-session-work-bridge-acceptance-v1.zhs.md`.
It is kept apart because the operator has rejected a card once for being unreadable, and because a
card buried inside a design contract is a card nobody runs.

What that card must cover, in English, for whoever implements it:

| # | Step | What passes | What gets rejected |
|---|---|---|---|
| 1 | Open a session that **has** distilled records (`claude-code`/`b0100cff` has 30) | a health line stating N records, M of which resolve; a non-empty list; every row carries kind, confidence and a line range | an empty list with no explanation; an empty line-range column; "no records" without saying which kind of none |
| 2 | Click any record | full text, the cited line range, and the source file path — and the path opens the real file | a summary with no full text; a line range that does not match the original file; a path that does not open |
| 3 | Open a session with **no** distilled records (most of the 183) | one plain sentence saying there are none, and separating the two reasons: outside the washing scope, or washed but contributed no qualifying record | an unexplained empty list; the two reasons merged into one "no data" |
| 4 | **Load-bearing.** Produce a "records exist but do not resolve" case | the screen states N records do not resolve and names what is missing (the session reference, or the file pointer) — and this state looks **different** from step 3 | the two states look alike. This project has already paid once for "looks right, is actually a guess" |
| 5 | Ask the explicit question: does this session have ledger events? | one plain sentence saying no, with the reason — the capture spine did not exist when it ran (earliest ledger event 2026-10-09; latest corpus session 2026-10-08) | an unexplained empty event list; worse, any event generated for a historical session |

## 10. Explicitly out of scope, with reasons

- **Writing ledger events for historical sessions.** Red line 1. It is the one action that would
  corrupt the product's core claim.
- **Widening the washing pipeline's scope beyond the 16 seatloom sessions.** Operator-fixed
  since 2026-10-10; expanding it is a separate decision, and this contract must not assume it.
- **Tier B attribution (commits, workitems, seats).** Needs its own contract, its own confidence
  model, and its own override surface. Named here so it is not silently folded in.
- **Nesting the 38 projects, re-segmenting topics, or touching the topic tree.**
- **Any re-layout of the interface skeleton.** Forbidden by the operator.
- **Touching the live database, emitting gate events, merging.**

## 11. Decomposition into dispatchable slices

Dependency edges are **event ids**, never prose. `blocked` is not an error and not a stall.

| # | Slice | Seat | Dispatch | Edges (all `dep_kind=event`) |
|---|---|---|---|---|
| 0 | This contract | Lyra | `disp-WUh_tJ0EHV-DKd7U_te1l` | — → emits `ev-session-work-bridge-contract-landed` |
| 1 | Make the join executable and measurable (SQL) | Onyx | **`disp-session-key-view`** (new) | `ev-session-work-bridge-contract-landed` → blocked |
| 2 | The read model, two thin surfaces | Nimbus | **`disp-session-key-readmodel`** (new) | `ev-session-key-view-landed` → blocked |
| 3 | Pin the producer side: exhaustive validation at write-in | Vela | **`disp-session-key-writein-guard`** (new) | none → ready |
| 4 | The session detail's distilled-records section | Mira | **`disp-session-key-ui`** (new) | `ev-session-key-readmodel-live` **and** `ev-history-sessions-read-live` → blocked |
| 5 | Verify the join adversarially | Flux | **`disp-session-key-verify`** (new) | `ev-session-key-readmodel-landed` → blocked; pre-declared verdict `ev-verdict-session-key` |

**Why these edges.** Slice 2 reads through the view slice 1 creates, so it waits for the
announcement. Slice 4 *calls* the capability, so it waits for the measured `-live` gate, not the
announcement — the same rule Mira's own corpus slice 3 was issued under — and it also waits for
the corpus read gate, because its screen does not exist without the corpus screens. Slice 3 has
no edge because it hardens a producer that already exists and runs today; making it wait would
leave a silent break in place for no reason.

**Why slice 4 carries no edge on `disp-history-sessions-ui`.** Mira holds exactly one in-flight
dispatch — a database fact, not a convention — and a dispatch row *exists* the moment it is
issued, so an edge on a dispatch id would read as satisfied immediately. The payload instead
requires her to measure the tree at pull time.

### 11.1 The event contract — exact ids and required payloads

| Event id | Emitter | Type | Required payload |
|---|---|---|---|
| `ev-session-work-bridge-contract-landed` | Lyra | `ArtifactCreated` | `contract`: this file's path + commit. Announcement; no `verified` field. |
| `ev-session-key-view-landed` | Onyx | `ArtifactCreated` | `branch`, `commit`, `files`, and the exact view definition. Announcement; no `verified` field. |
| `ev-session-key-readmodel-landed` | Nimbus | `ArtifactCreated` | `branch`, `commit`, `files`, the exact shape of both read functions, and **the measured drift number**. Announcement; no `verified` field. |
| `ev-session-key-readmodel-live` | Nimbus | `ArtifactCreated` | **`verified`** (mandatory): which call, exactly what it returned, on which database, when. A gate; only after a real call succeeded against the live database. |
| `ev-verdict-session-key` | Flux | `ReviewVerdictIssued` | Pre-declared at enqueue time; Flux must emit exactly this id. |

### 11.2 Done definition, per slice

**Slice 1 (Onyx) — make the join executable in SQL and measurable.**
- One **read-only view** (not a table, not materialized, zero rows inserted) that joins both Tier A
  keys: a memory's `l2_file_path` with the corpus-root prefix stripped against
  `archived_sessions.raw_path`, and a memory's `source_ref` split on the first colon against
  `archived_sessions (harness, id)`.
- The view must answer one number: **how many memories carry a session reference that resolves
  against no row.** This number is the load-bearing wall — before 021, 3 codex references resolved
  against nothing and nothing reported it, so a session looked like it had produced nothing.
- Do not change the behaviour of any existing column on `memories` or `archived_sessions`; add the
  view, touch no data.
- Applied end-to-end on a scratch database, and a second apply writes zero rows, before it counts
  as landed. (This rule was bought with a real defect in 019.)

**Slice 2 (Nimbus) — the read model, one function two surfaces.**
- One read-only function: given a historical session, return its distilled records (title, kind,
  confidence, line range, source path) **and report three separate numbers**: how many resolve, how
  many do not, and what exactly is missing for the ones that do not.
- ONE implementation, TWO thin surfaces over the same code: an MCP read tool and a Tauri command.
  No second implementation.
- The reverse direction too: given a memory, return the session it came from — by key, never by
  topic id.
- **Never use a topic id as a key.** `wash-sess-cc-621ae382` is a session id truncated to 8
  characters and formatted into a string. That is a guess.
- Reads return pointers and line ranges only, never transcript bodies (011's red line).
- Emit the announcement carrying the measured drift number first; only after a real call succeeds
  against the live database emit `-live`.

**Slice 3 (Vela) — weld the producer side shut.**
- S5 already validates every L2 digest exhaustively rather than by sample — its README records why
  (a 5-record sample reported 3 bad anchors as clean). The session key gets the same treatment:
  **every record in the write set must carry a `source_ref` and an `l2_file_path` that both resolve
  against `archived_sessions`, or the manifest is refused.** No sampling.
- Pin both spellings in the README (A1 and A2 of §3), and record why A1 is primary: it does not
  break when ids are re-keyed, and A2 already broke once.
- Do not modify the 90 memory rows already written. A correction is forget + remember (015's rule);
  the old row stays as a tombstone.
- Do not widen the washing scope to give more sessions records. That is the operator's decision,
  not this dispatch's.

**Slice 4 (Mira) — one more section in the session detail.**
- Mounted inside the existing session detail as **one added section** — not a new window, not a new
  route, no re-layout (his explicit instruction).
- Blocked on slice 2's measured `-live` gate: you call this capability, so an announcement is not
  enough.
- A health line at the top: how many records resolve, how many do not. The three states (no records
  / records that do not resolve / records that resolve) must **look different** — he has to be able
  to tell "genuinely none" from "some, but broken" at a glance.
- Each record opens to its full text and the lines it cites; the source file path is shown and
  opens.
- The acceptance card is written to the plain-language rule: all Chinese, no self-invented
  abbreviations, every step says what to do → what to see → what passes → what gets rejected.
- The interface skeleton is not re-laid-out. Checks are file-granular and the baseline is already
  red: deliver a MEASURED zero-new-issues result.

**Slice 5 (Flux) — verify the join adversarially.**
- Attack "looks right, is actually a guess" specifically. At least: a memory with a misspelled
  `source_ref` (must report non-resolution and name what is missing); a session with no distilled
  records (must say "no records", not "broken"); a session whose source file is no longer on disk;
  an attempt to look a session up by topic id (must be refused, or explicitly labelled derived); an
  empty topic.
- Report the exact input and exact output for every case. An implementation that usually refuses is
  a FAIL.
- Verify one thing above all: **"no records" and "records that do not resolve" must be
  distinguishable in the output.** They look identical in the ledger; they must not look identical
  in the product.
- The verdict must be emitted with exactly the pre-declared id `ev-verdict-session-key`.

---
