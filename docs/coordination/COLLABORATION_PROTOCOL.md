# SeatLoom Collaboration Protocol v1.0

| Item | Content |
|------|---------|
| Document | Collaboration Protocol v1.0 |
| Status | Draft — Pending Lyra Review |
| Author | Aegis |
| Date | 2026-04-28 |
| Complements | COORDINATION_RULES.md, AI_NATIVE_WORKFLOW_PRINCIPLES.md |

---

## 0. Why This Exists

Role definition files (LYRA.md, NIMBUS.md, etc.) define what each seat *can* do, but not:

- Who talks to whom
- What triggers a seat to act
- What shape the message takes
- What the end-to-end flow looks like

Without this, all coordination falls on the supervisor's runtime memory. This document closes that gap.

---

## 1. Role Roster

| Seat | Type | Responsibility | Receives From | Delivers To |
|------|------|---------------|---------------|-------------|
| **Aegis** | Supervisor | Global orchestration, cross-seat conflict resolution, phased review | Mr. Zhang (human) | All seats |
| **Lyra** | Supervisor-PO | Product truth, gate decisions, task packaging, daily cadence | Aegis, Flux (reports), Mira/Nimbus (delivery) | Mira, Nimbus, Flux (task packets) |
| **Mira** | Worker-UX | UI/UX prototypes, interaction design, component delivery | Lyra (task packets) | Flux (verification), Lyra (acceptance) |
| **Nimbus** | Worker-Eng | Architecture, implementation, backend/frontend code | Lyra (task packets), Mira (prototype handoff) | Flux (verification), Lyra (acceptance) |
| **Flux** | Verifier | Acceptance testing, evidence capture, environment ops | Lyra (verification packets), Mira/Nimbus (artifacts) | Lyra (verification reports) |

### Authority Hierarchy

```
Mr. Zhang (Human)
  └── Aegis (Supervisor — global view, phased review, escalation)
        └── Lyra (Supervisor-PO — daily driver, task packaging, gate decisions)
              ├── Mira (Worker — UX/UED)
              ├── Nimbus (Worker — Engineering)
              └── Flux (Verifier — QA/Ops)
```

Rules:
- Worker seats take instructions only from Lyra (or Aegis when Lyra is unavailable)
- Worker seats do not instruct each other
- Flux verifies but does not direct Mira/Nimbus; findings go to Lyra for triage
- Aegis intervenes only at phase boundaries or on Mr. Zhang's explicit request

---

## 2. Collaboration Graph

### 2.1 Primary Flow (Happy Path)

```
┌──────────┐    task packet    ┌──────────┐    artifact    ┌──────────┐
│   Lyra   │ ───────────────→  │  Worker  │ ────────────→  │   Flux   │
│   (PO)   │                   │(Mira/Nim)│                │(Verifier)│
└────┬─────┘                   └──────────┘                └────┬─────┘
     │                                                          │
     │              verification report                         │
     │ ◄────────────────────────────────────────────────────────┘
     │
     ├── PASS → update MEMORY.md + issue next task packet
     └── FAIL → issue fix packet to worker (with evidence)
```

### 2.2 Exception Flows

```
Worker blocked ──→ blocker report to Lyra ──→ Lyra resolves or escalates to Aegis
Flux finds P0 ──→ urgent report to Lyra ──→ Lyra freezes + issues hotfix packet
Cross-doc conflict ──→ Lyra freezes conflict ──→ Aegis arbitrates if needed
Mr. Zhang directive ──→ Aegis translates ──→ Lyra packages ──→ Worker executes
```

### 2.3 Prohibited Paths

| From | To | Why Prohibited |
|------|----|---------------|
| Mira | Nimbus (direct) | Creates shadow contracts outside Lyra's view |
| Nimbus | Mira (direct) | Same — scope changes must go through Lyra |
| Flux | Mira/Nimbus (direct fix) | Flux reports, does not fix product code |
| Worker | Mr. Zhang (direct) | Escalation goes through Aegis |

---

## 3. Trigger-Action Table

This table defines **when** each seat should act. No seat acts without a trigger.

### 3.1 Lyra Triggers

| Trigger | Action | Output |
|---------|--------|--------|
| New/updated authority doc in `docs/` | Review for impact on active tasks | Decision memo or task packet update |
| Worker delivery artifact appears in `docs/coordination/` | Read and assess against acceptance spec | Accept/reject with evidence |
| Flux verification report appears | Gate decision (PASS/FAIL) | Gate decision file + next task packet |
| Blocker report from any worker | Triage: resolve, re-scope, or escalate to Aegis | Resolution memo or escalation note |
| Start of day | Create/update daily memory file | `memory/YYYY-MM-DD.md` |
| End of day | Close daily log | Deferred items + tomorrow first action |
| Aegis phased review complete | Incorporate findings into task queue | Updated task packets |

### 3.2 Mira Triggers

| Trigger | Action | Output |
|---------|--------|--------|
| New task packet in `tasks/mira/` | Read packet + referenced contracts, execute | React components + delivery report |
| Lyra fix packet (post-Flux rejection) | Read rejection evidence, fix, re-deliver | Updated components + updated delivery report |
| Dependency blocked (missing contract/schema) | Write blocker report to Lyra | Blocker file in `tasks/mira/` |

Mira does NOT act on:
- Nimbus implementation changes (unless Lyra issues a re-sync packet)
- Flux reports directly (unless Lyra re-routes as fix packet)
- Global timeline/memory updates

### 3.3 Nimbus Triggers

| Trigger | Action | Output |
|---------|--------|--------|
| New task packet in `tasks/nimbus/` | Read packet + referenced contracts, implement | Code changes + implementation report |
| Mira prototype handoff (routed via Lyra) | Integrate UI components into Tauri app | Integration report |
| Lyra fix packet (post-Flux rejection) | Read rejection evidence, fix, re-deliver | Updated code + updated report |
| Dependency blocked (missing contract/API) | Write blocker report to Lyra | Blocker file in `tasks/nimbus/` |

Nimbus does NOT act on:
- Mira's UI changes directly (waits for Lyra to package)
- Flux reports directly
- Product scope changes not in approved docs

### 3.4 Flux Triggers

| Trigger | Action | Output |
|---------|--------|--------|
| New verification packet in `tasks/flux/` | Execute acceptance tests per spec | Verification report with pass/fail/evidence |
| Lyra requests environment setup | Prepare runnable environment | Setup runbook + confirmation |
| Worker artifact appears (when Lyra routes for verification) | Verify against acceptance criteria | Verification report |

Flux does NOT act on:
- Worker artifacts directly (waits for Lyra to issue verification packet)
- Product design discussions
- Architecture decisions

---

## 4. Message Contracts (Packet Schemas)

All packets are Markdown files under `docs/coordination/tasks/<role>/`. Every packet must include the header table.

### 4.1 Task Packet (Lyra → Worker)

```markdown
# Task: <short title>

| Field | Value |
|-------|-------|
| Packet ID | TASK-<role>-YYYY-MM-DD-NNN |
| From | Lyra |
| To | <target role> |
| Priority | P0 / P1 / P2 |
| Deadline | YYYY-MM-DD or "before <gate name>" |
| Status | issued / in_progress / delivered / accepted / rejected |

## Objective

<1-3 sentences: what to deliver>

## Input Files

- `docs/<file1>.md` — <why needed>
- `docs/<file2>.md` — <why needed>

## Done Definition

- [ ] <verifiable criterion 1>
- [ ] <verifiable criterion 2>
- [ ] Delivery report written to `docs/coordination/tasks/<role>/`

## Acceptance Spec Reference

- `docs/acceptance-spec-v1.1.md` §<section>

## Constraints

- <any scope limits, budget limits, or prohibitions>
```

### 4.2 Delivery Report (Worker → Lyra)

```markdown
# Delivery: <task title>

| Field | Value |
|-------|-------|
| Packet ID | DEL-<role>-YYYY-MM-DD-NNN |
| Task Ref | TASK-<role>-YYYY-MM-DD-NNN |
| From | <worker role> |
| To | Lyra |
| Status | delivered |

## Changed Files

- `path/to/file1` — <what changed>
- `path/to/file2` — <what changed>

## Done Checklist

- [x] <criterion 1>
- [x] <criterion 2>
- [ ] <criterion 3 — incomplete, reason: ...>

## Blockers

- <none, or: description + suggested owner + suggested action>

## Verification Request

Ready for Flux verification: yes / no (reason)
```

### 4.3 Verification Report (Flux → Lyra)

```markdown
# Verification: <task title>

| Field | Value |
|-------|-------|
| Packet ID | VER-<role>-YYYY-MM-DD-NNN |
| Task Ref | TASK-<role>-YYYY-MM-DD-NNN |
| From | Flux |
| To | Lyra |
| Verdict | PASS / FAIL / PARTIAL |

## Criteria Results

| # | Criterion | Result | Evidence |
|---|-----------|--------|----------|
| 1 | <criterion> | PASS/FAIL | <file path, screenshot, command output> |
| 2 | <criterion> | PASS/FAIL | <evidence> |

## Issues Found

| ID | Severity | Description | Repro Steps | Suggested Fix |
|----|----------|-------------|-------------|---------------|
| V1 | P0/P1/P2 | <description> | <steps> | <suggestion> |

## Environment

- OS: <os>
- Node: <version>
- Commands run: <list>

## Recommendation

<pass to gate / fix required before gate / needs Lyra decision>
```

### 4.4 Gate Decision (Lyra → All)

```markdown
# Gate Decision: <gate name>

| Field | Value |
|-------|-------|
| Packet ID | GATE-YYYY-MM-DD-NNN |
| From | Lyra |
| Verdict | GO / HOLD / CONDITIONAL GO |

## Entry Criteria Status

| Criterion | Met? | Evidence |
|-----------|------|----------|
| <criterion 1> | Yes/No | <ref> |

## Decision

<1-3 sentences explaining verdict>

## Follow-up Packets Issued

- TASK-mira-YYYY-MM-DD-NNN: <title>
- TASK-nimbus-YYYY-MM-DD-NNN: <title>

## Next Gate

<name and expected date>
```

### 4.5 Blocker Report (Worker → Lyra)

```markdown
# Blocker: <short title>

| Field | Value |
|-------|-------|
| From | <role> |
| To | Lyra |
| Blocking Task | TASK-<role>-YYYY-MM-DD-NNN |
| Severity | P0 / P1 / P2 |

## Description

<what is blocked and why>

## Root Cause

<best understanding of the cause>

## Suggested Resolution

<what Lyra/Aegis could do to unblock>

## Workaround (if any)

<temporary path forward, or "none">
```

---

### 4.6 Review Change Tier Record (Reviewer ↔ Worker)

Use this packet when review findings require follow-up edits. This packet is mandatory so change-tier decisions are durable and queryable for later productization.

Tier policy:

- `L1` micro edit: reviewer direct patch allowed (no worker dispatch)
- `L2` clear semantic edit: worker patch + compact acknowledgment
- `L3` contract/scope edit: worker patch + full gate review

```markdown
# Review Change Tier Record: <short title>

| Field | Value |
|-------|-------|
| Packet ID | RCT-YYYY-MM-DD-NNN |
| From | <reviewer> |
| To | <executor or self> |
| Tier | L1 / L2 / L3 |
| Task Ref | TASK-<role>-YYYY-MM-DD-NNN (optional for L1) |
| Impact Level | none / minor / major |

## Reason

<why this tier was selected>

## Changed Clauses

- `<doc-path>#<section>`
- `<doc-path>#<section>`

## Execution Mode

- `direct_patch` (L1)
- `worker_patch_compact_ack` (L2)
- `worker_patch_full_gate` (L3)

## Evidence

- `<file path>`
- `<file path>`

## Compact Ack (required for L2)

- TaskRef:
- Changed files:
- Changed clauses:
- Impact:
- Evidence:
- Status: done / blocked
```

---

## 5. End-to-End Lifecycle Example

A complete cycle for one task:

```
1. Lyra creates task packet
   → writes TASK-mira-2026-04-28-001.md to tasks/mira/
   → sends Chinese summary via terminal: "新任务包已发，请查看 tasks/mira/"

2. Mira reads packet + referenced docs
   → executes (writes React components)
   → writes DEL-mira-2026-04-28-001.md to tasks/mira/
   → sends Chinese summary: "交付完成，请查看 DEL 报告"

3. Lyra reads delivery report
   → if done checklist complete → issues verification packet to Flux
   → writes TASK-flux-2026-04-28-001.md to tasks/flux/

4. Flux reads verification packet
   → runs acceptance tests
   → writes VER-flux-2026-04-28-001.md to tasks/flux/
   → sends Chinese summary: "验收完成，2 PASS / 1 FAIL"

5. Lyra reads verification report
   → if PASS → writes gate decision, updates MEMORY.md, issues next task
   → if FAIL → writes fix packet back to Mira with Flux evidence attached

6. Loop until gate passes
```

---

## 6. Routing Rules

### 6.1 Who Routes What

| Event | Router | Logic |
|-------|--------|-------|
| New authority doc update | Lyra | Assess impact → issue task packets to affected workers |
| Worker delivery | Lyra | Check completeness → route to Flux or reject |
| Flux verification done | Lyra | Gate decision → next task or fix |
| Blocker from worker | Lyra | Resolve if within scope; escalate to Aegis if cross-cutting |
| Mr. Zhang directive | Aegis | Translate into contract language → hand to Lyra for packaging |
| Phase boundary review | Aegis | Comprehensive review → findings to Lyra |

### 6.2 Escalation Path

```
Worker → Lyra (first attempt to resolve)
  → Aegis (if Lyra cannot resolve or conflict is cross-seat)
    → Mr. Zhang (if Aegis needs human judgment)
```

Maximum escalation latency: Lyra should respond to blocker within same session. Aegis responds within same day.

---

## 7. Packet Naming Convention

```
TASK-<role>-YYYY-MM-DD-NNN.md    (task assignment)
DEL-<role>-YYYY-MM-DD-NNN.md     (delivery report)
VER-<role>-YYYY-MM-DD-NNN.md     (verification report)
GATE-YYYY-MM-DD-NNN.md           (gate decision)
BLK-<role>-YYYY-MM-DD-NNN.md     (blocker report)
```

NNN is a zero-padded sequential number within that day (001, 002, ...).

---

## 8. Relationship to SeatLoom Product

This manual protocol is the live prototype of what SeatLoom automates:

| Manual Protocol Element | SeatLoom Product Equivalent |
|------------------------|----------------------------|
| Role `.md` files | Seat profile (`.seatloom/seats/`) |
| Task packet files | WorkItem + Handoff |
| Delivery report | Artifact + Checkpoint |
| Verification report | Pipeline output (verification_loop) |
| Gate decision | WorkItem state transition gate |
| MEMORY.md | Ledger (append-only event store) |
| Trigger-action table | Inbox rules engine |
| Routing rules | Handoff routing + Pipeline triggers |
| Escalation path | Inbox priority escalation |
| Blocker report | WorkItem `blocked` status + Inbox item |

Insights from operating this protocol should feed back into product design refinement.

---

## 9. Adoption Checklist

- [ ] Lyra reviews and accepts this protocol
- [ ] All future task assignments use Task Packet schema
- [ ] All future deliveries use Delivery Report schema
- [ ] Flux verifications use Verification Report schema
- [ ] Gate decisions use Gate Decision schema
- [ ] Prohibited paths are enforced (no direct worker-to-worker instruction)
- [ ] MEMORY.md updated with "Collaboration Protocol v1.0 adopted"

---

*This document defines the collaboration operating protocol for SeatLoom's AI-native team. It complements role definitions in `roles/`, coordination rules in `COORDINATION_RULES.md`, and workflow principles in `AI_NATIVE_WORKFLOW_PRINCIPLES.md`.*
