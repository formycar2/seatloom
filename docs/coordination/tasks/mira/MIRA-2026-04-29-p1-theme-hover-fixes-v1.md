# Task: Mira P1 Theme + Hover Fixes

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | MIRA-2026-04-29-p1-theme-hover-fixes-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-system-acceptance.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | mira, ui, fix, theme, hover, contrast, p1 |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | One active styling packet only. Do not run this in parallel with any other UI edit packet. Keep the write set bounded to the files below. |

## Objective

Implement only the five immediate `P1` fixes from the accepted Flux visual audit.

This packet is **not** a broad restyle. Do **not** redesign the theme system, do **not** tune all `P2` polish items, and do **not** widen into unrelated UI surfaces.

The goal is to remove the most visible theme/hover/readability defects so the next UI cycle starts from a trustworthy baseline.

## Input Files

Read only these inputs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/tasks/flux/FLUX-2026-04-29-ui-visual-theme-hover-audit-delivery-v1.md`
6. `docs/coordination/acceptance/2026-04-29-lyra-flux-ui-visual-theme-hover-audit-acceptance.md`
7. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-system-acceptance.md`
8. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md`
9. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
10. `docs/coordination/COORDINATION_RULES.md`
11. this packet

## Need-to-Know Scope

Read only:

- the input files above;
- `ui/src/styles/globals.css`;
- `ui/src/components/EventRow.tsx`;
- `ui/src/components/ArtifactDetail.tsx`;
- `ui/src/layouts/Sidebar.tsx`;
- `ui/tailwind.config.js`;
- optional supporting theme files only if required for token-safe implementation.

Do not reopen by default:

- mobile companion surfaces;
- command bar / delegation / continuity / prompt banner;
- artifact chip icon contrast (`P2`);
- button hover polish (`P2`);
- broad preset palette redesign;
- repo-wide token naming migration.

## Write Boundary

Primary targets:

- `ui/src/styles/globals.css`
- `ui/src/components/EventRow.tsx`
- `ui/src/components/ArtifactDetail.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/tailwind.config.js`

Optional support targets only if strictly required:

- `ui/src/styles/theme.ts`

If you need any other file, state the reason first in the delivery artifact and keep the change minimal.

## Required Outcome

### 1. Fix hover cascade bug in shared clickable styling

Close Flux finding `D-01`:

- remove or narrow the `sl-clickable:hover *` cascade so hover does **not** override semantic child colors;
- row hover may still change background, but status-bearing children must keep their meaning;
- do not introduce a new global selector that is equally broad.

### 2. Remove hardcoded non-theme colors from Timeline and Artifact detail

Close Flux findings `D-02` and `D-03`:

- `EventRow.tsx` must stop using hardcoded Tailwind palette classes for event-type tones;
- `ArtifactDetail.tsx` must stop using hardcoded T1-T7 badge colors;
- use theme-aware semantic tokens only;
- after this packet, theme switching must not leave those two surfaces visually stuck on hardcoded colors.

### 3. Repair low-contrast sidebar session text

Close Flux finding `D-07`:

- inactive sidebar session labels must no longer rely on low opacity over small text;
- use stable text tokens that preserve contrast across all three shipped themes;
- keep active-state contrast intact.

### 4. Repair low-contrast object label in Timeline rows

Close Flux finding `D-10`:

- the EventRow object reference label must no longer depend on low-opacity primary text at tiny size;
- use a readable token combination that remains legible across all three themes.

### 5. Normalize text-token naming only inside touched files

Bounded interpretation of Flux recommendation `R-05`:

- in the files touched by this packet, prefer the `text-text-*` naming direction over `text-ink*` aliases;
- do **not** run a repo-wide migration in this packet;
- do **not** remove alias support from `tailwind.config.js` yet unless it is required for a safe bounded cleanup.

## Acceptance Criteria

Your delivery must satisfy all of the following:

1. `ui/src/components/EventRow.tsx` contains no hardcoded color classes like `text-violet-*`, `text-indigo-*`, or `text-orange-*` for the audited event tones.
2. `ui/src/components/ArtifactDetail.tsx` contains no hardcoded template-family badge palette classes for T1-T7.
3. `ui/src/layouts/Sidebar.tsx` no longer uses low opacity on inactive small session text in the audited lines.
4. Hovering shared clickable rows no longer forces semantic child text into a single primary color through a broad descendant selector.
5. `cd ui && pnpm build` passes.
6. The packet stays inside the bounded file set above.

## Non-goals

- No new theme preset
- No palette redesign
- No P2 polish fixes unless they are unavoidable side-effects of the P1 repair
- No live data or backend wiring changes
- No broad typography rewrite outside the touched files

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] The five targeted `P1` issues are addressed.
- [ ] `cd ui && pnpm build` passes.
- [ ] Any residual `P2` items are left untouched and called out explicitly.
- [ ] tmux reply is sent to Lyra after the artifact is written.

## Validation

Required command:

```bash
cd ui && pnpm build
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-29-p1-theme-hover-fixes-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Fix-by-fix mapping (`D-01`, `D-02`, `D-03`, `D-07`, `D-10`)
4. Validation commands and results
5. Residual notes (if any)
6. Recommended next owner

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_p1_theme_hover_fixes.txt
[Mira -> Lyra] P1 Theme + Hover Fixes
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-29-p1-theme-hover-fixes-delivery-v1.md
MSG

tmux load-buffer -b mira_to_lyra_p1_theme_hover_fixes /tmp/mira_to_lyra_p1_theme_hover_fixes.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_p1_theme_hover_fixes
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
