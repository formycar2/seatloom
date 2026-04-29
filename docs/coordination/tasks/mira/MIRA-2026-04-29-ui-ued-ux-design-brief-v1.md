# Task: Mira UI/UED/UX Design Brief After P1 Repair

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-29-ui-ued-ux-design-brief-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P1 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/reviews/2026-04-29-lyra-mira-ui-ued-ux-readiness-note.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-p1-theme-hover-fixes-verification-v1.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | mira, ui, ued, ux, brief, typography, theme, hover |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | Read-only packet only. Do not patch product code in this packet. Keep at most one active design packet while Flux verification is still pending. |

## Objective

Prepare the next UI/UED/UX improvement cycle as a **read-only design brief**, not as a direct styling patch.

This packet exists to convert the accepted Flux audit plus sponsor dissatisfaction into one coherent execution brief that Mira can implement in bounded packets after Lyra closes the current Flux verification loop.

## Start Gate

This packet is **pre-issued now** so there is no restart delay.

Execution rule:

1. read the required sources now;
2. do not patch code;
3. treat `D-01`, `D-02`, `D-03`, `D-07`, and `D-10` as closed baseline items unless Flux's pending verification explicitly reopens them later;
4. if Flux's pending verification returns `HOLD`, update the brief to absorb only those reopened findings, not unrelated scope.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/reviews/2026-04-29-lyra-mira-ui-ued-ux-readiness-note.md`
6. `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`
8. `docs/coordination/acceptance/2026-04-29-lyra-mira-p1-theme-hover-fixes-acceptance.md`
9. `docs/coordination/tasks/flux/FLUX-2026-04-29-p1-theme-hover-fixes-verification-v1.md`
10. this packet

## Need-to-Know Scope

You are solving only the next UI/UED/UX cycle design problem:

- overall visual identity strength;
- Chinese/English typography harmony;
- hover / disabled / active interaction affordance consistency;
- theme-token discipline for the remaining open issue set;
- correct packetization into small implementation slices.

Do not widen into:

- IA redesign;
- new product capability proposals;
- backend or Tauri implementation;
- re-auditing closed SG-01 flows;
- broad aesthetic moodboarding with no file-level consequences.

## Source Issue Baseline

Treat the following as the active open backlog unless Flux verification later changes the status:

- `D-04` secondary-button hover too weak
- `D-05` artifact icon contrast too weak
- `D-06` primary action hover relies on opacity only
- `D-08` terminal preview color hardcoded
- `D-09` sidebar icon-button hover affordance weak
- `D-11` MobileCompanion desktop hover missing
- `D-12` transition rhythm inconsistent
- `D-13` `text-ink*` and `text-text*` token families coexist
- `D-14` disabled buttons lose semantic identity
- `D-15` SessionDetail budget bar still placeholder-driven
- `D-16` EventRow object-label overlap / re-baseline needed only if still open after recheck

Also incorporate sponsor-level concerns already accepted by Lyra:

1. the current theme presets are functional but still not persuasive enough as SeatLoom's visual point of view;
2. Chinese and English typography still feels uneven on several surfaces;
3. hover and disabled states are still not governed as one interaction system;
4. the theme system exists, but UED discipline is not yet coherent enough.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-29-ui-ued-ux-design-brief-delivery-v1.md`

Required sections:

1. Baseline assumptions and closed items not to reopen
2. Open issue inventory grouped by system problem
3. Typography recommendations (Chinese / English / mixed-script)
4. Theme preset direction recommendations with rationale
5. Hover / disabled / active-state interaction policy
6. Exact packet split for the next 2 implementation packets
7. File targets and acceptance criteria per packet
8. Risks / tradeoffs / what not to change yet
9. Recommended next owner

## Required Output Quality

The brief must be implementation-driving, not descriptive only.

For every recommendation, answer all three:

1. what user pain it solves;
2. what happens if we do not fix it now;
3. what user-visible behavior changes after the fix.

If any recommendation cannot answer those three, exclude it from the implementation packets.

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] The brief keeps closed `P1` fixes closed unless Flux evidence reopens them.
- [ ] Typography, theme direction, and interaction-state policy are all covered.
- [ ] The next implementation cycle is split into 2 bounded packets with exact file targets.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_ui_ued_ux_design_brief.txt
[Mira -> Lyra] UI/UED/UX Design Brief
completed:
- ...
blockers:
- none / ...
next action:
- wait for acceptance and implementation packet issuance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-ui-ued-ux-design-brief-delivery-v1.md
MSG

tmux load-buffer -b mira_to_lyra_ui_ued_ux_design_brief /tmp/mira_to_lyra_ui_ued_ux_design_brief.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_ui_ued_ux_design_brief
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
