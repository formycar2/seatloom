# Task: Close Session-Recovery Context and Return to GPU Resource Mainline

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-09-28-session-recovery-handoff-gpu-mainline-v1 |
| status | issued |
| author | human-directed-codex |
| date | 2026-09-28 |
| version | v1 |
| to | next-agent |
| priority | P0 |
| depends_on | `docs/coordination/reviews/2026-09-28-session-recovery-lookup-and-response-method.md` |
| tags | handoff, session-continuity, iterm2, tmux, gpu-resource, capacity-planner, context-reset |

## Objective

Take over after the local iTerm2/tmux session lookup has been documented. Do not repeat the recovery investigation unless the human explicitly asks for another recovery operation. Return the active workstream to GPU resource calculation and capacity-planning work in `cn-gpu-infer-fabric`.

## Copy-Paste Prompt

Use the following prompt as the next Agent's starting instruction:

```text
You are taking over after a completed local iTerm2/tmux/Codex session-recovery investigation.

Read this artifact once:
  /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/docs/coordination/reviews/2026-09-28-session-recovery-lookup-and-response-method.md

The recovery incident is already resolved and documented. Do not:
- run workspace-restore with --execute;
- close, rebuild, or rearrange all iTerm2 windows;
- kill tmux sessions;
- attach or resume a historical Codex/Claude session;
- search the recovery evidence again unless the human explicitly asks;
- modify SeatLoom code or documentation beyond this handoff unless explicitly requested.

Treat the recovery artifact as durable reference only. Do not carry its detailed terminal/session context into the active engineering task.

Return to the GPU resource and capacity-planning mainline:

  repository:
    /Users/jyxc-dz-0100609/Documents/GitHub/cn-gpu-infer-fabric

First inspect the current repository state without destructive commands:

  cd /Users/jyxc-dz-0100609/Documents/GitHub/cn-gpu-infer-fabric
  git status --short
  sed -n '1,240p' README.md

Then read the current capacity-planning and Step-5 evidence sources before making assumptions:

  docs/prd/stepfun-capacity-planner-v4.2.md
  docs/v4/step-5-preview-dashboard-capacity-reference-2026-09-23.md
  docs/v4/a800-pd-deployment-details.md
  docs/v4/a800-pd-disaggregated-benchmark-plan.md
  docs/v4/step-5-preview-token-cost-estimation.md
  docs/sales/step-5-preview-sales-quote-v2.md
  docs/sales/step-5-preview-sales-quote-v2-user-manual.md

Continue the resource-calculation work from the repository's actual uncommitted state. The mainline objective is to turn observed inference behavior into explainable GPU resource and capacity estimates, including:
- replica definition and deployment topology;
- GPU type and GPU count;
- prefill/decode or equivalent serving split;
- concurrency and QPS;
- input/output token throughput;
- context length and cache behavior;
- single-replica capacity;
- SLA or capacity reservation implications;
- sales-facing quote inputs versus architecture-only inputs.

Use existing project conventions and preserve all unrelated user changes. Never use git reset --hard, git checkout --, or broad cleanup commands.

When reporting progress, keep the output on the GPU/capacity topic. Mention only:
- files inspected or changed;
- resource/capacity conclusion;
- evidence and formulas;
- test or validation results;
- unresolved data gaps and the next bounded action.

The SeatLoom recovery task is complete. The active destination for this Agent is cn-gpu-infer-fabric.
```

## Handoff Constraints

- The recovery work is reference-only after this packet is issued.
- The current `cn-gpu-infer-fabric` worktree is dirty by design; preserve its existing changes.
- The GPU mainline must be resumed from repository evidence, not from a guessed prior conversation.
- The next Agent must not infer that a passing iTerm/tmux layout report proves every model context was recovered.
- No credentials, cookies, authorization headers, or secret values belong in this packet.

## Done Definition

- [ ] Next Agent reads the recovery review once.
- [ ] Next Agent does not perform a new iTerm/tmux recovery operation.
- [ ] Next Agent changes operational focus to `cn-gpu-infer-fabric`.
- [ ] Next Agent inspects the current capacity-planning documents and dirty worktree.
- [ ] Next Agent reports the next bounded GPU resource calculation action.

