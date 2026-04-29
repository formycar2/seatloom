# Task: Mira S7B Enter Confirm Hotfix

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-29-s7b-enter-confirm-hotfix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-29 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`, `docs/coordination/tasks/mira/MIRA-2026-04-29-sg01-ui-closeout-delivery-v1.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | ui, sg-01, supervisor, command-bar, keyboard, hotfix |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | Execute this packet alone. Do not run any other UI packet until Lyra accepts or re-routes this hotfix. |

## Objective

Close the single remaining `SG-01` UI blocker in the Supervisor Command Bar.

Only repair the proposal-surface keyboard confirm path. Do not reopen any accepted surface.

## Input Files

Read only these inputs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/interaction-spec-v1.1.md` (`INT-02`)
3. `docs/acceptance-spec-v1.1.md` (`U-06`)
4. `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`
5. `docs/coordination/tasks/mira/MIRA-2026-04-29-sg01-ui-closeout-delivery-v1.md`
6. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
7. `docs/coordination/COORDINATION_RULES.md`
8. this packet

## Need-to-Know Scope

Read only:

- the input files above
- `ui/src/components/SupervisorCommandBar.tsx`

Do not reopen:

- delegation overlay files
- Session / mobile / theme / artifact / continuity / search surfaces
- shortcut routing unless you prove a direct blocker inside the command bar itself

## Write Boundary

Primary target only:

- `ui/src/components/SupervisorCommandBar.tsx`

## Required Outcome

Make the `Enter` confirm path truthful on the live proposal surface:

1. When a valid proposal card is open and inline edit is not active, pressing `Enter` must trigger the same confirm path as the `Confirm Proposal` button.
2. Keep `Enter` generating the proposal from the initial input when no proposal exists.
3. Keep inline edit behavior intact (`Enter` exits the inline title edit only, not full confirm).
4. Keep `Esc` close behavior intact.
5. Keep the shared UI build green.

## Non-goals

- No new product scope
- No visual redesign
- No changes to accepted dropdown, delegation, mobile, continuity, prompt, theme, or artifact-reader slices
- No store or routing changes unless you prove the command bar alone cannot satisfy `U-06`

## Done Definition

- [ ] `Enter` confirms an open valid proposal from the live proposal surface.
- [ ] `Enter` still generates a proposal from the initial input when no proposal exists.
- [ ] Inline edit behavior is preserved.
- [ ] `Esc` still closes the bar.
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

- `docs/coordination/tasks/mira/MIRA-2026-04-29-s7b-enter-confirm-hotfix-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Keyboard-path coverage
4. Build result
5. Blockers
6. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s7b_enter_hotfix.txt
[Mira -> Lyra] S7B Enter Confirm Hotfix
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-s7b-enter-confirm-hotfix-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s7b_enter_hotfix /tmp/mira_to_lyra_s7b_enter_hotfix.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s7b_enter_hotfix
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
