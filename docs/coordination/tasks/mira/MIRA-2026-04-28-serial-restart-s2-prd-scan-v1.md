# MIRA-2026-04-28-Serial-Restart-S2-PRD-Scan-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 2 |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-task only / no concurrency |

## 1. Purpose

Continue the restart flow in strict serial mode.
This step reads the active PRD only.

## 2. Rules

1. One task only.
2. No parallel work.
3. No code edits.
4. No extra file reads.
5. Finish this step, report, then wait.

## 3. Step S2

Do only this:

1. Read `docs/prd-v0.5.md`
2. Do not read any other file in this step
3. Reply with only:
   - `P0 UI priorities from PRD:` short bullets
   - `must-not-regress baselines named or implied by PRD:` short bullets
   - `blockers:` none / exact clause conflict

## 4. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S2 PRD Scan
P0 UI priorities from PRD:
- ...
must-not-regress baselines named or implied by PRD:
- ...
blockers:
- none / ...
next action:
- wait for S3
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s2-prd-scan-v1.md
- docs/prd-v0.5.md
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.

## 5. Done definition

All must be true:

- only `docs/prd-v0.5.md` is read in this step
- no code changes
- tmux reply sent to Lyra
- wait for S3
