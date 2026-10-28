# Delivery — the session↔work read model's three design gaps, ruled

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | lyra-disp-0S12su7xBeaULFhCkFdLy-delivery |
| status | completed |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| tags | ruling, readmodel, session-work-bridge, contract-correction, delivery |

## Decision

All three of Nimbus's open items are ruled, and one of them is ruled **better than "accept it"**.

**A — the "two sources disagree" state is recoverable, in the read model and not in the view.** He
was right that 022 coalesces `s1`/`s2` and so cannot express it, and right that the view should not
be changed. But the read model already computes both join predicates (`a1_corpus_path`,
`a2_harness`/`a2_id`) and stops one join short: two LEFT JOINs against `archived_sessions` recover
per-key resolution at read time, with no migration and no data change. So `join_state` gains five
values — `both_agree`, `disagree`, `path_only`, `ref_only`, `none` — and `disagree` is reachable in
code, **measured 0 on live today, and reported as a measurement rather than hidden**. Ruling "accept
two states" would have written "claimed but not computed" into the acceptance criteria, which is the
failure this product exists to prevent.

The same change fixes a defect Nimbus flagged in his report: `v.session_harness AS a1_harness`
labels the view's **coalesced** session as A1's, which is false whenever A2 is the key that resolved.

**F2 — report `source_file_state` ∈ `present` / `missing` / `not_checked`.** `stat` only, never a
read; `~/`-expanded and non-`~` paths are `not_checked` rather than `missing`; never merged with
`join_state`. A dead file and an unresolvable reference are two different facts.

**F3 — read the session's own row by `(harness, id)`, unconditionally.** `rows.first()` is removed
as a source of session facts: its assumption breaks the moment resolution is per-key, and today it
throws away `raw_path`/`message_count`/`corpus_session` for exactly the zero-record case the
acceptance card needs to explain.

**F4 — rewrite the session-level note** to report five per-state counts plus the drift number as
separate numbers, say `agree` only when both keys resolve to the same row, and never use `disagree`
for the view's `resolved_by` distribution.

**C — the write-in guard's criterion is "both keys resolve", not "both keys agree".** The guard is
the lock, the read model is the alarm, the drift number is what the alarm reads. A guard written
against the stronger predicate would fail closed on the wrong condition (the 3 `a1_path` rows are
healthy today).

## Files

- `docs/coordination/tasks/lyra/LYRA-2026-10-28-session-work-bridge-readmodel-rulings-v1.md` (new) — the ruling memo, with a "what I verified before ruling" table, option-by-option reasoning for A, and the done definition.
- `docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md` (v1 → v2) — §11.2 slice 2 gains the four judged items as done-definition; §9 step 4 amended to name the five states and who computes them; §7.2 amended for `source_file_state`; slice 3 gains the resolve-vs-agree constraint; **new §12 appendix quoting track's 20 columns** and the columns that must be derived. v1's text kept intact.
- `docs/coordination/acceptance/2026-10-16-session-work-bridge-acceptance-v1.zhs.md` (v1 → v2) — step 4 no longer offers "a session whose file was deleted" as the load-bearing case (that conflates two states); **new step 4b** for the file-missing case.
- `docs/coordination/ENGINEERING_STANDARDS.md` (v2 → v3) — new section `## Read models and the contracts that describe them`, six rules, plus follow-up 8.

## Verification

- `bash scripts/verify-docs.sh HEAD` — all changed docs conform (header, naming, English, template directory).
- `git diff --check` on staged files — clean.
- The 20-column list in §12 was read out of `track/infra-foundation:infra/postgres/schema/022_session_work_bridge_view.sql` and cross-checked against Nimbus's `SESSION_WORK_SELECT` derivation comment, not taken from either summary.
- Authored with `write`/`edit`; zero code-modifying heredocs; zero `cd <absolute path> &&`.

## Escalation / notes

- **I could not verify the live numbers.** No database access from this seat. Every live figure
  (`both=87, a1_path=3, drift=0`) is Nimbus's, taken 2026-10-28 from
  `packet/nimbus/session-work-readmodel` @ `1572154`. Whoever emits
  `ev-session-key-readmodel-live` must re-run the count — Aegis's own 10-28 rule
  (`the_gate_condition_was_met_on_a_branch_but_not_on_track`) applies to me as much as to him.
- **The read model is still not on track.** `git merge-base --is-ancestor 1572154
  track/infra-foundation` returns non-zero; the three `session_work` files exist only on the packet
  branch. Nothing in this ruling should be read as making the gate emittable.
- **These rulings do not unblock `disp-session-key-ui`.** They define what the read model must
  compute; the implementation, its verification, and the integration are still owed. Aegis asked for
  the rule so he can release the following dispatches — the rule is the rule, not the work.
- Shared coordination logs (`MEMORY.md`, `docs/coordination/memory/`) untouched.
- Pre-existing unrelated dirty files left exactly as found, not in this commit.
