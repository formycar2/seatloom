# LYRA-2026-04-27-Mira-Seat-Switch-Checkpoint-v1

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Active decision |
| Topic | Mira seat migration and checkpoint protocol |
| Effective date | 2026-04-27 |

## 1. Decision

Before switching Mira from the current Gemini-driven seat to a new OpenCode seat driven by DSV4Pro, we should **freeze a source checkpoint first**.

## 2. Why this is the right move

1. It prevents the typography/readability pass from being mixed with unfinished drift-signal cleanup.
2. It gives the replacement Mira seat a stable baseline to continue from.
3. It preserves auditability for PO review and stage-gate evidence.
4. It reduces the risk of losing UI state if the current seat is near model/runtime limits.

## 3. Checkpoint rule

The checkpoint should capture only durable project sources, not generated artifacts.

### Include
- `docs/`
- `ui/src/`
- `ui/package.json`
- `ui/pnpm-lock.yaml`
- `ui/package-lock.json` if still intentionally used
- UI config files (`vite`, `tailwind`, `tsconfig`, `postcss`, etc.)

### Exclude
- `.local/`
- `test-results/`
- `ui/node_modules/`
- `ui/dist/`
- `.DS_Store`
- any other generated cache or local runtime output

## 4. Recommended sequence

1. Freeze current Mira seat from making further UI edits.
2. Tighten ignore/staging scope so generated artifacts are excluded.
3. Create one checkpoint commit for the current UI/doc baseline.
4. Hand the new Mira seat only the checkpointed baseline plus the active task packets.
5. Resume UI work on the replacement seat.

## 5. Handoff rule for the new Mira seat

The replacement Mira seat must inherit these active references:
- `docs/prd-v0.4.md`
- `docs/interaction-spec-v1.0.md`
- `docs/acceptance-spec-v1.0.md`
- `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-27-typography-harmony-fix-v1.md`
- `docs/coordination/acceptance/2026-04-27-mira-v52-drift-signal-verification.md`

## 6. PO recommendation

**Yes: checkpoint first, then continue modifications on the replacement Mira seat.**

This is the lowest-risk transition path.
