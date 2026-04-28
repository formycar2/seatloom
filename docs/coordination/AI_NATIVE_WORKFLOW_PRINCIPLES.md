# AI Native Workflow Principles v1.0

| Item | Value |
|------|-------|
| Document | AI Native Workflow Principles v1.0 |
| Status | Active |
| Updated | 2026-04-27 |
| Owner | Lyra |

---

## 1. Why this exists

Seat-based AI collaboration is not the same as human-only project management.
LLM seats are bounded by context window, token cost, latency, and output limits.

The workflow must optimize for:

- minimum necessary context
- deterministic handoffs
- high signal-to-token ratio
- verifiable outputs over verbose narration

---

## 2. Core principles

### P1. Need-to-know by default

Non-supervisor seats should not consume global project context by default.
They receive only:

- assigned task packet
- required dependencies
- acceptance contract

### P2. Passive execution first

Worker seats (Mira/Nimbus/Flux) are packet-driven by default.
Active discovery is allowed only when:

- dependency is blocked
- required evidence is missing
- acceptance needs explicit cross-object lookup

### P3. Two-layer instruction protocol

Every instruction should have:

1. Micro-brief (short, actionable, <=10 lines)
2. Contract pack link(s) (full constraints in files)

This prevents overloading live context while preserving precision.

### P4. Token budget as policy

Each seat run should have explicit budget targets:

- input token budget
- output token budget
- truncation strategy when budget is exceeded

### P5. Minimum verifiable output

Preferred output shape:

- changed files
- flow checklist
- evidence links
- blockers + owner + due date

Long freeform narrative is discouraged unless explicitly requested.

### P6. Supervisor-only global view

Global timeline/risk/priority view is owned by supervisor seats (Lyra/Aegis).
Worker seats should not continuously ingest unrelated seat activity.

### P7. Delta-only synchronization

Seats should consume only "what changed since last accepted state".
Avoid full-history replay unless required for diagnosis.

---

## 3. Role behavior model

### Lyra (Supervisor PO)

- owns global priorities, gate decisions, conflict resolution
- issues task packets with owner/deadline/done-definition
- enforces language and persistence policy

### Mira/Nimbus (Worker seats)

- execute scoped packets
- avoid unsolicited global analysis
- return compact verifiable outputs

### Flux (Verifier seat)

- validates against acceptance contracts
- publishes reproducible runbook and evidence
- does not redefine product scope

---

## 4. Workflow protocol

1. Lyra publishes/updates contract files.
2. Lyra issues role-specific packet.
3. Worker seat executes packet within scoped context.
4. Worker writes artifact file and short Chinese terminal summary.
5. Flux verifies evidence against acceptance spec.
6. Lyra issues PASS/FAIL and updates coordination memory.

---

## 5. What to avoid

- broadcasting all-seat live activity to all seats
- replacing packet contracts with long chat threads
- accepting completion claims without artifact links
- allowing critical decisions to exist only in terminal scrollback

---

## 6. Adoption checklist

- [x] Lyra confirms this principles file as active workflow baseline
- [x] Lyra issues packet updates to Mira/Nimbus/Flux referencing this file
- [x] All new artifacts are in English
- [x] All terminal summaries are in Chinese
- [x] Gate decisions include token-efficiency and context-scope compliance

---

*This document defines AI-native collaboration behavior for SeatLoom and complements COORDINATION_RULES.md.*
