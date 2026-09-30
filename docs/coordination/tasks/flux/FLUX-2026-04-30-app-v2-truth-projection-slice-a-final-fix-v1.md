# Task: app-v2 Truth Projection Slice A Final Hover Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | FLUX-2026-04-30-app-v2-truth-projection-slice-a-final-fix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P1 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-followup-review-v2.md`, commit `7677672`, `ui/src/app-v2/AppV2.tsx` |
| tags | flux, ui, app-v2, truth-projection, hover, final-fix |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Single-thread only. Do not parallelize this with any other Flux task. |

## Base Commit

Work from commit `7677672` on `track/infra-foundation`.
If local HEAD is not `7677672` when you start, stop and report drift.

## Objective

Close the last remaining P1 bug in the bounded truth-projection slice.

This is a very small patch.
Do not widen scope.
Do not refactor unrelated helpers.
Do not touch layout, routing, styling, or data models.

## Required Read Order

1. `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-followup-review-v2.md`
2. `ui/src/app-v2/AppV2.tsx`
3. this packet

## Exact Bug

The activity panel now correctly falls back to `justNow` mock rows when `projectedEvents.length === 0`.
However, those fallback rows still use `type === 'event'`, and `handleMouseMove()` still routes all `event` rows through `buildEventHover()` whenever `truthData` exists.

Current failure path:
- project has `truthData`
- project has no `events`
- activity list renders `justNow`
- hover a mock row
- `buildEventHover()` assumes `evidenceRefs.map(...)` and crashes because mock rows do not have truth-event shape

## Required Fix

You must make the event hover path safe for both row shapes:
- canonical projected event rows
- mock `justNow` fallback rows

### Acceptable implementation patterns

You may choose either approach:

1. **Shape guard inside `buildEventHover()`**
   - detect whether the incoming row is a projected canonical event or a mock timeline row
   - return a safe hover payload for both

or

2. **Separate mock-event hover path**
   - keep `buildEventHover()` truth-only
   - add something like `buildMockEventHover()`
   - branch at the call site when rendering fallback rows

### Hard requirements

- No crash when `truthData` exists but `projectedEvents.length === 0`
- Mock fallback rows must still hover safely
- Truth-backed event rows must keep their current enriched hover payload
- Touch only `ui/src/app-v2/AppV2.tsx` unless compile safety forces otherwise

## Validation Required

Run exactly:
- `cd ui && pnpm build`
- `cd ui && npx tsc --noEmit`

## Delivery Required

After finishing:
1. create one dedicated commit
2. reply via tmux with:
   - completed
   - validation
   - blockers
   - commit hash
   - artifact path(s)
