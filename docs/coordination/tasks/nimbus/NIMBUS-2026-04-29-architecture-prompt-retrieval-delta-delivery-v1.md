# NIMBUS-2026-04-29-architecture-prompt-retrieval-delta-delivery-v1

| Field | Value |
|-------|-------|
| Document | Architecture Prompt + Retrieval Delta — Delivery |
| Template | T3 |
| Subtype | fix |
| Status | Delivered |
| Issuer | Lyra |
| Assignee | Nimbus |
| Issued | 2026-04-28 |
| Delivered | 2026-04-29 |
| Task Ref | NIMBUS-2026-04-28-architecture-prompt-retrieval-delta-v1 |
| Acceptance Boundary | docs/coordination/acceptance/2026-04-28-lyra-architecture-baseline-alignment-acceptance.md |

---

## 1. Micro-brief

This delivery closes all four FAIL/WARNING findings from Lyra's CONDITIONAL acceptance verdict (ABA-01 through ABA-04) on the Architecture Baseline Alignment delivery. Three P0 changes and one P1 metadata fix were applied across `docs/architecture-decisions.md` and `docs/architecture-design.md`.

---

## 2. Facts

| Item | Status |
|------|--------|
| ABA-01: Prompt architecture missing | CLOSED — AD-012 added; §3.2 + §5.1 + §12.4 updated |
| ABA-02: Rejected/Rescoped durable states violate event-first contract | CLOSED — states removed from WorkItemStatus; event payloads carry verdict + scope-change |
| ABA-03: INT-13 through INT-16 re-read required | CLOSED — re-read complete; architecture text updated per findings |
| ABA-04: Delivery metadata dual-key (template + subtype) | CLOSED — both previous delivery header and this delivery header carry `Template: T3 / Subtype: fix` |
| BLOCKER-001: Retrieval L1/L2 backend | CLOSED — Lyra froze SQLite FTS5 via dispatch 2026-04-28 |

---

## 3. Exact doc patches

### docs/architecture-decisions.md

**AD-010 (rewritten in previous session, confirmed correct in this delta):**
- Removed durable `Rejected` and `Rescoped` WorkItem states
- Replaced with event-first contract: `ReviewVerdictIssued` event carries verdict + reason + linked evidence artifact ID; `WorkItemRescoped` event carries scope-change summary + new AC refs
- WorkItem lifecycle path on rejection: `InReview → (ReviewVerdictIssued) → Blocked → Ready → Active`
- Authoritative reference: INT-05

**AD-011 (updated):**
- BLOCKER-001 marked closed
- Added table: L1/L2 → SQLite FTS5 (persistent, authoritative); in-memory index → cache only, rebuilt on startup; L3/L4 → P1 deferred

**AD-012 (new):**
- Interactive prompt architecture decision
- Defines: `PromptKind` (Deterministic / WizardMenu / Freeform / Sensitive), `PromptPolicy` (AutoAllowed / NeedsApproval / HumanRequired), `PromptAction` (Approve / HumanTakeover / SupervisorAssist / Stop), `AssistBudget` (max 5 steps / 2048 tokens)
- Sensitive rule: `HumanRequired` always set; `SupervisorAssist` disabled at architecture level
- Audit events: `PromptDetected`, `PromptInputInjected`

### docs/architecture-design.md

**§3.2 session.rs — SessionStatus enum:**
- Added `PromptBlocked` variant with clarifying comment
- Added `PromptState`, `PromptKind`, `PromptPolicy`, `PromptAction`, `AssistBudget` structs

**§3.2 workitem.rs — WorkItemStatus enum:**
- Removed `Rejected` and `Rescoped` variants
- Added clarifying comment explaining event-first path and INT-05 reference

**§3.3 event.rs — EventType enum:**
- Removed `WorkItemRejected` (redundant with `ReviewVerdictIssued` event)
- Added `PromptDetected` and `PromptInputInjected` with inline payload documentation
- Added clarifying comment on `WorkItemRescoped` and `ReviewVerdictIssued` as the evidence chain

**§5.1 IPC — Prompt command group added:**
- `get_prompt_state(session_id)` → `PromptState`
- `approve_prompt(session_id)` → `PromptResult`
- `takeover_prompt(session_id)` → `()`
- `supervisor_assist_prompt(session_id, budget)` → `PromptResult`
- `stop_on_prompt(session_id)` → `Session`

**§12.3 Retrieval Layer Contract:**
- BLOCKER-001 open text replaced with frozen decision
- SQLite FTS5 specified as persisted L1/L2 backend
- In-memory index explicitly labelled as cache/projection rebuilt from SQLite on startup
- PostgreSQL/pgvector upgrade path noted for P1 multi-user/cloud deployments

**§12.4 Prompt Engine Architecture (new section):**
- End-to-end flow diagram: stdin blocked → PromptClassifier → Ledger event → Frontend banner → user action dispatch
- AssistLoop constraints documented in-line
- Sensitive prompt architecture-level enforcement noted

---

## 4. How INT-13 through INT-16 changed architecture text

| Spec | Finding | Architecture impact |
|------|---------|---------------------|
| INT-13 | Evidence search must enter L1 structured first, then L2 full-text; no LLM wait; grouped exact matches | §12.3 retrieval ladder confirmed; order fixed; SQLite FTS5 covers both L1 and L2 with two virtual tables |
| INT-14 | Why-decision = exact refs first; semantic in P1 only | §12.3 L3 semantic explicitly P1-deferred; no architecture change at L1/L2 |
| INT-15 | ROI/routing comparison | P2 — no P0 architecture impact; not reflected in design |
| INT-16 | Full prompt handling flow: auto for deterministic, approval for wizard/freeform, human-only for sensitive; Supervisor assist bounded window; `prompt.detected` and `prompt.input_injected` events required | AD-012 and §3.2 PromptState structs + §3.3 EventType + §5.1 IPC + §12.4 flow diagram all derived directly from INT-16 |

---

## 5. Remaining blockers

None. All P0 blockers from the task packet are closed.

**P1 items noted (not blocking this delivery):**
- L3 semantic search backend (PostgreSQL/pgvector vs other): deferred to P1 milestone
- HandoffStatus::Working live-activity state implementation: P1 per prd-v0.5
- `NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md` section §8 (BLOCKER-001) now historical; SQLite FTS5 frozen

---

## 6. Commands run and results

All patches applied via GitHub Copilot CLI edit tool against the working directory `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom`. No build or test commands exist yet at P0 milestone; all changes are architecture document patches.

Files modified:
- `docs/architecture-decisions.md` — AD-010 event-first, AD-011 SQLite freeze, AD-012 new
- `docs/architecture-design.md` — SessionStatus, WorkItemStatus, EventType, IPC Prompt group, §12.3, §12.4 new
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-architecture-baseline-alignment-delivery-v1.md` — header dual-key metadata patch
- `docs/coordination/MEMORY.md` — (writeback pending, see §7)
- `docs/coordination/memory/2026-04-28.md` — (writeback pending, see §7)

---

## 7. Recommendation to Lyra

**Recommend: ACCEPT**

All four acceptance findings from the CONDITIONAL verdict are fully addressed:
1. ABA-01 — Prompt architecture implemented at decision, struct, IPC, and flow diagram levels
2. ABA-02 — Durable Rejected/Rescoped states removed; event-first contract restored to match INT-05
3. ABA-03 — INT-13 through INT-16 re-read complete; §12.3 and §12.4 directly derived from findings
4. ABA-04 — Dual-key metadata applied to both delivery artifacts

BLOCKER-001 closed per Lyra dispatch 2026-04-28.

No new blockers introduced. P1 items are explicitly labelled and do not affect P0 completeness.
