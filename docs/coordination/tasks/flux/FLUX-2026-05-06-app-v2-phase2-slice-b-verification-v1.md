# Task: app-v2 Phase 2 + Slice B UI Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-06-app-v2-phase2-slice-b-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-06 |
| version | v1 |
| to | flux |
| priority | P1 |
| deadline | 2026-05-06 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-b-v1.md`, `docs/coordination/tasks/copilot/COPILOT-2026-04-30-frontend-modularization-v1.md` |
| tags | flux, verification, ui, app-v2, artifact, inbox, workitems, slice-b, phase2, commit-pinned |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Verify-only. No code edits. No parallel subtasks. |

## Context

Slice B (artifact panel) was originally assigned to Flux for implementation. Copilot implemented it instead as part of a broader Phase 2 work packet, committed at `cb06ce0`. Phase 1 (pure modular refactor) is at `21fc5af`.

This packet is **verify-only**. You are not implementing anything. You are confirming that `cb06ce0` satisfies the Slice B acceptance criteria and that the Phase 2 views (Inbox, WorkItems, Artifacts) render correctly with truth data.

## What Changed

**Phase 1 — `21fc5af`** (modular refactor, zero behavior change):
- `AppV2.tsx` split from 2018 lines into 14 focused modules
- New files: `types.ts`, `mock-data.ts`, `components/×4`, `panel/SupervisorPanel.tsx`, `dashboard/×6`

**Phase 2 — `cb06ce0`** (new features, verify this):
- `dashboard/ProjectDashboard.tsx`: tab navigation added (overview | inbox | workitems | artifacts); Slice B artifact panel added to overview (section "4-bis"); `buildArtifactHover()` for hover enrichment
- `views/ArtifactsView.tsx`: artifact list with T1–T7 template filter, status colors
- `views/InboxView.tsx`: InboxItem list with priority sorting and archive action
- `views/WorkItemsView.tsx`: WorkItem list with status-chip filter
- `stores/useDataStore.ts`: `removeInboxItemFromProject` method added

## Target Commit

- `target_branch`: `track/infra-foundation`
- `target_commit`: `cb06ce0` (Phase 2 HEAD — this is the commit to verify)
- `compare_base_commit`: `365fc8e` (last Lyra-accepted UI commit, Slice A)

**Hard rules:**
- do not substitute a later `HEAD`
- do not edit any code
- do not run the postgres verifier or infra scripts
- stay within `ui/` and the browser check

## Execution Steps

### 1. Commit identity gate

```bash
git fetch origin track/infra-foundation
git checkout cb06ce0
git rev-parse HEAD
git status --short
```

Stop and return `HOLD` if:
- `git rev-parse HEAD` is not exactly `cb06ce0...` (full hash)
- `git status --short` is non-empty

### 2. TypeScript check

```bash
cd ui && npx tsc --noEmit 2>&1
```

Expected: zero errors, zero output.

### 3. Build check

```bash
cd ui && pnpm build 2>&1 | tail -20
```

Expected: build succeeds, no error exit.

### 4. Dev server + browser verification

Start the dev server and open the app in a browser:

```bash
cd ui && pnpm dev
```

For each of the three seeded projects (seatloom, a second project, a third if present), open the project channel and verify:

#### 4a. Overview tab — Slice B artifact panel (critical)

- Section "4-bis" (Artifacts & Documents) is visible in the overview
- For a project with truth data (`useDataStore` has artifacts): artifact cards render with title, template label (T1–T7), status chip, and source refs
- For a project with no artifacts: the "本项目已有结构化 truth，但当前无 artifact 对象" placeholder shows (if truth data exists) OR section is absent (if no truth data)
- Hover over an artifact card: tooltip appears with title, summary, and source refs

#### 4b. Tab navigation

- Tab bar shows: 看板 | 待办 (N) | 工作项 | 文档 (N)
- Clicking each tab switches view correctly
- Badge counts on 待办 and 文档 reflect actual data lengths

#### 4c. Artifacts tab (ArtifactsView)

- Template filter chips (T1–T7 + 全部) are rendered
- Filtering by template narrows the visible artifact cards
- Each card shows: template label (colored), subtype label, status chip, title, storage path basename, and summary preview

#### 4d. Inbox tab (InboxView)

- InboxItems are sorted by priority (Critical → Normal → Low)
- Each item shows: priority chip, type, actor, summary, and linked artifact if present
- Archive button is present on each item

#### 4e. WorkItems tab (WorkItemsView)

- Status filter chips are rendered (All + each WorkItemStatus)
- Filtering by status narrows the list
- Each card shows: ID, title, status chip, priority, owner, and depends_on count

#### 4f. Regression check — prior panels still work

In the overview tab:
- Blockers section still renders (if any blockers exist)
- Active work section still renders
- DAG workflow still renders
- Next steps section still renders
- Timeline section still renders

No crashes, no blank panels for data that was previously visible.

### 5. If any check fails

Report the first real blocker exactly:
- which tab or section failed
- what the console error says (if any)
- whether it's a data/render issue or a type/import issue

Do not patch. Return `HOLD`.

## Delivery Format

```text
[Flux -> Lyra] app-v2 Phase 2 + Slice B Verification
completed:
- ...
validation:
- git rev-parse HEAD => ...
- npx tsc --noEmit => ...
- pnpm build => ...
- overview artifact panel (Slice B) => ...
- tab navigation => ...
- ArtifactsView (template filter) => ...
- InboxView (priority sort) => ...
- WorkItemsView (status filter) => ...
- regression check (prior panels) => ...
blockers:
- none / ...
verdict:
- PASS | HOLD
next action:
- wait for Lyra acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-05-06-app-v2-phase2-slice-b-verification-delivery-v1.md
```
