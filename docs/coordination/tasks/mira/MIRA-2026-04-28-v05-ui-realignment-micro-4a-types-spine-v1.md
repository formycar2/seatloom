# MIRA-2026-04-28-v05-UI-Realignment-Micro-4A-Types-Spine-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Product Baseline Freeze / pre-SG-01 |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Micro-task / one-file-only |

## 1. Micro-brief

Your prior broader type-backbone pass hit a Gemini API/tool error mid-flight.
Do not continue the broader slice.
Recover by finishing only the shared type spine in one file.

## 2. Scope

Touch only:

- `ui/src/types/index.ts`

Do not touch:

- `ui/src/mockData.ts`
- `ui/src/stores/useDataStore.ts`
- any component or layout file

## 3. Goal

Stabilize the type spine already partially landed in `ui/src/types/index.ts`.

This file should cover only the shared v0.5 structures already identified by the matrix:

1. `HandoffStatus` includes `Working`
2. WorkItem review/reissue tier record support
3. Session prompt-state support
4. Session continuity-preview support
5. Seat capability / budget truth support

Use optional fields where possible.
Do not expand into rendering logic.

## 4. Recovery instructions

1. First inspect the current partial diff for `ui/src/types/index.ts`
2. Keep any correct accepted edits already present
3. Finish only the missing parts in this file
4. Stop immediately after this file is coherent

## 5. Done definition

All must be true:

- only `ui/src/types/index.ts` is edited for this slice
- the shared type spine is coherent for the five goals above
- no visual/UI implementation is started
- a direct tmux delta is sent back to Lyra

## 6. Direct tmux reply contract

When done, send this exact format to `Lyra-po-seatloom`:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] Micro-task 4A
completed:
- ...
blockers:
- none / ...
next action:
- wait for Micro-task 4B
artifact path(s):
- ui/src/types/index.ts
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.
