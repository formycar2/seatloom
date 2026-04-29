# Task: Nimbus Foundation Scaffold Entrypoint + Metadata Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-29 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md`, `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md` |
| tags | architecture, scaffold, rust, tauri, metadata, fix, nimbus |
| owner | Nimbus |
| acceptance owner | Lyra |
| execution mode | One bounded fix packet. No parallel expansion. |

## Objective

Close the two remaining acceptance blockers on the foundation scaffold without expanding into runtime-engine work.

## Input Files

- `docs/PRODUCT_TRUTH.md`
- `docs/architecture-decisions.md`
- `docs/architecture-design.md`
- `docs/coordination/DOCUMENT_TEMPLATES.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md`
- `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md`

## Scope

Primary write targets:

- `src-tauri/src/main.rs`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md`

Optional support target only if strictly needed for the minimal entrypoint:

- `src-tauri/src/state.rs`

## Required Outcome

### 1. Add the smallest valid Tauri binary entrypoint

Requirements:

- `src-tauri/src/main.rs` must define a minimal `fn main()`;
- keep the runtime behavior stub-level only;
- do not expand into full command wiring, state initialization, or engine behavior;
- it is acceptable to leave command handlers unregistered if that is the smallest compile-safe shape;
- keep comments boundary-oriented and brief.

### 2. Normalize delivery metadata to the active taxonomy

Requirements:

- rewrite the delivery artifact header to expose machine-readable `template` + `subtype` fields;
- use a valid `T3` subtype from `docs/coordination/DOCUMENT_TEMPLATES.md` §11.1;
- preserve the existing delivery content unless a small wording correction is needed.

## Non-goals

- no runtime-engine implementation
- no adapter-engine work
- no compile-toolchain installation
- no new crates or dependency changes unless strictly required by the minimal entrypoint

## Done Definition

- [ ] `src-tauri/src/main.rs` contains a minimal valid binary entrypoint.
- [ ] The delivery artifact header uses a valid `T3` subtype and explicit `template` field.
- [ ] Scope stays bounded to entrypoint completeness and metadata normalization.
- [ ] If you cannot run compile locally, retain the existing environment blocker and do not expand scope.
- [ ] A delivery artifact is written.
- [ ] A tmux reply is sent to Lyra and Nimbus waits for acceptance.

## Validation

If local Rust is still unavailable, do not broaden scope. Keep the existing verification blocker and state that this packet is a static fix only.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-delivery-v1.md`

Required sections:

1. Scope completed
2. Entrypoint fix
3. Metadata fix
4. Changed files
5. Validation status
6. Blockers
7. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_foundation_fix.txt
[Nimbus -> Lyra] Foundation Scaffold Entrypoint + Metadata Fix
completed:
- ...
validation:
- local compile unavailable / compile command result ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-delivery-v1.md
- ...
MSG

tmux load-buffer -b nimbus_to_lyra_foundation_fix /tmp/nimbus_to_lyra_foundation_fix.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_foundation_fix
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
