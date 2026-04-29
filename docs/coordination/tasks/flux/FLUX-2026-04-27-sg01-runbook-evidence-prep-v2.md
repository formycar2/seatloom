# FLUX-2026-04-27-sg01-runbook-evidence-prep-v2

| Field | Value |
|---|---|
| Owner | Flux |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-30 18:00 |
| Acceptance owner | Lyra |
| Write boundary | `.local/**`, `docs/coordination/tasks/flux/**` |
| Execution mode | Need-to-know / token-efficient |

## 1. Micro-brief

1. Prepare the SG-01 verification runbook and evidence checklist.
2. Stay inside verifier boundaries; do not patch product code.
3. Cover project switch, Inbox, Timeline, Handoff, Terminal, recovery, and required shortcuts.
4. Mark dependency-blocked checks explicitly.
5. Persist the runbook in English; keep terminal progress summaries in Chinese.

## 2. Contract pack

Mandatory reads, in this order:

1. `docs/archive/product-history/prd-v0.4.md`
2. `docs/archive/product-history/interaction-spec-v1.0.md`
3. `docs/archive/product-history/acceptance-spec-v1.0.md`
4. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
5. `docs/coordination/COORDINATION_RULES.md`
6. `docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md`
7. `docs/coordination/acceptance/2026-04-27-sg-01-ui-contract-baseline-decision.md`
8. `docs/coordination/reviews/2026-04-27-product-alignment-review.md`

## 3. Need-to-know scope

Read only:

- this packet
- the contract pack above
- exact scripts/config files required to define commands and evidence capture

Do not consume by default:

- unrelated product source files
- full repo history
- unrelated seat packets

If a verification dependency is missing, mark the check blocked instead of widening scope or patching source.

## 4. Token-efficiency rules

| Item | Rule |
|---|---|
| Input budget target | <= 5,000 tokens total live context |
| Output budget target | <= 900 words in the runbook body, excluding tables |
| Sync policy | Delta-only from `FLUX-2026-04-27-sg01-runbook-evidence-prep-v1.md` |
| Preferred output | verification matrix, commands, evidence paths, blockers |
| Truncation strategy | If output grows too long, keep executable checks and evidence paths; compress commentary into blocker bullets |

## 5. Delta from v1 packet

New requirements in this v2 packet:

- `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md` is now mandatory authority for execution behavior.
- Need-to-know scope and token budgets are explicit.
- Terminal progress must be Chinese; durable artifacts must remain English.
- The runbook must be minimum-verifiable-output first, not narrative-first.

## 6. Required work

| Priority | Requirement | Expected coverage |
|---|---|---|
| P0 | Build SG-01 verification matrix | Project switch, Inbox, Timeline, Handoff, Terminal, recovery, and required keyboard shortcuts |
| P0 | Publish reproducible commands | Build, local run, and evidence capture commands with expected outputs |
| P0 | Define evidence checklist | Screenshots, logs, command results, and file paths required for gate review |
| P1 | Mark blocked checks explicitly | Note which checks depend on Mira recovery or Nimbus alignment completion |
| P1 | Keep artifacts inside allowed write boundaries | Use `.local/**` for raw evidence and `docs/coordination/tasks/flux/**` for durable summaries |

## 7. Done definition

All items below must be true:

- Flux publishes a runbook under the required artifact path.
- Runbook includes exact commands, expected results, evidence paths, and blocker annotations.
- Verification matrix maps each SG-01 requirement to a check method.
- No product source files are edited during this packet.

## 8. Required delivery artifact

Publish completion as:

- `docs/coordination/tasks/flux/FLUX-2026-04-30-sg01-runbook-evidence-delivery-v2.md`

Required sections:

1. Micro-brief
2. Verification matrix
3. Commands
4. Expected evidence paths
5. Blocked / unblocked status
6. Risks / next owner

## 9. Reporting rules

- Durable artifact: English only.
- Terminal progress summary: Chinese only.
- Keep raw evidence inside approved write boundaries.
- If blocked, report exact dependency and recommended owner.
