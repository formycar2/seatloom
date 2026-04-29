# Lyra Operational Note - Nimbus Model Routing Protocol

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-04-29-nimbus-model-routing-protocol-v1 |
| status | active |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| scope | Nimbus task issuance policy |
| references | `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md` |

## Decision

Adopt the following Nimbus model-routing rule immediately:

- prefer `claude-opus-4-6` for code-writing, architecture implementation, migration work, schema work, repository work, or verification-script authoring;
- prefer `deepseek-v4-pro` for lower-complexity dirty work such as evidence gathering, inventory extraction, structured data generation, or bulk seed preparation;
- if a packet contains both, split the packet by work type instead of mixing model expectations in one step.

## Transport Safety Rule

Do **not** send `/model ...` commands inside Nimbus tmux task-dispatch messages.

Reason:

- when `/model ...` is prepended to a longer tmux payload, the model switch may not execute cleanly and the whole packet can fail or be partially ignored.

Operational rule going forward:

1. Nimbus task dispatches from Lyra must contain task content only.
2. Model preference may be recorded in the task packet as advisory context, but not emitted as an inline seat-chat command.
3. If a model switch is ever truly required operationally, it must be handled out of band as a standalone interaction, not bundled with task text.
4. Keep infrastructure packets serial unless explicit parallelism is justified.
5. Prefer `claude-opus-4-6` for any task that changes Rust code or storage architecture.
6. Prefer `deepseek-v4-pro` for bounded collection / normalization / seed-generation work that does not require strong architectural reasoning.

## Immediate Application

Current active Nimbus packet is code-writing infrastructure work:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`

Therefore the preferred active Nimbus model for that class of work is `claude-opus-4-6`, but Lyra should not send `/model` inline in Nimbus tmux task messages.

## Non-Goals

- This note does not change product scope.
- This note does not authorize business logic expansion.
- This note does not replace task packets; it only governs how Lyra should route Nimbus model selection before packet issuance.

## Artifact paths

- `docs/coordination/tasks/lyra/LYRA-2026-04-29-nimbus-model-routing-protocol-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
