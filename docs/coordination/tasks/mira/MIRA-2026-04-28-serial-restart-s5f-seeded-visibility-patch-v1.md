# MIRA-2026-04-28-Serial-Restart-S5F-Seeded-Visibility-Patch-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 5F |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-packet implementation / depends on S5B+S5C+S5D+S5E |

## 1. Purpose

Populate the seeded prototype with enough contract-shaped data to prove the repaired surfaces visibly in the UI.
This is a visibility patch, not a product-model expansion.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md` (`US-P0-05`, `US-P0-06`, `US-P0-08`, `US-P0-09`, `US-P0-11`)
3. `docs/acceptance-spec-v1.1.md` (`US-P0-05`, `US-P0-06`, `US-P0-08`, `US-P0-09`, `US-P0-11`)
4. `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5-queue-control-v1.md`
5. this packet

## 3. Dependency

Start only after `S5B`, `S5C`, `S5D`, and `S5E` are accepted.

## 4. Scope

Edit only:

- `ui/src/stores/useDataStore.ts`
- `ui/src/mockData.ts`

Do not edit:

- any component file
- `ui/src/types/index.ts`

## 5. Required outcome

Seed at least one credible example for each repaired surface using the current 2026-04-28 coordination narrative.

Minimum requirements:

1. Add one `WorkItem` with a realistic `change_tier_record`.
2. Add one `Session` with a realistic `prompt_state`.
3. Add one `Session` continuity preview with Tier 0/1/2, skills, playbook matches, budget estimate, and fallback path.
4. Add one `Seat` with capability truth fields populated.
5. Keep all seeded text in Chinese and aligned with the current SeatLoom workday narrative.
6. Do not invent unsupported enum values or subtype values.

## 6. Guardrails

1. Keep the patch minimal but visible.
2. Reuse existing seats/work items/sessions where possible instead of duplicating the entire dataset.
3. No speculative future-state stories unrelated to the current contract baseline.
4. Preserve existing typed artifact data.

## 7. Validation

Run:

```bash
cd ui && pnpm build
```

Record pass/fail in the reply.

## 8. Done definition

All must be true:

- only `useDataStore.ts` and `mockData.ts` are edited,
- each repaired surface has at least one visible seeded proof state,
- seeded copy remains Chinese and coordination-realistic,
- build result is reported,
- tmux reply is sent to Lyra,
- stop and wait for acceptance.

## 9. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] S5F Seeded Visibility Patch
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5f-seeded-visibility-patch-v1.md
- ui/src/stores/useDataStore.ts
- ui/src/mockData.ts
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
