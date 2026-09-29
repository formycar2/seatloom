# Review: Flux app-v2 Truth Projection Slice A

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | 2026-04-30-lyra-flux-truth-projection-slice-a-review |
| status | active |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-v1.md`, `ui/src/app-v2/AppV2.tsx`, commit `ab52edd` |
| tags | review, flux, ui, app-v2, truth-projection, pinned-commit |

## Scope

Review Flux delivery commit `ab52edd` against the bounded requirements in `FLUX-2026-04-30-app-v2-truth-projection-slice-a-v1.md`.

## Verdict

- `HOLD`

The slice is directionally correct and build-safe, but it is not yet acceptable because three bounded regressions remain in the exact surface that was just rewired.

## Findings

### 1. Inbox `object_ref` is rendered with the wrong formatter
- Status: `open`
- Severity: `P1`
- Evidence:
  - `ui/src/types/index.ts:248` defines `InboxItem.object_ref` as `string`
  - `ui/src/app-v2/AppV2.tsx:617` assumes `formatObjectRef(ref)` receives an object ref
  - `ui/src/app-v2/AppV2.tsx:1047` passes `nextStepSource.object_ref` into that object-only formatter
  - `ui/src/app-v2/AppV2.tsx:836` repeats the same mistake in next-step hover enrichment
- Impact:
  - The next-step card can render malformed references such as character-derived junk instead of `WI-402` / `SES-402` / `HO-404`.
  - This breaks the truth surface exactly where the packet required deterministic projection from `projectData.inboxItems`.
- Required fix:
  - Add a string-safe formatter path for inbox refs.
  - Do not use the object-ref formatter for `InboxItem.object_ref`.

### 2. Hovering next-step can crash when the project has no inbox item
- Status: `open`
- Severity: `P1`
- Evidence:
  - `ui/src/app-v2/AppV2.tsx:833` `buildNextHover(item)` dereferences `item.priority`
  - `ui/src/app-v2/AppV2.tsx:1038` and `ui/src/app-v2/AppV2.tsx:1039` call it even when `nextStepSource` may be `null`
- Impact:
  - Unseeded or inbox-empty projects can regress from mock fallback into a runtime hover crash.
  - This violates the packet requirement to preserve current behavior when truth data is missing.
- Required fix:
  - Guard hover enrichment when `nextStepSource` is absent.
  - Preserve the fallback card as a non-crashing mock-backed surface.

### 3. Activity panel lost its mock fallback path
- Status: `open`
- Severity: `P1`
- Evidence:
  - `ui/src/app-v2/AppV2.tsx:765` returns `[]` when `truthData.events` is missing
  - `ui/src/app-v2/AppV2.tsx:1062` renders only `projectedEvents`
  - Prior baseline at commit `99c5646` rendered `justNow` rows from `MOCK_CHANNEL_DATA`
- Impact:
  - Non-seeded projects now show an empty primary activity list instead of the previous mock timeline.
  - This is a direct regression against the packet's hybrid rule.
- Required fix:
  - Restore mock `justNow` rows whenever truth events are unavailable.
  - Keep the current truth path for seeded projects.

## Notes

### Follow-up but not blocker for this packet
- `ui/src/app-v2/AppV2.tsx:609` hardcodes `now` to `2026-04-28T23:59:00+08:00` for blocker age calculation.
- This should be replaced later with a deterministic but current-time-safe strategy if relative age remains part of the dashboard truth contract.

## Acceptance Condition

Accept the slice only after a bounded follow-up commit closes all three P1 issues above without expanding scope beyond the existing dashboard truth projection.
