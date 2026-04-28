# FLUX-2026-04-27-sg01-runbook-evidence-prep-v1

| Field | Value |
|---|---|
| Owner | Flux |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-30 18:00 |
| Acceptance owner | Lyra |
| Write boundary | `.local/**`, `docs/coordination/tasks/flux/**` |

## 1. Objective

Prepare the SG-01 verification runbook and evidence checklist so acceptance can proceed immediately once Mira and Nimbus clear their handoffs.

## 2. Authority and evidence

Required sources:

1. `docs/acceptance-spec-v1.0.md`
2. `docs/interaction-spec-v1.0.md`
3. `docs/prd-v0.4.md`
4. `docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md`
5. `docs/coordination/acceptance/2026-04-27-sg-01-ui-contract-baseline-decision.md`
6. `docs/coordination/reviews/2026-04-27-product-alignment-review.md`

## 3. Scope restriction

- Do not patch product source to make tests pass.
- Do not reinterpret product meaning.
- Record blockers that depend on Mira or Nimbus instead of papering them over.

## 4. Required work

| Priority | Requirement | Expected coverage |
|---|---|---|
| P0 | Build SG-01 verification matrix | Project switch, Inbox, Timeline, Handoff, Terminal, recovery, and required keyboard shortcuts |
| P0 | Publish reproducible commands | Build, local run, and evidence capture commands with expected outputs |
| P0 | Define evidence checklist | Screenshots, logs, command results, and file paths required for gate review |
| P1 | Mark blocked checks explicitly | Note which checks depend on Mira recovery or Nimbus alignment completion |
| P1 | Keep artifacts inside allowed write boundaries | Use `.local/**` for raw evidence and `docs/coordination/tasks/flux/**` for durable summaries |

## 5. Done definition

All items below must be true:

- Flux publishes a runbook under `docs/coordination/tasks/flux/`.
- Runbook includes exact commands, expected results, evidence paths, and blocker annotations.
- Verification matrix maps each SG-01 requirement to a check method.
- No product source files are edited during this packet.

## 6. Delivery artifact required

Publish completion as:

- `docs/coordination/tasks/flux/FLUX-2026-04-30-sg01-runbook-evidence-delivery-v1.md`

Include:

- verification matrix
- exact commands
- expected evidence files
- blocked/unblocked status by check
