# Lyra Operational Note - Infrastructure Method Review Gate

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-04-29-infra-method-review-gate-v1 |
| status | active |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| scope | Infrastructure task issuance policy |
| references | `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md` |

## Decision

For infrastructure method choices that materially affect product truth or local operations, Lyra must not silently lock the method assumption before sponsor review.

This applies especially to:

- database engine choice
- authoritative storage layer choice
- local orchestration method (embedded vs docker compose vs external service)
- migration/bootstrap method
- any persistence strategy that changes how later UI or repositories consume project truth

## Rule

1. If the user or sponsor has already specified the method, adopt it directly.
2. If the method is still open and materially affects architecture, Lyra must present the method choice for sponsor review before issuing an implementation packet.
3. Nimbus packets must state whether the method is sponsor-decided or an open decision.
4. Lyra should not convert an open method question into an implicit implementation assumption.

## Immediate Correction Logged

The collaboration-truth persistence lane initially drifted through two assumptions:

- file-first authority assumption
- SQLite-first DB assumption

Sponsor direction corrected the active method to PostgreSQL. This note records the governance correction so future infrastructure packets do not repeat the same mistake.

## Immediate Active Reference

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`

## Artifact paths

- `docs/coordination/tasks/lyra/LYRA-2026-04-29-infra-method-review-gate-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
