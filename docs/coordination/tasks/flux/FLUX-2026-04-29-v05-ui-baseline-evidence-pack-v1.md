# Task: Flux v0.5 UI Baseline Evidence Pack

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | FLUX-2026-04-29-v05-ui-baseline-evidence-pack-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-29-sg-01-ui-contract-baseline-decision.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | flux, qa, evidence, ui, baseline, v0.5 |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. Read-only scope. Do not patch product code, docs, or seed data outside the required delivery artifact and raw evidence folder. |

## Objective

Produce the final read-only evidence pack for the accepted v0.5 UI baseline.

`SG-01 UI Contract Baseline` is now `GO`. Your job is not to reopen accepted surfaces. Your job is to publish a clean verification matrix, command evidence, and any residual non-gate risks against the current accepted prototype.

## Input Files

Read only these inputs before validating:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/acceptance/2026-04-29-sg-01-ui-contract-baseline-decision.md`
6. `docs/coordination/acceptance/2026-04-29-lyra-mira-s7b-supervisor-command-bar-acceptance.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-mira-s7c-scoped-delegation-overlay-acceptance.md`
8. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
9. `docs/coordination/COORDINATION_RULES.md`
10. this packet

## Need-to-Know Scope

Read only:

- the input files above
- exact UI source files needed to verify the claimed surfaces
- exact config or command files needed to run validation

Do not reopen by default:

- archived product docs
- architecture source unless a verification claim directly depends on it
- Nimbus Rust implementation files
- old SG-01 verifier packets as authority

If a check cannot be completed, mark it explicitly with the exact blocking condition instead of widening scope.

## Write Boundary

Allowed write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md`
- `.local/evidence/2026-04-29-v05-ui-baseline/**`

## Required Coverage

Validate and report these surfaces:

1. shell truth: Project Overview default state, terminal auto-open behavior, shortcut help truth
2. Inbox next-action loop and deterministic list search
3. Timeline replay + drill-through
4. Handoff lifecycle strip
5. Supervisor Command Bar (`Cmd/Ctrl+K`, recent intents, proposal card, `Enter` confirm, `Esc` close)
6. Seat capability truth + scoped delegation overlay visibility
7. Session prompt-blocked banner + continuity pack preview
8. typed Artifact detail / chip / template+subtype filter path
9. mobile companion monitor surfaces
10. shared build result

## Non-goals

- No code changes
- No contract rewrite
- No reopening of already accepted S5/S6/S7 surfaces unless you find a build-backed regression in the current branch
- No new product findings outside residual-risk note format

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Coverage matrix maps each required surface to PASS / HOLD with exact evidence.
- [ ] `cd ui && pnpm build` result is recorded.
- [ ] Raw evidence paths are listed under `.local/evidence/2026-04-29-v05-ui-baseline/`.
- [ ] Any residual risk is clearly labeled as non-gate unless it truly blocks the accepted baseline.
- [ ] tmux reply is sent to Lyra after the artifact is written.

## Validation

Required command:

```bash
cd ui && pnpm build
```

Optional supporting checks only if needed:

```bash
cd ui && npx tsc --noEmit
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md`

Required sections:

1. Scope completed
2. Coverage matrix
3. Validation commands and results
4. Raw evidence paths
5. Residual risks (gate-blocking vs non-gate)
6. Recommended next owner

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_v05_ui_evidence.txt
[Flux -> Lyra] v0.5 UI Baseline Evidence Pack
completed:
- ...
validation:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md
- .local/evidence/2026-04-29-v05-ui-baseline/
MSG

tmux load-buffer -b flux_to_lyra_v05_ui_evidence /tmp/flux_to_lyra_v05_ui_evidence.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_v05_ui_evidence
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
