# MIRA-2026-04-28-Serial-Restart-S4B-Workloop-Continuity-Scan-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 4B |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Bounded parallel read / max 3 |

## 1. Purpose

Continue the read-only UI gap scan after `S4A`.
This step checks the work loop, continuity, prompt-state, and seat-capability surfaces against the active v0.5 contract.

## 2. Rules

1. You may use up to **3 bounded parallel read actions** in this step.
2. Read only the files listed below.
3. Do not edit code.
4. Do not inspect unrelated files yet.
5. Finish this step, report, then wait for `S5`.

## 3. Step S4B

Read only:

1. `ui/src/components/WorkItemDetail.tsx`
2. `ui/src/components/HandoffDetail.tsx`
3. `ui/src/components/SessionDetail.tsx`
4. `ui/src/components/AddSeatDialog.tsx`
5. `ui/src/types/index.ts`

Reply with only:

- `surface/flow | status | exact file targets`
- include exactly these rows:
  - `workitem review / reissue tiers`
  - `handoff detail state strip`
  - `session continuity preview + prompt-blocked banner`
  - `seat capability truth / role binding`
  - `shared type support for review + prompt + continuity`
- allowed `status` values:
  - `already aligned`
  - `repair needed`
  - `missing`
  - `needs Lyra decision`
- `implementation-ready slice candidates for S5:` short bullets
- `blockers:` none / exact clause conflict

## 4. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S4B Workloop + Continuity Scan
surface/flow | status | exact file targets
workitem review / reissue tiers | ... | ...
handoff detail state strip | ... | ...
session continuity preview + prompt-blocked banner | ... | ...
seat capability truth / role binding | ... | ...
shared type support for review + prompt + continuity | ... | ...
implementation-ready slice candidates for S5:
- ...
blockers:
- none / ...
next action:
- wait for S5
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s4b-workloop-continuity-scan-v1.md
- ui/src/components/WorkItemDetail.tsx
- ui/src/components/HandoffDetail.tsx
- ui/src/components/SessionDetail.tsx
- ui/src/components/AddSeatDialog.tsx
- ui/src/types/index.ts
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.

## 5. Done definition

All must be true:

- only the five listed files are read
- no code changes are made
- the five required rows are returned in the exact matrix format
- tmux reply is sent to Lyra
- wait for `S5`
