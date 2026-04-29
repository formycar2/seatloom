# Task: Flux Verification of Mira P1 Theme + Hover Fixes

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-p1-theme-hover-fixes-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`, `docs/coordination/tasks/mira/MIRA-2026-04-29-p1-theme-hover-fixes-v1.md`, `docs/coordination/tasks/mira/MIRA-2026-04-29-p1-theme-hover-fixes-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | flux, verification, ui, theme, hover, p1 |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Read-only verification only. Do not patch product code. Do not reopen accepted scope beyond the five audited `P1` defects. |

## Objective

Run one bounded read-only verification pass against Mira's accepted `P1` theme + hover repair packet.

Your job is to determine whether the five accepted `P1` defects are now actually closed in the current UI baseline, and whether any new regression appeared in the same touched surfaces.

This is not a new audit of the full visual system. Do not widen into the deferred `P2` list unless a `P1` fix caused a direct regression in the same surface.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`
6. `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`
7. `docs/coordination/tasks/mira/MIRA-2026-04-29-p1-theme-hover-fixes-v1.md`
8. `docs/coordination/tasks/mira/MIRA-2026-04-29-p1-theme-hover-fixes-delivery-v1.md`
9. `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`
10. this packet

## Need-to-Know Scope

Review only:

- `ui/src/styles/globals.css`
- `ui/src/components/EventRow.tsx`
- `ui/src/components/ArtifactDetail.tsx`
- `ui/src/layouts/Sidebar.tsx`
- any minimal runtime evidence needed to verify the same surfaces

Do not reopen by default:

- mobile companion surfaces
- command bar / delegation / continuity / prompt banner
- deferred `P2` polish items
- broad typography or palette taste feedback

## Verification Questions

Answer all of the following:

1. Is the broad hover descendant override truly gone?
2. Do EventRow and ArtifactDetail now use theme-aware semantic colors rather than hardcoded palette classes?
3. Are inactive sidebar sessions and timeline object labels materially more legible across the shipped theme presets?
4. Did the repair packet introduce any direct regression in the same touched surfaces?
5. Should the five `P1` issues be considered fully closed?

## Required Validation

At minimum:

```bash
cd ui && pnpm build
```

If a runtime check is possible on your seat, include it and save the evidence. If runtime preview is unavailable, say so explicitly and keep the verification bounded to code + build + any available static evidence.

## Required Evidence Folder

Write any logs or notes under:

- `.local/evidence/2026-04-29-p1-theme-hover-fixes-verification/`

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-p1-theme-hover-fixes-verification-delivery-v1.md`

Required sections:

1. Scope reviewed
2. Verification method
3. Result by defect (`D-01`, `D-02`, `D-03`, `D-07`, `D-10`)
4. Regression check
5. Validation commands and results
6. Verdict (`PASS` / `HOLD`)
7. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] All five `P1` fixes are explicitly verified or disputed.
- [ ] Build result is recorded.
- [ ] Any runtime limitation is stated clearly if present.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_p1_theme_hover_verification.txt
[Flux -> Lyra] P1 Theme + Hover Fix Verification
completed:
- ...
validation:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
verdict:
- PASS / HOLD
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-p1-theme-hover-fixes-verification-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_p1_theme_hover_verification /tmp/flux_to_lyra_p1_theme_hover_verification.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_p1_theme_hover_verification
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
