# SeatLoom Roadmap — the single living plan

| Field | Value |
|---|---|
| template | T7 |
| subtype | governance |
| id | ROADMAP |
| status | living — updated as state changes |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| tags | roadmap, plan, status, gap-register, p0 |

## 0. How to read this

This is the **single entry point** to what SeatLoom is building and where it stands.
It is a dashboard, not a design doc — the "why/how" lives in the deep design reviews,
linked from each phase. When status changes, **this file is updated** (by Aegis), so a
glance here answers "what's done, what's in flight, what's blocked, on whom."

Deep references (unchanged, authoritative for detail):
`reviews/2026-09-29-aegis-platform-design-macro-to-micro-v1.md` (abstraction A1–A7),
`reviews/2026-09-30-aegis-p0-scope-takeover-and-capture-v1.md` (P0 thesis),
`reviews/2026-09-30-aegis-data-flow-and-layering-v1.md` (capture tiers T1–T3),
`reviews/2026-09-30-aegis-object-model-and-traceability-v1.md` (object hierarchy, topic),
`reviews/2026-09-30-aegis-p0-build-plan-and-gap-register-v1.md` (P0 build detail; its §1
gap register is now superseded by §4 below as the living list).

## 1. Current status — the live view (2026-09-30)

| Layer | State |
|---|---|
| Workbench (CI, gates, 6 seats) | **DONE** — hard gate green on GitLab `track` (`b6de097`); lint/test/docs stages |
| runtime-triple (harness/agent/model split) | **DONE** — both sides merged + reconciled to GitLab `b6de097` |
| **P0 step 1 — schema foundation (Onyx, migration 010)** | **IN FLIGHT** — delivered (`bc526ba`), Aegis signed off (vocab=12). **Next actor: Onyx** (one-line vocab edit) → Flux verify → Lyra merge → Aegis reconcile to GitLab |
| P0 steps 2–6 | **QUEUED** — serialize behind step 1 on the one shared working tree |
| P1 / P2 | not started |

**Blocked-on-whom right now:** nobody is blocked on a decision. Loop is with the seats
(Onyx → Flux → Lyra). Aegis's next action fires when 010 merges to local `track`.

## 2. Phases

| Phase | What it is | Solves | Done when | Status |
|---|---|---|---|---|
| **0. Workbench** | CI, quality gates, 6 seats on one harness | collaboration safety | hard gate green, red=no-merge enforced | ✅ done |
| **R. runtime-triple** | split `Runtime` into harness/agent/model | harness-agnostic identity groundwork | both sides on GitLab track, CI green | ✅ done |
| **P0. Capture loop** | take over the live agent loop; record it into one harness-agnostic data model | "the record, not the harness, holds the truth" | one real event exists because an agent did something, filed under project+topic, linked, retrievable, harness-independent (see §3) | 🔵 in progress (step 1) |
| **P1. Compounding** | richer capture + capability assets | reasoning/search/RAG traces; published Playbook/Skill/Tool; auto topic-segmentation | (scoped when P0 lands) | ⏳ queued |
| **P2. History & cost** | ingest the ~16 GB corpus; budget/token accounting | findability of past work; cost discipline | (scoped later) | ⏳ deferred |

## 3. P0 build — the six steps

P0 delivers **one true thing that does not exist today**: a record in SeatLoom because an
agent, on some harness, did something — filed under its project and topic, linked to what it
touched, retrievable, and independent of which harness produced it.

| Step | Work | Owner | Done when | Status |
|---|---|---|---|---|
| 1 | **Schema foundation** — event envelope (G2), ref roles (G3), G5 verify, topics (G6), turn key (G12) | Onyx | migration idempotent; value-domain contract enforced; seed still loads | 🔵 delivered, in verify |
| 2 | **Write API (A1/G1)** — `append(event)→projections`, single txn | Nimbus | a projection rebuildable from `canonical_events` alone | ⏳ queued |
| 3 | **`external_ref` (G4)** — pin seat sessions, harness-agnostic identity | Onyx+Nimbus | a session resolves to its object regardless of harness | ⏳ queued |
| 4 | **MCP emit server (G7)** — `emit_event(type, payload, refs)`, schema-validated | Nimbus | a seat's `--mcp-config` points at it; malformed event rejected | ⏳ queued |
| **6** | **Governed dispatch (G19)** — per-seat inbox + explicit `depends_on` + ready-aware scheduler (one-in-flight, pull next *ready* item, blocked items don't head-of-line-block) + cycle/stall detection → escalate | Nimbus | a seat receives the next message only when idle and its deps are satisfied; a cross-seat dependency cycle is detected and surfaced to Aegis, not hung; delivery recorded as events; no mid-turn injection | ⏳ **elevated to P0** (2026-09-30, Mr. Zhang) |
| 5 | **First slice** — one seat emits one real `ArtifactCreated` (`commit`+`digest`), lands via write API, links to workitem+topic+project, retrievable | Nimbus + Flux verify | a human queries the event and traverses to the artifact and back | ⏳ queued (acceptance of P0) |

Critical path: 1 → 2 → 4 → 6 → 3 → 5 (step 6 rides on 2+4; step 5 is the acceptance).

## 4. Gap register (living — supersedes build-plan §1)

| # | Gap | Priority | Status |
|---|---|---|---|
| G1 | Event as sole write primitive (A1) | P0 (step 2) | queued |
| G2 | Typed + versioned event envelope (A2) | P0 (step 1) | 🔵 in 010 |
| G3 | `event_object_refs.role` (A3) | P0 (step 1) | 🔵 in 010 |
| G4 | `external_ref` (A4) | P0 (step 3) | queued |
| G5 | `sessions.project_id` | P0 — **verify only** (008 added it) | 🔵 confirmed in 010 |
| G6 | Topic layer (table + event→topic ref) | P0 reserve (step 1) / P1 auto-segment | 🔵 in 010 |
| G7 | MCP emit server | P0 (step 4) | queued |
| G8 | File events reference git `(commit, digest)` | P0 (step 5) | queued |
| **G19** | **Governed dispatch** — per-seat inbox + explicit `depends_on` + ready-aware scheduler + cycle/stall detection→escalate (one-in-flight kills context pollution; dependency-awareness prevents head-of-line blocking; only a true cross-seat cycle deadlocks, and SeatLoom detects it because it holds the dispatch graph as data) | **P0 (step 6)** — elevated; auto-resolution/preemption → P1 | queued |
| G9 | Dual-authority resolution (A5) | P1 | deferred |
| G10 | Typing de-duplication (A6) | P1 | deferred |
| G11 | Capability asset (Playbook/Skill/Tool), published+versioned | P1 | deferred |
| G12 | T2 process traces; reserve `(session_id, turn_seq)` now | P0 reserve (step 1) / P1 capture | 🔵 key in 010 |
| G13 | Recording shell / terminal capture | P1 | deferred |
| G14 | Standing commitment / recurring task | P1 | deferred |
| G15 | SUT-version dimension for test results | P1 | deferred |
| G16 | Budget as first-class object (A7) | P2 | deferred |
| G17 | Historical corpus ingestion (~16 GB) | P2 | deferred |
| G18 | Interposition proxies / egress gateway | P2 | deferred |

## 5. Coordination model (so escalations don't evaporate)

- **Aegis is the supervisor session** (Mr. Zhang's assistant), **not a tmux seat.** There is
  no `Aegis-*-seatloom` seat by design. Seats escalate by parking in the daily log and/or via
  Mr. Zhang; Aegis observes `track` + daily log and dispatches decisions to Lyra directly.
  (COORDINATION_RULES §12 to note that the supervisor is a session, not a seat.)
- **Dispatch chain:** Aegis → Lyra (PO) → worker seats. Lyra authors packets; Aegis signs off
  load-bearing/irreversible decisions; Flux verifies; Lyra merges to local `track`.
- **One shared working tree:** never two code packets in flight; serialize. (This is *code*
  serialization; *message* serialization is G19, step 6.)
- **GitLab reconcile is Aegis-only** (only Aegis holds the PAT); local `track` → GitLab `track`
  via MR through the hard gate.

## 6. The irony, stated plainly

Until G19 and the P0 capture loop land, this roadmap is a **markdown file a human updates** and
seat coordination is **human-paced tmux send-keys** — exactly the manual, attention-burning,
pollution-prone loop SeatLoom exists to replace. The plan is to make SeatLoom itself hold this
view and run this loop. This file is the interim; the SeatLoom UI over the captured event model
is the destination.
