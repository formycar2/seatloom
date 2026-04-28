# SG-01 UI Contract Baseline Decision - 2026-04-27

| Field | Value |
|---|---|
| Gate | SG-01 UI Contract Baseline |
| Owner | Lyra |
| Reviewer | Aegis |
| Current verdict | HOLD |

## 1. Gate purpose

Confirm that SeatLoom has one contract-safe UI baseline before Nimbus implementation proceeds.

## 2. Entry criteria

- Product alignment review exists in files.
- Mira acceptance review exists in files.
- P0 contract conflicts are resolved or explicitly waived in durable artifacts.
- `ui` build passes locally.
- Task packets exist for Mira, Nimbus, and Flux.

## 3. Evidence list

- `docs/coordination/reviews/2026-04-27-product-alignment-review.md`
- `docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md`
- `docs/coordination/tasks/lyra/LYRA-2026-04-27-sg01-recovery-plan-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-27-sg01-ui-contract-recovery-v2.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-27-sg01-schema-event-alignment-prep-v2.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-27-sg01-runbook-evidence-prep-v2.md`
- `ui` build evidence: `cd ui && pnpm build` on 2026-04-27

## 4. Exit criteria for Go

- Terminal is a bottom toggle panel inside the main shell, not a main tab.
- Inbox behaves as an action queue with state-correct accept/return/resolve behavior.
- Timeline supports seat/workitem/type/time filtering and an event-detail review path.
- Handoff create/send/accept/return/complete flow is represented in UI truth.
- Required keyboard shortcuts and focus behavior are implemented for SG-01 scope.
- Role packets and gate evidence comply with AI-native execution rules: micro-brief, need-to-know scope, token-efficient output, English artifacts, and Chinese terminal summaries.
- No Critical findings remain in Lyra acceptance review.

## 5. Hold triggers

- Any P-class acceptance item fails.
- Shell, Inbox, Timeline, Handoff, or continuity fallback flow remains off-contract.
- Any required gate artifact violates language policy or AI-native context-scope rules.
- Critical product decisions still live only in terminal text.

## 6. Current verdict rationale

Hold remains necessary because Mira's current redesign fails the shell, Inbox, Timeline, Handoff, and keyboard contract checks. Nimbus may prepare alignment notes, but full implementation handoff is blocked.

## 7. Follow-ups

| Owner | Deadline | Follow-up |
|---|---|---|
| Lyra | 2026-04-28 | Freeze the four P0 contract conflicts in files |
| Mira | 2026-04-29 | Resubmit prototype for SG-01 recovery review |
| Nimbus | 2026-04-30 | Publish schema/event alignment memo only |
| Flux | 2026-04-30 | Publish SG-01 runbook and evidence checklist |
