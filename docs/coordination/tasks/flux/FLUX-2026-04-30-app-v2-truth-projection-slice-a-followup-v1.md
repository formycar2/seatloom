# Task: app-v2 Truth Projection Slice A Follow-up Fixes

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | FLUX-2026-04-30-app-v2-truth-projection-slice-a-followup-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P1 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-review.md`, commit `ab52edd`, `ui/src/app-v2/AppV2.tsx`, `ui/src/types/index.ts` |
| tags | flux, ui, app-v2, truth-projection, followup, pinned-commit |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Single-thread only. Do not parallelize this with any other Flux task. |

## Base Commit

Work from commit `ab52edd` on `track/infra-foundation`.
If your local HEAD is not `ab52edd` when you start, stop and report drift.

## Objective

Close the three bounded P1 issues found in Lyra review after Slice A.

This is not a refactor.
Do not widen scope.
Do not touch routing, layout, DAG scaffolding, or unrelated styling.

## Required Read Order

1. `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-review.md`
2. `ui/src/app-v2/AppV2.tsx`
3. `ui/src/types/index.ts`
4. this packet

## Exact Fixes Required

### Fix 1 — `InboxItem.object_ref` formatting

Current bug:
- `InboxItem.object_ref` is a string, but current code passes it into the object-ref formatter.

Required behavior:
- The next-step subtitle and next-step hover must display inbox refs correctly for strings like `WI-402`, `SES-402`, `HO-404`.

Implementation rule:
- Add a separate formatter for string refs, for example `formatInboxObjectRef(ref: string)`.
- Use that formatter only for `InboxItem.object_ref`.
- Do not break existing event/object-ref formatting for canonical event objects.

Acceptance examples:
- `WI-402` stays `WI-402`
- `SES-402` stays `SES-402`
- `HO-404` stays `HO-404`

### Fix 2 — no hover crash when `nextStepSource` is absent

Current bug:
- The next-step card always calls hover enrichment even if `nextStepSource` is `null`.

Required behavior:
- If truth inbox data is absent, the mock fallback card still renders and hover must not throw.
- You may either disable hover enrichment in fallback mode or provide a safe mock hover payload.

Implementation rule:
- No unguarded access to `item.priority`, `item.actor`, `item.object_ref`, or `item.timestamp` when `item` is nullish.

### Fix 3 — restore activity mock fallback

Current bug:
- When truth events are absent, the main activity list is empty.

Required behavior:
- Seeded projects keep using truth-driven activity rows.
- Non-seeded or event-empty projects must fall back to the old `justNow` mock rows.

Implementation rule:
- Keep the current canonical-event projection for truth-backed projects.
- Restore previous `justNow` rendering path only when `projectedEvents.length === 0`.
- Do not remove the earlierToday / yesterday section.

## File Boundary

Required target:
- `ui/src/app-v2/AppV2.tsx`

Do not touch other files unless compile safety requires it.
If you must touch another file, explain why in the delivery artifact.

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
