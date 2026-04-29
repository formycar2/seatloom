# Task: Mira Packet A - Interaction Affordance + Disabled-State Semantics

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-29-packet-a-interaction-affordance-state-semantics-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P1 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-p1-theme-hover-fixes-verification-acceptance.md`, `docs/coordination/tasks/mira/MIRA-2026-04-29-ui-ued-ux-design-brief-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-ui-ued-ux-design-brief-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | mira, ui, ued, ux, hover, disabled, affordance, packet-a |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | One active UI implementation packet only. Do not start Packet B in parallel. Do not widen into new theme presets, IA changes, or product capability work. |

## Objective

Implement the first bounded UI/UED/UX follow-up packet after the accepted `P1` repair and verification closure.

This packet fixes the remaining interaction-affordance and disabled-state semantics issues only:

- `D-04` secondary-button hover too weak
- `D-06` primary CTA hover relies on opacity only
- `D-09` sidebar icon-button hover affordance weak
- `D-11` MobileCompanion desktop hover missing
- `D-14` disabled buttons lose semantic identity

## Product Intent

This is not a broad restyle.

The user-visible goal is simple: when the operator hovers, taps, or encounters a disabled action, the system must feel deliberate and readable rather than vague, washed out, or inconsistent.

For every change in this packet, keep the causal chain visible:

1. what ambiguity the operator feels now;
2. what will still feel broken if we leave it untouched; and
3. what clearer behavior the operator sees after the change.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`
6. `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`
8. `docs/coordination/acceptance/2026-04-29-lyra-flux-p1-theme-hover-fixes-verification-acceptance.md`
9. `docs/coordination/tasks/mira/MIRA-2026-04-29-ui-ued-ux-design-brief-delivery-v1.md`
10. `docs/coordination/acceptance/2026-04-29-lyra-mira-ui-ued-ux-design-brief-acceptance.md`
11. this packet

## Need-to-Know Scope

In scope only:

- stronger and clearer hover states on light backgrounds;
- system-wide primary/secondary/button hover semantics in the touched files;
- disabled-state semantics that keep action identity visible;
- desktop hover parity for `MobileCompanionView` list rows;
- no regressions to the accepted `P1` fixes.

Out of scope:

- new theme preset creation;
- dark mode or dark-leaning preset shifts;
- texture, grain, ornamental borders, or decorative theme experiments;
- token-family cleanup (`text-ink` vs `text-text`) repo sweep;
- budget bar truth wiring;
- terminal preview palette work;
- typography-wide refactor.

## Hard Guardrails

1. All presets must remain **light-first** in this packet.
2. Do not use `hover:opacity-*` as the only affordance on any touched primary CTA.
3. Do not use `disabled:grayscale` on touched buttons.
4. Disabled actions must preserve semantic family identity and expose the lock reason through visible microcopy, a `title`, or an equally truthful bounded mechanism.
5. Do not reopen `D-01`, `D-02`, `D-03`, `D-07`, or `D-10`.
6. Keep Chinese-first copy intact unless a touched control needs a clearer disabled reason label.

## Write Boundary

Primary files only:

- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/DelegationOverlay.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/views/MobileCompanionView.tsx`
- `ui/src/views/InboxView.tsx`

Optional shared support only if truly required:

- `ui/src/styles/globals.css`
- `ui/src/styles/theme.ts`

Do not widen beyond these files without first recording the reason in the delivery artifact.

## Required Outcomes

### 1. Secondary-action hover becomes visible on light surfaces

Close `D-04` in:

- `ui/src/components/SessionDetail.tsx`
- `ui/src/views/InboxView.tsx`

Acceptance intent:

- hover must be perceptible at a glance on `paper-ledger` and `sage-archive`;
- do not rely on nearly invisible accent tint alone;
- use theme-token-based tint, border, or text contrast change.

### 2. Primary CTA hover becomes explicit, not opacity-only

Close `D-06` in:

- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/DelegationOverlay.tsx`

Acceptance intent:

- touched primary actions must feel clickable immediately;
- hover may use brightness, tint, or border emphasis, but must stay theme-token-based;
- active/pressed contrast must still fit the accepted shell language.

### 3. Icon-only sidebar buttons gain real hover affordance

Close `D-09` in:

- `ui/src/layouts/Sidebar.tsx`

Acceptance intent:

- hover must include more than icon-color change alone;
- small plus buttons should gain a visible surface or border state without becoming noisy.

### 4. Mobile companion rows gain desktop hover parity

Close `D-11` in:

- `ui/src/views/MobileCompanionView.tsx`

Acceptance intent:

- desktop pointer users must get hover feedback;
- touch active-state behavior must stay intact;
- keep the current mobile-monitor positioning and information hierarchy unchanged.

### 5. Disabled actions keep semantic identity and explain themselves

Close `D-14` in:

- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/DelegationOverlay.tsx`

Acceptance intent:

- disabled primary actions remain visually identifiable as primary;
- do not flatten them into generic gray blocks;
- reason exposure must be truthful and specific enough that the operator knows what unlocks the action.

## Validation

Required:

```bash
cd ui && pnpm build
```

If a local runtime preview is available on your seat, include one bounded note on hover/disabled-state behavior in the delivery artifact. If not, say so explicitly.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-29-packet-a-interaction-affordance-state-semantics-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Fix-by-defect (`D-04`, `D-06`, `D-09`, `D-11`, `D-14`)
4. Guardrails respected
5. Validation commands and results
6. Residual notes / what was intentionally not changed
7. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] `D-04`, `D-06`, `D-09`, `D-11`, and `D-14` are all explicitly addressed.
- [ ] `cd ui && pnpm build` passes.
- [ ] No darkening or preset-redefinition work is introduced.
- [ ] No accepted `P1` baseline item is reopened.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_packet_a_interaction_affordance.txt
[Mira -> Lyra] Packet A Interaction Affordance + Disabled-State Semantics
completed:
- ...
validation:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-packet-a-interaction-affordance-state-semantics-delivery-v1.md
MSG

tmux load-buffer -b mira_to_lyra_packet_a_interaction_affordance /tmp/mira_to_lyra_packet_a_interaction_affordance.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_packet_a_interaction_affordance
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
