# Acceptance: Frontend Refactor Runtime Acceptance Routing

| Field | Value |
|---|---|
| template | T5 |
| subtype | gate_decision |
| id | LYRA-2026-04-29-frontend-refactor-runtime-acceptance-decision-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| target | `Frontend Refactor P1-P7 Baseline` |
| verdict | HOLD |
| tags | acceptance, gate, frontend, ui, runtime, verification, refactor |

## Verdict

**HOLD**

Aegis's frontend refactor is structurally promising and the local build gates are green, but Lyra will not sign the new baseline yet.

The refactor changed the shell, dashboard, navigation model, responsive layout, and mobile path all at once. That blast radius is too large for build-only acceptance. The next step is a dedicated read-only runtime verification pass before any detail-fix packet is issued.

## Coverage Matrix

| Criterion | Result | Evidence | Gap |
|---|---|---|---|
| Local build passes on the refactored UI | PASS | `cd ui && pnpm build` | None |
| TypeScript type check passes on the refactored UI | PASS | `cd ui && npx tsc --noEmit` | None |
| New shell primitives exist in code | PASS | `ui/src/App.tsx`; `ui/src/layouts/AppShell.tsx`; `ui/src/layouts/MasterDetail.tsx`; `ui/src/components/NavRail.tsx`; `ui/src/components/SupervisionDashboard.tsx`; `ui/src/components/MobileTabBar.tsx` | None |
| Refactor broadly matches accepted direction at structural level | PASS | `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md`; files above | None |
| Responsive runtime behavior is independently proven across desktop / compact / tablet / mobile | HOLD | not yet independently verified | Needs Flux runtime verification |
| Mobile detail behavior is proven against the accepted bottom-sheet requirement | HOLD | `ui/src/components/BottomSheet.tsx`; `ui/src/layouts/MasterDetail.tsx` | Implementation path exists, but live wiring is not yet proven |
| Command Bar behavior is proven inside the new shell and mobile path | HOLD | `ui/src/components/SupervisorCommandBar.tsx`; `ui/src/App.tsx` | Needs runtime verification |
| Hover / active / affordance quality is proven after the refactor | HOLD | not yet independently verified | Needs live QA |

## Lyra Review Notes

Lyra's local code recheck confirms the following are present:

- collapsible `NavRail` with `48px` / `180px` width logic,
- `SupervisionDashboard` as the default desktop entry surface,
- `MasterDetail` as the main list/detail shell,
- mobile bottom tab bar path via `MobileTabBar`,
- successful local `build` and `tsc` execution.

However, the code review also highlights two runtime-risk areas that must be explicitly tested before sign-off:

1. `ui/src/components/BottomSheet.tsx` exists, but the narrow-layout path in `ui/src/layouts/MasterDetail.tsx` currently appears to render full-screen detail rather than a bottom sheet.
2. The accepted mobile-shell requirement includes a usable mobile command-entry path and desktop-only terminal behavior; these behaviors are not yet proven by build output alone.

## Gate Decision

- **Gate status:** **HOLD**
- **What happens next:** Flux runs one read-only runtime/interaction verification pass against the refactored frontend baseline.
- **What does not happen yet:** no new frontend repair packet is issued, and Lyra does not sign final frontend acceptance yet.
- **What can close this gate:** a verification artifact that proves the refactor at runtime or identifies bounded gaps for Aegis to fix.

## Follow-up Action

1. Flux executes `docs/coordination/tasks/flux/FLUX-2026-04-29-frontend-refactor-runtime-verification-v1.md`.
2. Lyra reviews Flux's evidence package and then issues either:
   - `GO` for the new frontend baseline, or
   - one bounded repair packet with exact file targets.

## Evidence Paths

- Design direction: `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md`
- Shell entry: `ui/src/App.tsx`
- New app shell: `ui/src/layouts/AppShell.tsx`
- Master-detail layout: `ui/src/layouts/MasterDetail.tsx`
- Navigation rail: `ui/src/components/NavRail.tsx`
- Dashboard: `ui/src/components/SupervisionDashboard.tsx`
- Mobile tab bar: `ui/src/components/MobileTabBar.tsx`
- Local build gate: `cd ui && pnpm build`
- Local type gate: `cd ui && npx tsc --noEmit`
