# Lyra Decision Memo - Persistence Protocol Adoption

| Field | Value |
|---|---|
| Owner | Lyra |
| Date | 2026-04-27 |
| Status | Accepted / Active |
| Scope | Coordination operating protocol |
| References | `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/memory/2026-04-27.md` |

---

## 1. Decisions needed

- None for adoption itself.
- All subsequent substantial outputs must follow file-first writeback immediately.

## 2. Decisions made

- SeatLoom adopts the file-first persistence protocol immediately.
- Terminal/chat output is summary-only and must include artifact path(s).
- Every substantial output must produce a durable artifact under `docs/coordination/`.
- `docs/coordination/MEMORY.md` and `docs/coordination/memory/YYYY-MM-DD.md` must be updated for accepted milestones and required writebacks.
- Draft terminal analysis without writeback is not source-of-truth.

## 3. Action packets

| Owner | Deadline | Done definition |
|---|---|---|
| Lyra | Immediate | Every decision/review/task packet includes durable file artifact + short terminal summary + artifact path(s) |
| Mira | Next substantial output | UI delivery/report is persisted under `docs/coordination/` with changed files, flow coverage, blockers |
| Nimbus | Next substantial output | Implementation packet is persisted with commands run, results, contract deviation notes |
| Flux | Next substantial output | Acceptance report is persisted with severity, reproducible runbook, evidence paths |

## 4. Blockers

- Earlier terminal-only reasoning from 2026-04-27 is not durable until rewritten to file artifacts.
- The Mira acceptance review and product alignment output still need formal writeback before any gate use.

## 5. Stage-gate status

- `SG-01 UI Contract Baseline`: Hold
- Reason: persistence protocol is now active, but the current alignment/review results are not yet published as coordination artifacts.
- Next required writeback: Lyra product alignment + Mira acceptance review packet.

## Artifact paths

- `docs/coordination/tasks/lyra/LYRA-2026-04-27-persistence-protocol-v1.md`
- `docs/coordination/MEMORY.md`
- `docs/coordination/memory/2026-04-27.md`
