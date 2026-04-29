# Task: Mira Re-entry Delta Scan

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-29-reentry-delta-scan-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-29 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/MEMORY.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-s7a-mobile-queue-count-alignment-acceptance.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-sg01-post-s7a-ui-baseline-verification-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | ui, reentry, delta, scan, mira, product-truth |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | Read-only serial task. Do not edit `ui/src/**` in this packet. Do not start a new implementation slice until Lyra issues it after this scan. |

## Objective

Re-enter the active v0.5 UI contract with a minimal, truthful delta scan.

You were offline while Lyra closed the last `S7A` mobile truth fix directly and while Flux completed the post-`S7A` SG-01 verification packet. Lyra has now reviewed that verification and narrowed the real live gaps to delegation overlay plus Supervisor Command Bar wiring. This packet remains intentionally read-only so you can recover context without colliding with the accepted mobile surfaces or reopening disputed findings.

## Input Files

Mandatory reads, in order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/ux-spec-v1.1.md`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/coordination/MEMORY.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-mira-s7a-mobile-queue-count-alignment-acceptance.md`
8. `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-delivery-v1.md`
9. `docs/coordination/acceptance/2026-04-29-lyra-flux-sg01-post-s7a-ui-baseline-verification-acceptance.md`
10. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
11. `docs/coordination/COORDINATION_RULES.md`

## Need-to-Know Scope

Read only:

- this packet
- the input files above
- the exact UI files listed below

Do not consume by default:

- archived product docs as authority
- unrelated backend scaffold files
- old broken packet history unless a current artifact explicitly references it

## UI Read Scope

Inspect only these current prototype files:

- `ui/src/components/ProjectOverview.tsx`
- `ui/src/views/InboxView.tsx`
- `ui/src/views/WorkItemsView.tsx`
- `ui/src/views/TimelineView.tsx`
- `ui/src/components/HandoffDetail.tsx`
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/SeatDetail.tsx`
- `ui/src/components/ArtifactDetail.tsx`
- `ui/src/views/MobileCompanionView.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/styles/theme.ts`

## Required Outcome

Produce a concise re-entry matrix that tells Lyra three things:

### 1. What is already closed and should not be re-opened

List the surfaces that appear aligned enough that Mira should treat them as frozen unless Lyra issues a fix packet.

### 2. What the next implementation-ready UI slices should be

Name up to three candidate slices only.

For each candidate, provide:

- short slice name
- why it matters to the active contract
- exact primary file targets
- whether it overlaps with Flux's current verification scope
- recommended priority (`now` / `after Flux verdict`)

### 3. What Mira should explicitly avoid touching next

Call out any surfaces that are too unstable, already under verification, or blocked by product/engineering truth.

## Non-goals

- No code edits
- No style experiments
- No re-interpretation of product scope
- No new UI slice implementation in this packet

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] The artifact contains a frozen/next/avoid matrix.
- [ ] No `ui/src/**` file is modified.
- [ ] tmux reply is sent to Lyra after the artifact is written.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-29-reentry-delta-scan-delivery-v1.md`

Required sections:

1. Scope completed
2. Closed surfaces not to reopen
3. Next-slice candidates
4. Avoid list
5. Blockers
6. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_reentry_delta_scan.txt
[Mira -> Lyra] Re-entry Delta Scan
completed:
- ...
blockers:
- none / ...
next action:
- wait for next implementation packet
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-reentry-delta-scan-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_reentry_delta_scan /tmp/mira_to_lyra_reentry_delta_scan.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_reentry_delta_scan
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
