# SeatLoom Process Mapping: Real Workflow → Product Model

| Item | Content |
|------|---------|
| Document | Process Mapping Review |
| Status | Draft — Pending Lyra Review |
| Author | Aegis |
| Date | 2026-04-28 |
| Purpose | Map our actual 2-day collaboration (Apr 27-28) onto SeatLoom's object model to validate product completeness |

---

## 0. Why This Matters

Our current collaboration on the SeatLoom project is itself the exact use case SeatLoom is designed to serve: multiple AI seats working on a shared project, switching runtimes, handing off context, and needing traceability.

By mapping our real work onto the product model, we can:
1. Validate that the object model can express real workflows
2. Discover gaps where the model cannot represent what actually happened
3. Generate realistic test data for UI prototyping

---

## 1. Seat Registration

| Seat ID | Name | Role | Actual Runtime | Actual Environment |
|---------|------|------|---------------|-------------------|
| seat-aegis | aegis | supervisor | claude_code | Cursor Agent (Claude) |
| seat-lyra | lyra | product_owner | gemini → opencode | tmux: Lyra-po-seatloom |
| seat-mira | mira | designer | gemini | tmux: Mira-UX/UED-seatloom |
| seat-nimbus | nimbus | architect | copilot | tmux: Nimbus-TechArchi-seatloom |
| seat-flux | flux | verifier | opencode | tmux: Flux-Quality&Ops-seatloom |

Product action: User opens SeatLoom → Init → Add 5 Seats.

---

## 2. WorkItem Lifecycle

### Phase 1: Product Design (Apr 27 morning)

| WorkItem | Title | Owner | Status Flow | Gate Events |
|----------|-------|-------|-------------|-------------|
| WI-001 | Define product form and tech stack | aegis | draft → ready → active → done | AC: architecture-decisions.md written |
| WI-002 | Write MVP scenarios v2.0 | aegis | draft → ready → active → done | AC: mvp-scenarios.md v2.0 approved |
| WI-003 | Write PRD v0.4 design-first contract | aegis | draft → ready → active → done | AC: prd-v0.4.md written with state machines, inbox rules, capability guarantees |
| WI-004 | Deliver Phase 1 UX prototypes | mira | draft → ready → active → **blocked** | Lyra rejected: SG-01 FAIL |
| WI-005 | Verify Mira Phase 1 delivery | flux | draft → ready → active → **blocked** | Environment issues (path error, pnpm missing) |

### Phase 2: Design Refinement (Apr 27 afternoon → Apr 28)

| WorkItem | Title | Owner | Status Flow | Gate Events |
|----------|-------|-------|-------------|-------------|
| WI-006 | Comprehensive interaction design review | aegis | draft → ready → active → done | AR: review doc written, HO sent to Lyra |
| WI-007 | Define collaboration protocol | aegis | draft → ready → active → done | AR: COLLABORATION_PROTOCOL.md written |
| WI-008 | Lyra product contract alignment review | lyra | draft → ready → active → done | 4 P0 conflicts frozen, SG-01 Hold decision |
| WI-009 | Flux acting-Mira contract repair | flux (acting as mira) | draft → ready → active → in_progress | Chinese high-density demo refresh |
| WI-010 | Multica benchmark analysis | lyra | draft → ready → active → done | Adaptation guardrails accepted by Aegis |
| WI-011 | Cross-priority value/dataflow baseline | lyra | draft → ready → **active** | Implementation frozen until this is done |

---

## 3. Session History

| Session | Seat | Runtime | Started | Ended | Status | Key Output |
|---------|------|---------|---------|-------|--------|------------|
| SES-001 | aegis | claude_code | Apr 27 ~10:00 | Apr 27 ~22:00 | completed | 6 design docs (AR-001..AR-006) |
| SES-002 | mira | gemini | Apr 27 ~13:00 | Apr 27 ~14:30 | completed | 15 React components (AR-007..AR-021) |
| SES-003 | flux | opencode | Apr 27 ~15:00 | Apr 27 ~16:00 | completed (partial) | Verification attempt, env issues |
| SES-004 | lyra | gemini→opencode | Apr 27 ~16:00 | Apr 27 ~22:00 | completed | SG-01 review, task packets, daily log |
| SES-005 | lyra | opencode | Apr 28 ~09:00 | ongoing | running | Acting-Mira repair, Multica benchmark, protocol review |
| SES-006 | aegis | claude_code | Apr 28 ~10:00 | ongoing | running | Interaction review, collab protocol, process mapping |

### Rehydrate Events

| From Session | To Session | Trigger | LaunchPack Content |
|-------------|-----------|---------|-------------------|
| SES-001 (aegis/claude) | SES-006 (aegis/claude) | Next day, new conversation | Conversation summary + MEMORY.md + all design docs |
| SES-004 (lyra/gemini) | SES-005 (lyra/opencode) | Runtime switch + new day | Lyra role file + MEMORY.md + active task packets |

---

## 4. Handoff Chain

| Handoff | From | To | WorkItem | Purpose | Status |
|---------|------|----|----------|---------|--------|
| HO-001 | aegis | mira | WI-004 | Deliver Phase 1 UX prototypes per ux-spec | accepted → completed (output rejected at gate) |
| HO-002 | aegis | flux | WI-005 | Acceptance test Mira Phase 1 output | accepted → completed (partial, env issues) |
| HO-003 | aegis | lyra | WI-008 | Take over as PO driver, review all product docs | accepted → completed |
| HO-004 | aegis | lyra | WI-006 | Interaction design review findings | sent (pending accept/dispute) |
| HO-005 | aegis | lyra | WI-007 | Collaboration protocol for review | sent (pending review) |
| HO-006 | lyra | flux | WI-009 | Acting-Mira contract repair packet | accepted → in_progress |
| HO-007 | aegis | lyra | — | Process mapping analysis + gap findings | sent (this document) |

---

## 5. Artifact Registry

| Artifact | Type | Source Session | WorkItem | Content |
|----------|------|--------------|----------|---------|
| AR-001 | decision_doc | SES-001 | WI-001 | architecture-decisions.md |
| AR-002 | scenario_spec | SES-001 | WI-002 | mvp-scenarios.md v2.0 |
| AR-003 | product_contract | SES-001 | WI-003 | prd-v0.4.md |
| AR-004 | ux_spec | SES-001 | WI-003 | ux-spec.md |
| AR-005 | interaction_spec | SES-001 | WI-003 | interaction-spec-v1.0.md |
| AR-006 | acceptance_spec | SES-001 | WI-003 | acceptance-spec-v1.0.md |
| AR-007..021 | ui_component | SES-002 | WI-004 | 15 React components in ui/src/ |
| AR-022 | daily_log | SES-004 | WI-008 | coordination/memory/2026-04-27.md |
| AR-023 | acceptance_verdict | SES-004 | WI-008 | SG-01 FAIL verdict |
| AR-024 | task_packet | SES-004 | WI-008 | Worker task packets (AI-native v2) |
| AR-025 | review_doc | SES-006 | WI-006 | interaction-design-review.md |
| AR-026 | protocol_doc | SES-006 | WI-007 | COLLABORATION_PROTOCOL.md |
| AR-027 | review_doc | SES-006 | — | This process mapping document |

---

## 6. Timeline Reconstruction (Chronological)

```
Apr 27
──────
10:00  aegis   Aegis started a Claude Code session                     SES-001
10:30  aegis   Aegis created "Define product form and tech stack"      WI-001
10:45  aegis   Aegis produced architecture decisions document          AR-001
10:50  aegis   WI-001 moved to done                                    WI-001
11:00  aegis   Aegis produced MVP scenarios v2.0                       AR-002
11:00  aegis   WI-002 moved to done                                    WI-002
11:30  aegis   Aegis produced PRD v0.4                                 AR-003
12:00  aegis   Aegis produced UX Spec v1.0                             AR-004
12:30  aegis   Aegis produced Interaction Spec v1.0                    AR-005
12:30  aegis   Aegis produced Acceptance Spec v1.0                     AR-006
12:30  aegis   WI-003 moved to done                                    WI-003

13:00  aegis   Aegis sent UX prototype task to Mira                    HO-001
13:00  mira    Mira started a Gemini session                           SES-002
13:05  mira    Mira accepted handoff from Aegis                        HO-001
13:10  mira    Mira produced AppShell.tsx                              AR-007
       ...     (14 more components)
14:30  mira    Mira session completed                                  SES-002
14:30  auto    Checkpoint created for SES-002                          CP-001

15:00  aegis   Aegis sent verification task to Flux                    HO-002
15:00  flux    Flux started an OpenCode session                        SES-003
15:05  flux    Flux session requires input (path error)                SES-003
15:10  aegis   Aegis resolved Flux input (corrected path)              SES-003
15:30  flux    Flux session requires input (pnpm not found)            SES-003
15:40  aegis   Aegis resolved Flux input (npm install -g pnpm)         SES-003
16:00  flux    Flux session completed (partial verification)           SES-003

16:00  aegis   Aegis sent PO driver task to Lyra                       HO-003
16:00  lyra    Lyra started a Gemini session                           SES-004
16:05  lyra    Lyra accepted handoff from Aegis                        HO-003
17:00  lyra    Lyra produced SG-01 acceptance verdict: FAIL            AR-023
17:30  lyra    Lyra issued recovery plan                               AR-024
17:30  auto    WI-004 moved to blocked (SG-01 Hold)                    WI-004
18:00  lyra    Lyra adopted file-first coordination                    DECISION
19:00  lyra    Lyra adopted AI-native workflow principles              DECISION
20:00  lyra    Lyra reissued worker packets under AI-native rules      AR-024
22:00  lyra    Lyra session completed                                  SES-004
22:00  aegis   Aegis session completed                                 SES-001
22:00  auto    Checkpoint created for SES-001, SES-004                 CP-002, CP-003

Apr 28
──────
09:00  lyra    Lyra started an OpenCode session                        SES-005
       ──── runtime switch: gemini → opencode ────
09:30  lyra    Lyra issued acting-Mira packet to Flux                  HO-006
10:00  lyra    Lyra produced Multica benchmark analysis                AR-025
10:30  lyra    Lyra conditionally accepted Collaboration Protocol      GATE

10:00  aegis   Aegis started a Claude Code session (rehydrated)        SES-006
       ──── rehydrate from SES-001 via conversation summary ────
11:10  aegis   Aegis produced interaction design review                AR-025
11:10  aegis   Aegis sent review findings to Lyra                      HO-004
11:24  aegis   Aegis produced Collaboration Protocol v1.0              AR-026
11:24  aegis   Aegis sent protocol to Lyra for review                  HO-005
11:35  aegis   Aegis started SeatLoom process mapping                  (this doc)
11:44  aegis   Aegis produced process mapping document                 AR-027
```

---

## 7. Product Gaps Discovered

Three real scenarios from our work that the current product design cannot express:

### Gap 1: Temporary Role Reassignment (Seat Delegation)

**What happened**: Mira went offline. Lyra assigned Flux to act as Mira with constrained authority (contract repair only, no IA redesign).

**Current model limitation**: A Seat has a fixed `role` field. There is no way to express "Flux is temporarily acting as designer with reduced scope."

**Recommendation**: Add a `delegation` concept:
- `Seat.active_delegation`: optional reference to another Seat's role
- Delegation has explicit scope constraints and expiry
- Timeline shows delegation events
- Detail Pane shows delegation badge on the Seat

### Gap 2: Supervisor Context Continuity

**What happened**: My (aegis) session spans two days. On day 2, I resumed via a "conversation summary" — a lossy compression of ~50k tokens into ~5k tokens. This is not the same as a Worker's LaunchPack (which is scoped to one WorkItem).

**Current model limitation**: LaunchPack is designed for Worker sessions resuming one WorkItem. Supervisor sessions carry cross-cutting context (all WorkItems, all Seats, all decisions, all active conflicts). The ContextPack compiler's 8192 token budget is far too small for supervisor rehydration.

**Recommendation**: Add a `SupervisorPack` concept:
- Separate token budget (e.g., 32k)
- Includes: MEMORY.md, active WorkItems summary, unresolved blockers, pending Handoffs, last gate decision
- Triggered on supervisor session start, not on runtime switch
- Optionally tiered: L1 full pack, L2 delta-only (changes since last supervisor checkpoint)

### Gap 3: WorkItem Rejection and Re-scoping

**What happened**: Lyra reviewed WI-004 (Mira's Phase 1) and issued a FAIL verdict. The WorkItem went to `blocked`, but what actually happened was a rejection + re-scoping: the original scope was deemed insufficient, new requirements were added, and the work was re-issued to a different seat (Flux acting as Mira).

**Current model limitation**: The state machine has `blocked` but no `rejected` → `re-scoped` → `re-issued` path. `blocked` implies waiting for an external dependency to clear, not that the work was judged inadequate and needs fundamental revision.

**Recommendation**: Add states to the WorkItem state machine:
- `rejected`: explicit verdict that delivered work does not meet AC
- `re-scoped`: AC or scope has been revised after rejection
- Allow transition: `active → in_review → rejected → re-scoped → ready → active`
- Rejection creates a Ledger event with evidence (link to acceptance verdict artifact)

---

## 8. Product Validation Summary

| Product Capability | Can Express Our Workflow? | Notes |
|-------------------|--------------------------|-------|
| Seat registration | Yes | 5 seats with distinct roles |
| Session lifecycle | Yes | start/run/input_required/complete all occurred |
| WorkItem state machine | Partial | Missing rejected/re-scoped states |
| Handoff chain | Yes | 7 handoffs with clear from/to/purpose |
| Artifact tracking | Yes | 27 artifacts with source session and WorkItem |
| Timeline reconstruction | Yes | Full chronological event chain recoverable |
| Inbox rules | Yes | blocked WI, pending handoffs, input_required all generate inbox items |
| Morning Digest | Yes | Day 2 start would show overnight changes |
| Rehydrate / LaunchPack | Partial | Worker rehydrate works; supervisor rehydrate needs separate design |
| Multi-project switching | Not tested | Only one project in this scenario |
| Seat delegation | No | Flux-as-Mira cannot be expressed |
| Pipeline | Not tested | No automated verification pipeline was run |

---

## 9. Recommended Actions

| Priority | Action | Owner |
|----------|--------|-------|
| High | Add `rejected` and `re-scoped` to WorkItem state machine | Lyra (product) |
| High | Design SupervisorPack for supervisor session rehydration | Lyra (product) + Nimbus (architecture) |
| Medium | Add Seat delegation mechanism | Lyra (product) |
| Low | Use this document's data as realistic mock data for Mira's prototypes | Mira (UX) |

---

*This process mapping serves as both a product validation exercise and a source of realistic test data for SeatLoom's UI prototyping.*
