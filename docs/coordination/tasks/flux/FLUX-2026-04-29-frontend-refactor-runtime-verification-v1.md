# Task: Flux Verification of Frontend Refactor Runtime Baseline

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-frontend-refactor-runtime-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md`, `docs/coordination/acceptance/2026-04-29-frontend-refactor-runtime-acceptance-decision.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | flux, verification, frontend, runtime, responsive, mobile, hover, navrail, dashboard |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Read-only verification only. Do not patch product code. Do not parallelize this with any other Flux task. If `docs/coordination/tasks/flux/FLUX-2026-04-29-foundation-hardening-verification-delivery-v1.md` is not yet written, finish that task first and start this packet second. |

## Objective

Run one strict read-only verification pass against Aegis's completed frontend refactor baseline.

This is not a taste review. This is a runtime and interaction truth review against the accepted design direction and active product contract.

Your job is to prove whether the new shell is actually usable and aligned, not merely whether the code compiles.

## Start Condition

Start this packet only after both conditions are true:

1. `docs/coordination/tasks/flux/FLUX-2026-04-29-foundation-hardening-verification-delivery-v1.md` is already written, or Lyra explicitly tells you that packet is not active.
2. The current repo already contains the refactor files listed in this packet.

If condition 1 is not true, finish the foundation hardening verification first.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md`
3. `docs/coordination/acceptance/2026-04-29-frontend-refactor-runtime-acceptance-decision.md`
4. `docs/prd-v0.5.md`
5. `docs/interaction-spec-v1.1.md`
6. `docs/ux-spec-v1.1.md`
7. this packet

## Need-to-Know Scope

Review only the refactored frontend shell and the surfaces directly affected by it.

Primary files to inspect:

- `ui/src/App.tsx`
- `ui/src/layouts/AppShell.tsx`
- `ui/src/layouts/MasterDetail.tsx`
- `ui/src/components/NavRail.tsx`
- `ui/src/components/SupervisionDashboard.tsx`
- `ui/src/components/MetricCard.tsx`
- `ui/src/components/WorkflowPanorama.tsx`
- `ui/src/components/ActionQueue.tsx`
- `ui/src/components/RecentActivity.tsx`
- `ui/src/components/MobileTabBar.tsx`
- `ui/src/components/BottomSheet.tsx`
- `ui/src/views/InboxView.tsx`
- `ui/src/views/TimelineView.tsx`
- `ui/src/views/WorkItemsView.tsx`
- `ui/src/views/SeatsView.tsx`
- `ui/src/views/ArtifactsView.tsx`
- `ui/src/views/PlaybookView.tsx`
- `ui/src/hooks/useResponsive.ts`
- `ui/src/styles/globals.css`
- `ui/src/styles/theme.ts`

Do not reopen:

- Rust/backend foundation code
- architecture docs
- product-contract rewriting
- old Mira packets as implementation scope
- any product-code patching

## Required Validation Commands

Run these commands from repo root and record the results:

```bash
cd ui && pnpm build
cd ui && npx tsc --noEmit
```

If your seat can do a live browser/dev-server pass, also run:

```bash
cd ui && pnpm dev --host 127.0.0.1 --port 4173
```

If a live browser pass is not possible on your seat, state that limitation explicitly and fall back to the strongest code-backed responsive inspection you can do. Do not fake runtime proof.

## Verification Questions

Answer every question explicitly:

1. Does the new shell expose a collapsible NavRail with the required 7 navigation items and the accepted `48px / 180px` behavior?
2. Is the default desktop landing surface the supervision dashboard with all four required blocks: metrics, workflow panorama, action queue, recent activity?
3. Does the app behave as master-detail on desktop, with the list narrowing to approximately `280px` and the detail using the main space?
4. Does tablet mode collapse into full-width detail after selection, with a clear return path?
5. Does mobile mode actually use the required bottom 4-tab navigation and hide the desktop NavRail?
6. Does mobile selected-detail behavior match the accepted bottom-sheet requirement, or is it still full-screen detail?
7. Does the Supervisor Command Bar still work correctly inside the refactored shell?
8. Is desktop terminal/status behavior still coherent, and is mobile correctly prevented from pretending to be a desktop terminal workspace?
9. Are hover, active, and selected states visually coherent in the new light-only theme?
10. Were the old shell references truly removed from the active path, or do dead-path regressions remain?

## Risk Focus from Lyra

You must explicitly inspect these risk areas:

1. `ui/src/components/BottomSheet.tsx` exists, but Lyra's code recheck suggests the narrow-layout path may still render full-screen detail in `ui/src/layouts/MasterDetail.tsx`. Treat this as a high-priority truth check.
2. The design direction requires a usable mobile command-entry path and desktop-only terminal affordance. Verify whether the new shell behavior actually honors this.
3. Because the refactor replaced the old sidebar/top-shell model, check for hover/selection regressions in NavRail, list rows, and dashboard action surfaces.

## Breakpoint Matrix

Verify the strongest evidence you can for these four breakpoint bands:

| Breakpoint | Expected layout truth |
|---|---|
| `>= 1280px` | expanded/collapsible desktop shell, dashboard first, master-detail with right detail area |
| `1024-1279px` | compact desktop shell, collapsed rail, main content still usable |
| `768-1023px` | tablet mode, selected detail takes full width with return path |
| `< 768px` | mobile mode, bottom 4-tab bar, single-column list flow, bottom-sheet detail expectation |

## Required Evidence Folder

Write logs, screenshots, or notes under:

- `.local/evidence/2026-04-29-frontend-refactor-runtime-verification/`

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-frontend-refactor-runtime-verification-delivery-v1.md`

Required sections:

1. Scope reviewed
2. Verification method
3. Breakpoint matrix results
4. Findings by severity (`Critical` / `High` / `Medium` / `Low`)
5. Build/type/runtime command results
6. Contract-alignment verdict (`PASS` / `HOLD`)
7. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] `pnpm build` and `tsc --noEmit` are re-run and recorded.
- [ ] Responsive shell behavior is checked across the four breakpoint bands.
- [ ] The mobile bottom-sheet requirement is explicitly proven or disputed.
- [ ] Command Bar, hover states, and desktop/mobile shell boundaries are explicitly checked.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_frontend_refactor_runtime_verification.txt
[Flux -> Lyra] Frontend Refactor Runtime Verification
completed:
- ...
validation:
- `cd ui && pnpm build` => ...
- `cd ui && npx tsc --noEmit` => ...
- live runtime pass => completed / not possible on this seat (reason: ...)
blockers:
- none / ...
verdict:
- PASS / HOLD
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-frontend-refactor-runtime-verification-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_frontend_refactor_runtime_verification /tmp/flux_to_lyra_frontend_refactor_runtime_verification.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_frontend_refactor_runtime_verification
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
