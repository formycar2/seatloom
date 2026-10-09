# Delivery: D2 close-out — widen AcceptanceResult (status + reworkDispatchId), fixing the reject-path dead-end

[Nimbus -> Flux / Aegis] Delivery for governed dispatch `disp-58ZddqlToyI5dgkC1jiIP`
(D2 close-out, Nimbus lane). Fixes the Flux HOLD finding
(`FLUX-2026-10-09-mira-d2-ui-verify-delivery-v1` §4): the accept/reject decision
response was under-specified, so Mira's reject path fell through to a dead-end
"already handled" label.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | disp-58ZddqlToyI5dgkC1jiIP-delivery |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| governing dispatch | `disp-58ZddqlToyI5dgkC1jiIP` |
| Flux HOLD | `FLUX-2026-10-09-mira-d2-ui-verify-delivery-v1` §4 (reject-path dead-end) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/waker-hardening` |
| status | **COMPLETE** — gates green; Flux re-verify self-requested |

---

## 0. The fix (minimal, additive, Nimbus lane only)

`AcceptanceResult` (the accept/reject command response) was
`{ accepted, merge_commit?, error? }` — under-specified vs the acceptance-loop
design §5. On a REJECT it carried no status and no rework link, so Mira's
normaliser produced `status:""` / `mergeCommit:null` / `reworkDispatchId:null`
and her `DecisionResult` dead-ended — no rejected badge, no feedback echo, no
jump to the rework dispatch — even though `cmd_reject_delivery` HAD created the
rework and written `rework_dispatch_id` to the row.

Per Flux's routing, two additive fields make the response a SUPERSET of the
card normaliser (no Mira-side change, no accept-path break):

- `src-tauri/src/dto.rs` — `AcceptanceResult` gains `status: String` and
  `rework_dispatch_id: Option<String>` (kept `#[serde(rename_all="camelCase")]`,
  so the wire names are `status` / `reworkDispatchId`).
- `crates/seatloom-core/src/db/repositories.rs` — `decide_acceptance` now
  RETURNS the decided `AcceptanceRow` (read back inside the same transaction),
  so the response is populated from what the database actually landed, not from
  the caller's intentions.
- `src-tauri/src/commands/acceptance_cmds.rs` — `cmd_accept_delivery` and
  `cmd_reject_delivery` populate `status` + `merge_commit` + `rework_dispatch_id`
  from the returned row. Accept → `status:"accepted"`, `mergeCommit:<sha>`,
  `reworkDispatchId:null`. Reject → `status:"rejected"`, `mergeCommit:null`,
  `reworkDispatchId:<disp-...>` (feedback is already on the card).

No `ui/**` file touched (Mira's lane).

## 1. Verification

- `cargo build -p seatloom-core` / `-p seatloom-mcp-emit` — green.
- `cargo fmt --all --check` — green.
- `cargo clippy --workspace --all-targets -- -D warnings` — green.
- `cargo test --workspace` — green (104 core + 40 mcp-emit + 4 tauri), including
  a new wire-shape test asserting BOTH paths serialize `status` /
  `mergeCommit` / `reworkDispatchId` with the camelCase names the UI reads.
- Scratch DB (`seatloom_d2_nimbus`): the `decide_acceptance` read-back SELECT's
  14-column list validated against `row_to_acceptance` (matches `get_acceptance`).

## 2. STALE-BRANCH FLAG — report, not worked around

`packet/nimbus/waker-hardening` diverged from `track/infra-foundation` before
track merged the dispatch-UI redesign, the Flux waker proof, and several
coordination docs. A two-dot diff (`git diff track..this-branch`) shows a naive
WHOLE-BRANCH merge of this branch into track would REVERT:

- `ui/src/views/DispatchInboxView.tsx` (~1065 lines) and
  `ui/src/views/DispatchInboxView.test.tsx` (deleted) — Mira's merged redesign;
- `crates/seatloom-mcp-emit/proof/waker_loop_proof.sh` (deleted) — Flux's proof;
- `docs/coordination/MEMORY.md`, `memory/2026-10-09.md`, and the merged
  `ONYX-…-dispatch-states-design`, `MIRA-…-dispatch-ui-redesign`,
  `LYRA-…-governed-dispatch-milestone`, `FLUX-…-waker-loop-proof-delivery` docs.

These are NOT reconciled here (that is track-integration work, not this lane).
**Recommendation for Aegis: integrate the Rust backend path-scoped** — take only
`crates/seatloom-core/src/db/{event_domain,models,repositories}.rs`,
`crates/seatloom-mcp-emit/src/{dispatch,scheduler,protocol,main}.rs`,
`src-tauri/src/dto.rs`, `src-tauri/src/commands/{acceptance_cmds.rs,mod.rs}`,
`src-tauri/src/main.rs` — never a whole-branch merge. Emitted as a
`SupervisorMessage` for Aegis alongside this delivery.

## 3. Sequence

1. One commit on `packet/nimbus/waker-hardening` (this change + this doc).
2. `emit_event` SupervisorMessage with the stale-branch flag (§2).
3. `enqueue_dispatch` to `seat-flux-001` — re-verify the reject-path shape per
   `FLUX-2026-10-09-mira-d2-ui-verify` §5 recipe.
4. `complete_dispatch disp-58ZddqlToyI5dgkC1jiIP`.
