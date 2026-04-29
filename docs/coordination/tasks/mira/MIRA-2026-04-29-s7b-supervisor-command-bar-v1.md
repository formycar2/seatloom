# Task: Mira S7B Supervisor Command Bar

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-29-s7b-supervisor-command-bar-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-sg01-post-s7a-ui-baseline-verification-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-reentry-delta-scan-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | ui, sg-01, supervisor, command-bar, shortcuts, mira |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | This packet may run in parallel with `MIRA-2026-04-29-s7c-scoped-delegation-overlay-v1` only. Do not exceed 2 active SG-01 closure packets at once. Keep the write set disjoint. |

## Objective

Close the remaining `SG-01` blocker around the Supervisor entry surface.

Today `Cmd/Ctrl+K` still opens the wrong shell route. The prototype needs a real Supervisor Command Bar that is visible, keyboard-first, and truthful to the v0.5 contract without widening into backend orchestration.

## Input Files

Read only these inputs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md` (`INT-02`)
4. `docs/ux-spec-v1.1.md` (`UX-02`)
5. `docs/acceptance-spec-v1.1.md` (`US-P0-02`, `U-06`)
6. `docs/coordination/acceptance/2026-04-29-lyra-flux-sg01-post-s7a-ui-baseline-verification-acceptance.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-mira-reentry-delta-scan-acceptance.md`
8. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
9. `docs/coordination/COORDINATION_RULES.md`
10. this packet

## Need-to-Know Scope

Read only:

- the input files above
- `ui/src/App.tsx`
- `ui/src/hooks/useGlobalShortcuts.ts`
- `ui/src/components/WorkItemForm.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx`
- any new command-bar component you add

Do not reopen by default:

- mobile companion surfaces
- continuity preview / prompt-blocked surfaces
- theme-token baseline
- scoped delegation work (that belongs to `S7C`)

## Write Boundary

Primary targets:

- `ui/src/App.tsx`
- `ui/src/hooks/useGlobalShortcuts.ts`
- `ui/src/components/SupervisorCommandBar.tsx` *(new file allowed)*

Optional support targets only if required:

- `ui/src/components/ShortcutHelpDialog.tsx`
- `ui/src/components/WorkItemForm.tsx`

## Required Outcome

### 1. Correct keyboard entrypoint

- `Cmd/Ctrl+K` opens the Supervisor Command Bar from any screen.
- `Esc` closes the bar when it is open.
- The shortcut help surface must describe the real behavior, not the previous All Projects toggle.
- The All Projects surface must remain reachable from existing visible navigation; do not leave it silently bound to `Cmd/Ctrl+K`.

### 2. Real command bar surface

Implement a centered desktop command bar aligned to `UX-02`:

- target width: about `640px`,
- light-first theme-token styling only,
- placeholder aligned to the contract,
- recent commands dropdown on focus,
- visible mode choices for `Create`, `Explain`, and `Find evidence`.

Prototype boundary:

- only the `Create` mode must be fully wired in this packet;
- `Explain` and `Find evidence` may be visibly present but may stop at bounded non-mutating placeholder cards.

### 3. Deterministic suggestion card for `Create`

After submit in `Create` mode, show a structured suggestion card that includes:

- intent restatement,
- proposed WorkItem title,
- proposed owner seat,
- capability / constraint rationale,
- linked evidence refs,
- acceptance criteria,
- budget estimate,
- impact summary,
- actions: `Confirm`, `Edit`, `Cancel`.

Rules:

- keep proposal synthesis deterministic and local to the prototype;
- do not call any backend or external model;
- `Edit` must stay inline inside the card, not open a blank replacement screen.

### 4. Confirm path must be visible

- `Confirm` must produce a visible prototype result, not a dead button.
- Preferred bounded path: create one deterministic local WorkItem draft through existing store behavior, then bring the operator to that object.

## Non-goals

- No backend routing engine
- No real evidence retrieval implementation
- No why-answer grounding logic beyond a bounded placeholder card
- No changes to accepted mobile, continuity, prompt, or theme surfaces

## Done Definition

- [ ] `Cmd/Ctrl+K` opens the Supervisor Command Bar.
- [ ] Command Bar is visible, keyboard-first, and uses shared theme tokens.
- [ ] `Create` mode renders a structured suggestion card with owner, evidence, AC, impact, and budget before confirm.
- [ ] `Confirm` has a visible prototype outcome.
- [ ] Shortcut help is updated to the real behavior.
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

- `docs/coordination/tasks/mira/MIRA-2026-04-29-s7b-supervisor-command-bar-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Shortcut wiring behavior
4. Command Bar surface coverage
5. Suggestion card behavior
6. Build result
7. Blockers
8. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s7b_command_bar.txt
[Mira -> Lyra] S7B Supervisor Command Bar
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-s7b-supervisor-command-bar-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s7b_command_bar /tmp/mira_to_lyra_s7b_command_bar.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s7b_command_bar
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
