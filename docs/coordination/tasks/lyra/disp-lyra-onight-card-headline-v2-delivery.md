---
template: T5
subtype: delivery
doc_id: 2026-10-11-lyra-disp-lyra-onight-card-headline-v2-delivery
title: Delivery — headline acceptance-card step dispatched to Mira (disp-onight-panel-card)
status: active
author: lyra
doc_date: 2026-10-11
version: v1
depends_on: disp-lyra-onight-card-headline-v2, docs/coordination/acceptance/2026-10-11-lyra-mira-knowledge-panel-headline-acceptance-plan-v1.md
tags: delivery, lyra, headline, knowledge-panel, mira, acceptance-card
---

# Delivery — `disp-lyra-onight-card-headline-v2`

*(Language correction: this note was first written in Chinese, violating the mandatory
English-artifact policy in COORDINATION_RULES §10. Facts unchanged; rewritten for
compliance.)*

## What was done

The dispatch payload's single action: hand the headline carding step to `seat-mira-001`.

- **Enqueued:** `disp-onight-panel-card`
  - title: `头条建卡:读模型+真实数据+面板验绿齐 → request_acceptance(血缘+结构化方案)`
  - target `seat-mira-001`; priority 30; max_retries 1; **no dependencies** → returned
    `status=ready` (immediately pullable, not blocked)
  - payload.note passed through verbatim as given: Mira calls `request_acceptance` now,
    citing §2/§3 of the acceptance plan verbatim; confirms the broken card
    `acc-disp-me6Nm8e9D6ZRe4fKyVokY` is voided; the verification build is the integrated
    build Aegis assembles; the merge happens only on the operator's Accept click.

## Why v2 was issuable when v1 was not

v1 hung on `ev-typed-memory-tools-landed`, whose provenance stayed unverified. v2 rests on
three delivered, evidenced facts already in the ledger:

- `ev-verdict-knowledge-panel` (Flux PASS for the panel)
- `ev-knowledge-read-model-landed` (Nimbus's read model delivered)
- `ev-writein-landed` (Vela's real distilled memories in the DB)

So the card rests on delivered, evidenced reality rather than on an ambiguous gate.

## How it was verified

1. Read the referenced acceptance-plan document and confirmed §2 (the acceptance_plan JSON)
   and §3 (the carding parameters: `dispatch_id` / `branch` / `base` / `how_to_verify` /
   `flux_verdict_ref`) exist and match what the payload note instructs Mira to cite
   verbatim — the thing being quoted has a real source.
2. `get_dispatch(disp-onight-panel)` confirmed the target lane exists, targets
   `seat-mira-001`, and is on branch `packet/mira/knowledge-panel` (the
   `base=track/infra-foundation` in the note comes from the plan's §3).
3. `enqueue_dispatch` returned `enqueued disp-onight-panel-card (status=ready)` — no
   dependencies, pullable at once.

## Ledger

- `emit_event(ArtifactCreated)` → `ev-bqCCOC5xVOHwKxQlwxxZ2` (subject = the new dispatch,
  actor = seat-lyra-001, context = seat-mira-001), payload records the three evidence
  event ids.

## Escalations / notes for upstream

- **Voiding the broken card `acc-disp-me6Nm8e9D6ZRe4fKyVokY`:** the payload already assigns
  this correctly — Mira voids it if her process can; if it cannot (old binary) she reports
  via `emit_event` and Aegis owns the backfill/void path. Lyra does not void it: that is a
  terminal act in Aegis's backfill path, not a PO action.
- **The integrated build belongs to Aegis**, also already stated in the payload (panel +
  read model + real data, assembled as in the D2 bootstrap). If the operator looks and the
  panel is not wired, Mira says so rather than letting him click into an empty panel.
- **No out-of-scope changes:** this turn was one dispatch plus one ledger event. No shared
  config, service, or another seat's domain was touched.

## Commit

- Branch: `packet/lyra/milestone-summary` (see the branch note in the 2026-10-12 delivery)
- Commit: `3b5fb02`
