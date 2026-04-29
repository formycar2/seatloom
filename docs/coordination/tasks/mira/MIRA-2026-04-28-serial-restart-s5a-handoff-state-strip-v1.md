# MIRA-2026-04-28-Serial-Restart-S5A-Handoff-State-Strip-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 5A |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-task implementation / no parallelism |

## 1. Purpose

Begin implementation again with the smallest validated v0.5 slice.
This slice repairs the Handoff detail lifecycle visibility without opening broader review, prompt, or continuity work.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/interaction-spec-v1.1.md` (`INT-12`)
3. `docs/ux-spec-v1.1.md` (`WorkItems and Handoffs` / `Handoff detail`)
4. `docs/acceptance-spec-v1.1.md` (`US-P1-06` pass expectations)
5. this packet

## 3. Scope

Edit only:

- `ui/src/components/HandoffDetail.tsx`

Do not edit:

- `ui/src/types/index.ts`
- `ui/src/stores/useDataStore.ts`
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/SessionDetail.tsx`
- any other file

## 4. Required outcome

Implement a visible Handoff state strip in `HandoffDetail` that makes lifecycle progress legible.

Minimum requirements:

1. Show the canonical strip states:
   - `sent`
   - `accepted`
   - `working`
   - `completed`
   - `returned`
2. Highlight the current position based on `handoff.status`.
3. Preserve the exact current status label elsewhere; the strip is the normalized progress view, not a replacement for raw status.
4. Show a small contextual sentence that explains what the current stage means.
5. Keep the existing sender/receiver, purpose, expected outcome, linked WorkItem, artifacts, and recent-event sections intact.

## 5. Mapping rule for current data

Use this normalization unless you find a direct contract conflict:

- `Sent` -> `sent`
- `Received` or `Accepted` -> `accepted`
- `Working` -> `working`
- `Completed` -> `completed`
- `Returned` -> `returned`

If the source status is outside those paths (for example `Drafted` or `Expired`), keep the raw label visible and render the strip in the nearest safe neutral state without inventing unsupported progress.

## 6. Guardrails

1. No new fake actions in this slice. If `Open live activity` is not already wired, do not invent it here.
2. No data-model expansion in this slice.
3. No typography regression on touched text.
4. Keep the implementation deterministic; do not rely on inferred LLM summaries.

## 7. Validation

Run:

```bash
cd ui && pnpm build
```

Record pass/fail in the reply.

## 8. Done definition

All must be true:

- only `ui/src/components/HandoffDetail.tsx` is edited
- the Handoff lifecycle strip is visible and status-aware
- the raw status label remains visible somewhere in detail
- build result is reported
- tmux reply is sent to Lyra
- stop and wait for `S5B`

## 9. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S5A Handoff State Strip
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for S5B
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5a-handoff-state-strip-v1.md
- ui/src/components/HandoffDetail.tsx
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.
