# Delivery: night step 3 (re-issue) — S5 write-in, blocked on the live database again, with the write set now proven clean

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-11-writein-v2-delivery-v1 |
| status | delivered (work complete, insert blocked-environment) |
| author | vela |
| date | 2026-10-11 |
| version | v1 |
| dispatch | disp-onight-writein-v2 |
| depends_on | VELA-2026-10-10-washing-pipeline-p1-delivery-v1, VELA-2026-10-10-washing-pipeline-p2-delivery-v1, VELA-2026-10-11-writein-guard-delivery-v1, VELA-2026-10-11-writein-blocked-delivery-v1 |
| tags | washing, writein, blocked, stats, seatloom |

## 0. The one-paragraph version for Mr. Zhang

Your 16 sessions are distilled into **87 knowledge records** — decisions, failures, facts,
preferences and open threads — each one pointing back to the exact lines of the original
transcript it came from. The distillation work is finished and re-verified line by line.
**The records are not in the database yet**, and the reason is not the work: the tool that
creates a topic exists in my process now (it did not before), but the database still refuses
the write because one migration (019, the one that adds the "topic opened" event) has not been
applied to the live database. One command from Aegis — applying 019 — turns this into a
finished insert with no re-extraction.

## 1. The guard fired again — but on a different half of the same wall

The re-issue precondition was: *this seat's process reports `open_topic` and the 018-typed
`remember`*. **That is met, and I verified it from this process:**

- `open_topic` is advertised (`searchTools` returns it, with `parent_topic_id`).
- `remember` is advertised with the full 018 signature: `kind`, `confidence`,
  `l2_line_start`, `l2_line_end`, `relations`, `supersedes`.
- A fully valid 018 `remember` call passes tool validation — it is no longer rejected at the
  schema, which is what happened at 09:06 (`l2_line_start: schema is false`).

The insert still does not happen, because the **database** half is unchanged:

| attempt | verbatim result |
|---|---|
| `open_topic` (root topic, no parent) | `open_topic rejected: event write rejected: postgres error: db error` |
| `remember` (018 signature, fully valid, digest + line range) | `memory tool rejected: postgres error: db error` |

Diagnosis, proven rather than guessed:

- **Reads and the event-write machinery both work.** `get_knowledge_tree` returns
  `{"projectId":"seatloom","topics":[]}` (the project row and the `topics` table are live and
  readable), and a `SupervisorMessage` escalation **was written successfully** to the ledger
  from this seat (`ev-vela-writein-blocked-env-20261011`). So this is not a dead database and
  not a dead write API.
- **`open_topic` fails inside its own transaction**, after the project check passed — its code
  path inserts the `topics` row and appends `TopicOpened` in ONE transaction, and the error
  string is `event write rejected`. `TopicOpened` is migration **019**; 019's own header states
  the consequence verbatim: *"until 019 is landed AND applied to live, `open_topic` refuses at
  the database CHECK with the vocabulary violation"*. The 09:06 close-out recorded the same
  fact: live's event vocabulary is still 27 values, without `TopicOpened`.
- **`remember` then fails on the foreign key**, because `memories.topic_id` is `NOT NULL` ->
  `topics(id)` and `topics` holds 0 rows — no topic can be created, so no memory can be
  created. The 019 migration header names this exact cascade, and the code comment at
  `crates/seatloom-mcp-emit/src/dispatch.rs:4218` says it plainly: *"With open_topic live,
  remember finally works (the FK is satisfiable)."*

**So there is exactly one missing piece: 019 applied to live** (canonical order 018 -> 019, via
`apply-db.sh`). Not the binary, not the tool surface, not this seat's work.

Escalated to Aegis via `SupervisorMessage` `ev-vela-writein-blocked-env-20261011` with the
verbatim rejections, the reads-work evidence, and the staged state. No raw SQL, no bypass
files, no live-DB action — the same discipline as the 09:06 close.

## 2. What I did do: the write set is now proven clean, not just staged

Rather than re-stage what was already staged, I built and ran the insert driver
(`tools/wash/s5_writein.py`) that the manifest is executed from. It re-validates every record
against `raw/` **before** any write and refuses to emit a manifest if anything is off. Running
it found real defects in the staged set that the earlier 5-sample spot check could not see:

**Dropped — 1 record (prior count was 0):**

| record | kind | why it cannot be stored |
|---|---|---|
| `56ccb8df` "Codex history is 247 files / 14GB on disk with a ready index; Claude transcripts live under stepgo launch dirs" | fact | Its single claim spans **four disjoint regions** of the raw file (line 336 for the `.codex` 14G size, 484–501 for the 247 rollout files + `session_index.jsonl`, 572–579 for the stepgo launch homes, 7024–7044 for where the seats really write), and its L1 is about the $200/day codex credit rather than file locations. No single line range carries the record honestly, so it was dropped rather than force-anchored. |

**Repaired — 5 records:**

- **2 records were anchored against the wrong file.** Their `l2_body_digest` was
  `e3b0c442...`, which is the sha256 of an **empty input** — the tell-tale of the range
  hasher reading nothing because `start` was past EOF. Root cause: those line numbers were
  taken from the **live** session transcript (`~/.claude/projects/...b0100cff....jsonl`,
  13,596 lines) instead of the archived `raw/` copy (7,044 lines). `raw/` is the corpus of
  record; the live transcript is not in the washing corpus at all. Re-anchored to the raw
  window that actually carries each claim, and the digest recomputed:
  - `0dcce59c` (failure, "AgentHistory archive had silently gone stale for a week")
    7054–7097 → **6994–7044**
  - `7ea2faad` (fact, "The 2026-09-30 AgentHistory archive is a hand-off-oriented corpus")
    7047–7054 → **6981–7000**
  Both windows were read before being accepted, not guessed.
- **3 records carried a stale idempotency key** — `record_id` did not equal the content digest
  of the record's own fields, so a re-run would have computed a fresh digest and inserted a
  duplicate. Re-keyed: `857a99ca`, `f520586e`, `3de1a305`. (Their L2 pointers were already
  valid and verified; only the key was wrong.)

**A third defect the driver itself had, found by writing the test for it:** the staged
`record_id` was computed over the corpus-relative `l2_file_path` while the value written to
the database is the `~`-rooted path — so the idempotency key did not describe the row it was
supposed to de-duplicate. The driver now normalises the path *first* and computes the key over
exactly the bytes that get written. `tests/test_s5_writein.py::test_record_id_equals_content_digest`
is the regression guard; it failed until the driver was fixed.

One scope note on the cycle detector, so nobody over-trusts it: `deep_cycle_detect` walks the
**simple-name** space (the space 015's 1-cycle CHECK does not cover), not the full-path space.
It catches `a→b→c→a` and `a→b→c→d→a` and stays silent on a genuine forest — which is what the
test now pins. It is exercised on a degenerate input today (`project_paths()` is just
`["seatloom"]`), so it is proven for the claim being made about it and nothing more.

**Re-validated end to end, all green:** 87/87 L2 ranges resolve inside `raw/` and their sha256
recomputes exactly; 87/87 `record_id` == content digest of the exact bytes written; layering
clean (L0 one sentence, L1 decision-grade, L2 complete); projects-tree cycle detection clean;
**0 secret shapes in the text that was about to enter the table** (the redaction gate was re-run
on the *distilled* text, not just on the raw stream — red line 1 says before anything is stored,
not only before the LLM).

**Tests:** 37 pass (28 pre-existing + 9 new in `tools/wash/tests/test_s5_writein.py`). The new
suite asserts the gates above against the real manifest — every L2 pointer, not a sample — plus
the accounting (`inserted + dropped == staged`, every drop states its reason), the topic forest
(exactly one root, every parent opened before its child, no empty scope), and the cycle
detector's real behaviour. **Doc gate:** `verify-docs.sh track/infra-foundation` reports this
delivery doc conforming to §2 / §4 / §10.

## 3. The three required stats

**Pruning (S2.5) — raw lines to lines actually sent to the model, re-confirmed against this run:**

| metric | value |
|---|---|
| raw transcript lines (16 sessions) | 58,017 |
| raw bytes | 175,729,835 (167.6 MiB) |
| canonical turns after normalization | 58,084 |
| turns actually sent to the model stage | 25,156 |
| bytes sent to the model stage | 24,726,186 (23.6 MiB) — **14.1% of the bytes kept, 85.9% pruned** |
| tool_result bodies dropped | 45,270,117 bytes (replaced by tool name + ok + byte count + L2 line range) |
| meta rows dropped | 32,928 |

**Redaction (S1) — hits by type, total 1,026, re-confirmed against this run:**

| type | count |
|---|---|
| intranet_domain | 964 |
| stepplan_key65 | 56 |
| jwt_fragment | 2 |
| ak_api_key | 1 |
| basic_auth | 1 |
| session_token | 1 |
| high_entropy | 1 |

Independent grep audit of both washed streams after the run: **0 surviving secret shapes**.
Across the washed streams there are **47 distinct `(type, fingerprint)` pairs** — the same key
is recognisable as the same key without its content ever appearing. Replaced sample, verbatim
from the washed stream:

```
⟪REDACTED:intranet_domain:11e553⟫     x256 occurrences
⟪REDACTED:intranet_domain:7944ea⟫     x363 occurrences
⟪REDACTED:intranet_domain:e4ac3b⟫     x140 occurrences
⟪REDACTED:stepplan_key:831183⟫       x2
```

**Records dropped for no recoverable L2 line number: 1** (prior run reported 0). Why the prior
run said 0 and this one says 1: the earlier check sampled 5 records at random and recomputed
their digests; it did not test every record's range against the raw file's actual line count.
This run tested all 88 and found the two mis-anchored records (repaired, above) plus the one
that cannot be anchored at all (dropped). **The prior "0 dropped" was an under-count produced
by sampling, not by the data being clean.** That is a defect in my own verification method and
I am reporting it rather than quietly keeping the old number.

## 4. What is ready to insert, the moment 019 lands on live

`tools/wash/.local/wash/s5/writein_manifest.json` — machine-readable, committed-by-proxy in
this delivery, regenerable with `python3 tools/wash/s5_writein.py`:

- **24 topics** to open via `open_topic`, governed creation, no raw SQL seeding:
  1 root (`wash-root`) + 10 session scopes + 13 S2 segment scopes, parented
  root → session → segment. Only segments that actually contributed records become topics;
  the other 469 of the 482 S2 segments stay in `topics.json` / `topics.md` as the gate-1
  eyeball artifact rather than being published as empty recall scopes.
- **87 memories** to write via `remember`, ordered by session then record id, each carrying
  `kind` / `confidence` / `source_kind='archived_session'` / `source_ref='<harness>:<session>'`
  / `l2_ref_kind='file'` / `l2_file_path` / `l2_body_digest` / `l2_line_start` / `l2_line_end`
  / tags.
- Distribution: **42 decision, 26 failure, 13 fact, 4 preference, 2 unresolved**;
  **85 quoted / 2 inferred** (the 2 inferred ones state their inference reason in L1);
  10 sessions (7 claude-code contributing 81 records, 3 codex contributing 6);
  55 distinct tags.

**Idempotency is real this time, not asserted.** `remember` mints its own memory id, so the
driver's pre-write check is: read the topic's existing memories at depth l2 and skip any record
whose title + `l2_file_path` + line range is already present, recording it as a skip rather
than an insert. Combined with the corrected content-digest keys and the per-session sha256
checkpoint, a re-run after a partial failure inserts nothing twice.

**One deviation from the schema comment, flagged for Aegis/Lyra:** `memories.l2_file_path` is
documented as *repo-relative*, but the corpus of record lives outside this repository, at
`~/Documents/GitHub/AgentHistory/raw/`. I store the `~`-rooted path
(`~/Documents/GitHub/AgentHistory/raw/claude-code/<session>.jsonl`). The alternative — the bare
`raw/claude-code/...` the staged file carried — looks repo-relative while resolving to nothing
in this repo, i.e. an unresolvable pointer, which red line 2 forbids. Either the schema comment
needs widening to "corpus-relative", or the corpus needs a registered in-repo alias; that is a
decision above this seat.

## 5. What I deliberately did NOT do, and why

- **I did not emit the gate event `ev-writein-landed`.** The dispatch's step 9 asks for it on
  delivery, but the overnight plan fixes its release condition in writing — *"memories row count
  matches the delivery note, every memory has an L2 line range, the delivery note carries the
  three stats"* — and adds the standing rule, quoted here in English (the source is Chinese):
  *"never fire the 'the memory path is available' gate event without a real success — lying to
  the ledger that something happened is worse than being stuck"*. No memory row exists. Firing
  the event would be a false ledger claim that the headline acceptance card would then build on.
  **The event stays unfired until the insert actually lands.** This is the one place where the
  dispatch text and the operator's written gate condition conflict, and the gate condition wins.
- **I did not self-request Flux or open the carding self-dispatch** (step 10). A Flux verdict
  and an acceptance card both attest a *delivery*; there is no delivery to attest while the
  insert is blocked, and asking Flux to verify a write that did not happen would manufacture
  exactly the false-green signal the ruling exists to prevent. The p2 card
  (`acc-disp-wash-vela-p2`) already covers the pipeline and the staged records, with its
  unmet precondition 3 stated plainly.
- **I did not touch the live database.** No raw SQL, no seeding topics by hand, no migration.
  AGENTS.md puts live-DB changes outside this seat's remit; applying 019 is Aegis's call.

## 6. Two packet references are not on this branch

The dispatch payload names this delivery doc `disp-onight-writein-v2-delivery.md`; it is filed
as `VELA-2026-10-11-writein-v2-delivery-v1.md` instead, because COORDINATION_RULES §4
requires `ROLE-YYYY-MM-DD-<topic>-vN.md` and the doc gate
(`scripts/verify-docs.sh`) fails a new file that does not conform — a brand-new path is not
grandfather-able debt. The dispatch id is in the header, which is what routing actually needs.

Two other references are missing from the branch as well:

`docs/coordination/tasks/vela/VELA-2026-10-10-washing-pipeline-v1.md` (§S5) and
`docs/coordination/acceptance/2026-10-11-lyra-vela-writein-acceptance-plan-v1.md` (§2 plan
JSON, §3 card parameters) are both referenced by the dispatch payload and **neither exists on
`packet/vela/washing-pipeline`**. Per my standing instruction I proceeded from the payload's
own description plus the design packet that *is* present
(`AEGIS-2026-10-10-washing-pipeline-design-v2.md` §S5) and the overnight plan's gate
conditions. Flagging it because the missing §2 JSON is the one thing I could not reproduce
verbatim for a future carding step — whoever re-issues should re-attach it.

One more, for Aegis rather than for this dispatch: `docs/coordination/knowledge/INDEX.md`
(added by `ff6b037`) fails the doc gate on the universal header and is **not** in
`.gitlab/docs-grandfather.txt`. It is pre-existing debt on this branch, not something this
dispatch introduced, and fixing another lane's doc is outside my remit — but the gate goes red
on it for every seat that branches from here until it is either fixed or grandfathered.

## 6b. `.pi/` is not gitignored — a hygiene finding, and a correction I owe the ledger

While staging this commit, `git add -A` picked up an untracked `.pi/mcp.json` — local harness
config that was already in the worktree before this turn. I escalated it to Aegis as a
**committed database password**. **That escalation was wrong, and I have retracted it in the
ledger** (`ev-vela-dotpi-finding-correction-20261011`, correcting
`ev-vela-dotpi-credential-exposure-20261011`).

What is actually true: the `DATABASE_URL` is `postgresql://seatloom:seatloom@localhost:5432/seatloom`
— user and password are both the literal string `seatloom`, which is the **documented local-dev
default** already written in `scripts/apply-db.sh:55`. The file holds no key, token, cookie or
non-default secret. I reported a secret without reading its value, which is exactly the failure
mode I am supposed to be guarding against in the other direction: an overstated escalation would
have had Aegis rotate a non-secret and possibly rewrite another seat's branch for nothing.

The real, much smaller finding: **`.pi/` is not in `.gitignore`**, so `git add -A` on any worktree
carrying it commits machine-specific harness config — an absolute path to one seat's
`target/debug` binary plus that default DSN. Commit `6b380b4` on `packet/lyra/milestone-summary`
contains such a file. Worth one `.gitignore` line so worktrees stay portable; **not** a security
matter, and not mine to decide — escalated, not acted on.

What I did: excluded `.pi/` from my own commit by explicit path, and verified the staged diff
contains no real secret. What I did **not** do: modify the shared `.gitignore`, touch the other
branch, or rotate anything. It is **not** in this branch's history and not an ancestor of it, so
`packet/vela/washing-pipeline` is clean.

## 7. Forward recording (completion-protocol step 4)

The `remember` path is not live-available, so the forward records are written here instead of
into the memory store, exactly as in the 09:06 close-out. Four, one per class, kept short:

- **decision** — *The S5 write-in driver validates the whole write set against `raw/` and
  refuses to emit a manifest on any failure, rather than trusting a sampled spot check.*
  Sampling 5 of 88 records reported "0 dropped" while 3 of the 88 were mis-anchored. Verification
  that gates a write must be exhaustive over the write set, because the failure mode it is
  protecting against (an untraceable memory) is per-record, not statistical.
- **failure** — *An L2 anchor numbered against the live session transcript instead of the
  archived `raw/` copy is indistinguishable from a valid anchor until the digest is recomputed
  — and its digest is then the sha256 of an empty input.* Any pipeline that derives line numbers
  from more than one source must record which source each number came from. Fix: assert the
  range is inside the file and recompute the digest before the record is allowed to stage.
- **failure** (mine, this turn) — *I escalated a committed database password without reading its
  value; it was the documented dev default (`seatloom`/`seatloom`, `scripts/apply-db.sh:55`).*
  Retracted in the ledger the same turn. An overstated security report is not a safe error: it
  would have had Aegis rotate a non-secret and rewrite another seat's branch for nothing. Read
  the value before naming it a secret.
- **unresolved** — *`memories.l2_file_path` is documented "repo-relative" but the corpus of
  record is outside the repo; 87 records are about to be stored with a `~`-rooted path that
  satisfies no documented convention.* Needs a ruling (widen the comment to "corpus-relative",
  or register an in-repo alias for the corpus) before the schema comment and the data disagree
  in production.

## 8. Disposition

`complete_dispatch(disp-onight-writein-v2)` with `outcome='failed'` and the reason recorded
verbatim, so the slot is released rather than left hanging — a hanging in_flight is what made
the waker skip this seat for 17 hours, and that will not be repeated. The lane is re-issuable
and the work is not lost:

**The unblock is one command: apply 019 to live** (canonical order 018 -> 019 via
`apply-db.sh`). Then the insert is `execute_plan(manifest, writer=remember)` — 24 `open_topic`
calls, 87 `remember` calls, idempotent, resumable, no re-extraction. After that the three
things this dispatch owes are: the `ev-writein-landed` gate event (condition now genuinely
satisfied), the Flux self-request with a pre-declared verdict id, and the acceptance card.

Scope unchanged: seatloom only, 16 sessions. No expansion without the operator's explicit
decision.
