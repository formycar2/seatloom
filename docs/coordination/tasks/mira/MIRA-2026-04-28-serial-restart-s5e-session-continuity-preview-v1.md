# MIRA-2026-04-28-Serial-Restart-S5E-Session-Continuity-Preview-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 5E |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-packet implementation / depends on S5C |

## 1. Purpose

Make continuity-pack composition visible before runtime switch or recovery.
This slice upgrades the continuity preview only after the prompt-blocked visibility layer is complete.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md` (`US-P0-06`, `US-P0-09`, `US-P1-04`)
3. `docs/interaction-spec-v1.1.md` (`INT-06`, `INT-09` supervisor continuity notes)
4. `docs/ux-spec-v1.1.md` (`UX-06`)
5. `docs/acceptance-spec-v1.1.md` (`US-P0-06`, `US-P0-09`, `E-04`)
6. `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5-queue-control-v1.md`
7. this packet

## 3. Dependency

Start only after `S5C` is accepted, because both slices edit `ui/src/components/SessionDetail.tsx`.

## 4. Scope

Edit only:

- `ui/src/components/SessionDetail.tsx`

Do not edit:

- `ui/src/stores/useDataStore.ts`
- `ui/src/layouts/Sidebar.tsx`
- any other file

## 5. Required outcome

Add a continuity-pack preview block to `SessionDetail` when `session.continuity_pack` exists.

Minimum requirements:

1. Show Tier 0, Tier 1, and Tier 2 as separate visible blocks.
2. Show seat skills and playbook matches as separate sections.
3. Show budget estimate and a simple budget meter or budget summary.
4. Show fallback path and instruction text.
5. If any tier is missing, show precise incomplete-state text instead of fabricating content.
6. Keep deeper evidence hidden; do not invent `Load more evidence` behavior if the current prototype has no backing data.
7. Preserve the existing recovery and runtime-switch surfaces.

## 6. Guardrails

1. No prompt-state redesign in this slice.
2. No seed-data edits here.
3. No backend recovery actions.
4. Keep skill and playbook sections visually separate.

## 7. Validation

Run:

```bash
cd ui && pnpm build
```

Record pass/fail in the reply.

## 8. Done definition

All must be true:

- only `ui/src/components/SessionDetail.tsx` is edited,
- continuity preview is tiered and visibly budgeted when data exists,
- fallback path is visible,
- build result is reported,
- tmux reply is sent to Lyra,
- stop and wait for acceptance.

## 9. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S5E Session Continuity Preview
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5e-session-continuity-preview-v1.md
- ui/src/components/SessionDetail.tsx
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
