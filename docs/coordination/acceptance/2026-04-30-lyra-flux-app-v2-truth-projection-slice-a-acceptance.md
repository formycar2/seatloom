# Acceptance: Flux app-v2 Truth Projection Slice A

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-30-flux-app-v2-truth-projection-slice-a-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| target | commit `365fc8e` on `track/infra-foundation` |
| verdict | PASS |
| tags | acceptance, flux, ui, app-v2, truth-projection, postgres, pinned-commit |

## Verdict

**PASS**

Flux closed the full bounded `Slice A` packet chain on a commit-pinned basis.

Lyra rechecked the final fix at `ui/src/app-v2/AppV2.tsx:802` and reran:

- `cd ui && pnpm build`
- `cd ui && npx tsc --noEmit`

Both passed on commit `365fc8e`.

This acceptance means the bounded dashboard truth-projection slice is now safe and acceptable in its declared scope:

1. blockers project from `projectData` instead of mock-only stage text,
2. active work projects from truth-backed workitems and sessions,
3. next-step projects from inbox truth with string-safe object refs,
4. activity projects from canonical events with mock fallback preserved, and
5. hover enrichment remains safe for truth rows and fallback rows.

## Scope Reviewed

- `docs/coordination/reviews/2026-04-30-lyra-app-v2-postgres-data-coverage-review.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-v1.md`
- `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-review.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-followup-v1.md`
- `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-followup-review-v2.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-final-fix-v1.md`
- `ui/src/app-v2/AppV2.tsx`
- commit `ab52edd`
- commit `7677672`
- commit `365fc8e`
- `cd ui && pnpm build`
- `cd ui && npx tsc --noEmit`

## Coverage Matrix

| Requirement slice | Evidence paths | Result | Notes |
|---|---|---|---|
| Blockers panel uses truth-backed projection from sessions / workitems / handoffs | commit `ab52edd`; `ui/src/app-v2/AppV2.tsx`; `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-v1.md` | PASS | Mock `currentStage.blockers` is no longer the primary source for seeded projects. |
| Active work panel uses truth-backed workitems / sessions | commit `ab52edd`; `ui/src/app-v2/AppV2.tsx` | PASS | Active and in-review work now project from persisted work/session families. |
| Next-step panel uses inbox truth and preserves string refs | commit `7677672`; `ui/src/app-v2/AppV2.tsx` | PASS | `InboxItem.object_ref` is rendered with a string-safe formatter. |
| Next-step hover stays safe when no inbox item exists | commit `7677672`; `ui/src/app-v2/AppV2.tsx` | PASS | Null-safe branching preserves the mock fallback card without hover crash. |
| Activity panel uses canonical events when available and mock `justNow` fallback when not | commit `7677672`; `ui/src/app-v2/AppV2.tsx` | PASS | Hybrid rule is restored: seeded projects show truth, empty projects keep baseline fallback. |
| Event hover stays safe for both canonical rows and mock fallback rows | commit `365fc8e`; `ui/src/app-v2/AppV2.tsx:802` | PASS | `buildEventHover()` now guards row shape before truth enrichment and safely maps missing evidence refs. |
| Slice remains build-safe and type-safe on the accepted commit | commit `365fc8e`; `cd ui && pnpm build`; `cd ui && npx tsc --noEmit` | PASS | Lyra reran both checks locally on the accepted commit. |

## Findings

No blocking findings remain inside the bounded `Slice A` scope.

Residual out-of-scope note only:

- The wider `app-v2` truth gap remains open. Documents, typed artifacts, sessions, handoffs, role bindings, delegations, and reconcile freshness are still not fully projected in `app-v2`. Those are already captured in `docs/coordination/reviews/2026-04-30-lyra-app-v2-postgres-data-coverage-review.md` and need separate packets.

## Required Fixes for Flux

None.

## Go / No-Go Recommendation

- **Accept bounded Slice A:** **GO**
- **Use commit `365fc8e` as the accepted baseline for this slice:** **GO**
- **Treat the broader `app-v2` truth coverage review as closed:** **NO-GO**
- **Open the next bounded truth-projection slice only with a new pinned packet:** **GO**

## Evidence Paths

- Coverage review: `docs/coordination/reviews/2026-04-30-lyra-app-v2-postgres-data-coverage-review.md`
- Initial packet: `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-v1.md`
- First review: `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-review.md`
- Follow-up packet: `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-followup-v1.md`
- Follow-up review: `docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-followup-review-v2.md`
- Final fix packet: `docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-final-fix-v1.md`
- Accepted implementation: `ui/src/app-v2/AppV2.tsx`
- Accepted commits: `ab52edd`, `7677672`, `365fc8e`
