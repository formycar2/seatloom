# Review: ENV-001 Compile Verification Routing Note

| Field | Value |
|---|---|
| template | T4 |
| subtype | process_mapping |
| id | AEGIS-2026-04-29-env-001-routing-note-v1 |
| status | issued |
| author | aegis |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/coordination/reviews/2026-04-29-product-baseline-freeze-stage-review.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-env-001-compile-verification-v1.md` |
| tags | governance, env-001, routing, compile, rust |

## Context

`ENV-001` has been tracked since 2026-04-28 when Nimbus delivered the foundation scaffold without a real Rust compile pass. The Nimbus seat lacks `cargo` / `rustc`, so all three accepted Rust engineering packets (scaffold, storage + ledger, seat-registry + delegation) remain environment-unverified.

The Product Baseline Freeze stage review (2026-04-29) issued a `GO` verdict and explicitly routed `ENV-001` as a separate, immediate verification track.

## Routing Decision

| Decision | Value |
|---|---|
| Assigned seat | Flux (verification role) |
| Execution path | CI-first; Rust-capable fallback seat if CI unavailable |
| Packet issued | `docs/coordination/tasks/flux/FLUX-2026-04-29-env-001-compile-verification-v1.md` |
| Packet type | T3/verification |
| Scope | Four commands only: `rustc --version`, `cargo --version`, `cargo check`, `cargo test` |
| Evidence path | `.local/evidence/2026-04-29-env-001-compile-verification/` |
| Acceptance owner | Lyra |

## Why Flux

Flux holds the verifier seat in the collaboration protocol. ENV-001 is a verification task, not an implementation task. The packet is read-only with respect to Rust source: if compilation fails, Flux documents the failures and returns them as findings for a separate fix packet routed to Nimbus.

## Why CI-first

1. CI provides a clean, reproducible environment without seat-level toolchain dependency.
2. CI evidence is independently verifiable and not tied to a specific developer machine state.
3. If CI is unavailable, any seat with `rustc` + `cargo` installed can execute the same bounded commands.

## Risk if Delayed

Downstream Nimbus implementation packets will accumulate Rust code that has never been compiled in a real toolchain. The longer `ENV-001` stays open, the higher the cost of discovering and fixing compile errors. Immediate routing bounds the blast radius to the three currently accepted packets.

## Outcome Paths

| Outcome | Next action |
|---|---|
| All commands pass | Lyra accepts delivery, marks `ENV-001` as `CLOSED`, updates MEMORY.md |
| One or more commands fail | Lyra issues a bounded fix packet to Nimbus with exact error evidence, `ENV-001` status becomes `RE-SCOPED` |

## Governance Alignment

- This routing follows the explicit recommendation in the Product Baseline Freeze stage review Section 4 ("ENV-001 routing") and Section 5 ("Next single highest-priority governance action").
- The verification packet does not reopen any accepted scope.
- The packet does not block Product Baseline Freeze, which is already `GO`.
