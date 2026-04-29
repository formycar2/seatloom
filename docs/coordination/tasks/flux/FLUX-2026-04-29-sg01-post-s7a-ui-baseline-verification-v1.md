# Task: Flux SG-01 Post-S7A UI Baseline Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/MEMORY.md`, `docs/coordination/acceptance/2026-04-29-lyra-mira-s7a-mobile-queue-count-alignment-acceptance.md`, `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | verification, ui, sg01, mobile, artifact, workloop, flux |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Verification only. Do not patch product code. Keep support artifacts inside `.local/**` and the delivery file inside `docs/coordination/tasks/flux/**`. |

## Objective

Run the first post-`S7A` evidence-backed UI baseline verification against the active v0.5 contract set.

This packet exists to tell Lyra exactly what is now `PASS`, what remains `HOLD`, and whether `SG-01 UI Contract Baseline` can move forward after the mobile truth fix and scaffold closure.

## Input Files

Mandatory reads, in order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/ux-spec-v1.1.md`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/coordination/MEMORY.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-mira-s7a-mobile-queue-count-alignment-acceptance.md`
8. `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md`
9. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
10. `docs/coordination/COORDINATION_RULES.md`

## Need-to-Know Scope

Read only:

- this packet
- the input files above
- the exact UI files, commands, and evidence captures needed to verify the listed surfaces

Do not consume by default:

- archived product docs as primary authority
- unrelated backend scaffold files except when a UI claim explicitly depends on them
- long historical chat context

If blocked, mark the check blocked instead of widening scope or patching source.

## Verification Scope

Verify these current prototype claims only:

1. Project overview / shell truth
2. Inbox + WorkItems deterministic list search
3. Timeline drill-through and typed artifact opening path
4. Handoff state strip
5. WorkItem review tier strip (`L1` / `L2` / `L3` + `change_tier_record` visibility)
6. Session prompt-blocked banner and continuity pack preview
7. Seat capability-truth detail view
8. Typed artifact metadata strip / family + subtype visibility
9. Theme preset selector persistence and visible preset identity
10. Mobile companion truth after `S7A` close-out

## Required Outcome

### 1. Produce a verdict-ready matrix

For each surface above, report one of:

- `PASS`
- `HOLD`
- `BLOCKED`

Every non-pass row must include:

- expected behavior
- actual behavior
- exact reproduction path
- exact file path(s) or evidence path(s)
- severity (`P0` / `P1` / `P2`)

### 2. Use evidence, not inference alone

At minimum include:

- `cd ui && pnpm build`
- any run command needed to inspect the current UI locally
- screenshots or console/log evidence under `.local/evidence/2026-04-29-sg01-post-s7a/` if needed
- explicit note when a check is code-read-only versus runtime-observed

### 3. Stay inside verifier boundaries

- do not modify `ui/src/**`
- do not rewrite product interpretation
- do not silently fix issues
- if a claim fails, return it to Lyra as a concrete finding

## Non-goals

- No source-code patching
- No architecture redesign
- No reopening of the closed scaffold packet
- No broad regression sweep outside the ten surfaces above

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Each of the ten verification surfaces has a verdict.
- [ ] Every `HOLD` or `BLOCKED` row includes reproduction and evidence paths.
- [ ] Support artifacts stay inside `.local/**`.
- [ ] No product source file is edited.
- [ ] tmux reply is sent to Lyra after the artifact is written.

## Validation Commands

Minimum:

```bash
cd ui && pnpm build
```

Use additional local run / preview commands only if needed to produce truthful runtime evidence.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-delivery-v1.md`

Required sections:

1. Scope completed
2. Verification matrix
3. Commands run
4. Evidence paths
5. Findings by severity
6. Overall recommendation (`advance SG-01` / `hold SG-01`)
7. Blockers / next owner

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_sg01_post_s7a.txt
[Flux -> Lyra] SG-01 Post-S7A UI Baseline Verification
completed:
- ...
validation:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
recommendation:
- advance SG-01 / hold SG-01
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-sg01-post-s7a-ui-baseline-verification-delivery-v1.md
- ...
MSG

tmux load-buffer -b flux_to_lyra_sg01_post_s7a /tmp/flux_to_lyra_sg01_post_s7a.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_sg01_post_s7a
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
