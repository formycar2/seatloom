# Task: Mira SG-01 UI Close-out

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-29-sg01-ui-closeout-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | ui, sg-01, closeout, supervisor, delegation, mira |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | No new UI packet should run in parallel with this close-out. Work this packet serially and keep the write set bounded to the files below. |

## Objective

Close the last bounded UI gaps that still keep `SG-01` on hold.

Do **not** reopen accepted mobile, continuity, prompt, theme, artifact, or search surfaces. This packet only repairs the four live issues found in the `S7B` and `S7C` acceptance reviews.

## Input Files

Read only these inputs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md` (`INT-02`, `INT-03`, `INT-04`)
4. `docs/ux-spec-v1.1.md` (`UX-02`, `UX-04`, `UX-05`, `UX-11`)
5. `docs/acceptance-spec-v1.1.md` (`US-P0-02`, `US-P0-04`, `U-06`)
6. `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`
8. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
9. `docs/coordination/COORDINATION_RULES.md`
10. this packet

## Need-to-Know Scope

Read only:

- the input files above
- `ui/src/components/SupervisorCommandBar.tsx`
- `ui/src/components/DelegationOverlay.tsx`
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/hooks/useGlobalShortcuts.ts` *(only if needed for keyboard preservation)*

Do not reopen by default:

- `SessionDetail`
- mobile companion views
- theme preset files
- Timeline redesign beyond existing delegation projection
- seat-card capability truth beyond delegation visibility

## Write Boundary

Primary targets:

- `ui/src/components/SupervisorCommandBar.tsx`
- `ui/src/components/DelegationOverlay.tsx`
- `ui/src/components/WorkItemDetail.tsx`

Optional support target only if required:

- `ui/src/hooks/useGlobalShortcuts.ts`

## Required Outcome

### 1. Finish the Supervisor Command Bar keyboard contract

In `ui/src/components/SupervisorCommandBar.tsx`:

- show a visible recent-commands dropdown when the input receives focus,
- keep the dropdown deterministic and local-only,
- let `Enter` confirm the proposal card when a proposal is already open,
- preserve the current inline edit path and `Esc` close behavior.

### 2. Make delegation issuer explicit and validatable

In `ui/src/components/DelegationOverlay.tsx`:

- add a visible `issuer` control,
- block confirm when issuer is blank,
- keep self-delegation blocking,
- keep the overlay clearly temporary and scoped.

### 3. Gate delegation to routable-owner truth

In `ui/src/components/WorkItemDetail.tsx`:

- show `Delegate this work` only when the WorkItem has a real routable owner seat,
- remove the unsafe `owner!` overlay mount path,
- keep delegated-state rendering unchanged when a valid owner exists.

### 4. Keep SG-01 closure evidence clean

- keep the shared UI build green,
- write one delivery artifact that explicitly calls out which of the four close-out issues are now resolved,
- reply to Lyra through tmux after the artifact is written.

## Non-goals

- No new product scope
- No backend or store refactor
- No changes to accepted mobile, prompt, continuity, theme, or artifact-reader slices
- No redesign of Timeline, Seat Detail, or Project Overview outside the bounded fixes above

## Done Definition

- [ ] Recent commands dropdown opens on focus in the Supervisor Command Bar.
- [ ] `Enter` confirms an open Supervisor proposal card.
- [ ] Delegation overlay exposes and validates `issuer`.
- [ ] Delegation entrypoint only appears when a routable owner seat exists.
- [ ] The unsafe `owner!` overlay mount path is removed.
- [ ] `cd ui && pnpm build` passes.
- [ ] Delivery artifact is written at the required path.
- [ ] tmux reply is sent to Lyra after the artifact is written.

## Validation

Run:

```bash
cd ui && pnpm build
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-29-sg01-ui-closeout-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Supervisor Command Bar close-out coverage
4. Delegation close-out coverage
5. Build result
6. Blockers
7. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_sg01_closeout.txt
[Mira -> Lyra] SG-01 UI Close-out
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-sg01-ui-closeout-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_sg01_closeout /tmp/mira_to_lyra_sg01_closeout.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_sg01_closeout
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
