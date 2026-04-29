# Task: Mira S7C Scoped Delegation Overlay

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-29-s7c-scoped-delegation-overlay-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-sg01-post-s7a-ui-baseline-verification-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-reentry-delta-scan-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | ui, sg-01, delegation, workitem, seat-card, timeline, mira |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | This packet may run in parallel with `MIRA-2026-04-29-s7b-supervisor-command-bar-v1` only. Own the delegation write set end-to-end and do not touch the command-bar files. |

## Objective

Close the remaining `SG-01` blocker around scoped delegation.

Seat capability truth is already visible, but the prototype still cannot express temporary delegated responsibility without mutating the original owner. This packet adds the missing overlay and the visible delegated-state trail.

## Input Files

Read only these inputs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md` (`US-P0-04`)
3. `docs/interaction-spec-v1.1.md` (`INT-03`, `INT-04`)
4. `docs/ux-spec-v1.1.md` (`UX-04`, `UX-05`, `UX-11`)
5. `docs/acceptance-spec-v1.1.md` (`US-P0-04`)
6. `docs/coordination/acceptance/2026-04-29-lyra-flux-sg01-post-s7a-ui-baseline-verification-acceptance.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-mira-reentry-delta-scan-acceptance.md`
8. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
9. `docs/coordination/COORDINATION_RULES.md`
10. this packet

## Need-to-Know Scope

Read only:

- the input files above
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/SeatDetail.tsx`
- `ui/src/views/TimelineView.tsx`
- `ui/src/components/EventRow.tsx`
- `ui/src/stores/useDataStore.ts`
- `ui/src/types/index.ts`
- `ui/src/utils/display.ts`

Do not reopen by default:

- command-bar / shortcut wiring (that belongs to `S7B`)
- continuity preview / prompt-blocked surfaces
- mobile companion surfaces
- theme presets

## Write Boundary

Primary targets:

- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/SeatDetail.tsx`
- `ui/src/components/DelegationOverlay.tsx` *(new file allowed)*
- `ui/src/stores/useDataStore.ts`
- `ui/src/types/index.ts`

Optional support targets only if required:

- `ui/src/components/EventRow.tsx`
- `ui/src/views/TimelineView.tsx`
- `ui/src/utils/display.ts`

## Required Outcome

### 1. Delegation entrypoint from active work

Add a visible `Delegate this work` action from `WorkItemDetail` when a routable owner seat exists.

### 2. Scoped delegation overlay

Implement a bounded overlay / drawer that captures at least:

- source seat,
- delegate seat,
- scope description,
- issuer,
- expiry,
- authority limit note.

Rules:

- this must read as temporary and scoped, not permanent reassignment;
- block confirmation when delegate seat, scope, issuer, or expiry is missing;
- block confirmation when the delegate seat equals the source seat.

### 3. Visible delegated state after confirm

After confirm, the prototype must visibly preserve the overlay result in canonical UI state:

- WorkItem detail shows active delegated responsibility,
- Seat detail shows an active delegation badge with scope, issuer, and expiry,
- Timeline receives a readable delegation event row.

Prototype boundary:

- do not mutate the original owner seat identity;
- a lightweight local delegation record inside prototype state is acceptable;
- deterministic local event append is acceptable.

### 4. Readable delegation labeling

If support files need updates, ensure delegation rows read in human language rather than raw ids only.

## Non-goals

- No P3 module-topology surface
- No backend IPC or Rust-side persistence work
- No permanent ownership transfer
- No broader timeline redesign beyond what is needed for readable delegation visibility

## Done Definition

- [ ] `WorkItemDetail` exposes a `Delegate this work` entrypoint.
- [ ] The overlay captures scope, issuer, expiry, and authority limit note.
- [ ] Invalid delegation attempts are blocked inline.
- [ ] After confirm, delegated state is visible in WorkItem detail, Seat detail, and Timeline.
- [ ] Original seat identity remains unchanged.
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

- `docs/coordination/tasks/mira/MIRA-2026-04-29-s7c-scoped-delegation-overlay-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Overlay fields and validation
4. Delegated-state visibility coverage
5. Timeline projection behavior
6. Build result
7. Blockers
8. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s7c_delegation.txt
[Mira -> Lyra] S7C Scoped Delegation Overlay
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-s7c-scoped-delegation-overlay-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s7c_delegation /tmp/mira_to_lyra_s7c_delegation.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s7c_delegation
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
