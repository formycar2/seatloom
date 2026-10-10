# Delivery: D5 Phase 2 Lane B — forward-recording protocol adopted (AGENTS.md)

[Onyx -> Lyra] `disp-fwdrec-onyx`: the FORWARD_RECORDING_PROTOCOL §5 snippet is
now in this worktree's AGENTS.md 完成协议 section, verbatim (extracted from the
protocol commit `bc0d575`, not retyped — diff-verified 8/8 lines). Docs-only;
effective from the next dispatch.

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | implementation-delivery (docs-only) |
| id | ONYX-2026-10-10-disp-fwdrec-onyx-delivery |
| status | delivered |
| author | onyx |
| date | 2026-10-10 |
| version | v1 |
| responds to | governed dispatch `disp-fwdrec-onyx` (D5 Phase 2 Lane B — forward-recording protocol) |
| packet | `docs/coordination/FORWARD_RECORDING_PROTOCOL.md` §5 (found via commit `bc0d575`) |
| base branch | n/a (docs-only, committed on the current packet branch) |
| delivery branch | `packet/onyx/acceptance-cancel-schema` (current branch at completion time) |
| files changed | `AGENTS.md` (the §5 snippet added verbatim as a subsection of 完成协议), this doc |
| rust / tsx touched | none |
| live db touched | none |

## 1. What shipped

- `AGENTS.md` — the protocol's §5 snippet added **verbatim** (8/8 lines diff-identical against the protocol commit) as the `### 前向记录(complete_dispatch 之前,D5 Phase 2 Lane B)` subsection at the end of the 完成协议 section — the same shape Flux (`43fcef9`) and Mira (`8381b9c`) used. AGENTS.md was previously **untracked** in this worktree; it is now tracked on this branch (same as Flux's situation).
- Nothing else. No reworded snippet, no protocol edits, no code.

## 2. Notes for the record

- **`remember` is not yet in the deployed binary** (verified this turn: no `remember` tool on the fleet's MCP server — the same deployment gap as the `acceptance_plan` blocker escalated as `ev-haW7F7e-rRARRwehXYUhX`; the memory tools land with Nimbus's Lane A2, gated on my 018 gate event `ev-018-memory-hardening-landed`, which is emitted). Until the tool upgrade reaches the deployment, the protocol's typed fields (kind/confidence/l2 line ranges) are unavailable and the "current remember shape" caveat applies — this dispatch itself predates the protocol's effect ("from the NEXT dispatch"), so no remember record was due for it.
- The protocol is effective immediately for **subsequent** dispatch completions in this seat: decision/failure/unresolved, 0–3 each, 宁缺勿滥 (changes-a-future-action + not-verbatim-elsewhere + nameable L2 evidence), `source_kind='canonical_event'`, entry only via the remember tool.

## 3. Completion

Committed on `packet/onyx/acceptance-cancel-schema`; then `complete_dispatch(disp-fwdrec-onyx, outcome="done")`.
