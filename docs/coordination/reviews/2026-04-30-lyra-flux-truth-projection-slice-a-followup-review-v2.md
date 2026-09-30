# Review: Flux app-v2 Truth Projection Slice A Follow-up

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | 2026-04-30-lyra-flux-truth-projection-slice-a-followup-review-v2 |
| status | active |
| author | lyra |
| date | 2026-04-30 |
| version | v2 |
| depends_on | commit `7677672`, `ui/src/app-v2/AppV2.tsx`, `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-followup-v1.md` |
| tags | review, flux, ui, app-v2, truth-projection, hover, fallback |

## Scope

Review Flux follow-up commit `7677672` against the three P1 fixes requested after commit `ab52edd`.

## Verdict

- `HOLD`

Two of the three requested fixes are closed. One P1 regression remains in the new fallback path for activity rows.

## Closed Items

- `closed` — string-safe rendering for `InboxItem.object_ref`
- `closed` — null-safe next-step hover handling when `nextStepSource` is absent

## Remaining Finding

### 1. Mock activity fallback still routes through truth-only hover enrichment and can crash
- Status: `open`
- Severity: `P1`
- Evidence:
  - `ui/src/app-v2/AppV2.tsx:802`–`ui/src/app-v2/AppV2.tsx:812` — `buildEventHover(ev)` assumes `ev.evidenceRefs.map(...)` exists
  - `ui/src/app-v2/AppV2.tsx:878`–`ui/src/app-v2/AppV2.tsx:883` — all `event` hovers still call `buildEventHover(itemData)` whenever `truthData` exists
  - `ui/src/app-v2/AppV2.tsx:1099`–`ui/src/app-v2/AppV2.tsx:1105` — mock fallback rows from `justNow` now emit `type === 'event'`
- Failure mode:
  - When a project has `truthData` but no `events`, the UI correctly falls back to mock `justNow` rows.
  - Hovering one of those fallback rows still enters the truth hover path.
  - Mock rows do not carry `evidenceRefs` / `objectRefs`, so `ev.evidenceRefs.map(...)` can throw.
- Impact:
  - The newly restored activity fallback remains non-durable in the exact scenario it was meant to protect.
  - This is still a user-visible runtime bug in the bounded truth projection slice.
- Required fix:
  - Make event hover enrichment branch on row shape, not only on `truthData` existence.
  - Either:
    1. guard `buildEventHover()` so it safely handles mock timeline rows, or
    2. send mock fallback rows through a dedicated mock-event hover builder.
  - Do not expand scope beyond the activity hover path.

## Acceptance Condition

Accept the slice after one final bounded fix closes the remaining mock-activity hover crash without changing unrelated dashboard behavior.
