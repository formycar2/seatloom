# Flux verdict — the lineage read model (`lineage(ref)`, Nimbus `disp-lineage-readmodel`)

- **Dispatch**: `disp-lineage-verify` (issued by seat-lyra-001, priority from the plan's §3 slice 5).
  Edge `ev-lineage-readmodel-landed` measured **true** (the announcement exists in the ledger), which
  is the correct edge for a verifier — I verify the code, so the announcement is the right gate.
- **Verdict event**: `ev-verdict-lineage-readmodel` — **pre-declared id, emitted exactly as required**.
- **Verdict**: **FAIL** — narrow and precise. The central promise holds for every corpus session;
  two classes of *unmeasured assertion* and one *total-answer-loss* path do not.
- **Slice under test**: `36194e8` on `packet/nimbus/gate-vs-announcement` —
  `feat(lineage): the lineage read model — one read-only function, two thin surfaces, an honest
  'no record' for historical sessions`.
- **Contract**: `LYRA-2026-10-16-next-round-plan-v1` @ `b785071` on `packet/lyra/milestone-summary`.
  My acceptance criteria are the plan's **§3.2 slice 5** (the pre-declared attack list), read against
  slice 1's own acceptance criteria in §3.2.
- **Where I worked**: the slice is not on my branch (`packet/flux/intent-verdict` has no
  `lineage_ref.rs` — measured). I verified at Nimbus's pin, in Nimbus's worktree, **read-only**: I
  committed nothing there and changed no tracked file. All scratch work is on the disposable
  scratch database `seatloom_lineage_ref`; the live `seatloom` database was touched only by
  read-only `SELECT`s and by read-only `lineage` calls whose no-write property I measured.

---

## §14 captures (COORDINATION_RULES §14)

```
$ git -C <flux worktree> rev-parse --abbrev-ref HEAD  -> packet/flux/intent-verdict
$ git -C <flux worktree> rev-parse HEAD               -> 27a2b30 (docs(flux): verdict judged against live as Aegis requires)
$ git -C <flux worktree> status --short               -> ?? .scratch/*, .seatloom/rollout-ack.txt (pre-existing untracked; tracked tree clean)
$ git -C <nimbus worktree> rev-parse HEAD             -> 36194e8 (the slice under test)
$ git -C <nimbus worktree> status --short             -> ?? .scratch/*, .seatloom/rollout-ack.txt (pre-existing untracked; tracked tree clean)
```

---

## 0. The gates, run at the pin before any behavioural claim

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all -- --check` | **clean** |
| clippy | `cargo clippy --workspace --all-targets` | **clean, zero warnings** |
| test | `cargo test --workspace` | **262 passed, 0 failed** (10 of them `db::lineage_ref::tests::*`) |
| Nimbus's own proof | `proof/lineage_ref_proof.sh` on a scratch DB (canonical order + 015-019 + 021) | **ALL POINTS PASS** (all 7 points) |

Nimbus's proof is honest and it passes. Everything below is **outside** its seven points.

---

## 1. The thing this verification exists for — measured, and it PASSES

The dispatch named the failure mode to hunt: *a function that PRETENDS to know — a guessed join key,
a derived filename, or an empty list that reads as "nothing happened"*.

**Live, measured** (`DATABASE_URL` → the live `seatloom` DB, `SEATLOOM_HISTORY_CONTEXTS_DIR` → the
main checkout's `resume-contexts`; ledger snapshot before **and** after every call:
`1044 events / 2536 refs / 154 dispatches / 8 acceptances` — unchanged):

```
input : {"ref":{"kind":"history_session","id":"019cad2b-bc6e-7df0-b386-02137c512a38","harness":"codex"}}
isError: False
subject : history_session codex, 5 msgs, dated: True
noRecord: 这个会话在账本里没有任何记录。
facts   : []          produced: []
evidence: file /Users/…/AgentHistory/raw/codex/rollout-2026-03-02T14-10-50-019cad2b-….jsonl   (on disk: True)
          file /Users/…/resume-contexts/by-session/ai-insight__post-training__codex__019cad2b-….md  lines 13–63  (on disk: True)
notes   : ['被什么卡着为空:归档会话不在 dispatch_dependencies 里,它不依赖任何东西',
           '时间记录:起于 2026-03-02 06:11:01.392+00,终于 2026-03-02 06:15:30.723+00']
```

**PASS on the central test.**
- **No guessed join key.** The measurement is per call — `event_chain(db, &[("session", id)], None)`
  asks the ledger whether any event names `('session', <id>)`. When the answer is none, `no_record`
  carries `statement` / `reason` / `would_need`, and `would_need` names the exact missing row
  (`an event_object_ref row (ref_type='session', ref_id=…)`) and says inventing one would be a guess.
- **No derived filename.** `context_evidence` resolves the row's `raw_path` against the contexts
  *index* (`HistoryContexts::resolve_source_path`, both of whose strategies are verified against the
  index rather than configured), and when that fails it carries an `unresolved` pointer with the
  reason rather than a guessed path. 021's re-keying of 165 rows is respected: the id is taken from
  the referent, never reconstructed from the file name.
- **No empty list that reads as "nothing happened".** `facts == []` and `produced == []` occur
  **only** together with `noRecord != null`. The two states are structurally distinguishable.

**The parser is not "usually refusing"** (the plan's own FAIL criterion). Live, measured, all five
kinds resolve with real data: a dispatch (`disp-session-key-verify`, `blocked`), an event
(`ev-OzXRbRwZobSV0ACw9WCax`), an acceptance (`acc-disp-ol420Y586tNIsbbeh5nlU`, `accepted`), a
session (`ses-nimbus-infra-001`, `completed`), and the archived session above. Malformed input is
refused **by name** (unknown kind names the five-kind domain; empty id; a `history_session` with no
harness names the id it needs one for), never as an empty object.

**One implementation, two surfaces** (plan §3.2 slice 1, second criterion): verified in code.
`seatloom_core::db::lineage_ref::lineage` is the only assembler; `crates/seatloom-mcp-emit/src/lineage.rs`
is a thin shell over it; `src-tauri/src/commands/lineage_cmds.rs::cmd_lineage` shells out to the
same binary over stdio and adds the envelope and nothing else. No second assembler exists.

**Read only** (plan §3.2 slice 1, fifth criterion): measured, not asserted. Seven calls on scratch
(including four refusals) left `canonical_events` and `topics` counts unchanged; the gate ids
`ev-lineage-readmodel-landed` / `-live` were absent from the scratch ledger; and on live the
snapshot above is identical before and after my calls. No writes, no gate, no UI change.

---

## 2. The FAIL findings — exact input, exact output

### F1 — MAJOR, measured on LIVE: `history_session` refuses a row that EXISTS, with a remedy that is already done

```
input (live) : {"ref":{"kind":"history_session","id":"06ef2d08-ba8f-4439-bdd0-cacd018b324f","harness":"stepcode"}}
output (live): isError: True
  lineage rejected: the historical corpus is not loaded — archived_sessions.message_count does not
  exist. Apply the corpus migration (021) first; a zero here would be a false statement about a
  corpus that has not been counted.
```

Measured on live, in the same session:

```
the row itself    : 06ef2d08-ba8f-4439-bdd0-cacd018b324f harness=stepcode project=fde/field_evaluation message_count=NULL
021 applied?      : message_count column EXISTS on live  (SELECT … information_schema.columns → t)
corpus loaded?    : 183 corpus rows ARE loaded on live (measured independently: total=272,
                  corpus=183, inventory_only=89, sum(message_count)=36680)
rows in this state: 89 of 272 live archived_sessions rows  (SELECT count(*) … WHERE message_count IS NULL → 89)
ledger unchanged  : 1044 events / 2536 refs / 154 dispatches / 8 acceptances, before == after
```

So for **one row in three**, the answer tells the operator to apply a migration that is applied, and
asserts a schema fact (`archived_sessions.message_count does not exist`) that is false.

**Mechanism.** `history_session_lineage` (crates/seatloom-core/src/db/lineage_ref.rs) does:

```rust
if row.message_count.is_none() { return Err(LineageRefError::CorpusNotLoaded); }
```

`get_archived_session` (repositories.rs:937) returns `NULL` for `message_count` in **two** different
situations: the column is absent (021 not applied) **and** the row is one of the 89 the 2026-10-08
out-of-band manifest load left with no extracted conversation. 021's own header states the second
case plainly — "the live table also holds the 2026-10-08 manifest load — 271 rows, 89 of them with no
extracted conversation and therefore no `message_count`". The read collapses "this row has no
extracted conversation" into "the corpus migration is not applied", and the error text then makes a
claim about the database it never measured.

This is the same species as the failure mode the dispatch sent me to hunt, inverted: not a guess
about the *object*, but an unmeasured assertion about the *database*, shown to the operator as fact.

### F2 — MAJOR (latent): ONE unjoinable dependency edge destroys the WHOLE answer with a raw postgres error

```
input (scratch) : {"ref":{"kind":"dispatch","id":"disp-lref-proof"}}
fixture         : dispatch_dependencies holds ('disp-lref-proof','session','sess-lref-proof')
                  and that session row DOES exist (SELECT count(*) FROM sessions WHERE id='sess-lref-proof' → 1)
output (scratch): isError: True
  lineage rejected: postgres error: expected 0 parameters but got 1
```

**Mechanism.** `referent_exists` (repositories.rs:704-714):

```rust
let sql = match dep_kind {
    "event"    => "SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1)",
    "artifact" => "SELECT EXISTS (SELECT 1 FROM artifacts WHERE id = $1)",
    "dispatch" => "SELECT EXISTS (SELECT 1 FROM dispatches WHERE id = $1)",
    _ => "SELECT FALSE",          // <-- takes no parameters
};
let row = client.query_one(sql, &[&dep_value]).await?;   // <-- still binds one
```

The default arm's SQL has no placeholder, so tokio-postgres rejects the bound parameter and the
whole read fails. Every other "I cannot answer this" path in the module is typed and names what is
missing; this one is a raw driver error that loses the entire lineage — the plan's own
"不许白屏" criterion, in the backend.

**Reachability, stated honestly.** `dispatch_dependencies.dep_kind` carries
`CHECK (dep_kind IN ('event','artifact','dispatch'))` (012) and `enqueue_dispatch` validates the same
closed set, so today it needs an out-of-band write or a dropped constraint. I demonstrated it by
dropping the CHECK. Two things make that more than theoretical here: this project has a documented
out-of-band load precedent (the 271-row manifest load on 2026-10-08, recorded in 021's header), and
the scheduler's own twin of this function (`crates/seatloom-mcp-emit/src/dispatch.rs:625`) is safe
*only because* it matches on the closed `DepKind` enum rather than a `&str` — the repository version
is the one that took the string.

### F3 — MODERATE, measured: `no_record.reason` asserts a corpus-wide fact it did not measure

```
input (scratch) : {"ref":{"kind":"history_session","id":"019cad2b-bc6e-7df0-b386-02137c512a38","harness":"codex"}}
fixture         : event_object_refs ('session','ev-flux-other-archived-id') — ONE join key, for a DIFFERENT session
output reason   : measured: no event in canonical_events names ('session', "019cad2b-bc6e-7df0-b386-02137c512a38");
                  archived_sessions and canonical_events share no key — nothing writes an
                  event_object_ref for an archived session and no archived session id is ever an
                  event referent
SQL             : event_object_refs naming an archived session, whole corpus = 1   (the row I wrote)
```

The first clause is measured and true. The trailing clause — *"no archived session id is ever an
event referent"* — is a hardcoded universal inside an answer that is otherwise labelled `measured:`.
Once one join key exists, that sentence is false while being displayed. Nimbus's own proof asserts
the universal **in SQL** (`ANY = 0`), which is why it passes there; the code does not measure it per
call, and the plan's §2.1 point 3 says outright that "the join cannot be faked" and that a link
feature which guesses would be wrong "in exactly the way this project has already paid for once".

### F4 — MODERATE (latent on live): the evidence pointer asserts "on disk" without checking

```
input (scratch) : {"ref":{"kind":"history_session","id":"019cb37a-72c0-7ce1-a7de-1355ca53406a","harness":"codex"}}
fixture         : the contexts index rewritten to a fake archive root holding 1 of the 183 transcripts
output          : {"kind":"file",
                   "id":"/tmp/flux-fake-archive/raw/codex/rollout-2026-03-03T19-34-32-019cb37a-….jsonl",
                   "detail":"the original transcript on disk (raw_path \"raw/codex/rollout-…\" resolved
                             against the archive root the contexts index discovered)"}
os.path.isfile  : False
```

The dispatch's own attack list names this case: *"a session whose source file is no longer on disk"*.
`context_evidence` never checks. Its sibling does — `read_history_session_turns`
(crates/seatloom-core/src/history/read.rs) computes `source_present = Path::new(&source_path).is_file()`
and reports an `empty_reason` when it is false. The lineage read has the row and the path one
`is_file()` away.

**Latent, and I say so:** on live today all 183 declared source files are present (measured:
183 context files declare a source path, 183 exist on disk, 0 missing). The defect is real and
reachable the moment a transcript is moved or cleaned, which is the normal life of a 14 GB archive.

### F5 — MINOR, measured: the acceptance row-sourced fact invents an actor

```
input (scratch) : {"ref":{"kind":"acceptance","id":"acc-flux-dangling-verdict"}}
fixture         : the card, with NO AcceptanceRequested event (nothing records who opened it)
output fact     : eventType: null | occurredAt: null | actorRef: null
                  statement: 监督者 在 2026-10-16T06:58:21.494019+00:00 为派单 disp-flux-dangling
                             开了一张验收卡,分支 packet/flux/x,状态 pending
                  evidence : row acc-flux-dangling-verdict
schema          : acceptances columns = id, dispatch_id, project_id, branch, base, how_to_verify,
                  flux_verdict_ref, status, feedback, rework_dispatch_id, merge_commit, decided_by,
                  decided_at, created_at, acceptance_plan   — there is NO requested_by
```

`subject_statement_acceptance` does `seat_name(s.decided_by.as_deref().unwrap_or("human:supervisor"))`,
so the answer names an actor the schema never recorded. In the normal path an `AcceptanceRequested`
event supplies the actor and the row-sourced fact is unused; it is reachable exactly when F6 happens.

### F6 — MINOR, measured: an acceptance card whose `flux_verdict_ref` names a missing event says nothing

```
input (scratch) : {"ref":{"kind":"acceptance","id":"acc-flux-dangling-verdict"}}
fixture         : flux_verdict_ref = ev-this-verdict-was-never-emitted
                  SELECT count(*) FROM canonical_events WHERE id='ev-this-verdict-was-never-emitted' → 0
output          : facts: [null]   (one row-sourced fact)
                  notes: ['产出了什么为空:链上没有事件命名产出物或文档',
                          '被什么卡着:0 条依赖边,0 条的引用对象存在']
                  evidence: []
```

The card's own verdict pointer is absent from `facts`, from `evidence` **and** from `notes`. The
plan's attack list requires the same honesty for a dispatch's edge ("must name what is missing") —
measured **PASS** for dispatches (live: `disp-session-key-verify` blocked on
`ev-session-key-readmodel-landed` → `exists: false` + `missing: event "ev-session-key-readmodel-landed"
不在库里 — 这个引用对象不存在,所以这条边没有满足`) — but it is not extended to the acceptance's
verdict ref.

### F7 — MINOR, measured: `evidence` can be empty with no note naming it

Every other empty slot carries its reason (`produced == []` → a note; `blocked_on == []` → a note).
`evidence` is the exception, and it is the slot the plan calls 证据在哪:

| referent (measured) | `evidence` | a note explains it? |
|---|---|---|
| `dispatch disp-lref-proof` (scratch) | `[]` | **no** |
| `dispatch disp-session-key-verify` (**live**) | `[]` | **no** |
| `session ses-nimbus-infra-001` (**live**) | `[]` | **no** |
| `event ev-lref-delivered` (scratch) | `[]` | **no** |
| `event ev-lref-verdict` (scratch) | `[document doc-lref-verdict]` | n/a |
| `acceptance acc-disp-lref-proof` (scratch) | `[document doc-lref-verdict]` | n/a |

`dispatch_lineage` hardcodes `evidence: Vec::new()` — the only one of the five kinds that never
populates the slot at all.

### F8 — MINOR, code-level: the "this card gates a dispatch that is gone" note is unreachable

`get_acceptance` (repositories.rs:514-536) `JOIN dispatches d ON d.id = a.dispatch_id` — an inner
join — while `acceptance_lineage` carries a note for `dispatch_row.is_none()`. Measured with the FK
dropped and the dispatch deleted, the card present:

```
input  : {"ref":{"kind":"acceptance","id":"acc-disp-lref-proof"}}
the card exists (SELECT count(*) FROM acceptances WHERE id=… → 1); the dispatch is gone (→ 0)
output : lineage rejected: no acceptance named "acc-disp-lref-proof" exists — 你指的东西不在库里,没有猜一个给你
```

Today `acceptances.dispatch_id REFERENCES dispatches(id)` makes the state impossible, so this is
defensive-code drift rather than a live defect — but the two reads disagree about what the answer
*would* be, and the disagreement resolves toward the less honest one.

### F9 — MINOR, measured: a `history_session` refusal does not name the harness it looked up

```
input  : {"ref":{"kind":"history_session","id":"(1/2) 12-02→12-03 quotagroup 前段","harness":"codex"}}
the row exists, under harness 'cursor'
output : lineage rejected: no history_session named "(1/2) 12-02→12-03 quotagroup 前段" exists — 你指的东西不在库里,没有猜一个给你
```

The archive is keyed by `(harness, id)` — the module says so itself, in `MissingHarness`'s message.
A caller who mistyped the harness is not told which one was tried.

### F10 — MINOR, measured: a resolved artifact/dispatch edge gets a sentence that only restates itself

```
artifact  ar-lref-proof       exists=True  statement=artifact 对象 ar-lref-proof
dispatch  disp-lref-proof     exists=True  statement=dispatch 对象 disp-lref-proof
```

No title, no path, no time — though both rows carry all three one query away, and the plan's
complaint about the 派单 dependency column was precisely that the operator "看得见'被什么卡住',
看不出'那是什么、结论是什么'". For `event` the sentence is excellent (it is built by the same
`statement_for_event` the facts use, so the two surfaces cannot disagree — that part is a real
design win). For the other two kinds the sentence is a placeholder.

### F11 — MINOR, measured: the historical-session file evidence depends on the process's cwd

`default_contexts_dir()` is `$SEATLOOM_HISTORY_CONTEXTS_DIR` or `<resolve_repo_root()>/resume-contexts`,
and `resolve_repo_root()` is `$SEATLOOM_REPO_ROOT` or `current_dir()`. `cmd_lineage` does **not** pass
`SEATLOOM_HISTORY_CONTEXTS_DIR` to the binary it spawns (verified: `grep -rn
SEATLOOM_HISTORY_CONTEXTS_DIR src-tauri/src/commands/` → no hits). `resume-contexts/` exists only in
the main checkout (measured: present in `seatloom/`, **missing** in every worktree). Measured, with
cwd = the flux worktree and no env override:

```
input : {"ref":{"kind":"history_session","id":"019cad2b-bc6e-7df0-b386-02137c512a38","harness":"codex"}}
output: evidence: [{"kind":"unresolved","id":".codex/sessions/2026/03/02/rollout-2026-03-02T14-10-50-019cad2b-….jsonl",
                    "detail":"the extracted contexts are not on disk (the historical corpus is not on disk at
                              \"/Users/…/seatloom-worktrees/flux/resume-contexts\" …); the source path the row
                              records is offered instead — 历史会话的原文没加载,只有库里的这一行"}]
```

The answer is **honest** — it names the missing directory — so this is not a lie. But the corpus
half's only evidence (the file, with its line range) silently degrades to a bare relative path
depending on where the app was launched from, and nothing measures it. **Corroborated by Nimbus's
own gate**: `ev-lineage-readmodel-live`'s referent (A) records `evidence: [{kind:'unresolved', …
naming the missing resume-contexts directory}]` with the note "because this worktree has no corpus on
disk", and referent (B) is the same call re-run with the env var set — so the dependency is known and
documented, just not removed.

---

## 3. Attacks that PASSED — recorded so the FAIL is not read as "the slice is broken"

| # | Attack (from the plan's §3.2 slice 5 list, plus the dispatch's own list) | Measured result |
|---|---|---|
| A | a referent that does not exist at all — **all five kinds** | typed refusal naming kind **and** id, `不在库里`, never an empty object. `dispatch`/`event`/`acceptance`/`session`/`history_session` all measured. |
| B | a historical session | **PASS** for all 183 corpus rows: measured `no_record` with reason + `would_need`, `facts == []`, distinguishable. See §1. |
| C | a dispatch blocked on an event that does not exist | **PASS**: `exists: false` + `missing` naming the absent referent; a resolved edge gets a plain-language statement built by the same sentence function the facts use, so the dependency column stops showing `event:ev-…`. Measured on live. |
| D | a topic with no memories | **PASS**: `kind: "topic"` is outside the closed set, so a typed refusal that names the five-kind domain. A topic is not one of the contract's five referent kinds, so the refusal is the correct answer, and it names the domain rather than answering something else. |
| E | a session whose context file is absent | **PASS**: `context_evidence` carries an `unresolved` pointer naming the missing directory, plus the row's own source path. Measured both with a fake archive root and with a worktree cwd (F11). |
| F | a session whose source file is no longer on disk | **FAIL — F4.** The case is not checked at all. |
| G | anything that returns an empty collection must be distinguishable from "no record" | **PARTIAL**: `facts`/`produced` are guarded by `noRecord`; `blocked_on` and `produced` carry notes; **`evidence` does not (F7)**. |
| H | malformed refs (unknown kind / empty id / missing harness) | **PASS**: three distinct refusals, each naming the domain or the id. |
| I | the read writes nothing | **PASS**: measured on scratch (7 calls, counts unchanged, gate ids absent) and on live (snapshot identical before/after). |
| J | one implementation, two surfaces | **PASS**: verified in code; `cmd_lineage` shells out to the same binary, no second assembler. |
| K | the schema's closed kind set is advertised | **PASS**: `tools/list` advertises `lineage` with the five-kind enum and no identity parameter. |
| L | 中文 / no invented abbreviations / plain language | **PASS**: every statement is Chinese, every empty slot names its reason in Chinese, no invented abbreviations. |

---

## 4. What must change for a re-verdict

1. **F1** — separate the two states `get_archived_session` currently conflates. Either have the
   repository report *why* `message_count` is NULL (column absent vs. row has no extracted
   conversation), or have `history_session_lineage` ask `corpus_columns_present()` itself. The row
   that exists with no extracted conversation deserves its own honest answer ("this row is the
   2026-10-08 manifest inventory entry; it has no extracted conversation, so there is nothing to
   read"), not an instruction to apply a migration that is applied. **Measured on live: 89 of 272
   rows.**
2. **F2** — make `referent_exists`'s default arm match its parameter vector, and prefer the closed
   `DepKind` enum the scheduler already uses. An unknown kind must be a typed `exists: false` with a
   `missing` naming the kind it cannot check — never a raw driver error that loses the whole answer.
3. **F3** — measure the corpus-wide claim or drop it. The `reason` should say what was measured for
   *this* id; a universal statement belongs in a proof, not inside a per-call answer.
4. **F4** — one `Path::new(&source).is_file()`, exactly as `read_history_session_turns` already does,
   and an `unresolved` pointer (or a note) when it is false.
5. **F6** — when `flux_verdict_ref` names no event, say so in `notes`.
6. **F5** — do not default the actor. "开卡的人没有记录" is the honest sentence.
7. **F7** — a note whenever `evidence` is empty, the same as every other slot.
8. **F9** — name the harness in the `history_session` refusal.
9. **F11** — pass `SEATLOOM_HISTORY_CONTEXTS_DIR` (or an explicit root) through `cmd_lineage`, so the
   file evidence does not depend on where the app was launched.

F8 and F10 are consistency work and can ride along.

---

## 5. Out of my lane, reported not acted on

- **`ev-lineage-readmodel-live` exists on live** (measured: `SupervisorMessage` @ `2026-10-16T06:55:11Z`,
  subject `dispatch:disp-lineage-readmodel`), so slice 4 (`disp-lineage-ui`) is **unblocked** — my edge
  `ev-lineage-readmodel-landed` was the correct one for a verifier, and the plan's §3 slice-4 edge on
  the *measured* gate is satisfied. Two things about that gate belong in this verdict:
  1. it was emitted as `SupervisorMessage`, not the `ArtifactCreated` the plan's §3.1 table spells,
     with an `emission_note` recording that `emit_event`'s enforced gate criterion (Aegis ruling
     2026-10-11) refused the `ArtifactCreated` spelling. The plan's event table and the enforced
     criterion disagree; **reported to Lyra/Aegis, not decided by me**.
  2. its eight measured referents did **not** include an archived session with no extracted
     conversation — which is exactly the 89-row gap **F1** sits in. The gate measured the honest
     no-record half (referent `019dce05-…`, 312 messages) and measured it well; it did not measure
     the rows this read refuses with a false statement.
- The **live `documents` table has 0 rows** (measured: `SELECT count(*) FROM documents` → 0, while
  `artifacts` → 15). Consequence, measured on live: every `document` referent in `produced` reads
  `resolved: false` and every matching `evidence` entry reads `kind: "unresolved"`. **That is the
  honest answer** — there is nothing to join — but it means the plan's "证据在哪: what the verdict
  said" pointer is unavailable on live for every event and every acceptance card, because the
  reconcile scanner has never populated `documents`. I measured this independently; Nimbus's own
  `ev-lineage-readmodel-live` payload already carries it in `does_not_assert` ("measured at 0 rows on
  live, so every document referent reads resolved:false there, which is the honest answer for that
  database rather than a defect in the join"). Reported for Aegis; not patched here.
