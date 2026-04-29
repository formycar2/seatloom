# MIRA-2026-04-28-Serial-Restart-S5C-Session-Prompt-Blocked-Surfaces-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 5C |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-packet implementation / queue-safe |

## 1. Purpose

Expose interactive-prompt blocking in visible UI surfaces so wrapped sessions do not fail silently.
This slice covers the prompt-state visibility layer only, not the broader continuity-pack preview.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md` (`US-P0-11`)
3. `docs/interaction-spec-v1.1.md` (`INT-16`)
4. `docs/ux-spec-v1.1.md` (`UX-12`)
5. `docs/acceptance-spec-v1.1.md` (`US-P0-11` expectations)
6. `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5-queue-control-v1.md`
7. this packet

## 3. Scope

Edit only:

- `ui/src/components/SessionDetail.tsx`
- `ui/src/layouts/Sidebar.tsx`

Do not edit:

- `ui/src/stores/useDataStore.ts`
- `ui/src/components/WorkItemDetail.tsx`
- `ui/src/components/SeatDetail.tsx`
- any other file

## 4. Required outcome

Implement the prompt-blocked visibility layer for wrapped sessions.

Minimum requirements:

1. In `SessionDetail`, when `session.prompt_state` exists or `session.status === 'InputRequired'`, show a prominent `Prompt blocked` banner near the top.
2. The banner must show:
   - runtime + session id,
   - classification chip,
   - policy chip,
   - bounded preview,
   - expected next prompt (if present),
   - step count and token budget when present.
3. Show visible action controls:
   - `Approve`,
   - `Human takeover`,
   - `Supervisor assist`,
   - `Stop`.
4. `Supervisor assist` must be visibly disabled for `sensitive` prompts or `human required` policy.
5. In `Sidebar`, session rows with blocked prompts must show a small `Prompt` chip so the blocked state is discoverable before opening detail.
6. Keep all controls non-destructive and UI-only in this slice; do not invent backend actions.

## 5. Guardrails

1. Do not add continuity-pack UI in this slice.
2. Keep preview bounded; never expand full terminal history.
3. No seed-data edits here.
4. Preserve existing session recovery panels and runtime-switch surfaces.

## 6. Validation

Run:

```bash
cd ui && pnpm build
```

Record pass/fail in the reply.

## 7. Done definition

All must be true:

- only `SessionDetail.tsx` and `Sidebar.tsx` are edited,
- blocked prompt state is visible in both detail and session-list surfaces,
- `Supervisor assist` is disabled when the contract requires,
- build result is reported,
- tmux reply is sent to Lyra,
- stop and wait for acceptance before `S5E`.

## 8. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S5C Session Prompt Blocked Surfaces
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance before `S5E`
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5c-session-prompt-blocked-surfaces-v1.md
- ui/src/components/SessionDetail.tsx
- ui/src/layouts/Sidebar.tsx
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
