# Task: Mira S7A Mobile Queue + Count Alignment

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | MIRA-2026-04-28-s7a-mobile-queue-count-alignment-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-29 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-mobile-overview-inbox-companion-acceptance.md`, `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-monitor-truth-fix-acceptance.md`, `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-v1.md` |
| tags | ui, prototype, mobile, companion, queue, count, truth, mira, s7a |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | One serial fix only. Do not start `S7B`. Do not open parallel edits inside this packet. |

## Objective

Close the last remaining `S7A` truth gaps without redesigning the accepted mobile shell.

This is a semantics-tightening fix, not a new surface.

## Input Files

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md` — `INT-17`
- `docs/ux-spec-v1.1.md` — `UX-13`
- `docs/acceptance-spec-v1.1.md` — `US-P0-12`, `P-13`, `U-11`
- `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-mobile-overview-inbox-companion-acceptance.md`
- `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-monitor-truth-fix-acceptance.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-monitor-truth-fix-delivery-v1.md`

## Scope

Primary write target:

- `ui/src/views/MobileCompanionView.tsx`

Optional support target only if you need a small shared helper for truth-safe projection:

- `ui/src/stores/useDataStore.ts`

## Required Outcome

### 1. Tighten the approval / gate count rule

The `待审批 / 闸门` card must count approval-worthy or gate-waiting records that are actually represented in the canonical seed set.

Minimum allow-list for this packet:

- `待处理交接`
- `需要验收`
- `需确认范围`
- `验收待决策`
- `输入请求`

Rules:

- do not count `背景记录`, `已记录决策`, or other FYI-only records;
- if a seed item is used by the count rule, name it exactly in the delivery artifact;
- keep the rule deterministic and local; no token dependency.

### 2. Tighten the urgent mobile queue rule

The `URGENT ONLY` list must stop admitting every `Normal` row.

Allowed-by-default rows for this packet:

- any `Critical` item, plus
- action-needed types such as `待处理交接`, `需要验收`, `需确认范围`, `验收待决策`, `输入请求`, `会话恢复`, `待补产物`

Disallowed-by-default rows for this packet:

- `背景记录`
- `已记录决策`
- `需要同步`
- any other FYI-only note that does not require near-term mobile action

### 3. Keep ranking deterministic

After filtering, rank items deterministically:

1. `Critical` first
2. then non-critical action-needed rows
3. newest timestamp first within the same priority band

If you use the timestamp string directly, state why it stays stable for the seeded format.

### 4. Preserve accepted shell boundaries

- no layout redesign;
- no new route or navigation model;
- no new mobile mutation flow;
- no palette/theme changes outside the existing preset token system;
- preserve `cd ui && pnpm build` success.

## Done Definition

- [ ] `待审批 / 闸门` count includes the seed-backed gate/approval types above.
- [ ] `URGENT ONLY` excludes background and FYI-only rows by default.
- [ ] Ranking is deterministic and documented.
- [ ] `cd ui && pnpm build` passes.
- [ ] A delivery artifact is written at the required path.
- [ ] A tmux reply is sent to Lyra and Mira waits for acceptance.

## Validation

Run:

```bash
cd ui && pnpm build
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-delivery-v1.md`

Required sections:

1. Scope completed
2. Approval/gate count rule
3. Urgent queue rule
4. Ranking rule
5. Changed files
6. Build result
7. Blockers
8. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s7a_queue_count.txt
[Mira -> Lyra] S7A Mobile Queue + Count Alignment
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s7a_queue_count /tmp/mira_to_lyra_s7a_queue_count.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s7a_queue_count
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
