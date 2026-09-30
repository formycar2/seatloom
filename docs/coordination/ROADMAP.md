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
| Workbench (CI, gates, 6 seats) | **DONE** — hard gate green on GitLab `track` |
| runtime-triple (harness/agent/model split) | **DONE** — reconciled to GitLab |
| **P0 step 1 — schema foundation (010)** | **DONE** — Flux PASS, merged, Lyra accepted, reconciled to GitLab (`231d0a5`) |
| **P0 step 2 — write API (A1)** | **DONE** — Flux PASS, merged, Lyra accepted, reconciled to GitLab (`b691ca4`). `append(event)→projections`, projections rebuildable from ledger |
| **A — live mirror (UM0)** | **DELIVERED, BLOCKED** — Mira's UI proven correct; blocked by a backend bug: `pty/mod.rs` uses `tmux pipe-pane -o` (a toggle), so re-attach kills the mirror. Fix dispatched to Nimbus (drop `-o`). Reliable UM0 = A + fix both merged, then Aegis restarts the shared UI |
| P0 steps 4 (MCP), 6 (dispatch), 5 (first event) | **QUEUED** |
| P1 / P2 | not started |

**Blocked-on-whom right now:** UM0 waits on Nimbus (pipe-pane `-o` re-attach fix); A (Mira) is
delivered and in Flux's queue. No decision is blocked on Mr. Zhang or Aegis.

## 2. Phases

> **Governing principle (Mr. Zhang, 2026-09-30):** every milestone ships a
> **user-perceivable capability**. Pure-plumbing steps (schema, write API, MCP) are
> sub-tasks *inside* a milestone, never milestones themselves. Every milestone's
> acceptance carries a "the user can now see / do X" clause. §2b is the acceptance lens;
> §3's engineering steps are the scaffolding beneath it.

| Phase | What it is | Solves | Done when | Status |
|---|---|---|---|---|
| **0. Workbench** | CI, quality gates, 6 seats on one harness | collaboration safety | hard gate green, red=no-merge enforced | ✅ done |
| **R. runtime-triple** | split `Runtime` into harness/agent/model | harness-agnostic identity groundwork | both sides on GitLab track, CI green | ✅ done |
| **P0. Capture loop** | take over the live agent loop; record it into one harness-agnostic data model | "the record, not the harness, holds the truth" | one real event exists because an agent did something, filed under project+topic, linked, retrievable, harness-independent (see §3) | 🔵 in progress (step 1) |
| **P1. Compounding** | richer capture + capability assets | reasoning/search/RAG traces; published Playbook/Skill/Tool; auto topic-segmentation | (scoped when P0 lands) | ⏳ queued |
| **P2. History & cost** | ingest the ~16 GB corpus; budget/token accounting | findability of past work; cost discipline | (scoped later) | ⏳ deferred |

## 2b. User-perceivable milestones (the acceptance lens)

Each rung is something the user can **see or do** that they could not before. Engineering
steps (§3) slot beneath a rung; a rung is "done" only when the capability is perceivable.

| Milestone | What the user can now do (couldn't before) | Backed by | Status |
|---|---|---|---|
| **UM0 — one screen for the whole team** | open SeatLoom, watch all 6 seats' live terminals + IM in one window, instead of attaching 6 tmux sessions | existing tmux mirror + viewer | **available now — not yet switched on** |
| **UM1 — work leaves a clickable trace** | when a seat produces an artifact, a structured, clickable record appears in the timeline (filed under project + commit), not just terminal scroll | step 1 ✅ + steps 2, 4, 5 | step 1 done |
| **UM2 — progress at a glance** | dispatches / handoffs / verdicts render as a live work-graph: who's doing what, who's blocked on whom, what's queued — the UI answers "how is progress?" at a glance | step 6 governed dispatch + capture generalized | queued |
| **UM3 — find & replay** | "show me everything about topic X across sessions/harnesses"; deterministic search + T1→T2→T3 zoomable replay | P1 | not started |
| **UM4 — capability library** | see and reuse the team's published Playbooks / Skills / Tools | P1 | not started |
| **UM5 — history & cost** | the ~16 GB of past work is findable; per-project token spend is visible | P2 | deferred |

Interim reality until UM2: this ROADMAP (a markdown file) + Aegis are the "progress view";
the destination is the SeatLoom UI rendering it from the captured event model.

## 3. P0 build — the six steps

P0 delivers **one true thing that does not exist today**: a record in SeatLoom because an
agent, on some harness, did something — filed under its project and topic, linked to what it
touched, retrievable, and independent of which harness produced it.

| Step | Work | Owner | Done when | Status |
|---|---|---|---|---|
| 1 | **Schema foundation** — event envelope (G2), ref roles (G3), G5 verify, topics (G6), turn key (G12) | Onyx | migration idempotent; value-domain contract enforced; seed still loads | 🔵 vocab edit done, awaiting Flux verify |
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

## 4b. Dev-velocity TODO (iteration-speed optimizations)

Running list — any optimization that speeds the build/test/coordinate loop is logged here as it
is observed (Mr. Zhang, 2026-09-30). These are dev-ergonomics, not product gaps; none blocks a
milestone, each pays back time on every cycle.

| # | Optimization | Bottleneck it kills | Effort | Status |
|---|---|---|---|---|
| DV1 | **Shared compile cache** — `sccache` (`RUSTC_WRAPPER`, per-crate cache, concurrency-safe) across all worktrees + CI; or `CARGO_TARGET_DIR` (simpler, but serializes concurrent builds on a target lock) | each worktree cold-compiles ~450 deps from zero; repeated per worktree and per CI pipeline | S | **proven recipe (no infra): `cp -Rc <shared>/target ./target` in a fresh worktree — APFS copy-on-write, 18s + ~0 disk; then `tauri dev` rebuilt 2/452 and ran in 36s. Verify backend identical first (`git diff --stat`). sccache/CI still todo.** |
| DV2 | **Local pre-push gate dry-run** — `scripts/preflight.sh` = `cargo fmt --check` + `clippy` + `verify-docs.sh` + `cargo test` before push | red-pipeline round-trips on reconcile (MR !4 took 2) | S | todo |
| DV3 | **Doc-header conformance at source** — seat pre-commit hook running `verify-docs.sh`, and/or `version` baked into the doc template seats copy | every reconcile Aegis hand-fixes seat docs missing the `version` header (recurring tax) | S | todo |
| DV4 | **Worktree bootstrap script** — `scripts/seat-worktree.sh <seat>` creates the worktree + wires `node_modules` + shared target in one step | manual per-seat `git worktree add` + `pnpm install` each time | S | todo |
| DV5 | **Parallel DB tests** — per-test transaction/schema isolation to drop the `--test-threads=1` serial constraint | DB integration tests run serially (race avoidance) | M | todo |
| DV6 | **CI cargo cache** — cache registry + incremental `target` across pipelines (GitLab cache key), beyond the baked image | CI rebuilds from cold each pipeline | S–M | todo |

Related but tracked elsewhere: coordination routing latency (Aegis→Lyra→worker takes minutes per
hop) is addressed by **G19 governed dispatch** (§4, P0 step 6), not duplicated here.

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
