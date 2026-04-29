# MIRA-2026-04-28-Serial-Restart-S1-Product-Truth-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 1 |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-task only / no concurrency |

## 1. Purpose

Restart from the beginning in strict serial mode.
Ignore all prior micro-task numbering.

## 2. Rules

1. One task only.
2. No parallel work.
3. No code edits.
4. No extra file reads.
5. Finish this step, report, then wait.

## 3. Step S1

Do only this:

1. Read `docs/PRODUCT_TRUTH.md`
2. Do not read any other file yet
3. Reply with:
   - `ACK`
   - the active authority set named by `docs/PRODUCT_TRUTH.md`
   - any blocker accessing the active truth entrypoint

## 4. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S1 Product Truth
completed:
- read docs/PRODUCT_TRUTH.md
- identified active authority set
blockers:
- none / ...
next action:
- wait for S2
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s1-product-truth-v1.md
- docs/PRODUCT_TRUTH.md
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.

## 5. Done definition

All must be true:

- only `docs/PRODUCT_TRUTH.md` is read
- no code changes
- tmux reply sent to Lyra
- wait for S2
