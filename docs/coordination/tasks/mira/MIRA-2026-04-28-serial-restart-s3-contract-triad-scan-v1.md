# MIRA-2026-04-28-Serial-Restart-S3-Contract-Triad-Scan-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 3 |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Bounded parallel read / max 3 |

## 1. Purpose

Continue the restart flow after the PRD scan.
This step builds the cross-contract UI obligation map from the active interaction, UX, and acceptance contracts.

## 2. Rules

1. You may use up to **3 bounded parallel read actions** in this step.
2. Read only the three files listed below.
3. Do not edit code.
4. Do not inspect UI files yet.
5. Finish this step, report, then wait.

## 3. Step S3

Read only:

1. `docs/interaction-spec-v1.1.md`
2. `docs/ux-spec-v1.1.md`
3. `docs/acceptance-spec-v1.1.md`

Reply with only:

- `required interactive surfaces from INT/UX:` short bullets
- `acceptance-visible behaviors that UI must prove:` short bullets
- `highest-risk mismatch areas to inspect later in code:` short bullets
- `blockers:` none / exact clause conflict

## 4. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S3 Contract Triad Scan
required interactive surfaces from INT/UX:
- ...
acceptance-visible behaviors that UI must prove:
- ...
highest-risk mismatch areas to inspect later in code:
- ...
blockers:
- none / ...
next action:
- wait for S4
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s3-contract-triad-scan-v1.md
- docs/interaction-spec-v1.1.md
- docs/ux-spec-v1.1.md
- docs/acceptance-spec-v1.1.md
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```

If direct paste fails, print the same payload in your own pane with the `[Mira -> Lyra]` header.

## 5. Done definition

All must be true:

- only the three active contract files are read
- no code changes
- tmux reply sent to Lyra
- wait for S4
