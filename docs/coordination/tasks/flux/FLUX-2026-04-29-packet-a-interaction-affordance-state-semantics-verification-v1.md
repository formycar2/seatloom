# Task: Flux Verification of Mira Packet A Interaction Affordance + Disabled-State Semantics

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-packet-a-interaction-affordance-state-semantics-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`, `docs/coordination/tasks/mira/MIRA-2026-04-29-ui-ued-ux-design-brief-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-ui-ued-ux-design-brief-acceptance.md`, `docs/coordination/tasks/mira/MIRA-2026-04-29-packet-a-interaction-affordance-state-semantics-v1.md`, `docs/coordination/tasks/mira/MIRA-2026-04-29-packet-a-interaction-affordance-state-semantics-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-packet-a-interaction-affordance-state-semantics-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | flux, verification, ui, affordance, hover, disabled, packet-a |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Read-only verification only. Do not patch product code. Do not widen into Packet B typography, budget truth, or broader visual taste feedback. |

## Objective

Run one bounded read-only verification pass against Mira's accepted Packet A interaction-affordance slice.

Your job is to verify that the five targeted fixes below are now actually closed in the current UI baseline, and that no direct regression was introduced in the same touched surfaces:

- `D-04` secondary hover visibility
- `D-06` primary CTA hover discoverability
- `D-09` sidebar icon-button hover affordance
- `D-11` desktop hover parity for Mobile Companion rows
- `D-14` disabled-state semantic identity

This is not a new full-system audit. Keep the review tightly bounded to Packet A and the already accepted audit/design-brief basis.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`
6. `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`
7. `docs/coordination/tasks/mira/MIRA-2026-04-29-ui-ued-ux-design-brief-delivery-v1.md`
8. `docs/coordination/acceptance/2026-04-29-lyra-mira-ui-ued-ux-design-brief-acceptance.md`
9. `docs/coordination/tasks/mira/MIRA-2026-04-29-packet-a-interaction-affordance-state-semantics-v1.md`
10. `docs/coordination/tasks/mira/MIRA-2026-04-29-packet-a-interaction-affordance-state-semantics-delivery-v1.md`
11. `docs/coordination/acceptance/2026-04-29-lyra-mira-packet-a-interaction-affordance-state-semantics-acceptance.md`
12. this packet

## Need-to-Know Scope

Review only these implementation surfaces and any minimal evidence needed to verify them:

- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/DelegationOverlay.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/views/MobileCompanionView.tsx`
- `ui/src/views/InboxView.tsx`
- `ui/src/styles/globals.css` only if the touched surfaces depend on it

Do not widen into:

- Packet B typography work
- budget truth presentation
- artifact palette follow-ups
- new theme preset direction
- command bar / delegation logic outside hover and disabled-state semantics

## Verification Questions

Answer all of the following:

1. Are secondary hover states now perceptible on light presets in the touched Inbox and Session surfaces?
2. Do touched primary CTAs now use explicit hover affordance rather than opacity alone?
3. Do sidebar icon-only buttons now gain a visible hover surface rather than color-only change?
4. Do Mobile Companion rows now expose desktop pointer hover feedback without conflicting with touch active states?
5. Do disabled primary actions preserve semantic identity and explain the lock reason truthfully?
6. Did Packet A reopen any accepted `P1` hover/theme repair in the same surfaces?
7. Should Packet A be considered fully closed and safe to unblock Packet B dispatch?

## Required Validation

At minimum:

```bash
cd ui && pnpm build
```

If a runtime check is possible on your seat, include one bounded note and any saved evidence. If runtime preview is unavailable, say so explicitly and keep the verification limited to code + build + available static evidence.

## Required Evidence Folder

Write any logs or notes under:

- `.local/evidence/2026-04-29-packet-a-affordance-verification/`

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-packet-a-interaction-affordance-state-semantics-verification-delivery-v1.md`

Required sections:

1. Scope reviewed
2. Verification method
3. Result by defect (`D-04`, `D-06`, `D-09`, `D-11`, `D-14`)
4. Regression check
5. Validation commands and results
6. Verdict (`PASS` / `HOLD`)
7. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] All five Packet A defects are explicitly verified or disputed.
- [ ] Build result is recorded.
- [ ] Any runtime limitation is stated clearly if present.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_packet_a_affordance_verification.txt
[Flux -> Lyra] Packet A Affordance Verification
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
- docs/coordination/tasks/flux/FLUX-2026-04-29-packet-a-interaction-affordance-state-semantics-verification-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_packet_a_affordance_verification /tmp/flux_to_lyra_packet_a_affordance_verification.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_packet_a_affordance_verification
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
