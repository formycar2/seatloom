# MIRA-2026-04-28-Serial-Recovery-R0-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Recovery / serial execution reset |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-task only / no concurrency |

## 1. Purpose

Your recent requests failed before completion.
Reset to a safe serial mode before any more implementation work.

## 2. Rules

1. Do not run concurrent work.
2. Do not queue multiple tasks.
3. Do not open or edit multiple files for this step.
4. Finish one task, report, then wait.

## 3. Task R0

Do only this:

1. Cancel/exit any failed in-flight request in your CLI.
2. Return to a stable idle prompt.
3. Do not read files yet.
4. Do not edit code yet.
5. Send a direct tmux ACK to `Lyra-po-seatloom`.

## 4. Direct tmux reply contract

Send this exact payload:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] Recovery R0
completed:
- seat reset to stable idle prompt
- serial mode adopted
blockers:
- none / ...
next action:
- wait for R1
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-recovery-r0-v1.md
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.

## 5. Done definition

All must be true:

- no code changes made in this step
- no file reads needed in this step
- tmux ACK sent to Lyra
- wait for R1 after ACK
