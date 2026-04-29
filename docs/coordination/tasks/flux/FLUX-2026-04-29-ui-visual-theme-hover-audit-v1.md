# Task: Flux UI Visual Theme + Hover Audit

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-ui-visual-theme-hover-audit-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-29-sg-01-ui-contract-baseline-decision.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-v05-ui-baseline-evidence-pack-acceptance.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-system-acceptance.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | verification, flux, ui, theme, hover, qa, visual, presets |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. Do not patch product code. Keep all outputs inside the delivery artifact and `.local/evidence/**`. |

## Objective

Run a focused visual QA pass on the current UI baseline because sponsor feedback indicates two concrete risks:

1. the current light-first theme direction is not yet visually satisfactory; and
2. multiple hover-state defects or inconsistencies are likely present.

Your job is not to redesign the product. Your job is to produce a precise, evidence-backed defect list and a fix-ready recommendation set for the next UI owner.

## Input Files

Read only these inputs before validating:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/acceptance/2026-04-29-sg-01-ui-contract-baseline-decision.md`
6. `docs/coordination/acceptance/2026-04-29-lyra-flux-v05-ui-baseline-evidence-pack-acceptance.md`
7. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-system-acceptance.md`
8. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md`
9. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
10. `docs/coordination/COORDINATION_RULES.md`
11. this packet

## Need-to-Know Scope

Read only:

- the files above;
- the exact UI files needed to inspect reported visual issues;
- the live prototype or local preview needed to reproduce hover behavior.

Do not do:

- product-contract reinterpretation;
- code patching;
- token-heavy broad critique with no reproduction path;
- reopening already accepted non-visual baselines unless the current branch visibly regressed.

If runtime inspection is blocked, record the exact blocker and fall back to code-backed findings only.

## Verification Scope

Test all three shipped theme presets:

- `paper-ledger` (`纸账本`)
- `harbor-blueprint` (`港湾蓝图`)
- `sage-archive` (`鼠尾档案`)

Focus on these risk areas:

1. color harmony and legibility in light mode;
2. hover / focus / active / selected state correctness;
3. whether hover causes layout shift, contrast loss, border jumps, or wrong cursor/state semantics;
4. whether selected state and hover state are visually distinguishable;
5. whether typography, size, weight, or mixed-font choices reduce Chinese readability;
6. whether component-local styling bypasses theme tokens and creates inconsistent behavior.

Inspect these surfaces at minimum:

1. Sidebar navigation, selected seat rows, and theme selector
2. Top navigation, project switcher, and top-level action triggers
3. Inbox rows, chips, and quick-action buttons
4. Timeline rows, filters, and drill-through triggers
5. WorkItems list, WorkItem detail actions, and delegation entrypoint
6. Handoff detail strip and related action controls
7. Seat detail cards, badges, capability sections, and chips
8. Session detail actions, prompt banner actions, and continuity blocks
9. Artifact chips, artifact references, and detail metadata rows
10. Mobile companion cards and high-priority action rows, if the current preview exposes them

## Required Output

Write one delivery artifact that contains:

### 1. Visual defect matrix

For every defect, include:

- defect ID;
- severity (`P0` / `P1` / `P2`);
- category (`theme`, `hover`, `typography`, `state semantics`, `token inconsistency`);
- affected preset(s);
- affected surface;
- exact reproduction path;
- expected behavior;
- actual behavior;
- suspected file target(s);
- evidence path(s).

### 2. Triage summary

Summarize defects into three buckets:

- immediate bug fixes;
- visual-system fixes;
- optional polish items.

### 3. Fix-ready recommendation set

Give Lyra a short packet-ready recommendation list for the next UI owner. Each recommendation must map to exact file targets and must say whether it should be fixed by:

- token-layer adjustment;
- component-level state fix; or
- typography/system cleanup.

## Validation and Evidence

Minimum command:

```bash
cd ui && pnpm build
```

Use live preview evidence if available. Preferred order:

1. inspect `http://localhost:5173/` if already running;
2. otherwise run a bounded local preview command and record it;
3. capture screenshots, notes, or logs under the required evidence folder.

## Write Boundary

Allowed write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`
- `.local/evidence/2026-04-29-ui-visual-theme-hover-audit/**`

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] At least one runtime-based check is attempted and recorded, unless blocked.
- [ ] Defects are grouped by severity and category with exact reproduction steps.
- [ ] Every finding includes suspected file targets.
- [ ] The artifact clearly separates bugs from subjective polish.
- [ ] tmux reply is sent to Lyra after the artifact is written.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`

Required sections:

1. Scope completed
2. Runtime / code inspection method
3. Visual defect matrix
4. Triage summary
5. Fix-ready recommendation set
6. Validation commands and results
7. Evidence paths
8. Recommended next owner

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_ui_visual_theme_hover_audit.txt
[Flux -> Lyra] UI Visual Theme + Hover Audit
completed:
- ...
validation:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md
- .local/evidence/2026-04-29-ui-visual-theme-hover-audit/
MSG

tmux load-buffer -b flux_to_lyra_ui_visual_theme_hover_audit /tmp/flux_to_lyra_ui_visual_theme_hover_audit.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_ui_visual_theme_hover_audit
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
