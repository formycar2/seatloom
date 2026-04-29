# MIRA-2026-04-28-v05-UI-Realignment-Implementation-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-29 16:00 |
| Acceptance owner | Lyra |
| Stage | Product Baseline Freeze / pre-SG-01 |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Need-to-know / token-efficient |

## 1. Micro-brief

1. You are back online and resume **primary UI ownership** for SeatLoom.
2. The active product contract has advanced to the v0.5 set; the current prototype is **not yet aligned** to that contract.
3. Your job is to realign the React prototype to the active v0.5 product/interaction/UX/acceptance contract and implement the highest-value UI gaps without regressing already-accepted baselines.
4. Start from the current codebase and accepted Flux/Mira carry-forward work; do not restart the UI from scratch.
5. Persist the delivery artifact in English; keep terminal/tmux progress updates in Chinese.

## 2. Decisions already frozen

- `docs/PRODUCT_TRUTH.md` is the only entrypoint.
- Active authority set:
  - `docs/prd-v0.5.md`
  - `docs/interaction-spec-v1.1.md`
  - `docs/ux-spec-v1.1.md`
  - `docs/acceptance-spec-v1.1.md`
- Architecture docs may constrain implementation, but they do not redefine UI behavior:
  - `docs/architecture-decisions.md`
  - `docs/architecture-design.md`
- Preserve these accepted baselines unless the active contract explicitly requires refinement:
  - typed Artifact objectization baseline
  - Chinese high-density demo content baseline
  - drift-signal policy (`no standalone drift alert`; only transient reconcile summary after explicit reconcile flows)
  - bottom Terminal panel, not a main tab
  - Inbox row -> Detail linkage
  - basic Timeline filters already added by Flux
- Do **not** use archived v0.4 / v1.0 product docs as primary authority for this packet.
- If a required UI behavior still appears ambiguous, escalate the exact clause conflict to Lyra instead of inventing product meaning.

## 3. Contract pack

Mandatory reads, in this order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/ux-spec-v1.1.md`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
7. `docs/coordination/COORDINATION_RULES.md`
8. `docs/coordination/reviews/2026-04-28-lyra-review-response.md`
9. `docs/coordination/reviews/2026-04-28-lyra-pre-implementation-freeze-decision.md`
10. `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`
11. `docs/coordination/tasks/flux/FLUX-2026-04-28-acting-mira-contract-repair-delivery-v1.md`

## 4. Need-to-know scope

Read only:

- this packet
- the contract pack above
- the exact UI files you must inspect or patch

Primary write scope:

- `ui/src/App.tsx`
- `ui/src/layouts/*`
- `ui/src/views/*`
- `ui/src/components/*`
- `ui/src/hooks/*`
- `ui/src/stores/*`
- `ui/src/mockData.ts`
- `ui/src/styles/*`
- `ui/src/types/*`
- `ui/src/utils/*`

Do not consume by default:

- archived product docs as authority
- unrelated backend / Rust implementation details
- broad repo history
- old verifier packets based on superseded contracts

## 5. Required work

### P0 - Realign the current UI to the active contract

| Slice | Requirement | Minimum outcome |
|---|---|---|
| Shell and default state | Align the main shell to the active v0.5 layout and default states | Sidebar / Main / Detail / bottom Terminal stay coherent; default Detail states are intentional and contract-safe |
| Project switching | Recheck and repair project switcher, recent/pinned behavior, switch protection, project-memory restoration, and All Projects entry behavior | Switching behavior matches the active contract instead of the older SG-01 recovery baseline |
| Inbox loop | Ensure Inbox remains a human action queue tied to canonical object detail and state transitions | Row selection, actions, and resulting object-state presentation stay coherent |
| Timeline replay | Keep and refine event replay, filters, and event-to-object drill-through | Current Flux-added filters remain, but the flow becomes v0.5-safe and legible |
| WorkItem / Handoff loop | Realign WorkItem detail, Handoff detail, and creation / review / reissue surfaces to the newer contract | Visible states, available actions, and linked evidence no longer drift from the active stories/specs |
| Continuity and execution surfaces | Reflect capability truth and prompt-state handling where the active contract now requires it | Attach / Wrap / Switch / Session detail / Prompt-blocked surfaces do not promise unsupported behavior |
| Artifact surfaces | Preserve and refine the typed Artifact baseline across Timeline / Inbox / Detail / object detail | Do not regress `template + subtype`, type chips, or filterability |

### P0 - Include a compact realignment matrix in the delivery

Before the code summary, publish a matrix:

`surface or flow -> already aligned / repaired / still missing / needs Lyra decision -> exact file targets`

This matrix is part of the required delivery artifact and becomes the acceptance map.

### P1 - Quality guardrails on touched surfaces

- Preserve detailed Chinese content and do not reintroduce shallow English fallback copy.
- Maintain mixed CJK/Latin readability; no tiny uppercase/tracking-heavy text on touched surfaces.
- Keep focus states, keyboard visibility, and clear status signaling on touched interactions.

## 6. Non-goals

- Do not redesign the entire product IA from zero.
- Do not revert the accepted Artifact objectization baseline.
- Do not turn this into backend/runtime implementation work.
- Do not rely on static screenshots as the sole artifact.
- Do not wait for a new Lyra acceptance packet before starting code unless you hit a true contract blocker.

## 7. Done definition

All items below must be true:

- The prototype is updated in code, not only as notes.
- The delivery artifact includes the required realignment matrix and changed-file summary.
- `cd ui && pnpm build` is run and the result is recorded.
- Touched surfaces preserve the accepted Artifact baseline and Chinese content baseline.
- The delivery artifact explicitly lists any remaining blockers or decisions still needed from Lyra.

## 8. Required delivery artifact

Publish completion as:

- `docs/coordination/tasks/mira/MIRA-2026-04-28-v05-ui-realignment-implementation-delivery-v1.md`

Required sections:

1. Micro-brief
2. Realignment matrix
3. Changed files
4. Flow coverage checklist
5. Build command + result
6. Remaining blockers / decision requests
7. Evidence paths

## 9. Reporting rules

- Durable artifact: English only.
- Terminal/tmux progress summary: Chinese only.
- Direct seat-to-seat dispatch remains English with artifact-path references.
- Keep updates delta-first and short.
