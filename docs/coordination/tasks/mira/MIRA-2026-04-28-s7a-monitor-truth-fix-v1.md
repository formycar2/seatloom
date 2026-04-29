# Task: Mira S7A Monitor Truth Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | MIRA-2026-04-28-s7a-monitor-truth-fix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-29 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-mobile-overview-inbox-companion-acceptance.md`, `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-v1.md` |
| tags | ui, prototype, mobile, companion, truth-fix, inbox |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | One serial fix only. Do not open internal parallel edits for this packet. |

## Objective

Close the remaining contract gaps in `S7A` without reopening the accepted mobile-shell structure.

This is a truth-completion fix, not a redesign.

## Input Files

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md` — `INT-17`
- `docs/ux-spec-v1.1.md` — `UX-13`
- `docs/acceptance-spec-v1.1.md` — `US-P0-12`, `P-13`, `U-11`
- `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-mobile-overview-inbox-companion-acceptance.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-overview-inbox-companion-v1.md`

## Scope

Primary write target:

- `ui/src/views/MobileCompanionView.tsx`

Optional support target only if needed for truthful projection:

- `ui/src/stores/useDataStore.ts`

## Required Outcome

### 1. Add the missing overview monitor signals

The mobile overview must now show these as explicit, separate signals:

- `pending approvals / gates`
- `Prompt blocked`
- `pending handoffs`

Rules:

- do not collapse these into `Alerts` or a generic blocker count;
- derive them from canonical seeded data when possible;
- if a metric is genuinely not modeled, show an explicit unavailable state instead of a fake number;
- document the exact deterministic counting rule in the delivery artifact.

Preferred deterministic rules for this packet:

- `Prompt blocked` = sessions with `prompt_state`
- `pending handoffs` = handoffs still in flight (not `Completed` / not `Expired`)
- `pending approvals / gates` = decision-waiting objects other than prompt-blocked sessions; if you use a narrower rule, state it exactly and keep it evidence-backed

### 2. Make the mobile inbox truly urgent-by-default

Replace raw inbox slicing with a deterministic urgency filter.

Rules:

- default rows must represent `high priority / action needed`, not raw history order;
- background records, already-recorded notes, and low-priority FYI items must not appear ahead of urgent action rows by default;
- keep the current mobile-safe row layout and existing detail drill-through.

### 3. Keep the accepted shell constraints intact

- no new route system;
- no new approval mutation flow;
- no theme redesign;
- no sidebar/detail-rail reintroduction;
- preserve `cd ui && pnpm build` success.

## Done Definition

- [ ] `pending approvals / gates`, `Prompt blocked`, and `pending handoffs` are visible as separate mobile monitor signals.
- [ ] The count rules are deterministic and described in the delivery artifact.
- [ ] Mobile Inbox defaults to urgent/action-needed rows rather than raw list order.
- [ ] The build passes with `cd ui && pnpm build`.
- [ ] A delivery artifact is written at the required path.
- [ ] A tmux reply is sent to Lyra and Mira waits for acceptance.

## Validation

Run:

```bash
cd ui && pnpm build
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-delivery-v1.md`

Required sections:

1. Scope completed
2. Counting rules
3. Urgent filter rule
4. Changed files
5. Build result
6. Blockers
7. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s7a_fix.txt
[Mira -> Lyra] S7A Monitor Truth Fix
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s7a_fix /tmp/mira_to_lyra_s7a_fix.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s7a_fix
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
