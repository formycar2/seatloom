# Acceptance: app-v2 viewMode Orthogonal State (chan-09, AD-013 v2)

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-05-08-app-v2-viewmode-chan-09-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-08 |
| version | v1 |
| target | `docs/coordination/tasks/flux/FLUX-2026-05-07-app-v2-viewmode-chan-09-verification-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, nimbus, flux, ui, app-v2, chan-09, viewmode, AD-013-v2, breadcrumb, orthogonal-state |

## Verdict

**PASS**

Lyra accepts app-v2 viewMode orthogonal state (chan-09) as scope-complete and fully verified.

Accepted target:
- branch: `track/infra-foundation`
- Nimbus implementation commit: `cadf36e` (feat: viewMode orthogonal state)
- Nimbus delivery commit: `eb0ddaa`
- verification: Aegis 12/12 static pre-flight PASS + Flux automated layer PASS (tsc/build/scope/mutex) + Mr. Zhang 14/14 browser smoke scenarios PASS (6 new + 8 chan-03 regression)
- verify cycle: single-cycle, zero HOLD events (process improvement first realized)

## Scope Reviewed

- `docs/coordination/reviews/2026-05-07-aegis-supervisor-viewmode-orthogonal-state-design.md`
- `docs/architecture-decisions.md` (AD-013 v2, `4f051b3`)
- `docs/architecture-design.md` (§7.1 supervisorStore v2, `4f051b3`)
- `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-05-07-app-v2-viewmode-chan-09-verification-delivery-v1.md`
- `ui/src/app-v2/types.ts`
- `ui/src/app-v2/panel/SupervisorPanel.tsx`

## Coverage Matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| `SupervisorViewMode = 'dashboard' \| 'chat'` type exists | Layer A scope check | PASS | `types.ts` |
| `enterGlobal()` clears activeProjectId + sets viewMode='dashboard' | Layer A static (line 123-127) | PASS | AD-013 v2 §6 invariant maintained |
| `enterProject()` sets contextMode + activeProjectId + viewMode='dashboard' | Layer A static (line 129-133) | PASS | |
| `enterProjectFromBreadcrumb()` syncs activeContactId to channel (Lyra opinion #1) | Layer A static (line 136-140) | PASS | Contact list highlight ↔ right pane alignment guaranteed |
| `switchContact` for seat/supervisor sets only viewMode='chat', does NOT mutate contextMode | Layer A static (line 191) | PASS | Key behavioral change from chan-03; explicitly verified |
| AD-013 v1 mutex invariant `contextMode==='global' ⇔ activeProjectId===null` preserved | Layer A static | PASS | Holds across all mutators by construction |
| Three localStorage keys present with correct stale fallback | Layer A static (line 29-31) | PASS | `seatloom.supervisor.contextMode/.activeProjectId/.viewMode`; viewMode fallback → `'dashboard'` |
| Pre-existing localStorage keys preserved | Layer A static | PASS | `sl-supervisor-*` series intact |
| TypeScript clean | `npx tsc --noEmit` | PASS | Zero errors |
| Build clean | `pnpm build` | PASS | 353.04 kB / 102.35 kB gzip |
| Chat with seat → 3-level breadcrumb `全局 › <project> › <seat>` | Scenario 9 (Mr. Zhang) | PASS | contextMode untouched during chat |
| Chat with Supervisor → 1-level breadcrumb `全局` | Scenario 10 (Mr. Zhang) | PASS | |
| Mid-link click in chat → ProjectDashboard + contact list highlight syncs | Scenario 11 (Mr. Zhang) | PASS | `enterProjectFromBreadcrumb` wired correctly |
| `全局` breadcrumb click from chat → full global reset | Scenario 12 (Mr. Zhang) | PASS | |
| Cross-project divergence (dashboard p-1 + chat p-2) is legal state | Scenario 13 (Mr. Zhang) | PASS | Breadcrumb projectName from seat contact, not activeProjectId |
| Mid-link visual affordance (hover cursor + color) | Scenario 14 (Mr. Zhang) | PASS | |
| Chan-03 8-scenario regression | Scenarios 1–8 (Mr. Zhang) | PASS | All 8 pass; GlobalDashboard/ProjectDashboard/tabs/artifact panel unaffected |

## Findings

### Closed: chan-09 (viewMode 正交状态)

AD-013 v2 fully implemented and verified:
- `SupervisorViewMode` type in `types.ts`
- `SupervisorPanel.tsx` (net +202/−112 lines): `viewMode` state + 3-localStorage-key persistence + breadcrumb 4-case render + mid-link affordance + `enterProjectFromBreadcrumb` + decoupled `switchContact`
- Lyra opinion #1 (project channel activeContactId sync) resolved via `enterProjectFromBreadcrumb` — no contact list highlight / right pane mismatch possible
- Lyra opinion #2 (localStorage key清单) resolved via AD-013 v2 §7 complete 6-key table

### Process improvement adopted

**Combined two-layer verify is endorsed as the default for all future UI verify packets.**

Comparison:

| Metric | chan-03 (old flow) | chan-09 (new flow) |
|---|---|---|
| Intermediate HOLD events | 1 | 0 |
| Delivery commits | 2 | 1 |
| Verify cycle | Multi-day | 1 day |

Future UI task packets should structure verify as a single document with explicit §Layer-A (Flux automated: tsc/build/scope/static checks) and §Layer-B (Mr. Zhang: numbered browser scenarios) sections, both gated before the single verdict is written.

### Non-blocking notes

1. `viewMode` uses local `useState` in `SupervisorPanel` (same pattern as chan-03). If cross-component reads become necessary, a `supervisorStore` Zustand slice should be introduced via a separate AD — not from this packet.
2. The cross-project divergence state (dashboard p-1 + chat p-2) is explicitly legal per AD-013 v2 §7. No forced sync between chat context and dashboard context.

## Required Fixes

None.

## Go / No-Go Recommendation

- **Close chan-09 as final PASS:** **GO**
- **Mark chan-09 closed in AEGIS-2026-04-30-pending-changes-register:** **GO**
- **Adopt combined two-layer verify as default for UI packets:** **GO**
- **Reopen or widen this packet:** **NO-GO**
- **Introduce supervisorStore Zustand slice from this packet:** **NO-GO**

## Evidence Paths

- Design proposal: `docs/coordination/reviews/2026-05-07-aegis-supervisor-viewmode-orthogonal-state-design.md`
- Architecture decision: `docs/architecture-decisions.md` (AD-013 v2)
- Nimbus task: `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-v1.md`
- Flux combined verify delivery: `docs/coordination/tasks/flux/FLUX-2026-05-07-app-v2-viewmode-chan-09-verification-delivery-v1.md`
- Implementation: `ui/src/app-v2/panel/SupervisorPanel.tsx`, `ui/src/app-v2/types.ts`
