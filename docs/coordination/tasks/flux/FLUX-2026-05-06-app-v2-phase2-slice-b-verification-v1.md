# Task: app-v2 Phase 2 + Slice B — Verify-Only

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
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-b-v1.md`, `docs/coordination/tasks/copilot/COPILOT-2026-04-30-frontend-modularization-v1.md`, `docs/coordination/deliveries/2026-04-30-frontend-v2-phase1-phase2-delivery.md`, commit `cb06ce0` |
| tags | flux, verification, ui, app-v2, artifact, inbox, workitems, slice-b, phase2, commit-pinned |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Verify-only. No code edits. No parallel subtasks. |

## Context

`FLUX-2026-04-30-app-v2-truth-projection-slice-b-v1` was originally assigned to Flux for implementation. Copilot implemented it instead as part of a broader Phase 2 work packet, committed at `cb06ce0` on `track/infra-foundation`. Phase 1 (pure modular refactor) is at `21fc5af`.

This packet is **verify-only**. You are not implementing anything. You are confirming that `cb06ce0` satisfies the Slice B acceptance criteria and that the Phase 2 views (Inbox, WorkItems, Artifacts) render correctly with truth data.

## What Changed

**Phase 1 — `21fc5af`** (modular refactor, zero behavior change):
- `AppV2.tsx` split from 2018 lines into 14 focused modules
- New files: `types.ts`, `mock-data.ts`, `components/×4`, `panel/SupervisorPanel.tsx`, `dashboard/×5 sections`

**Phase 2 — `cb06ce0`** (new features — verify this):
- `dashboard/ProjectDashboard.tsx`: tab navigation (overview | inbox | workitems | artifacts); Slice B artifact panel in overview tab; `buildArtifactHover()` for hover enrichment
- `views/ArtifactsView.tsx`: artifact list with T1–T7 template filter, status colors
- `views/InboxView.tsx`: InboxItem list with priority sorting and archive action
- `views/WorkItemsView.tsx`: WorkItem list with status-chip filter
- `stores/useDataStore.ts`: `removeInboxItemFromProject` method added

## Target Commit

- `target_branch`: `track/infra-foundation`
- `target_commit`: `cb06ce0`
- `compare_base_commit`: `365fc8e` (last Lyra-accepted UI commit, Slice A)

**Hard rules:**
- do not substitute a later HEAD
- do not edit any code
- do not run the postgres verifier or infra scripts
- stay within `ui/` and browser verification

## Execution Steps

### 1. Commit identity gate

```bash
git rev-parse HEAD
git diff --stat HEAD -- ui/ crates/ infra/ scripts/
```

Stop and return `HOLD` if:
- HEAD is not exactly `cb06ce0`
- `git diff --stat` shows tracked code modifications in `ui/`, `crates/`, `infra/`, or `scripts/`

Pre-existing untracked files in `docs/coordination/` or `.gemini/` are acceptable — only tracked code modifications are blockers.

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

Start the dev server:

```bash
cd ui && pnpm dev
```

Open the app and navigate to a project channel. For seeded project `p-1` (SeatLoom 协调核心):

#### 4a. Overview tab — Slice B artifact panel (critical)

- Section titled `文档与证据 (ARTIFACTS)` is visible in the overview
- Artifact cards render with: title, template label (T1–T7), status chip, source refs line
- Hover over an artifact card: popup appears with title, template, subtype, status, full path, source refs, summary
- For a project with no artifacts: empty state message renders; no crash
- Existing panels still render: blockers, active work, DAG, next-step, activity

#### 4b. Tab navigation

- Tab bar shows: 看板 | 待办 (N) | 工作项 | 文档 (N)
- Clicking each tab switches view correctly
- Badge counts reflect actual data lengths
- Tab state resets on contact switch

#### 4c. Artifacts tab (ArtifactsView)

- Template filter chips render (only templates present in data)
- Filtering by template chip narrows visible artifact cards
- Each card shows: template label, subtype, status, title, storage path basename

#### 4d. Inbox tab (InboxView)

- InboxItems sorted by priority: Critical → Normal → Low
- Each item shows: priority chip, type, summary
- Archive button present per item

#### 4e. WorkItems tab (WorkItemsView)

- Status filter chips render (进行中 / 待处理 / 阻塞 / 审阅中 / 已完成 / 全部)
- Filtering by status chip narrows list
- Each card shows: ID, title, status chip, priority

#### 4f. Regression check — prior panels still work in overview

- Blockers section renders (if blockers exist)
- Active work section renders
- DAG workflow renders
- Next steps section renders
- Timeline section renders
- No crashes, no blank panels for previously-visible data

## Verdict Options

- `PASS` — commit `cb06ce0` satisfies all requirements above
- `HOLD` — real gap found; name the first gap with file:line
- `FAIL` — fundamental contract breach

## Reporting Format

```text
[Flux -> Lyra] app-v2 Phase 2 + Slice B Verification
commit:
- cb06ce0
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
- PASS | HOLD | FAIL
next action:
- wait for Lyra acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-05-06-app-v2-phase2-slice-b-verification-delivery-v1.md
```
