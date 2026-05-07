# Task: app-v2 Supervisor viewMode (chan-09) — Combined Verify

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-07-app-v2-viewmode-chan-09-verification-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-07 |
| version | v1 |
| to | flux + Mr. Zhang (combined two-layer) |
| priority | P0 |
| deadline | 2026-05-08 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-viewmode-orthogonal-state-delivery-v1.md`, `docs/architecture-decisions.md` AD-013 v2 §7, commits `cadf36e`, `eb0ddaa` |
| tags | flux, verification, ui, app-v2, supervisor, viewmode, chan-09, AD-013-v2, pinned-commit, two-layer |
| owner | Flux (auto layer) + Mr. Zhang (human layer) |
| acceptance owner | Lyra |
| concurrency rule | Verify-only. No code edits. |

## Process Improvement Note

This packet is the **first** to apply the two-layer verify model upfront (per Aegis observation after chan-03 HOLD→PASS upgrade). Flux runs automated checks, then **without returning a HOLD**, hands off directly to Mr. Zhang for browser scenarios. Single delivery doc captures both layers.

## Target Commit

- `target_branch`: `track/infra-foundation`
- `target_commit`: `eb0ddaa` (delivery doc record); Nimbus impl: `cadf36e`
- `compare_base_commit`: `4f051b3` (AD-013 v2 design commit before Nimbus impl)

## Aegis pre-flight (already done)

- ✅ Independent tsc PASS
- ✅ Scope check: only 2 ui/ files (types.ts + panel/SupervisorPanel.tsx)
- ✅ Mutator anchors verified (enterGlobal/enterProject/enterProjectFromBreadcrumb/switchContact line 123-192)
- ✅ Breadcrumb 4-case render verified (line 264/269/290/295)
- ✅ Mid-link affordance click handler verified (line 311 → enterProjectFromBreadcrumb)
- ✅ stale fallback for viewMode verified (line 67-71)
- ✅ GlobalDashboard.onSelectProject correctly routes through enterProjectFromBreadcrumb (line 455) — solves Lyra opinion #1

## Layer A — Flux automated checks

```bash
git rev-parse HEAD                                  # must be eb0ddaa or descendant
git diff --stat HEAD -- ui/ crates/ infra/ scripts/ # must be empty
git diff --stat 4f051b3..HEAD -- ui/                # must show only types.ts and panel/SupervisorPanel.tsx
cd ui && npx tsc --noEmit                           # zero errors
cd ui && pnpm build                                 # exit 0
```

Static invariant inspection on `panel/SupervisorPanel.tsx`:

- [ ] `enterGlobal()` always sets `setActiveProjectIdState(null)` AND `setViewMode('dashboard')` (line 123-127)
- [ ] `enterProject(projectId)` sets all three: contextMode, activeProjectId, viewMode='dashboard' (line 129-133)
- [ ] `enterProjectFromBreadcrumb(projectId)` calls enterProject + sets activeContactId to that project's channel (line 136-140)
- [ ] `switchContact` for `type === 'project-channel'` → calls `enterProject(channel.projectId)` (line 187)
- [ ] `switchContact` for `type === 'seat'` or `'supervisor'` → only sets `viewMode='chat'`, **does NOT** touch contextMode/activeProjectId (line 191)
- [ ] AD-013 v1 invariant `contextMode === 'global' ⇔ activeProjectId === null` cannot be violated by any mutator path
- [ ] No code path can produce `contextMode='global' && activeProjectId !== null` or `contextMode='project' && activeProjectId === null`
- [ ] Three localStorage keys present: `seatloom.supervisor.contextMode`, `seatloom.supervisor.activeProjectId`, `seatloom.supervisor.viewMode` (line 29-31)
- [ ] Pre-existing keys (`sl-supervisor-open`, `sl-supervisor-pos`, `sl-supervisor-size`, `sl-supervisor-active-contact`, `sl-supervisor-draft-*`) preserved unchanged

If any item fails → `HOLD` with specific file:line. Otherwise → proceed to Layer B (no upgrade ceremony needed, single delivery).

## Layer B — Mr. Zhang browser smoke (14 scenarios)

Open dev server (Aegis already started it; check terminal for the local URL).

Open Supervisor panel (⌘K). DevTools → Application → Local Storage.

### chan-09 NEW behavior (6 scenarios)

| # | Scenario | Pass |
|---|---|---|
| 9 | Clear localStorage, reload, click Lyra (seat, p-1) directly from contact list | viewMode='chat' (Local Storage check), breadcrumb shows `全局 › SeatLoom 主项目 › Lyra`, contextMode stays `'global'` (NOT auto-synced) |
| 10 | Click Aegis Supervisor contact | viewMode='chat', breadcrumb shows just `全局` (no expansion) |
| 11 | In chat with Lyra (#9 state), click breadcrumb middle `SeatLoom 主项目` link | viewMode flips to `dashboard`, ProjectDashboard for p-1 renders, contact list highlight moves to `ch-p1` channel automatically |
| 12 | In chat with Lyra (#9 state), click breadcrumb `全局` | viewMode='dashboard', contextMode='global', activeProjectId=null (DevTools), GlobalDashboard renders |
| 13 | **Cross-project divergence**: in ProjectDashboard for p-1 (click ch-p1), then click p-2 contact `Iris` | viewMode='chat', contextMode stays `'project'`, activeProjectId stays `'p-1'`, breadcrumb shows `全局 › DataForge 数据平台 › Iris` (projectName from Iris.projectId='p-2', NOT activeProjectId) |
| 14 | Mid-link affordance hover | Hover over `<projectName>` link in chat-with-seat breadcrumb → cursor changes to pointer + color shifts to `var(--sl-brand)` |

### chan-03 regression (8 scenarios)

| # | Scenario | Pass |
|---|---|---|
| 1 | Clear localStorage, reload | Default global, GlobalDashboard renders, breadcrumb `全局` |
| 2 | Click p-1 row in GlobalDashboard | enters p-1 ProjectDashboard, breadcrumb `全局 › SeatLoom 主项目`, contact list highlight on `ch-p1` |
| 3 | Click breadcrumb `全局` (from #2) | back to GlobalDashboard, activeProjectId=null |
| 4 | In project mode, click `ch-p2` channel in contact list | switches to p-2 ProjectDashboard, breadcrumb updates |
| 5 | Close + reopen browser tab | Last context restored (contextMode + activeProjectId + viewMode + activeContactId) |
| 6 | Manually set `localStorage['seatloom.supervisor.activeProjectId'] = 'p-nonexistent'`, reload | Degrades to global, no white screen |
| 7 | In project mode, switch tabs (overview/inbox/workitems/artifacts) | All 4 tabs work as Phase 2 acceptance |
| 8 | In overview tab, verify Slice B artifact panel | Artifact rows + hover popup work |

## Verdict

- `PASS` — all 14 scenarios pass + Layer A all green
- `HOLD` — gap in Layer A (rare; Aegis already pre-flighted) OR ≥1 scenario fail in Layer B
- `FAIL` — fundamental contract breach

## Reporting Format

```text
[Flux + Mr. Zhang -> Aegis] chan-09 Combined Verify
commit:
- eb0ddaa (Nimbus impl: cadf36e)

Layer A (Flux automated):
- git rev-parse HEAD => ...
- scope (only types.ts + panel/SupervisorPanel.tsx) => ...
- npx tsc --noEmit => ...
- pnpm build => ...
- mutator anchor checks (10 items) => ...

Layer B (Mr. Zhang browser):
- chan-09 #9 (chat-with-seat 3-level breadcrumb) => ...
- chan-09 #10 (chat-with-supervisor 1-level breadcrumb) => ...
- chan-09 #11 (mid-link click → dashboard + contact sync) => ...
- chan-09 #12 (chat → 全局 click resets state) => ...
- chan-09 #13 (cross-project divergence) => ...
- chan-09 #14 (mid-link visual affordance) => ...
- chan-03 #1-#8 regression => ...

verdict:
- PASS | HOLD | FAIL
next action:
- Lyra acceptance + chan-09 closure in pending changes register
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-05-07-app-v2-viewmode-chan-09-verification-delivery-v1.md
```
