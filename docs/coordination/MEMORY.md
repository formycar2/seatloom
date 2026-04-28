# MEMORY - Project Governance and Milestone Log

This file tracks major decisions, sign-offs, and state transitions for the project.

## Project Metadata

- **Sponsor:** Mr. Zhang
- **PO:** Lyra
- **Architect:** Nimbus
- **Designer:** Mira
- **Lead Engineer:** Flux

## Durable Decisions

| Date | Decision | Rationale | Owner |
| :--- | :--- | :--- | :--- |
| 2026-04-24 | Team Onboarding | Lyra/Nimbus/Mira/Flux transitioned to seatloom | Lyra |
| 2026-04-27 | Product form: Desktop app + CLI | Desktop GUI (Tauri 2 + React) as primary, CLI for scripting. Ref: Claude Code Desktop, Cherry Studio | Aegis / Mr. Zhang |
| 2026-04-27 | Tech stack: Tauri 2 + React/TS + Rust | 3-10MB bundle, 40-80MB RAM, sub-500ms startup. Electron rejected for resource overhead | Aegis / Mr. Zhang |
| 2026-04-27 | 13 MVP scenarios defined | All user-facing scenarios documented with GUI interactions in mvp-scenarios.md v2.0 | Aegis / Mr. Zhang |
| 2026-04-27 | ContextPack: pure rules, no LLM in MVP | Selector→Budgeter→Assembler→Verifier, 8192 token budget, deterministic | Aegis / Mr. Zhang |
| 2026-04-27 | Checkpoint: 2 triggers in MVP | Session end + Artifact output. Timed snapshot → P1 | Aegis / Mr. Zhang |
| 2026-04-27 | Pipeline: foreground sync execution in MVP | No daemon. GUI real-time progress. Daemon mode → P1 | Aegis / Mr. Zhang |
| 2026-04-27 | Reconciliation: 3 trigger points | App launch + pre-Pipeline + manual. No file watcher in MVP | Aegis / Mr. Zhang |
| 2026-04-27 | PRD v0.2 and v0 archived | Superseded by v0.3 | Aegis |
| 2026-04-27 | Mira role: React component prototypes | UX expressed as React/TS code for Tauri webview, not Figma | Aegis / Mr. Zhang |
| 2026-04-27 | Architecture Design v1.0 drafted | Cargo workspace, core types, Tauri IPC, PTY, storage, 4-phase plan. Pending Nimbus/Mr. Zhang review | Aegis (for Nimbus) |
| 2026-04-27 | UX Spec v1.0 drafted | Full screen specs (Sidebar, Inbox, Timeline, WorkItems, Detail Pane, Terminal, Dialogs, Pipeline), 30 components, 3-phase delivery. Pending Mr. Zhang review | Aegis (for Mira) |
| 2026-04-27 | PRD v0.4 drafted (design-first contract) | Shift from MVP feasibility narrative to pain-point closure. Added state machines, inbox rules, capability guarantees, acceptance criteria | Aegis / Mr. Zhang |
| 2026-04-27 | Acceptance Spec v1.0 drafted | Unified PO/UI/Engineering/QA checklists and Go/Hold rules for baseline gate | Aegis / Mr. Zhang |
| 2026-04-27 | Interaction Spec v1.0 drafted | Defined trigger/success/failure flows for init, attach/wrap, inbox, timeline, workitem/handoff, rehydrate, recovery | Aegis / Mr. Zhang |
| 2026-04-27 | Multi-project switching promoted to P0 | Project switcher/recent projects/state memory/switch protection/all-projects summary are required in first delivery | Aegis / Mr. Zhang |
| 2026-04-27 | File-first coordination adopted | All substantial outputs require durable writeback under `docs/coordination/`; terminal output is summary-only with artifact paths | Lyra |
| 2026-04-27 | File-first coordination protocol adopted | Durable writeback required for all key outputs. Terminal messages become summaries + artifact paths | Aegis / Lyra / Mr. Zhang |
| 2026-04-27 | Active product contract frozen for current cycle | Execution contract = `docs/prd-v0.4.md` + `docs/interaction-spec-v1.0.md` + `docs/acceptance-spec-v1.0.md`; approved constraints = `docs/mvp-scenarios.md` + `docs/architecture-decisions.md` | Lyra |
| 2026-04-27 | Team precedence rule frozen | Approved `docs/` -> latest accepted coordination artifacts -> draft artifacts -> terminal/chat never alone | Lyra |
| 2026-04-27 | Mira redesign rejected for SG-01 baseline | Acceptance verdict = `FAIL`; shell, Inbox, Timeline, Handoff, and keyboard contract gaps block Nimbus full handoff | Lyra |
| 2026-04-27 | SG-01 UI Contract Baseline placed on Hold | Four P0 contract conflicts and Mira P0 UI gaps remain unresolved in files | Lyra / Aegis |
| 2026-04-27 | AI-native workflow principles adopted (v1.0) | Need-to-know default, packet-driven execution, token-efficiency, supervisor-only global view, delta-only sync | Aegis / Lyra / Mr. Zhang |
| 2026-04-27 | Language policy frozen | Persistent artifacts in English; terminal/screen updates in Chinese | Aegis / Lyra / Mr. Zhang |
| 2026-04-27 | SG-01 worker packets reissued under AI-native rules | Mira/Nimbus/Flux packets now require micro-briefs, need-to-know scope, token budgets, delta-only sync, English artifacts, and Chinese terminal summaries | Lyra |
| 2026-04-27 | Drift signal policy clarified | Drifted WorkItems belong in Inbox; no standalone floating drift alert; only transient reconcile-summary notice is allowed after explicit reconcile flows | Lyra |
| 2026-04-27 | Mira seat migration requires checkpoint-first handoff | Before replacing a saturated seat/runtime, freeze the current seat and create one durable-source checkpoint commit that excludes generated artifacts | Lyra |
| 2026-04-28 | Flux temporarily reassigned as acting Mira seat | Mira is offline; Flux receives a constrained UI/UED contract-repair packet under Lyra supervision with no authority to redesign the product IA | Lyra |
| 2026-04-28 | Frontend demo refreshed to a Chinese high-density baseline | Core demo content now follows the real `2026-04-28` coordination narrative; `ui/src/stores/useDataStore.ts` is authoritative, `ui/src/mockData.ts` is compatibility-only, and locale fallback no longer reintroduces English | Lyra |

## Milestone Status

- [x] PRD v0.3 Audit & Alignment (2026-04-27, Aegis)
- [x] Architecture Design v1.0 Draft (2026-04-27, Aegis for Nimbus)
- [x] PRD v0.4 Draft (2026-04-27, Aegis for product-contract baseline)
- [x] Acceptance Spec v1.0 Draft (2026-04-27, Aegis)
- [x] Interaction Spec v1.0 Draft (2026-04-27, Aegis)
- [x] File-First Coordination Adoption (2026-04-27, Lyra)
- [x] Product Contract Alignment Review (2026-04-27, Lyra)
- [x] Mira UI Acceptance Review v1 Issued - FAIL / NO-GO (2026-04-27, Lyra)
- [x] SG-01 Recovery Plan Issued (2026-04-27, Lyra)
- [x] SG-01 Role Task Packets Issued (2026-04-27, Lyra)
- [x] SG-01 Role Task Packets Reissued - AI-native v2 (2026-04-27, Lyra)
- [x] Flux Acting-Mira Contract Repair Packet Issued (2026-04-28, Lyra)
- [x] Frontend Demo Chinese High-Density Refresh (2026-04-28, Lyra)
- [ ] Architecture Design Baseline (pending review)
- [ ] Product Baseline Freeze (pending P0 conflict resolution)
- [ ] SG-01 UI Contract Baseline (HOLD pending Mira recovery and re-review)
- [ ] Nimbus Implementation Handoff (blocked by SG-01 Hold)
- [ ] Flux Evidence Pack Ready
- [ ] MVP Implementation Start

---
*Last Updated: 2026-04-28*
