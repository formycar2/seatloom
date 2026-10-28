# Delivery — `disp-lineage-verify` (Flux): adversarial verification of the lineage read model

- **Dispatch**: `disp-lineage-verify`, issued by seat-lyra-001. Edge
  `ev-lineage-readmodel-landed` measured **true** in the ledger — the correct edge for a verifier
  (I verify the code, so the announcement is the right gate).
- **Verdict**: **FAIL** — narrow. The central promise holds for every corpus session; two classes of
  unmeasured assertion and one total-answer-loss path do not.
- **Verdict event**: `ev-verdict-lineage-readmodel` — pre-declared id, emitted exactly as required
  (`ReviewVerdictIssued`, `verdict: "fail"`).

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-16-lineage-readmodel-verdict-v1.md` | new — the verdict: every attack with its exact input and exact output, the PASS table, and the nine must-fix items |
| `.scratch/flux_lineage_attack.sh` | new — the attack script (9 cases), reproducible against the scratch DB `seatloom_lineage_ref` |
| `.scratch/flux_lineage_attack2.sh` | new — the two follow-ups (A9 retry with the FK cleared, A10 the invented actor) |
| `.scratch/flux_show_evidence.py` | new — prints one evidence pointer per line with whether the path is on disk |

No source file was changed. I verified at Nimbus's pin `36194e8` on
`packet/nimbus/gate-vs-announcement`, in Nimbus's worktree, **read-only**: nothing was committed
there and no tracked file was modified.

## How it was verified

**Gates at the pin** (before any behavioural claim):
`cargo fmt --all -- --check` clean · `cargo clippy --workspace --all-targets` zero warnings ·
`cargo test --workspace` 262 passed / 0 failed (10 of them `db::lineage_ref::tests::*`) ·
Nimbus's own `proof/lineage_ref_proof.sh` **ALL POINTS PASS** on a scratch DB.

**Gates on my branch** (this commit): fmt clean · clippy zero warnings · `cargo test --workspace`
238 passed / 0 failed.

**Behavioural** — the real `seatloom-mcp-emit` binary driven over stdio:
- scratch DB `seatloom_lineage_ref` (canonical order + 015-019 + 021) for the cases that need a
  fixture the live DB does not have (an inventory-only row, a written join key, a dangling
  `flux_verdict_ref`, a dropped CHECK, a deleted dispatch);
- the **live** `seatloom` DB, read-only, for the cases that must be measured where the operator
  actually is — with a ledger snapshot before and after every call
  (`1044 events / 2536 refs / 154 dispatches / 8 acceptances`, unchanged), which is also how the
  read-only property was measured rather than assumed.

## The findings, in one line each

| # | Severity | Finding |
|---|---|---|
| F1 | **MAJOR, live** | `history_session` refuses a row that EXISTS with "apply migration 021 first" — 021 IS applied and the column exists. **89 of 272 live rows.** |
| F2 | **MAJOR, latent** | `referent_exists`'s `_ => "SELECT FALSE"` arm still binds a parameter → one unjoinable edge destroys the whole answer with `postgres error: expected 0 parameters but got 1`. |
| F3 | MODERATE | `no_record.reason` hardcodes "no archived session id is ever an event referent" — a corpus-wide claim it did not measure, inside an answer labelled `measured:`. |
| F4 | MODERATE, latent | `context_evidence` says "the original transcript on disk" without an `is_file()`; its sibling `read_history_session_turns` does check. |
| F5 | MINOR | the acceptance row-sourced fact defaults the actor to 监督者; `acceptances` has no `requested_by`. |
| F6 | MINOR | an acceptance card whose `flux_verdict_ref` names a missing event says nothing anywhere. |
| F7 | MINOR | `evidence == []` is the one empty slot with no note naming it (measured on live too). |
| F8 | MINOR, unreachable | the "this card gates a dispatch that is gone" note is dead — `get_acceptance` inner-joins `dispatches`. |
| F9 | MINOR | a `history_session` refusal does not name the harness it looked up. |
| F10 | MINOR | a resolved artifact/dispatch edge gets `artifact 对象 <id>` — no title, no path, no time. |
| F11 | MINOR | the historical-session file evidence depends on the process's cwd; `cmd_lineage` passes no contexts dir. |

## What PASSED (recorded so the FAIL is not over-read)

The central test: for a live corpus session the answer is the measured `no_record` with
statement/reason/would_need, `facts == []`, and file evidence with line ranges — **no guessed join
key, no derived filename, no empty list that reads as "nothing happened"**. All five kinds resolve
with live data. Malformed input is refused by name. The read is measured read-only. One
implementation, two thin surfaces (`cmd_lineage` shells out to the same binary). The parser is not
"usually refusing". Full table in the verdict doc §3.

## To report upward (not acted on)

- The live `documents` table has **0 rows** while `artifacts` has 15, so every document referent on
  live reads `resolved: false` — the honest answer, but it means the plan's "what the verdict said"
  evidence pointer is unavailable on live. Nimbus's own `ev-lineage-readmodel-live` payload already
  carries this in `does_not_assert`; I measured it independently. **For Aegis.**
- `ev-lineage-readmodel-live` was emitted as `SupervisorMessage`, not the `ArtifactCreated` the
  plan's §3.1 table spells, with an `emission_note` recording that `emit_event`'s enforced gate
  criterion refused the ArtifactCreated spelling. **The plan's event table and the enforced
  criterion disagree — for Lyra/Aegis.**
- That gate's eight measured referents did not include an archived session with no extracted
  conversation — which is exactly where F1 sits. **For Lyra/Aegis** (the gate's coverage, not the
  code).

## Forward recording (D5 Phase 2 Lane B)

Nothing recorded via `remember`. Every finding above is either (a) a code fact with a file:line and
a reproduced input/output, or (b) already written down in this verdict and in Nimbus's own gate
payload. Nothing here changes a future action of mine in a way that is not already in the verdict
doc, and nothing is a preference of Mr. Zhang's — so per the protocol's "三条全满足才记;存疑不记",
zero entries.
