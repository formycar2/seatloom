# MIRA-2026-04-28-Serial-Restart-S4A-Shell-Artifact-Scan-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 4A |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Bounded parallel read / max 3 |

## 1. Purpose

Move from contract reading into the first read-only code inspection slice.
This step checks whether the current UI shell, action queue, replay surface, and typed artifact surface match the active v0.5 contract.

## 2. Rules

1. You may use up to **3 bounded parallel read actions** in this step.
2. Read only the files listed below.
3. Do not edit code.
4. Do not inspect unrelated files yet.
5. Finish this step, report, then wait for `S4B`.

## 3. Step S4A

Read only:

1. `ui/src/App.tsx`
2. `ui/src/layouts/Sidebar.tsx`
3. `ui/src/views/InboxView.tsx`
4. `ui/src/views/TimelineView.tsx`
5. `ui/src/components/ArtifactDetail.tsx`
6. `ui/src/components/ArtifactChip.tsx`

Reply with only:

- `surface/flow | status | exact file targets`
- include exactly these rows:
  - `project switching / project memory restoration`
  - `inbox next-action loop`
  - `timeline replay + drill-through`
  - `artifact metadata strip / family badges`
- allowed `status` values:
  - `already aligned`
  - `repair needed`
  - `missing`
  - `needs Lyra decision`
- `top gaps to carry into S4B:` short bullets
- `blockers:` none / exact clause conflict

## 4. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S4A Shell + Artifact Scan
surface/flow | status | exact file targets
project switching / project memory restoration | ... | ...
inbox next-action loop | ... | ...
timeline replay + drill-through | ... | ...
artifact metadata strip / family badges | ... | ...
top gaps to carry into S4B:
- ...
blockers:
- none / ...
next action:
- wait for S4B
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s4a-shell-artifact-scan-v1.md
- ui/src/App.tsx
- ui/src/layouts/Sidebar.tsx
- ui/src/views/InboxView.tsx
- ui/src/views/TimelineView.tsx
- ui/src/components/ArtifactDetail.tsx
- ui/src/components/ArtifactChip.tsx
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.

## 5. Done definition

All must be true:

- only the six listed UI files are read
- no code changes are made
- the four required rows are returned in the exact matrix format
- tmux reply is sent to Lyra
- wait for `S4B`
