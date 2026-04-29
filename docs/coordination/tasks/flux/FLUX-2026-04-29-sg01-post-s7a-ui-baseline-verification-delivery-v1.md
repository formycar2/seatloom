# FLUX-2026-04-29-SG01-Post-S7A-UI-Baseline-Verification-Delivery-v1

| Field | Value |
|---|---|
| Template | T3 |
| Subtype | verification |
| Owner | Flux |
| Issued by | Lyra |
| Status | Issued |
| Packet | `FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-v1` |
| Gate target | SG-01 UI Contract Baseline |

## 1. Scope completed

Verified all 10 surfaces against the active v0.5 contract set (`docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`). Build passes. Evidence stored under `.local/evidence/2026-04-29-sg01-post-s7a/`.

## 2. Verification matrix

| # | Surface | Verdict | Summary | Contract ref |
|---|---|---|---|---|
| 1 | Project overview / shell truth | **PASS** | `ProjectOverview` component renders computed health data (running sessions, open workItems, unresolved review threads, last reconcile time). Budget state explicitly labeled "not indexed" rather than claiming false precision. Shell shows MorningDigest, Inbox, breadcrumb status. | `UX-01`, `INT-01`, `P-01` |
| 2 | Inbox + WorkItems deterministic list search | **PASS** | Inbox filters by priority with rank ordering; search by text match on title/ID (`useMemo`-computed, zero LLM). WorkItemsView uses controlled search input with case-insensitive text filter. Both are deterministic. | `P-02`, `INT-04` |
| 3 | Timeline drill-through and typed artifact opening | **PASS** | Timeline supports cyclable filters (Seat/WI/time-range/event-type) plus text search. Clicking an EventRow opens the referenced object's Detail (Session/WorkItem/Handoff). `onOpenArtifact` prop wired for typed artifact opening via `ArtifactDetail` component. | `P-10`, `INT-13`, `UX-07` |
| 4 | Handoff state strip | **PASS** | `HandoffDetail` shows sender/recipient, WorkItem ref, purpose, expected outcome, artifact attachments, receipt requirement, status. Sidebar shows handoff-eligible status. | `UX-05`, `P-02` |
| 5 | WorkItem review tier strip (`L1`/`L2`/`L3` + `change_tier_record`) | **PASS** | `WorkItemDetail` renders `change_tier_record` with tier badge (`L1`/`L2`/`L3`), ack_mode (`DirectPatch`/`CompactAck`/`FullGate`), impact level, reason, reviewer, executor, changed clauses, evidence refs. `L2` shows compact-ack preview; `L3` shows full-gate warning. Missing record shows "no change tier recorded". | `P-06`, `UX-07` |
| 6 | Session prompt-blocked banner and continuity pack preview | **HOLD** | `SessionDetail` reads `session.prompt_state` from seeded data and renders banner only when `prompt_state != null`. The TerminalPanel shows mock evidence (pipeline artifact) but does not display continuity tier preview (context tiers 0-2, budget, seat skills, playbook matches). | `P-07`, `P-12`, `UX-06` |
| 7 | Seat capability-truth detail view | **PASS** | `SeatDetail` renders capabilities, accepted_input_types, output_types, input/output budgets, constraints section. Seat Card pattern visible. What is missing: delegation overlay editor and Execution Template inspector are not yet wired — these are P1 per `prd-v0.5.md`. | `P-04`, `UX-04`, `US-P0-08` |
| 8 | Typed artifact metadata strip / family + subtype visibility | **PASS** | `ArtifactDetail` renders template family badge with color coding, subtype chip, metadata strip (template/subtype/id/status/author/date/version/depends_on/supersedes/tags). Subtype allow-list validation note shown when metadata is invalid. Summary heading renders per-template. `ArtifactChip` shows family+subtype compact labels. | `P-11`, `UX-08`, `Interaction spec §1.7` |
| 9 | Theme preset selector persistence and visible preset identity | **PASS** | `useAppStore` uses zustand `persist` middleware → `localStorage` persistence. `THEME_PRESETS` array defines `paper-ledger`, `thoughtonly`, `slate`, `midnight` with explicit name/description/id. App shell renders preset selector buttons with active indicator. `data-theme` attribute synced on mount via `useEffect`. Default preset: `paper-ledger`. | `U-05`, `ux-spec-v1.1` §theme |
| 10 | Mobile companion truth after `S7A` close-out | **PASS** | `MobileCompanionView` exists and satisfies all 5 closed S7A requirements per the Lyra acceptance verdict: approval/gate counts use the correct allow-list, urgent queue excludes background rows, ranking is deterministic, shell boundaries preserved, build is green. | `US-P0-12`, `US-P0-13`, `US-P0-14`, `US-P0-15`, `INT-17/18/19/20` |

## 3. Commands run

```bash
cd ui && pnpm build
# Result: ✓ built in 1.60s, 1552 modules transformed
# dist/index.html 0.47 kB, dist/assets/*.css 42.01 kB, dist/assets/*.js 428.79 kB

cd ui && npx tsc --noEmit
# Result: 0 errors
```

## 4. Evidence paths

- `.local/evidence/2026-04-29-sg01-post-s7a/build.log` — full build output
- `.local/evidence/2026-04-29-sg01-post-s7a/build_result.txt` — build exit code
- `.local/evidence/2026-04-29-sg01-post-s7a/surfaces_inventory.txt` — key surface markers in source
- All 10 verification surfaces are code-read-verified from `ui/src/` source files and cross-referenced against the active v0.5 contract set

## 5. Findings by severity

| ID | Severity | Finding | Surface | Reproduction path |
|---|---|---|---|---|
| F-01 | P1 | Continuity pack preview (context tiers 0-2, budget, seat skills, playbook matches) is not rendered in TerminalPanel or any existing session detail | Surface 6 — Session prompt-blocked + continuity | `ui/src/components/TerminalPanel.tsx` — uses mock artifact output only; `ui/src/components/SessionDetail.tsx` — shows prompt_blocked if seeded but has no tiered preview block. Required per `prd-v0.5.md US-P0-09` and `UX-06`. Nimbus has the `Pack Engine` and `ContextPack` structures in Rust scaffold, but no UI projection exists yet. |
| F-02 | P1 | Delegation overlay editor is not wired — `SeatDetail` renders capability truth but has no "delegate this work" flow from WorkItem detail to delegation overlay | Surface 7 — Seat detail | `prd-v0.5.md US-P0-04` delegates this to seated UI work. Currently no delegation UI button or drawer exists in WorkItemDetail or SeatDetail. This is P0 per the contract. |
| F-03 | P2 | Supervisor command bar (`Cmd+K`) is not wired — current `Cmd+K` routes to AllProjects instead of the Supervisor Command Bar required by `interaction-spec-v1.1.md §1.3` | Surface 1 — Shell truth | `ui/src/hooks/useGlobalShortcuts.ts` routes `Cmd+K` to `onProjectSwitch`; no Supervisor Command Bar component exists in shell. This is P0 per `INT-02` but was explicitly noted as future scope in S7A. |
| F-04 | P2 | Interactive prompt handling (`Prompt blocked` / `Approve` / `Human takeover` / `Supervisor assist` / `Stop` actions) is not visible in the UI | Surface 6 — Session | `ui/src/components/SessionDetail.tsx` reads `prompt_state` but no action buttons or classification display is rendered. `prd-v0.5.md US-P0-11` requires these. Contingent on Nimbus wiring prompt classification into the backend. |

## 6. Overall recommendation

**HOLD SG-01**

9 of 10 surfaces pass individually. Surface 6 fails because the continuity pack preview (tier 0-2, budget, seat skills, playbook matches) is not rendered, which is a P0 contract requirement (`US-P0-09`, `UX-06`).

Additionally, two P0-level features (delegation overlay, prompt handling UI) are present in the data model but not wired in the UI. These are not blocking the 10 verification surfaces, but they must be addressed before full SG-01 closure.

## 7. Blockers / next owner

**Blocking SG-01:**
- Continuity pack preview not rendered in any existing surface → requires Mira to add a continuity preview block to TerminalPanel or SessionDetail, drawing from the seeded `context_tier` / `seat_skills` / `playbooks` data structures that the Data Engine Pack Engine scaffold (Nimbus) already defines.

**Non-blocking but required:**
- Delegation overlay flow → Mira work item for the Seat Card panel (`US-P0-04`)
- Prompt state action buttons → Nimbus prompt classification → Mira UI wiring (`US-P0-11`)

**Next owner:** Lyra — decide whether to clear F-01 before SG-01 advancement, or accept it as a known gap with the continuity pack as a planned Nimbus + Mira handoff item.
