# Delivery: acceptance-card lineage read model + structured acceptance-plan enforcement

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-10-acceptance-lineage-readmodel-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| tags | nimbus, acceptance-lineage, lineage, read-model, acceptance-plan, d2 |
| governing dispatch | `disp-accplan-lane-b-nimbus` (Lane B, acceptance-lineage milestone) |
| governing packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-acceptance-lineage-readmodel-v1.md` (on `packet/lyra/milestone-summary`; not on this branch) |
| value-domain contract | `VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1` (Lyra sign-off 2026-10-10), carried verbatim in the packet §2 |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/acceptance-lineage-readmodel` |
| branch lineage | `packet/nimbus/waker-hardening` (D2 defect fixes `cc5ad22`/`9ff5e6a`) -> this branch. The defect dispatch was COMPLETED first; this branch continues FROM it, so the two never diverge on `dispatch.rs` / the acceptance write path (packet §1's serialisation rule). |
| scratch database | `seatloom_gate3` (001–012 + seed + 013/014 from `track/infra-foundation` + 016 from `packet/onyx/acceptance-plan-schema` + scratch 017; NEVER live `seatloom`) |
| ledger gate event | `ev-acceptance-lineage-readmodel-landed` (unblocks Mira's Lane A) |

---

## 0. Summary

The operator's blocking feedback on the pending-acceptance card — "no
cause-and-effect, no lineage" — was two DISTINCT gaps. Both are closed on the
Nimbus lane:

1. **Lineage (a pure DISPLAY gap).** Everything was already in the ledger; only
   the assembly was missing. New `seatloom_core::db::lineage` assembles one
   card into the contract's seven blocks — `acceptance`, `dispatch` (with the
   ORIGINAL task statement), `events` (the chain in `occurred_at` order),
   `fluxVerdict`, `products`, `dependencies` (with each referent's live
   existence), `rework` — with ZERO new storage and ZERO new event types.
2. **Acceptance plan (a REAL data gap).** `request_acceptance` now REQUIRES a
   shape-v1 plan and validates it BEFORE any DB contact: a missing, empty or
   malformed plan is a typed rejection naming the rule broken. A valid plan
   rides the `AcceptanceRequested` payload AND the `acceptances` row in the SAME
   transaction, and is immutable afterwards (there is no update path).

Both surfaces share ONE assembler: the Tauri command `cmd_get_acceptance_lineage`
and the MCP read tool `get_acceptance_lineage` call the same
`seatloom_core::db::lineage::get_acceptance_lineage`, so they cannot drift — and
no seat needs `psql` to answer "what is this card and where did it come from?"
any more.

---

## 1. What changed

| File | Change |
|---|---|
| `crates/seatloom-core/src/db/lineage.rs` | **NEW** — the pure-read assembler + its seven block structs (camelCase wire shape) + unit tests |
| `crates/seatloom-core/src/db/mod.rs` | registers `pub mod lineage` |
| `crates/seatloom-core/src/db/repositories.rs` | the row-level reads the assembler composes: `get_dispatch`, `get_event`, `list_events_for_refs` (two-hop chain walk), `list_object_refs_for_events`, `product_title_and_path`, `list_dispatch_dependencies`, `referent_exists`; the acceptance read models now select `acceptance_plan` (016) |
| `crates/seatloom-core/src/db/models.rs` | `AcceptanceRow.acceptance_plan` + `ObjectRefRow` |
| `crates/seatloom-mcp-emit/src/dispatch.rs` | `AcceptancePlanV1` + `validate_acceptance_plan` (strict shape v1); `request_acceptance` guard 0b (plan, before any DB contact) + plan on the row and in the event payload; new tool `get_acceptance_lineage`; new rejections `MissingAcceptancePlan` / `InvalidAcceptancePlan` / `AcceptanceNotFound` |
| `crates/seatloom-mcp-emit/src/protocol.rs` | `get_acceptance_lineage` schema; `acceptance_plan` REQUIRED in the `request_acceptance` schema with the full shape; `emit_event` enum 22 values |
| `crates/seatloom-mcp-emit/src/main.rs` | tool wired into `tools/call` (8 tools) |
| `src-tauri/src/commands/acceptance_cmds.rs` | `cmd_get_acceptance_lineage` |
| `src-tauri/src/dto.rs` | `AcceptanceLineageDto` + the seven block DTOs (one `From` each) |
| `src-tauri/src/main.rs` | command registered |
| `ui/src/lib/types-dto.ts`, `ui/src/lib/api.ts` | the wire-shape mirror + the `getAcceptanceLineage` wrapper for Mira's card UI (no UI logic — that is Lane A) |
| `crates/seatloom-core/tests/acceptance_lineage_integration.rs` | **NEW** — 3 DB tests (seven blocks; rejected/void lineage; unknown id) |
| `crates/seatloom-mcp-emit/proof/acceptance_lineage_proof.sh` | **NEW** — the 5-point behavioural proof |
| `.scratch/017_acceptance_cancel.sql` | renamed from 016 — Onyx's 016 (acceptance_plan) took that slot first; see §4 |

### The two-hop event chain

`list_events_for_refs` walks the chain in TWO bounded hops: hop 1 = every event
naming the dispatch or the acceptance (any role) plus the Flux verdict the card
names; hop 2 = every event naming a PRODUCT found in hop 1. Without hop 2 a
delivery's `ArtifactCreated` that names only the artifact (not the dispatch)
would leave the products listed but the event that produced them missing. This is
a context chain, not a graph walk — the depth is fixed and documented.

### Reads never invent

A referent that cannot be joined is carried with `resolved: false` and its id; a
missing Flux verdict leaves the block absent rather than defaulted; an unknown
acceptance id is a typed refusal, not an empty card.

---

## 2. Validation

Commands run (Rust 1.95.0; scratch DB `seatloom_gate3` with 001–014 + 016 +
scratch 017 applied in `apply-db.sh`'s canonical order; `seatloom-cli reconcile`
run first, as CI does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| compile | `cargo check --workspace --all-targets` | PASS |
| core tests | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (112 lib + 3 lineage + 6 D2 + 27 baseline + 3 seed) |
| mcp-emit tests | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (53, incl. the plan matrix) |
| UI | `pnpm typecheck` + `pnpm test` | PASS |
| proof | `proof/acceptance_lineage_proof.sh` | PASS (5 points; re-runnable) |

New / updated tests:

- `dispatch::tests::acceptance_plan_shape_v1_rejects_every_malformed_branch` —
  the whole negative matrix in one table: `{}`, `null`, non-object,
  `plan_version` 2 / non-integer, unknown key (named in the error), empty
  `steps`/`expected`, blank `action`/`input`/`output`/`expect`, blank
  `preconditions` entry; plus the happy path returning the typed plan.
- `dispatch::tests::request_acceptance_base_is_optional_but_the_plan_is_not` —
  the two optionality rules side by side on the wire.
- `dispatch::db_itests::request_acceptance_enforces_the_plan_and_lands_it` —
  against a real DB: 8 malformed plans each refuse AND `canonical_events` does
  not move AND no card is created; a valid plan then lands identically on the row
  and in the `AcceptanceRequested` payload.
- `lineage::*` (core) — the seven blocks assembled from seeded rows; the
  rejected/void lineage; the unknown-id refusal; the `{}`-is-not-a-plan and
  never-defaulted-verdict-field unit tests.
- `acceptance_lineage_integration.rs` — seven blocks incl. the two-hop chain,
  products joined and carried, per-edge dependency existence, rework block.
- Updated pre-existing fixtures for 016: the D2 acceptance tests now seed a real
  plan (016's CHECK rejects `{}` for NEW rows — a test fixture is a new row) and
  the duplicate-pending test carries a plan so it reaches the unique index it is
  actually about.

Proof output (abridged; full run in the script):

```
POINT 1  ERR ... no acceptance plan ... (x2) / plan_version must be 1 / unknown field `surprise`
         / steps must be a non-empty array / expected must be a non-empty array
         / steps[0].action ... / expected[0].expect ... / the plan must be a JSON object
         canonical_events: before=5 after=5   acceptances rows: 0
POINT 2  OK requested acceptance acc-disp-lin-proof ... base track/infra-foundation
         plan identical in row and event: t
POINT 3  blocks present: acceptance, dispatch, events, fluxVerdict, products, dependencies, rework
         original task payload: {"note": "the original task words", "packet": "p/lineage.md"}
         event chain: SeatDispatchIssued -> SeatDispatchDelivered -> SeatDispatchCompleted
                      -> ReviewVerdictIssued -> ArtifactCreated -> AcceptanceRequested
         flux verdict: pass | evidence: gates green | lane: nimbus
         dependencies: [('dispatch','disp-lin-proof',True), ('event','ev-lin-missing',False)]
         card plan scenario: operator verifies the delivered slice end to end
POINT 4  canonical_events: before=6 after=6 (the read is pure); unknown id = typed refusal
POINT 5  tools: [... cancel_acceptance, get_acceptance_lineage]; lineage params: ['acceptance_id']
         request_acceptance required: [... 'acceptance_plan']
```

---

## 3. Escalations and decisions (not worked around)

1. **`tools/list` is 8, not the packet's "6 -> 7".** The packet assumed only
   `request_acceptance` was extra on this branch; the branch also carries
   `cancel_acceptance` from the completed defect dispatch (`cc5ad22`). The
   lineage tool is therefore the 8th. Declared rather than silently renumbered.
2. **Scratch migration slot moved 016 -> 017.** Onyx's 016 (`acceptance_plan`,
   `a6f2ac8`) landed first — its gate event `ev-016-acceptance-plan-landed` is
   what unblocked this dispatch — so the D2 withdrawal migration drafted last
   round as 016 is renamed `.scratch/017_acceptance_cancel.sql` (015 remains
   Onyx's D5 knowledge-layer slot). Still NOT FOR COMMIT: Onyx owns
   `infra/postgres/schema/` and `scripts/apply-db.sh`.
3. **The legacy pending card still has no plan.** `acc-disp-me6Nm8e9D6ZRe4fKyVokY`
   reads as `acceptancePlan: null` in the lineage, and 016's CHECK is NOT VALID
   precisely so that row survives until Aegis backfills Lyra's authored plan and
   runs `VALIDATE CONSTRAINT`. That backfill is a live-DB act on the Aegis lane;
   it is NOT done here. The card also still cannot be cancelled until 017 lands.
4. **The plan is authored by the PO.** Contract §5: never by Mira/Nimbus/Flux.
   The plan used in this delivery's own acceptance request (§5 below) is
   assembled from Lyra's packet §5/§7 — the PO's own done-definition and
   one-step verification — not authored by the seat.
5. **No operator-side cancel, no UI cancel button.** As with the defect dispatch:
   a Tauri command with no button is dead code and the pending-acceptance surface is Mira's
   lane. Lane A can consume the lineage DTO + `api.getAcceptanceLineage` that
   this delivery adds.
6. **Pre-existing docs-gate debt on this branch** (unchanged from the previous
   delivery doc): `scripts/verify-docs.sh` is already red here for D2-era docs
   not in `.gitlab/docs-grandfather.txt`. This delivery's doc conforms.

---

## 4. Branch lineage and the serialisation rule (packet §1)

```
packet/nimbus/waker-hardening      39e8709 (D2 regression tests)
                                -> cc5ad22 (defect 1 + 2: base + cancel)
                                -> 9ff5e6a (ledger event id)
packet/nimbus/acceptance-lineage-readmodel  (this branch, continues from the above)
```

The defect dispatch was completed FIRST, so `dispatch.rs` and the acceptance
write path were never edited on two branches at once. Merging this branch into
`track/infra-foundation` carries the defect fixes with it — both are delivered,
both await Flux verification, and one verification pass covers the chain.

Not done here: `infra/**` untouched; no whole-branch merge; no second
`request_acceptance` implementation (the enforcement lands where the tool
actually lives — this branch, which carries it).

---

## 5. Self-requested Flux verification + acceptance request

After this delivery doc: emit the gate event `ev-acceptance-lineage-readmodel-landed`
(exact id, per packet §6.3) to unblock Mira's Lane A, then `complete_dispatch`,
then enqueue a commit-pinned Flux verification for `seat-flux-001`
(branch `packet/nimbus/acceptance-lineage-readmodel`, base
`track/infra-foundation`), and on Flux green call `request_acceptance` with the
packet §7 one-step verification verbatim:

The one-step verification carried into the acceptance card is the packet §7 text
VERBATIM (Chinese, as Lyra authored it — reproduced in the packet, not re-worded
here, because this document must be English per §10 while the tool argument must
not be paraphrased). In English it reads: call `cmd_get_acceptance_lineage` on the
pending card `acc-disp-me6Nm8e9D6ZRe4fKyVokY` and get, in one response, the
upstream dispatch (title / issuer / time / original payload), the full event
chain, the Flux verdict and its evidence, the product and delivery-doc entries,
and the dependency edges; the MCP `request_acceptance` refuses a missing
acceptance plan with a typed error and zero writes, and only creates a card with
a valid plan.

The seat never merges its own work: that request is handed to the operator, and
the merge happens only on an explicit accept.
