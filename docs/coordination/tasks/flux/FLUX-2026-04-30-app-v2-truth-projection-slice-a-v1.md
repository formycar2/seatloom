# Task: app-v2 Truth Projection Slice A

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | FLUX-2026-04-30-app-v2-truth-projection-slice-a-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P1 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/reviews/2026-04-30-lyra-app-v2-postgres-data-coverage-review.md`, `ui/src/app-v2/AppV2.tsx`, `ui/src/types/index.ts`, `ui/src/stores/useDataStore.ts`, `ui/src/mockData.ts` |
| tags | flux, ui, app-v2, truth-projection, dashboard, events, artifacts, inbox, hybrid-read-model |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Single-thread only. Do not parallelize this with any other Flux task. Do not open unrelated cleanup work while doing this packet. |

## Objective

Upgrade `ui/src/app-v2/AppV2.tsx` from a pure channel-story mock into a **hybrid truth-driven dashboard** for project channels.

This packet is intentionally narrow.

You are **not** rewriting the whole app. You are **not** building a backend API. You are **not** changing Nimbus schema or store shapes.

You are doing one bounded thing:

**For project-channel dashboards, replace the most misleading mock-only sections with deterministic projections from the existing front-end truth store (`useDataStore`) while keeping the current visual shell intact.**

## Why This Matters

Lyra's review established that `app-v2` currently fails the truth test:

- It still renders dashboard content from `MOCK_CHANNEL_DATA` in `ui/src/app-v2/AppV2.tsx`.
- It does not surface the structured project truth already present in `useDataStore` / `mockData`.
- The newest UI direction therefore looks persuasive but still hides real collaboration data.

This packet is a test of execution discipline:

- can you wire the existing UI to already-modeled truth,
- without overreaching into a full refactor,
- and without breaking the current shell.

## Scope Boundary

### In scope

Only these surfaces inside the project-channel dashboard:

1. blockers panel
2. active-items panel
3. next-step panel
4. activity log panel
5. hover detail content for `event`, `blocker`, `node`, and `next`

### Explicitly out of scope

Do **not** change these in this packet:

1. chat contact list and message thread model
2. global shell / resizing / routing guard
3. goals roadmap section
4. stage roadmap section
5. DAG layout algorithm or node renderer
6. backend / Tauri / PostgreSQL code
7. Zustand store schema
8. document parser / reconciliation infra
9. theme redesign
10. multi-language expansion

### Important hybrid rule

For this packet, you **may keep** these sections powered by the existing mock plan structure:

- goals
- current goal / stages
- current stage workflow DAG container

Reason: the store does not yet contain a full persisted project-plan / goal / stage model.

So the required end state is **hybrid truth**:

- plan scaffolding may still come from `MOCK_CHANNEL_DATA`
- operational truth must come from `useDataStore().projectData[projectId]`

## Required Read Order

Read in this exact order before changing code:

1. `docs/coordination/reviews/2026-04-30-lyra-app-v2-postgres-data-coverage-review.md`
2. `ui/src/app-v2/AppV2.tsx`
3. `ui/src/types/index.ts`
4. `ui/src/stores/useDataStore.ts`
5. `ui/src/mockData.ts`
6. this packet

## Primary File Targets

Required target files:

- `ui/src/app-v2/AppV2.tsx`

Optional helper extraction if needed:

- `ui/src/app-v2/truthProjection.ts`

Do not touch other product files unless strictly required for compile safety.
If you must touch another file, explain exactly why in the delivery artifact.

## Current Code Anchors

Use these anchors so you do not get lost:

- `ProjectDashboard` begins around `ui/src/app-v2/AppV2.tsx:580`
- current mock dashboard lookup is `const data = MOCK_CHANNEL_DATA[channelId];`
- dashboard call site is around `ui/src/app-v2/AppV2.tsx:1385`
- mock channel source starts around `ui/src/app-v2/AppV2.tsx:246`
- real store path exists in `ui/src/stores/useDataStore.ts`
- real project shape is `ProjectData` in `ui/src/types/index.ts:262`

## Required Implementation Strategy

### Step 1 — Wire `AppV2` to the real store

Inside `AppV2.tsx`, import `useDataStore`.

Read at least:

- `projectData`

You do **not** need to rewrite contacts/messages to come from the store.
Keep the existing `MOCK_CONTACTS` and `MOCK_MESSAGES` logic.

### Step 2 — Pass project identity into the dashboard

The dashboard currently receives only `channelId`.
That is not enough for truth projection.

Change the call so the dashboard also knows the project id.

Recommended shape:

```ts
<ProjectDashboard channelId={activeContact.id} projectId={activeContact.projectId} />
```

And update the component signature accordingly.

### Step 3 — Build a hybrid source model

Inside `ProjectDashboard`, create two layers:

1. `mockPlanData` — from `MOCK_CHANNEL_DATA[channelId]`
2. `truthData` — from `projectData[projectId]` when `projectId` exists and the store has data

If `truthData` is missing, preserve the current dashboard behavior.
Do not regress non-seeded projects.

### Step 4 — Add deterministic projection helpers

You may keep them inline or extract them to `ui/src/app-v2/truthProjection.ts`.

Recommended helpers:

```ts
formatActorRef(actorRef, seats)
formatObjectRef(ref)
formatClock(iso)
formatSince(iso)
getEventHeadline(event)
projectDashboardTruth(projectTruth)
```

Recommended projected shapes:

```ts
type DashboardBlocker = {
  sourceKind: 'WorkItem' | 'Session' | 'Handoff';
  sourceId: string;
  text: string;
  owner: string;
  since: string;
  detail?: string;
};

type DashboardActiveItem = {
  sourceKind: 'WorkItem' | 'Session';
  sourceId: string;
  title: string;
  owner: string;
  ownerAvatar?: string;
  ownerColor?: string;
  statusLabel: string;
  waitingSince?: string;
  refLabel?: string;
  description?: string;
  reviewTier?: 'L1' | 'L2' | 'L3';
  promptBadge?: string;
};

type DashboardNextAction = {
  summary: string;
  actor: string;
  priority: 'Critical' | 'Normal' | 'Low';
  objectRef: string;
  linkedArtifactIds?: string[];
  timestamp: string;
};

type DashboardEventRow = {
  eventId: string;
  eventType: string;
  time: string;
  headline: string;
  actor: string;
  objectRefs: string[];
  evidenceRefs: string[];
  rawTimestamp: string;
};
```

## Exact Projection Rules

Follow these rules exactly.

### A. Blockers panel

Use real truth when available.

Build blockers from these sources, in this order:

1. `sessions` with `status === 'InputRequired'`
2. `workItems` with `status === 'Blocked'`
3. `handoffs` with `status === 'Returned' || status === 'Sent' || status === 'Received'`

Sort order:

1. session blockers first
2. workitem blockers second
3. handoff blockers third
4. newest / most recent first within each family

Cap visible rows at `5`.

Text rules:

- session blocker text:
  - if `prompt_state` exists: `Prompt blocked: <seat name> requires <policy>`
  - else: `Session requires input: <seat name>`
- workitem blocker text:
  - `<workitem title>`
- handoff blocker text:
  - `Handoff awaiting action: <purpose>`

Owner rules:

- session owner = seat name from `seat_id`
- workitem owner = owning seat name
- handoff owner = receiver seat if `to_ref` is a seat, otherwise fallback to sender

Since rules:

- use `formatSince(created_at)` for sessions/handoffs
- use `formatSince(updated_at)` for workitems

### B. Active-items panel

Use real truth when available.

Include:

1. `workItems` where `status === 'Active' || status === 'InReview' || status === 'Reopened'`
2. `sessions` where `status === 'Running' || status === 'Launching'`

Sort order:

1. active workitems first
2. then live sessions
3. then newest first

Cap visible rows at `6`.

Display rules:

- workitem title = `<owner>: <title>`
- workitem ref label = `id`
- workitem status label:
  - `Active` => `推进中`
  - `InReview` => `审阅中`
  - `Reopened` => `已重开`
- if `change_tier_record` exists, include a compact tier badge such as `L2` or `L3`

- session title = `<seat name>: <runtime> session`
- session ref label = `session id`
- session status label:
  - `Running` => `运行中`
  - `Launching` => `启动中`
- if `prompt_state` exists, include compact badge text such as:
  - `deterministic`
  - `freeform`
  - `sensitive`

### C. Next-step panel

Use the real inbox.

Source:

- `projectTruth.inboxItems`

Sort order:

1. `Critical`
2. `Normal`
3. `Low`
4. newest timestamp first inside same priority

Pick only the top item.

Display rules:

- main text = `summary`
- secondary text = `actor + object_ref`
- hover detail must include linked artifact ids if present

If no inbox items exist, fall back to the current mock `currentStage.nextStep` text.

### D. Activity log panel

Replace the current `justNow` mapping with real canonical events when `truthData` exists.

Source:

- `projectTruth.events`

Sort order:

- newest first by `occurred_at`

Visible rows:

- top `6`

Headline rules:

Use this priority:

1. `payload.title` if present
2. `payload.summary` if present
3. fallback to `event_type`

Actor label:

- `Automation` stays `Automation`
- `{ Seat: id }` resolves to seat name

Time label:

- render as `HH:MM`

Event color rules:

- decision-like events (`WorkItemStatusChanged`, `ReconcileCompleted`, `CheckpointCreated`) => green
- delivery/handoff events (`ArtifactCreated`, `HandoffSent`, `HandoffAccepted`, `HandoffReturned`, `HandoffCompleted`) => blue
- all others => amber

Subtext rules:

Replace the current generic subtext with structured metadata:

- `actor` label
- object ref count
- evidence ref count

Example:

`Lyra · 2 refs · 1 evidence`

### E. Hover detail behavior

The hover card already exists. Improve it instead of replacing it.

#### For `event`

Show all of the following:

- headline
- `event_id`
- `event_type`
- `occurred_at`
- actor label
- linked object refs as chips
- evidence refs as chips

#### For `blocker`

Show all of the following:

- source family (`Session` / `WorkItem` / `Handoff`)
- source id
- owner
- since
- detail text if available

#### For `node`

If `node.workItemRef` matches a real workitem in `truthData.workItems`, enrich hover detail with:

- workitem id
- status
- priority
- owner seat
- review tier if `change_tier_record` exists
- active delegation scope if `active_delegation` exists
- linked artifacts count (`artifacts` whose `source_workitem_id` matches)

If no match exists, preserve current behavior.

#### For `next`

If driven by a real inbox item, show:

- priority
- actor
- object ref
- timestamp
- linked artifact ids as chips if present

### F. Chips and labels

Implement simple chips using existing inline style patterns.
Do not import a new design library.

Required chip families:

1. object ref chips
2. evidence ref chips
3. tier chips
4. prompt classification chips

Object chip labels:

- `WI` for workitems
- `SES` for sessions
- `HO` for handoffs
- `AR` for artifacts
- `SEAT` for seats

Evidence chip text rule:

- show only the basename in the chip
- keep full repo path in `title` attribute

## Fallback Rules

These must remain true after your patch:

1. If `projectId` is undefined, current mock dashboard still works.
2. If `projectData[projectId]` is missing, current mock dashboard still works.
3. Goals / stage / roadmap / DAG still render exactly as before unless compile-safe local refactor forces a harmless move.
4. No empty screen is introduced.

## Things You Must Not Do

1. Do not invent new store schema fields.
2. Do not change the seeded truth payloads in `ui/src/stores/useDataStore.ts` or `ui/src/mockData.ts` for convenience.
3. Do not refactor chat contacts/messages into the store.
4. Do not rewrite the whole dashboard.
5. Do not add backend fetches.
6. Do not touch PostgreSQL or Rust code.
7. Do not stage unrelated dirty files in your commit.

## Validation Commands

Run from repo root and record exact results:

```bash
cd ui && pnpm build
cd ui && npx tsc --noEmit
```

Optional if your seat can run it safely:

```bash
cd ui && pnpm dev --host 127.0.0.1 --port 4173
```

If runtime preview is not possible, say so explicitly.

## Git Discipline

This repo is already dirty.

You must:

1. touch only the minimal files required for this packet
2. stage only your files
3. create one dedicated commit after successful build/typecheck
4. report the commit hash in the delivery artifact

Recommended commit message:

```bash
feat(ui): add app-v2 truth projection slice A
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-delivery-v1.md`

Required sections:

1. Scope implemented
2. Files changed
3. Projection rules implemented
4. Fallback behavior preserved
5. Validation command results
6. Git branch and commit hash
7. Open limitations
8. Recommendation to Lyra (`ready for verification` / `needs follow-up`)

## Done Definition

- [ ] `ProjectDashboard` reads real `projectData` when available.
- [ ] blockers panel is store-driven for seeded projects.
- [ ] active-items panel is store-driven for seeded projects.
- [ ] next-step panel is inbox-driven for seeded projects.
- [ ] activity log panel is canonical-event-driven for seeded projects.
- [ ] hover cards for `event`, `blocker`, `node`, and `next` show structured truth, not only narrative text.
- [ ] fallback to the old mock behavior still works when truth data is absent.
- [ ] `pnpm build` passes.
- [ ] `tsc --noEmit` passes.
- [ ] one dedicated commit is created with only this packet's code changes.
- [ ] delivery artifact is written.

## Direct tmux Reply Contract

When done, send Lyra this exact structure:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_appv2_truth_projection_slice_a.txt
[Flux -> Lyra] app-v2 Truth Projection Slice A
completed:
- ...
validation:
- `cd ui && pnpm build` => ...
- `cd ui && npx tsc --noEmit` => ...
- runtime preview => completed / not possible on this seat (reason: ...)
branch:
- ...
commit:
- ...
blockers:
- none / ...
next action:
- wait for verification
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_appv2_truth_projection_slice_a /tmp/flux_to_lyra_appv2_truth_projection_slice_a.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_appv2_truth_projection_slice_a
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

## Final Reminder

This is a **precision wiring task**, not a redesign task.

If you are unsure between:

- a small truthful projection that keeps the shell stable, and
- a larger refactor that feels cleaner,

choose the **small truthful projection**.
