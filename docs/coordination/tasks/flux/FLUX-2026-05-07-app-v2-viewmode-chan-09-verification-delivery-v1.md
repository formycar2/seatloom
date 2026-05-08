# Delivery: chan-09 Combined Verify (Layer A + Layer B)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification_delivery |
| id | FLUX-2026-05-07-app-v2-viewmode-chan-09-verification-delivery-v1 |
| status | delivered |
| author | aegis (combined report on behalf of Flux automated layer + Mr. Zhang human layer) |
| date | 2026-05-08 |
| target_commit | `eb0ddaa` (Nimbus impl: `cadf36e`) |
| compare_base | `4f051b3` |
| verdict | **PASS** |
| tags | delivery, verification, chan-09, AD-013-v2, two-layer, combined |

## Verdict

**PASS** — Layer A (Flux automated checks) and Layer B (Mr. Zhang 14 browser scenarios) both PASS in a single verify cycle. No HOLD→PASS upgrade needed (process improvement realized).

## Layer A — Flux automated checks (Aegis pre-flight)

| Check | Result | Evidence |
|---|---|---|
| `git rev-parse HEAD` | PASS | `eb0ddaa` (or descendant) |
| Scope: only `ui/src/app-v2/types.ts` + `ui/src/app-v2/panel/SupervisorPanel.tsx` modified | PASS | `git show cadf36e --stat`: 2 files (+202/−112) |
| `cd ui && npx tsc --noEmit` | PASS | zero errors (Aegis independent run) |
| `cd ui && pnpm build` | PASS | tsc + vite build green; 353.04 kB / 102.35 kB gzip (Nimbus) |
| Mutator: `enterGlobal()` clears activeProjectId + sets viewMode='dashboard' | PASS | line 123-127 |
| Mutator: `enterProject()` sets all three fields including viewMode='dashboard' | PASS | line 129-133 |
| Mutator: `enterProjectFromBreadcrumb()` syncs activeContactId to channel (Lyra opinion #1) | PASS | line 136-140 |
| Mutator: `switchContact` for project-channel → calls enterProject | PASS | line 187 |
| Mutator: `switchContact` for seat/supervisor → only sets viewMode='chat', no contextMode mutation | PASS | line 191 |
| AD-013 v1 invariant `contextMode === 'global' ⇔ activeProjectId === null` preserved | PASS | by construction across all mutators |
| Three localStorage keys present | PASS | line 29-31 (`seatloom.supervisor.contextMode/.activeProjectId/.viewMode`) |
| Pre-existing localStorage keys preserved unchanged | PASS | `sl-supervisor-*` series intact |

## Layer B — Mr. Zhang browser smoke (14 scenarios)

### chan-09 NEW behavior (6 scenarios)

| # | Scenario | Result |
|---|---|---|
| 9 | Click Lyra (seat, p-1) directly from contact list | PASS (chat with 3-level breadcrumb; contextMode untouched) |
| 10 | Click Aegis Supervisor contact | PASS (1-level breadcrumb `全局`) |
| 11 | Mid-link click `SeatLoom 主项目` from chat with Lyra | PASS (dashboard switch + contact list highlight syncs) |
| 12 | Click `全局` from chat with Lyra | PASS (state fully resets to global) |
| 13 | Cross-project divergence (dashboard p-1 + chat with p-2 Iris) | PASS (legal divergent state, breadcrumb projectName from contact, not activeProjectId) |
| 14 | Mid-link visual affordance (hover) | PASS (cursor pointer + brand color shift) |

### chan-03 regression (8 scenarios)

| # | Scenario | Result |
|---|---|---|
| 1 | Clear localStorage default global | PASS |
| 2 | GlobalDashboard project row click | PASS |
| 3 | Breadcrumb `全局` back from project | PASS |
| 4 | Project channel switch via contact list | PASS |
| 5 | Persistence across reload | PASS |
| 6 | Stale activeProjectId fallback | PASS |
| 7 | 4-tab regression (overview/inbox/workitems/artifacts) | PASS |
| 8 | Slice B artifact panel regression | PASS |

## Resolution log

| Date | Actor | Action |
|------|-------|--------|
| 2026-05-07 | Nimbus | chan-09 implementation delivered at commit `cadf36e`, doc record `eb0ddaa` |
| 2026-05-07 | Aegis | Static pre-flight: tsc PASS, scope PASS, all mutator anchors verified, mutex invariant preserved |
| 2026-05-07 | Aegis | Issued combined two-layer verify packet `FLUX-2026-05-07-app-v2-viewmode-chan-09-verification-v1` (process improvement: no separate HOLD ceremony) |
| 2026-05-07 | Aegis | Started dev server at http://localhost:5174 |
| 2026-05-08 | Mr. Zhang | All 14 browser smoke scenarios verified PASS (6 chan-09 new + 8 chan-03 regression) |
| 2026-05-08 | Aegis | Combined Layer A + Layer B delivery written; verdict PASS |

## Process Improvement Note

This is the **first** delivery to apply combined two-layer verify upfront. Compare to chan-03:

| Aspect | chan-03 (old flow) | chan-09 (improved flow) |
|---|---|---|
| Verify packets issued | 1 (Flux verify-only) | 1 (Flux + Mr. Zhang combined) |
| Intermediate HOLD events | 1 (Flux HOLD because cannot do browser) | 0 |
| Delivery commits | 2 (HOLD delivery + HOLD→PASS upgrade) | 1 (single PASS delivery) |
| Aegis coordination overhead | Higher (escalation, upgrade ceremony) | Lower (linear path) |

Recommend Lyra adopt this two-layer model as the **default** for all future UI verify packets.

## Next Action

- Lyra to issue acceptance: `docs/coordination/acceptance/2026-05-08-lyra-app-v2-viewmode-chan-09-acceptance.md`
- Lyra to close `chan-09` in `AEGIS-2026-04-30-pending-changes-register`
- After acceptance: chan-03 acceptance can be issued in parallel (still pending from previous turn)
