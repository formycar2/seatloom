# MIRA-2026-04-28-Serial-Restart-S5D-Seat-Card-Capability-Truth-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 5D |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-packet implementation / queue-safe |

## 1. Purpose

Repair the Seat Card so routing decisions can be based on visible capability truth instead of role-only labels.
This slice upgrades the existing seat detail surface rather than reopening creation flows.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md` (`US-P0-03`, `US-P0-08`)
3. `docs/interaction-spec-v1.1.md` (`INT-02`, `INT-03` seat-card expectations)
4. `docs/ux-spec-v1.1.md` (`UX-04` Seat Card panel)
5. `docs/acceptance-spec-v1.1.md` (`US-P0-08`, `P-04`)
6. `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5-queue-control-v1.md`
7. this packet

## 3. Scope

Edit only:

- `ui/src/components/SeatDetail.tsx`

Do not edit:

- `ui/src/layouts/Sidebar.tsx`
- `ui/src/components/AddSeatDialog.tsx`
- `ui/src/stores/useDataStore.ts`
- any other file

## 4. Required outcome

Upgrade the existing seat detail into a capability-truth surface.

Minimum requirements:

1. Add a visible Seat Card section that surfaces:
   - capabilities,
   - accepted input types,
   - output types,
   - input budget,
   - output budget,
   - constraints,
   - attached seat skills.
2. Make the card visually signal one of three states using only existing seat data:
   - assignable,
   - under-specified,
   - budget-constrained.
3. Reuse the best available runtime hint from current sessions when a default runtime field does not exist.
4. Keep the current role and activity sections intact.
5. If seat fields are missing, show precise incomplete-state messaging instead of fake values.

## 5. Guardrails

1. No new editable forms in this slice.
2. No project-role drawer implementation yet.
3. No seed-data edits here.
4. Do not invent authority docs that are not present in data.

## 6. Validation

Run:

```bash
cd ui && pnpm build
```

Record pass/fail in the reply.

## 7. Done definition

All must be true:

- only `ui/src/components/SeatDetail.tsx` is edited,
- seat capability truth is clearly visible or explicitly marked incomplete,
- assignable / under-specified / budget-constrained state is legible,
- build result is reported,
- tmux reply is sent to Lyra,
- stop and wait for acceptance.

## 8. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S5D Seat Card Capability Truth
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5d-seat-card-capability-truth-v1.md
- ui/src/components/SeatDetail.tsx
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
