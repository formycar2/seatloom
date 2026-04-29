# Task: Mira S7A Mobile Overview + Inbox Companion

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-29 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-s6ab-shell-truth-fixes-acceptance.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md` |
| tags | ui, prototype, mobile, companion, inbox, overview |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | Up to 3 Lyra packets may exist overall, but treat this packet as one serial implementation slice and do not run internal parallel edits |

## Objective

Deliver the first prototype-visible mobile companion slice for the active v0.5 contract.

This slice is monitor-first only: prove `Mobile Overview` plus `Mobile Inbox` as a governed companion surface, using real seeded project data and existing theme tokens.

## Input Files

- `docs/PRODUCT_TRUTH.md` — active entrypoint
- `docs/prd-v0.5.md` — mobile stories `US-P0-12` to `US-P0-15`
- `docs/interaction-spec-v1.1.md` — `INT-17` mobile monitoring flow
- `docs/ux-spec-v1.1.md` — `UX-13` mobile overview and inbox companion
- `docs/acceptance-spec-v1.1.md` — mobile acceptance clauses for `US-P0-12`, `P-13`, and `U-11`
- `docs/coordination/acceptance/2026-04-28-lyra-mira-s6ab-shell-truth-fixes-acceptance.md` — latest accepted shell baseline
- `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md` — theme-token baseline

## Scope

Primary write targets:

- `ui/src/App.tsx`
- `ui/src/layouts/Sidebar.tsx`
- `ui/src/views/MobileCompanionView.tsx` *(new file allowed)*
- `ui/src/stores/useDataStore.ts` *(only if seeded data needs small truthful additions)*

Optional support targets:

- `ui/src/utils/display.ts`
- `ui/src/types/index.ts`

Prototype rule:

- because the current prototype is still a desktop-hosted shell, represent mobile as a dedicated in-app companion surface that uses a phone-like layout or narrow card stack;
- do **not** try to build a separate app, routing system, or backend.

## Required Outcome

### 1. Mobile Overview

The prototype must expose a mobile companion surface with:

- current project name,
- sync freshness,
- alert count,
- unresolved blocker count,
- pending approval / gate count,
- `Prompt blocked` count,
- pending handoff count,
- budget warning state (truthful or explicitly unavailable),
- one recommended next action.

Rules:

- values must come from current seeded project truth, not fabricated constants;
- if a metric is not yet modeled, show an explicit unavailable state rather than a fake healthy state;
- copy must be Chinese-first.

### 2. Mobile Inbox

The same surface must include a mobile-safe Inbox section with rows that show:

- priority chip,
- action type,
- owner seat,
- object ref,
- one-line summary,
- waiting-time indicator,
- evidence quick-view or open-detail entry,
- primary CTA.

Rules:

- default content is `high priority / action needed`, not full-history browsing;
- row actions may open existing Detail or a bounded quick-view pattern, but must not expand into the full desktop review rail;
- this packet does **not** need to mutate approval state yet.

### 3. Shell and theme guardrails

- use only the accepted preset-token system from `ui/src/styles/globals.css` and `ui/src/styles/theme.ts`;
- do not hardcode a new per-component palette;
- keep the mobile surface visually distinct from the desktop detail pane, but still clearly part of the same product family;
- preserve deterministic-first behavior.

## Done Definition

- [ ] A prototype-visible `Mobile Companion` surface exists and is reachable in-app.
- [ ] `Mobile Overview` shows truthful project-health projections or explicit unavailable states.
- [ ] `Mobile Inbox` shows high-priority action rows with the required row fields.
- [ ] The surface uses shared preset theme tokens only.
- [ ] The build passes with `cd ui && pnpm build`.
- [ ] A delivery artifact is written at the required path.
- [ ] A tmux reply is sent to Lyra and Mira waits for acceptance.

## Acceptance Spec Reference

- `docs/acceptance-spec-v1.1.md` — §3.1 `US-P0-12`
- `docs/acceptance-spec-v1.1.md` — §4 `P-13`
- `docs/acceptance-spec-v1.1.md` — §5 `U-11`

## Constraints

- No mobile approval mutation flow in this packet.
- No mobile feedback composer in this packet.
- No new backend state model beyond small truthful seed additions.
- No theme-system redesign.
- No heavy responsive rewrite of the whole desktop shell.

## Validation

Run:

```bash
cd ui && pnpm build
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Mobile Overview implementation
4. Mobile Inbox implementation
5. Theme-token compliance
6. Build result
7. Blockers
8. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s7a.txt
[Mira -> Lyra] S7A Mobile Overview + Inbox Companion
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s7a /tmp/mira_to_lyra_s7a.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s7a
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
