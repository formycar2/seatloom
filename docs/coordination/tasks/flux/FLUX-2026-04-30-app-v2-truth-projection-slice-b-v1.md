# Task: app-v2 Truth Projection Slice B — Typed Artifact Surface

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | FLUX-2026-04-30-app-v2-truth-projection-slice-b-v1 |
| status | superseded |
| superseded_by | `FLUX-2026-05-06-app-v2-phase2-slice-b-verification-v1` |
| superseded_note | Implementation completed by Copilot (session 704a5e63) at commit `cb06ce0`. Flux role changed to verify-only. |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P1 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/reviews/2026-04-30-lyra-app-v2-postgres-data-coverage-review.md`, `docs/coordination/acceptance/2026-04-30-lyra-flux-app-v2-truth-projection-slice-a-acceptance.md`, commit `365fc8e`, `ui/src/app-v2/AppV2.tsx`, `ui/src/stores/useDataStore.ts`, `ui/src/types/index.ts` |
| tags | flux, ui, app-v2, truth-projection, artifacts, documents, hover, pinned-commit |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Single-thread only. Do not parallelize this with any other Flux task. Do not open unrelated cleanup or styling work while doing this packet. |

## Base Commit

Work from commit `365fc8e` on `track/infra-foundation`.
If local HEAD is not `365fc8e` when you start, stop and report drift.

## Objective

Add one bounded truth-backed surface to `ui/src/app-v2/AppV2.tsx`:

**Project dashboard must show typed Artifact / Document objects from `useDataStore().projectData[projectId].artifacts` instead of hiding this truth family completely.**

This is a narrow follow-up to accepted Slice A.
You are not redesigning the shell.
You are not building routing.
You are not making every artifact clickable into a full detail page.

You are doing one thing:

**Introduce a truthful Artifact panel plus safe artifact hover enrichment inside the existing `ProjectDashboard` visual shell.**

## Why This Matters

Lyra's coverage review still marks `artifacts`, `documents`, `document_sections`, and `document_associations` as effectively absent from `app-v2`.

Right now the dashboard can show blockers, active work, next actions, and events, but it still cannot show the durable document evidence that actually defines SeatLoom collaboration.

That gap is misleading because Nimbus has already seeded rich artifact truth including:

- `title`
- `template`
- `subtype`
- `status`
- `storage_path`
- `summary`
- `content_preview`
- `source_workitem_id`
- `source_session_id`
- `source_handoff_id`

This packet tests whether you can surface that truth cleanly without widening scope.

## Scope Boundary

### In scope

Only these additions inside `ProjectDashboard` and its existing hover system:

1. recent typed artifact / document panel
2. artifact row projection from `truthData.artifacts`
3. artifact hover enrichment
4. minimal empty state for truth-backed projects with zero artifacts

### Explicitly out of scope

Do not change these in this packet:

1. contacts / message-thread model
2. routing or selected-detail architecture
3. session detail surfaces
4. seat / delegation panels
5. handoff detail surfaces
6. DAG layout / node renderer
7. goals / stages scaffolding
8. backend / Tauri / PostgreSQL code
9. Zustand schema
10. styling cleanup beyond what this slice strictly needs
11. document full-body reader
12. section-anchor navigation
13. review-thread / commenting system

## Required Read Order

Read in this exact order before changing code:

1. `docs/coordination/reviews/2026-04-30-lyra-app-v2-postgres-data-coverage-review.md`
2. `docs/coordination/acceptance/2026-04-30-lyra-flux-app-v2-truth-projection-slice-a-acceptance.md`
3. `ui/src/app-v2/AppV2.tsx`
4. `ui/src/stores/useDataStore.ts`
5. `ui/src/types/index.ts`
6. this packet

## Primary File Target

Required target file:

- `ui/src/app-v2/AppV2.tsx`

Do not touch other product files unless compile safety strictly requires it.
If you must touch another file, explain exactly why in the delivery artifact.

## Current Code Anchors

Use these anchors so you do not get lost:

- `ProjectDashboard` begins around `ui/src/app-v2/AppV2.tsx:581`
- truth store access already exists in Slice A
- next-step panel begins around `ui/src/app-v2/AppV2.tsx:1030`
- activity panel begins around `ui/src/app-v2/AppV2.tsx:1062`
- hover popup begins around `ui/src/app-v2/AppV2.tsx:1222`
- seeded artifact truth lives in `ui/src/stores/useDataStore.ts` under `P1_SEEDED_ARTIFACTS`
- `Artifact` type is defined in `ui/src/types/index.ts`

## Required Implementation Strategy

### Step 1 — derive a deterministic artifact projection

Inside `ProjectDashboard`, build `projectedArtifacts` from `truthData.artifacts`.

Required rule:
- if `truthData` is missing, keep current behavior and do not render the new panel
- if `truthData` exists, sort artifacts by `created_at` descending
- use the latest 5 rows only

Recommended projected row shape:

```ts
type DashboardArtifactRow = {
  id: string;
  title: string;
  template: string;
  subtype: string;
  status?: string;
  storageBasename: string;
  author?: string;
  date?: string;
  sourceRefs: string[];
  summary?: string;
  summaryPoints?: string[];
  contentPreview?: string[];
  fullPath: string;
};
```

### Step 2 — add minimal helper formatters

Add only the helpers you need, for example:

```ts
getBasename(path: string)
formatArtifactSourceRefs(artifact)
```

Source ref rules:
- `source_workitem_id` -> show raw id such as `wi-410`
- `source_session_id` -> show raw id such as `ses-406`
- `source_handoff_id` -> show raw id such as `ho-405`
- keep order: workitem, session, handoff
- omit missing refs

### Step 3 — render one new panel in the existing dashboard flow

Add a new panel titled:

- `文档与证据 (ARTIFACTS)`

Placement rule:
- insert it **after** the next-step panel and **before** the activity panel
- do not reorder the existing panels otherwise

Visual rule:
- keep current shell language and token style
- do not introduce a new layout system
- rows should look like existing dashboard cards, not a totally new UI pattern

Each artifact row must visibly show at minimum:

1. `title`
2. `template` chip
3. `subtype` chip
4. `status` chip when present
5. storage path basename
6. source refs line when available

Recommended secondary line:
- basename + author/date on one side
- source refs on the other side or underneath

### Step 4 — add truth-backed empty state

If `truthData` exists but `truthData.artifacts.length === 0`, render the panel with a small empty state.

Required empty-state meaning:
- this project has structured truth, but no typed artifact/document objects are currently available in the local read model

Do not invent fake artifact rows from mock data.

### Step 5 — artifact hover enrichment

Extend the existing hover system with a new type:

- `artifact`

Required hover payload must include at minimum:

1. title
2. template
3. subtype
4. status
5. full storage path
6. source refs
7. summary
8. up to 3 `summary_points`
9. up to 3 `content_preview` bullets

Implementation options:
- either store the projected row directly and enrich it for hover
- or build a dedicated `buildArtifactHover()` helper

Hard requirements:
- no crash when optional fields are absent
- no assumption that `summary_points` or `content_preview` always exist
- reuse the current global hover popup instead of creating a new modal

### Step 6 — update hover popup copy safely

The global hover popup currently handles `node`, `blocker`, `event`, and `next`.
Extend it to render artifact-specific content.

Required behavior:
- header label should distinguish artifact hover from event/blocker/node
- the popup body should show artifact metadata cleanly, not generic fallback text only
- keep existing hover behavior for other item types unchanged

## Exact Acceptance Expectations

On seeded project `p-1`, the dashboard should visibly surface real artifact rows such as contract docs / reviews / packets from the seeded store.

At minimum, Lyra must be able to see from the screen that an item is a typed object rather than a plain path string, because the row shows:

- title
- template
- subtype
- status or source linkage

Examples that should become visibly possible if the seeded rows are in the top 5:
- `PRD v0.5 active contract`
- `Interaction Spec v1.1`
- `Document template taxonomy`
- `Lyra artifact objectization baseline packet`

## Non-Goals Reminder

This packet does **not** close all document-authority gaps.
It does not build:

- section anchors
- full-body document reading
- association drill-through navigation
- revision history
- reconcile freshness

Those remain future slices.

## Validation Required

Run exactly:

- `cd ui && pnpm build`
- `cd ui && npx tsc --noEmit`

## Delivery Required

After finishing:

1. create one dedicated commit
2. reply via tmux in the standard format with:
   - completed
   - validation
   - blockers
   - commit hash
   - artifact path(s)

## Failure Rule

If you discover that this slice cannot be completed inside `ui/src/app-v2/AppV2.tsx` without widening scope, stop and report the concrete blocker instead of improvising a larger redesign.
