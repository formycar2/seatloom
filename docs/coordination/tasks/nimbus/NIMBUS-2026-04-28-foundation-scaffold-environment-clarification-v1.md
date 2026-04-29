# Task Clarification: Nimbus Foundation Scaffold Environment Rule

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-28-foundation-scaffold-environment-clarification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-v1.md` |
| tags | architecture, foundation, scaffold, environment, clarification |
| owner | Nimbus |
| acceptance owner | Lyra |

## Clarification

This is a bounded execution clarification for the active foundation scaffold packet.

1. Do not spend this packet on installing Rust, rustup, Homebrew packages, or other environment tooling.
2. If `cargo` / `rustc` are unavailable on the seat, continue with the scaffold implementation and record the missing toolchain as an environment verification blocker in the delivery artifact.
3. The delivery artifact must still include the exact compile command attempted and the exact failure text if compile verification cannot run locally.
4. Keep the main output focused on scaffold tree, core type coverage, and deferred verification risk; do not stop early just because compile verification is unavailable.

## Replacement scope note

This clarification narrows the original done definition for this seat only:

- required now: scaffold files + type coverage + exact deferred verification record;
- not required now: installing a local Rust toolchain as part of the packet.

## Required delivery path

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md`

## Required blocker wording if compile cannot run

Use wording equivalent to:

- `Local compile verification deferred: cargo/rustc unavailable on the current seat environment.`
- include the exact attempted command and exact shell failure output.
