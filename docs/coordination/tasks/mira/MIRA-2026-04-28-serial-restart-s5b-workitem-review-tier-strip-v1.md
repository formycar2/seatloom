# MIRA-2026-04-28-Serial-Restart-S5B-WorkItem-Review-Tier-Strip-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 5B |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-packet implementation / queue-safe |

## 1. Purpose

Make review-driven reissue logic visible in the WorkItem surface without reopening wider workflow or artifact-reader work.
This slice closes the missing `L1/L2/L3` review-change visibility gap found in `S4B`.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/interaction-spec-v1.1.md` (`INT-05`)
3. `docs/ux-spec-v1.1.md` (`WorkItem detail` review-change strip rules)
4. `docs/acceptance-spec-v1.1.md` (`US-P0-05`, `P-06`, `U-09`, `E-09`)
5. `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5-queue-control-v1.md`
6. this packet

## 3. Scope

Edit only:

- `ui/src/components/WorkItemDetail.tsx`

Do not edit:

- `ui/src/types/index.ts`
- `ui/src/stores/useDataStore.ts`
- `ui/src/components/SessionDetail.tsx`
- `ui/src/components/HandoffDetail.tsx`
- any other file

## 4. Required outcome

When `workItem.change_tier_record` exists, `WorkItemDetail` must surface a clear review-change strip near the top of the detail surface.

Minimum requirements:

1. Show a visible `Review change` strip with:
   - tier badge (`L1`, `L2`, `L3`),
   - execution-mode chip (`Direct patch`, `Compact ack`, `Full gate`),
   - changed-clause list,
   - impact-level chip,
   - reviewer label,
   - executor label,
   - evidence references.
2. Use `change_tier_record.reason` as the explanatory sentence for why the reissue path exists.
3. If the tier is `L2`, show a bounded compact-ack preview block only; do not invent a long threaded conversation.
4. If the tier is `L3`, keep a visible `Full gate review required` indicator.
5. If `change_tier_record` is absent, keep the current detail clean and avoid fake placeholders.
6. Preserve the existing owner, goal, AC, dependency, handoff, and evidence sections.

## 5. Guardrails

1. No new mutation controls in this slice.
2. No fake backend actions.
3. No changes to seeded data here.
4. Keep the strip deterministic and evidence-linked.

## 6. Validation

Run:

```bash
cd ui && pnpm build
```

Record pass/fail in the reply.

## 7. Done definition

All must be true:

- only `ui/src/components/WorkItemDetail.tsx` is edited,
- the review-change strip is visible when `change_tier_record` exists,
- `L2` remains compact and `L3` remains visibly gated,
- build result is reported,
- tmux reply is sent to Lyra,
- stop and wait for acceptance.

## 8. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S5B WorkItem Review Tier Strip
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance; `S5E` still depends on `S5C`
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5b-workitem-review-tier-strip-v1.md
- ui/src/components/WorkItemDetail.tsx
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
