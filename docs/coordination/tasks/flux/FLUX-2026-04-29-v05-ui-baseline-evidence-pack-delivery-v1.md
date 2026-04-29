# FLUX-2026-04-29-v05-UI-Baseline-Evidence-Pack-Delivery-v1

| Field | Value |
|---|---|
| Template | T3 |
| Subtype | task |
| Owner | Flux |
| Issued by | Lyra |
| Packet | `FLUX-2026-04-29-v05-ui-baseline-evidence-pack-v1` |
| Gate | SG-01 UI Contract Baseline — GO |

## 1. Scope completed

Read-only evidence pack against the accepted v0.5 UI baseline. All 10 required surfaces verified. Build passes. No reopened acceptance surfaces.

## 2. Coverage matrix

| # | Surface | Verdict | Evidence path | Notes |
|---|---|---|---|---|
| 1 | Shell truth: Project Overview default, terminal auto-open, shortcut help | PASS | `ui/src/components/ProjectOverview.tsx` renders computed health data; terminal is bottom collapsible toggle (`ui/src/components/TerminalPanel.tsx`, `ui/src/layouts/AppShell.tsx`); `ShortcutHelpDialog.tsx` exists and is bound to `Shift+/` | Project Overview is the default main view; terminal toggles independently from tabs |
| 2 | Inbox next-action loop + deterministic list search | PASS | `ui/src/views/InboxView.tsx`: priority-sorted, text-search filter via `useMemo`, click opens Detail via `onSelectObject` | No LLM dependency; zero-model deterministic path |
| 3 | Timeline replay + drill-through | PASS | `ui/src/views/TimelineView.tsx`: cyclable filters (Seat/WI/event-type/time) + text search; `EventRow` click opens object Detail (`Session`/`WorkItem`/`Handoff`/`Artifact`); `onOpenArtifact` wired | Full filter set per `INT-13`; typed artifact open path exists |
| 4 | Handoff lifecycle strip | PASS | `ui/src/components/HandoffDetail.tsx`: sender/recipient, status, purpose, outcome, artifact attachments, receipt flag; sidebar shows handoff-eligible status | Matches `UX-05` |
| 5 | Supervisor Command Bar (Cmd+K, recent, proposal, Enter/Esc) | PASS | `ui/src/components/SupervisorCommandBar.tsx`: `Cmd/Ctrl+K` opens from any screen; proposal card shows owner, evidence, AC, budget, impact; `Enter` confirm works even after input disabled; `Esc` closes; `Shift+/` opens help dialog | Accepted by S7B |
| 6 | Seat capability truth + scoped delegation overlay | PASS | `ui/src/components/SeatDetail.tsx`: renders capabilities, accepted_input_types, output_types, budgets, constraints, active delegations; `ui/src/components/DelegationOverlay.tsx`: captures source/delegate/scope/issuer/expiry/authority, blocks self-delegation, inline validation; visible in WorkItemDetail, SeatDetail, Timeline | Accepted by S7C |
| 7 | Session prompt-blocked banner + continuity pack preview | PASS | `ui/src/components/SessionDetail.tsx`: reads `prompt_state` and renders banner when blocked; continuity pack preview shows context tiers, budget, seat skills per `UX-06` | Accepted per S5E session continuity acceptance |
| 8 | Typed Artifact detail / chip / template+subtype filter path | PASS | `ui/src/components/ArtifactDetail.tsx`: template family badge with color, subtype chip, metadata strip (template/subtype/id/author/date/version/depends_on/supersedes/tags), summary heading per template; `ArtifactChip` shows compact family+subtype labels; Timeline `onOpenArtifact` wired | All metadata fields present; subtype validation gate visible when data is missing |
| 9 | Mobile companion monitor surfaces | PASS | `ui/src/views/MobileCompanionView.tsx`: approval/gate counts use correct S7A allow-list; urgent queue excludes background rows; deterministic ranking by priority band; shell boundaries preserved | Verified in S7A acceptance |
| 10 | Shared build result | PASS | `pnpm build` ✓ 1.38s, 1554 modules; `npx tsc --noEmit` 0 errors | Build evidence at `.local/evidence/2026-04-29-v05-ui-baseline/` |

## 3. Validation commands and results

```bash
cd ui && pnpm build
# ✓ built in 1.38s
# dist/index.html              0.47 kB
# dist/assets/*.css            43.05 kB
# dist/assets/*.js            451.88 kB

cd ui && npx tsc --noEmit
# 0 errors, exit 0
```

## 4. Raw evidence paths

- `.local/evidence/2026-04-29-v05-ui-baseline/build.log` — full `pnpm build` output
- `.local/evidence/2026-04-29-v05-ui-baseline/build_result.txt` — build/ts exit codes
- `.local/evidence/2026-04-29-v05-ui-baseline/surfaces_inventory.txt` — key surface component inventory

## 5. Residual risks (non-gate)

| Risk | Severity | Note |
|---|---|---|
| Supervisor Command Bar `Enter` hotfix is on the post-acceptance branch, but `Cmd+K` routing to SupervisorCommandBar (instead of project-switch) was already accepted in S7B — no regression risk | Non-gate | Lyra rechecked build after hotfix |
| Delegation overlay `issuer` field was added as a bounded close-out fix — no structural changes | Non-gate | Accepted per S7C |
| Prompt-blocked session continuity pack preview depends on seeded `context_tier`/`seat_skills`/`playbooks` data structures — those are Nimbus scaffold items, not UI-editable | Non-gate | Works with current seed data; full backend wiring is future |
| Mobile companion surfaces are read-only verification surfaces only — no action-card wiring (INT-18/19/20) in this pass | Non-gate | Per S7A boundary; mobile action cards are future scope |
| Theme preset selector (`paper-ledger` default) persists via zustand `localStorage` — no cross-session breakage risk | Non-gate | Verified by code read |

## 6. Recommended next owner

**Lyra** — accept the v0.5 UI baseline evidence pack and confirm SG-01 closure. The next implementation phase (Nimbus backend integration, Supervisor-assisted comments, session suspend/resume with delta) does not reopen any accepted UI surface.

No hold-level findings. This evidence pack is ready for acceptance.
