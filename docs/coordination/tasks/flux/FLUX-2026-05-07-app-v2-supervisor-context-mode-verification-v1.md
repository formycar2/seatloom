# Task: app-v2 Supervisor Context Mode (chan-03) — Verify-Only

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-07-app-v2-supervisor-context-mode-verification-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-07 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-05-08 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-context-mode-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-05-07-app-v2-supervisor-context-mode-delivery-v1.md`, `docs/architecture-decisions.md` AD-013, commits `f5b8423` and `6038ba6` |
| tags | flux, verification, ui, app-v2, supervisor, context-mode, chan-03, AD-013, pinned-commit |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Verify-only. No code edits. No parallel subtasks. |

## Context

Nimbus implemented chan-03 (Supervisor two-layer context mode per AD-013) as a cross-track assignment. Implementation is at commit `f5b8423`; delivery doc commit hash recorded at `6038ba6` (current branch HEAD).

This packet is **verify-only**. Confirm `f5b8423` satisfies AD-013 invariants and the Nimbus task packet (`NIMBUS-2026-05-07-app-v2-supervisor-context-mode-v1`) requirements.

## Target Commit

- `target_branch`: `track/infra-foundation`
- `target_commit`: `6038ba6` (branch HEAD; Nimbus implementation = `f5b8423`, doc record = `6038ba6`)
- `compare_base_commit`: `1c7935c` (last AD-013 design commit before Nimbus impl)

**Hard rules:**
- do not substitute a later HEAD
- do not edit any code
- do not run the postgres verifier or infra scripts
- stay within `ui/` and browser verification

## Files Changed by Nimbus

```
ui/src/app-v2/dashboard/GlobalDashboard.tsx        | 193 ++++++  (new)
ui/src/app-v2/mock-data.ts                         |  28 ++-     (modified)
ui/src/app-v2/panel/SupervisorPanel.tsx            | 191 ++++++- (modified)
ui/src/app-v2/types.ts                             |  11 ++      (modified)
docs/coordination/tasks/nimbus/...delivery-v1.md   | 170 ++++++  (new)
```

Verify nothing else was modified.

## Execution Steps

### 1. Commit identity gate

```bash
git rev-parse HEAD                                    # must be 6038ba6 (or descendant)
git diff --stat HEAD -- ui/ crates/ infra/ scripts/  # must be empty (no working-tree code edits)
git diff --stat 1c7935c..HEAD -- ui/                  # should match the 4 ui/ files above
```

Stop and return `HOLD` if files outside the listed scope (e.g. `ProjectDashboard.tsx`, `views/`, `useDataStore.ts`, Rust crates, infra) were modified.

### 2. TypeScript check

```bash
cd ui && npx tsc --noEmit 2>&1
```

Expected: zero errors, zero output.

### 3. Build check

```bash
cd ui && pnpm build 2>&1 | tail -20
```

Expected: build succeeds, exit 0.

### 4. Browser smoke test (8 scenarios)

Start dev server: `cd ui && pnpm dev`

Open Supervisor panel (⌘K). Test all 8 scenarios from the Nimbus packet §Validation:

| # | Scenario | Pass criteria |
|---|---|---|
| 1 | Clear localStorage, reload | Default lands in **global** mode; GlobalDashboard renders with project rows |
| 2 | Click a project row in global view | Switches to **project** mode; ProjectDashboard renders for that project; breadcrumb shows `全局 › <项目名>` |
| 3 | Click breadcrumb `全局` | Returns to global mode; `activeProjectId` clears; breadcrumb shows just `全局` |
| 4 | In project mode, click a different project channel in Contact list | ProjectDashboard re-renders with new project's data; breadcrumb updates |
| 5 | Close and reopen app | Last context restored: if was project mode, returns to that project; if global, returns to global |
| 6 | **Stale fallback**: manually set `localStorage['seatloom.supervisor.activeProjectId'] = 'p-nonexistent'`, reload | Degrades to global mode; no white screen, no console error |
| 7 | **Regression**: enter project mode, switch tabs (overview/inbox/workitems/artifacts) | All 4 tabs work identically to Phase 2 acceptance |
| 8 | **Regression**: in overview tab, verify Slice B artifact panel still renders | Artifact rows + hover popup work as before |

### 5. Mutex invariant inspection (static)

Inspect `ui/src/app-v2/panel/SupervisorPanel.tsx` and confirm:

- `enterGlobal()` always sets `setActiveProjectIdState(null)` — no path leaves activeProjectId non-null after this
- `enterProject(projectId)` sets both mode and activeProjectId in same render
- The contact-click handler around line 140-150 (which bypasses `enterProject` to avoid redirecting activeContact) **still maintains** `mode === 'project' ⇔ activeProjectId !== null` by setting both atomically
- No state mutator can produce `mode === 'global' && activeProjectId !== null` or `mode === 'project' && activeProjectId === null`

Report any path that breaks the invariant as a `HOLD`.

### 6. localStorage key inspection

Confirm the keys used are exactly:
- `seatloom.supervisor.contextMode`
- `seatloom.supervisor.activeProjectId`

And confirm no existing keys (`sl-supervisor-open`, position/size keys) were renamed.

## Verdict Options

- `PASS` — all 8 smoke scenarios pass + mutex invariant holds + scope respected
- `HOLD` — real gap; name first gap with file:line and which scenario/invariant it breaks
- `FAIL` — fundamental contract breach (e.g. AD-013 invariant fundamentally violated, ProjectDashboard regression)

## Reporting Format

```text
[Flux -> Aegis] app-v2 Supervisor Context Mode Verification (chan-03)
commit:
- 6038ba6 (Nimbus impl: f5b8423)
completed:
- ...
validation:
- git rev-parse HEAD => ...
- scope check (only 4 ui/ files modified) => ...
- npx tsc --noEmit => ...
- pnpm build => ...
- smoke scenario 1 (clear localStorage, default global) => ...
- smoke scenario 2 (enter project) => ...
- smoke scenario 3 (back to global, activeProjectId clears) => ...
- smoke scenario 4 (switch project via contact list) => ...
- smoke scenario 5 (persistence across reload) => ...
- smoke scenario 6 (stale fallback) => ...
- smoke scenario 7 (tab regression) => ...
- smoke scenario 8 (Slice B artifact panel regression) => ...
- mutex invariant (enterGlobal/enterProject + contact-click path) => ...
- localStorage keys correct => ...
blockers:
- none OR <first gap>
verdict:
- PASS | HOLD | FAIL
next action:
- wait for Lyra acceptance / chan-03 closure
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-05-07-app-v2-supervisor-context-mode-verification-delivery-v1.md
```
