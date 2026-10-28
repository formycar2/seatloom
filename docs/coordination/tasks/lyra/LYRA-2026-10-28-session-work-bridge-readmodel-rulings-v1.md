# Ruling: the session↔work read model's three open design gaps (F2/F3/F4), and the two contract defects behind them

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-session-work-bridge-readmodel-rulings-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | aegis, nimbus, vela, flux |
| depends_on | `LYRA-2026-10-16-session-work-bridge-contract-v1.md`, `docs/coordination/tasks/nimbus/disp-Yq3x6uWnKQn9j5MC_t-U2-delivery.md`, `infra/postgres/schema/022_session_work_bridge_view.sql` (track), `2026-10-16-session-work-bridge-acceptance-v1.zhs.md` |
| tags | ruling, readmodel, session-work-bridge, contract-correction, flux, vela, design-decision |

> Aegis dispatched this as `disp-0S12su7xBeaULFhCkFdJy` (2026-10-28): Nimbus left three items
> undone because they were design judgments, not implementation ones, and he was right to stop.
> This memo rules on all three, and on the two contract defects Nimbus measured behind them.

## 0. What I verified before ruling (not taken on anyone's word)

| Claim | How checked | Result |
|---|---|---|
| Track's 022 has 20 columns and names them differently from the 27-column view | `git show track/infra-foundation:infra/postgres/schema/022_session_work_bridge_view.sql` | **confirmed** — `resolved_by`, `a1_resolves`, `a2_resolves`, `session_harness`, `session_id`, `session_project`, `session_raw_path`, `session_message_count`, `session_cwd`, `missing`, plus `memory_id`, `project_id`, `topic_id`, `title`, `kind`, `confidence`, `l0_summary`, `source_ref`, `l2_file_path`, `l2_ref_kind`. No `join_state`, no `a1_harness`, no `a2_harness`/`a2_id`, no `status`/`created_at`/line range |
| The live error is real | `column "a1_harness" does not exist`, reproduced by Nimbus on live | **confirmed** (not re-run by me — no live DB access from this seat; see §6) |
| The read model derives the missing columns in the repository SELECT rather than changing the migration | read `SESSION_WORK_SELECT` in `crates/seatloom-core/src/db/repositories.rs` on `packet/nimbus/session-work-readmodel` | **confirmed**, and the derivation is documented in the constant's own comment |
| The view coalesces `s1`/`s2`, so `both` does not say whether the two keys named the same row | read 022's `coalesce(s1.*, s2.*)` | **confirmed** — this is the root of ruling A |
| Live distribution | Nimbus's read-only measurement: 90 rows, `both=87`, `a1_path=3`, `a2_source_ref=0`, `unresolved=0` | **accepted as measured**; re-verify before relying on it for a gate (§6) |
| The read model labels the coalesced session as `a1_harness`/`a1_id` | `v.session_harness AS a1_harness, v.session_id AS a1_id` in `SESSION_WORK_SELECT` | **confirmed — this is a defect**, ruled on in §2 |

## 1. Ruling A — "the two sources disagree" is recoverable, in the read model and not in the view

**The gap.** The contract §9 step 4 and the acceptance card step 4 both require a state that reads
*"the two recorded keys do not line up"* to be visibly distinct from *"no records"*. Nimbus found
he could not produce it, and reported that it was not recoverable — accurately, **for the view**.

**Why he is right about the view.** 022 coalesces: `coalesce(s1.harness, s2.harness)`, and
`resolved_by` is `both` when both resolve, without saying whether they named the same
`archived_sessions` row. That coalescing is deliberate — 022's own header records why it reports
both keys side by side instead of picking one — so a view that resolved through one key only would
hide the 3 rows the other cannot reach.

**Why "accept it" would have been the wrong ruling.** Two reasons, both fatal:

1. The read model already computes `a1_corpus_path` (the `regexp_replace` of `l2_file_path`) and
   `a2_harness`/`a2_id` (the `split_part` of `source_ref`). Those are exactly the two join
   predicates. A LEFT JOIN against `archived_sessions` **twice** — once per key — recovers the
   per-key resolution the view coalesced away, at read time, with no migration change and no data
   change. The technique is already in use in this very SELECT; it just stops one join short.
2. A contract state that is *claimed but not computed* is worse than an honest "not measurable".
   That is the failure mode this product exists to prevent. Ruling "two states" would have written
   that failure into the acceptance criteria.

**The ruling.**

1. **022 is not changed.** No `DROP VIEW`, no new migration to express `disagree`. The view's job
   stays what it is: make the join executable and the drift number countable, with both keys side
   by side. Changing a live migration is the operator's action and must be bought by need, not by
   contract symmetry.
2. **The read model computes per-key resolution.** The repository SELECT gains two LEFT JOINs
   against `archived_sessions`: `sa` on `sa.raw_path = a1_corpus_path`, `sb` on
   `sb.harness = a2_harness AND sb.id = a2_id`. `join_state` becomes:

   | `join_state` | Meaning |
   |---|---|
   | `both_agree` | both keys resolve unambiguously **and name the same row** |
   | `disagree` | both keys resolve unambiguously **and name different rows** |
   | `path_only` | only A1 resolves (A2 unresolvable, or A2 names no row) |
   | `ref_only` | only A2 resolves |
   | `none` | neither resolves |

   `a1_path_rows > 1` keeps its existing meaning: an ambiguous path is **reported, never resolved by
   picking a row** — it is not folded into `disagree`.
3. **The current live count of `disagree` is 0, and that is reported as a measurement, not hidden.**
   On today's data, 87 `both` and 3 `a1_path` means the state is reachable in the code and empty in
   the database. The forward read carries the per-state counts, so an operator sees
   `both_agree=87, path_only=3, disagree=0` and can tell "none today" from "not measured".
4. **Fix the aliasing defect at the same time.** Today `v.session_harness AS a1_harness` labels the
   view's *coalesced* session as A1's. When A2 is the key that resolved, that alias is false, and
   `join_state`'s `both_agree` would then rest on a lie about which key did the work. With the two
   joins, `a1_session` is A1's row and `a2_session` is A2's row, and the coalesced
   `session_harness`/`session_id` is kept only as a fallback for the no-key case.

**What this costs Nimbus:** two LEFT JOINs and one comparison in a SELECT that is already a
derivation, plus tests for the five `join_state` values — including a `disagree` case that must be
constructed in a scratch database (a memory whose `l2_file_path` normalises to a *different*
session's `raw_path`), because live has none today.

**What it does not cost:** a migration, a live-DB change, an operator action, or any change to the
view's contract with its readers.

**If the view ever should change** (operator's call, not this ruling's): a **new** migration 023
with `DROP VIEW` + `CREATE`, catalog-guarded and idempotent, never an edit to 022 in place — 022's
own header states that a later definition change is a follow-up migration's DROP+CREATE, *"never a
silent rewrite here"*. The trigger would be a measured need to distinguish same-row from
different-row agreement **for callers that cannot do the two joins themselves**. Today no such
caller exists.

## 2. Ruling F2 — yes, report whether the cited file is on disk, as three values, never a boolean

Contract §7.2 requires it: *"A record whose source file is no longer on disk. Show the record with
its line reference and say the file is missing. Do not hide the record."* Nimbus was right not to
invent it, because the obvious implementation — a `bool` — is a guess.

**The ruling.** Add `source_file_state` to `SessionWorkRecord`, one of exactly three values:

| Value | When |
|---|---|
| `present` | a `stat` on the expanded path succeeded |
| `missing` | a `stat` was attempted and returned not-found |
| `not_checked` | no stat was attempted (a caller that asked for pointers only, or a path that cannot be expanded) |

Four requirements, so this cannot decay into decoration:

1. **Metadata only.** `stat`, never a read. 011's red line 4 stands: no transcript body enters the
   database, and none is opened here either.
2. **Expand `~/` against `$HOME`; anything else is `not_checked`.** The L2 pointer is the
   `~`-rooted corpus path verbatim, and a bare `raw/...` form resolves to nothing — red line 2
   forbids guessing a base for it. A path that is not `~`-rooted is `not_checked`, not `missing`.
3. **`source_file_state` is a different fact from `join_state`, and the two are never merged.**
   A record can resolve by key and still have its file gone; a record can fail to resolve and still
   have its file present. One "healthy" flag over both would be the exact collapse §9 step 4
   forbids.
4. **The forward read reports how many it checked**, so `not_checked` is a visible choice rather
   than a silent default.

**And a contract correction this exposes.** Acceptance-card step 4 currently offers *"or use a
session whose original file has been deleted"* as a way to produce the non-resolution case. That
conflates two different states: a deleted file leaves the reference **resolving** and the pointer
**dead**; the load-bearing case is a reference that resolves against **no row**. The card is
amended below so step 4 asks for the broken reference and the deleted file becomes its own step.

## 3. Ruling F3 — yes, read the session row directly; it is required, not cosmetic

Today `SessionWorkSession` is built from `rows.first()`, so a session with **zero** records reports
`session: null` and loses `raw_path`, `project`, `message_count` and `corpus_session`. That breaks
acceptance-card step 3 exactly where it matters: the "no records" screen cannot say *why* there are
no records, because the fact that distinguishes the two reasons (inventory-only row vs corpus
session) was thrown away with the empty list.

**The ruling.**

1. **The forward read fetches the `archived_sessions` row for the `(harness, id)` it was given,
   unconditionally.** The read already calls `archived_session_exists` on that key; fetching the
   row is the same query with more columns, not a second round trip.
2. **`rows.first()` is removed as a source of session facts.** Its assumption — "every attributed
   record carries the same session facts" — is precisely the assumption ruling A breaks: with the
   view's coalescing, a record's session columns may be A1's row or A2's. Under ruling A the
   session's own row is the only authority for the session's own facts.
3. **`corpus_session` becomes a fact read from the row**, not a value inferred from a record's
   `message_count`. `message_count IS NULL` means inventory-only and is reported, never defaulted
   to 0.

## 4. Ruling F4 — yes, rewrite the session-level note, with four requirements

The note is the only place the read speaks in sentences, so it is where an unearned claim hides.
Nimbus already got the hardest part right: he renamed the note after the view's own word
(`resolved_by`) rather than the read model's (`join_state`), because the view cannot express a
disagreement and naming the note `join_state` would have claimed a distinction the database does
not make. Under ruling A the read model *can* express it, so the vocabulary question changes — but
the discipline that produced the rename is the one to keep.

**The ruling.** The session-level note must:

1. **Report the five per-state counts for this session** — `both_agree`, `disagree`, `path_only`,
   `ref_only`, `none` — and the view-wide drift number, as separate numbers. Never one merged
   "healthy" figure.
2. **Never say "agree" for something only "resolved" proves.** `both_agree` may be asserted only
   when both keys resolve **to the same row** (ruling A). Where they resolve to different rows the
   note says `disagree` and names both `(harness, id)` pairs, so the operator can check them.
3. **Never use the word "disagree" for the view's distribution.** The view-wide line reports
   `resolved_by` (its own column and its own vocabulary); the session-level line reports
   `join_state`. Two different words for two different measurements, and the note says which is
   which.
4. **State the zero-record case with its two reasons separated**, and state the inventory-only case
   from the row (ruling F3), not from a record.

## 5. Ruling C — how these three relate to Vela's write-in guard

They sit on **opposite sides of the same key**, and neither substitutes for the other:

| | Vela's guard (slice 3) | This read model (slice 2) |
|---|---|---|
| Direction | the **write** path | the **read** path |
| Question | may this new record be written? | what do the records already written say? |
| Effect of failure | the manifest is **refused** | the state is **reported** |
| Prevents | the drift number growing | the drift number being invisible |

So: **the guard is the lock, the read model is the alarm, and the drift number is what the alarm
reads.** The 3 rows A2 cannot reach were written before the guard existed; they are not repaired by
editing rows (contract §4.2: forget + remember, the tombstone stays, the ledger shows both).

**One precise constraint on the guard, so it cannot be written wrong:** the guard's criterion is
that **both keys resolve** — that each reaches an `archived_sessions` row. It is **not** that the
two keys *agree*. "Resolve" and "agree" are different predicates, and under ruling A the read model
is the thing that measures agreement. A guard written against the stronger predicate would refuse
records that are healthy today (the 3 `a1_path` rows resolve through A1) and would fail closed on
the wrong condition. Vela's dispatch should say this in as many words.

**And what the guard owes the read model:** the guard's exhaustive validation is what makes the
read model's `unresolved` count mean something over time. Without it, `unresolved` is a number that
only ever grows.

## 6. Contract corrections, and where they land

Three defects in my own 10-16 contract, each of which Nimbus hit:

1. **The contract never named the schema objects it depends on.** It specified the view's *job* and
   left the column list to whoever built it. That is how a 27-column view that never existed
   anywhere but one branch became the shape a read model was written against. **Correction:** the
   contract gains an appendix quoting track's 20 columns verbatim, with the note that the
   authoritative list is the one on the integration branch and in the live database.
2. **Acceptance-card step 4 offered the deleted file as the load-bearing case.** **Correction:**
   the card is amended (below).
3. **§9 step 4 said "three states" without saying which three, or who computes the third.**
   **Correction:** §9 step 4 now names the five `join_state` values and states that the read model
   computes them, that the view does not, and that the live count of `disagree` is 0 today.

The general rule behind all three goes into `ENGINEERING_STANDARDS.md`, because it is not specific
to this slice: **a read-model contract that names database columns is a claim about a database, and
every such claim is verified against the integration branch and the live database before the
contract is issued.** A branch-local artifact is not a contract.

**One thing I could not verify from this seat:** every live measurement in this memo is Nimbus's,
taken on 2026-10-28 from `packet/nimbus/session-work-readmodel` @ `1572154`. This seat has no
database access. Before any gate is emitted on these numbers, the emitter re-runs the count itself
— the rule Aegis recorded on 10-28 (`the_gate_condition_was_met_on_a_branch_but_not_on_track`)
applies to me exactly as it applied to Nimbus: *"X is on track" is always verified by the person
issuing the gate, never accepted as a relay.*

## 7. Done definition — the four amendments to slice 2

Appended to `LYRA-2026-10-16-session-work-bridge-contract-v1.md` §11.2 slice 2:

- [ ] Per-key resolution: two LEFT JOINs, five `join_state` values, `disagree` reachable in code
      and counted (0 on live today, reported as measured)
- [ ] `a1_harness`/`a1_id` no longer alias the coalesced session; A1's row and A2's row are
      separate
- [ ] `source_file_state` ∈ {`present`, `missing`, `not_checked`} by `stat` only, `~/`-expanded,
      never merged with `join_state`, with the checked count reported
- [ ] Session row read directly by `(harness, id)`; `rows.first()` removed as a source of session
      facts; `corpus_session` from the row
- [ ] Session-level note rewritten to the four requirements of §4
- [ ] Tests for all five `join_state` values including a scratch-built `disagree`, and for all
      three `source_file_state` values; mutation-proved (reverting the second join makes the
      `disagree` test fail)

**Not in scope for slice 2, and named so it is not folded in:** changing 022, touching the live
database, emitting any gate event, merging, widening the washing scope, Tier B attribution, or any
re-layout.
