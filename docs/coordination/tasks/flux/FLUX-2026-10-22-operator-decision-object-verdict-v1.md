# Verdict — the operator-decision object: **PASS**, and the default decider is measured, not assumed

- **Verification dispatch**: `disp-ewhjQhrVFlgpcrNp11ViZ`.
- **Implementation dispatch**: **`disp-dwSmYkULRU4HdSDJCDWH9`** (`verify_dispatch_id` in the payload);
  packet `AEGIS-2026-10-22-operator-decision-object-v1`, commit `a545a77`.
- **Branch under test**: `packet/nimbus/project-scoped-reads` @ `a545a77` — confirmed both **checked out**
  and the **tip**, tree clean. (I checked this first, because the previous dispatch in this loop was
  cancelled over a wrong branch name.)
- **Verdict event**: `ev-qGwjjmysUqELKo2haQP5c`, emitted with `turn_seq` null.
- **VERDICT: PASS** — with **one finding**, and it is the exact thing Aegis asked me to look at.

## Scope

One dispatch, its own verdict. Nothing merged in from earlier verifications.

## The two things Aegis asked me to check independently

Aegis's message named them: *"重点是默认 decider 是否为调用者本人(派单明确不许默认推给操作者)、以及
choices 空数组是否真被拒"*. I answered both with a throwaway probe inside the crate, printing the
measured values rather than asserting my expectations:

```
FLUX PROBE: caller=seat:seat-aegis-001  decider="human:supervisor"
FLUX PROBE: decider == the caller?  false
```

**1. Is the default decider the calling actor? NO — and that is not the bug it might look like.**
Omitted `decider` lands on the literal `"human:supervisor"`, hardcoded at
`decisions.rs:377`. So a caller is **never** implicitly the decider of the obligation it creates:
Aegis cannot create a row that only Aegis must rule on by leaving the field out, and no seat can
direct an obligation to itself by omission. That is the failure mode the packet forbids — an
obligation quietly routed to whoever asked for it — and it is absent.

The hardcoded value is instead **the operator**, which is what Nimbus's own comment quotes the
packet as saying ("默认是操作者,但不默认成所有事都推给他") and what the migration documents as
`decider ... DEFAULT is the configured operator ref`. So the dispatch's phrasing "不许默认推给操作者"
reads as the *opposite* of the packet's rule, and I am treating the **packet** as authoritative and
recording the discrepancy rather than silently picking one. See Finding 1 for why it still matters.

**2. Is an empty `choices` array really refused? YES**, by name. `validate_request` returns
`DecisionRejection::NoChoices` for `choices: []`, and a blank option is refused as
`Empty { field: "choices" }` — measured for both an Aegis caller and a `human:*` caller. Not
vacuous: the refusal names the rule.

## Method — four mutations, plus the probe

The probe's four assertions, then four mutations to prove they bite:

| mutation | result |
|---|---|
| `may_create_obligation` lets any `seat:*` create | **3 tests FAIL** (mine, the packet's cited test, and the missing-field test) |
| the `NoChoices` guard removed | **2 tests FAIL** |
| the decider defaults to the **caller** (what the dispatch feared) | **2 tests FAIL** — including the packet's own cited test |
| a near-miss Aegis ref (`seat:seat-aegis*`) allowed | **1 test FAILS** |

Every mutation restored; `git status` and `git diff` empty afterwards, so Nimbus's tree is
byte-identical to `a545a77`. The probe file was appended to the crate, never committed, and is gone.

Note that mutation 3 is the interesting one: defaulting the decider to the caller breaks the
project's own test, so the *cited* test is already defending against the exact regression the
dispatch worried about. Aegis's read of it is accurate.

## The packet's four hard requirements

**Only the operator and Aegis may create.** ✔ `may_create_obligation` is an allowlist: exactly
`seat:seat-aegis-001`, or a non-empty `human:*`. Every other seat in the fleet
(`nimbus`, `mira`, `flux`, `onyx`, `lyra`, `vela`) is refused outright, and so are the near-misses
(`seat:seat-aegis`, `aegis`, `human:`, `system`). Mutation 1 and 4 both caught. The test is named
`only_the_operator_and_aegis_may_create_an_obligation`, exists, and passes.

**Missing `choices` or `consequence` is refused by name.** ✔ Measured above. `consequence` goes
through the same `non_empty(...)` → `Empty { field }` path, so a caller is told which field is
missing rather than getting a generic failure.

**A resolved item is immutable.** ✔ Two independent walls. The Rust path refuses a second ruling
(`AlreadyResolved`); and `a_resolved_decision_is_immutable_at_the_database` goes **around** the Rust
path with raw `UPDATE` and asserts the refusal names
`operator_decisions_resolved_pairing`. I read the constraint: it is an IFF in **both** directions
(`(status='resolved') = (chosen IS NOT NULL AND resolved_by IS NOT NULL AND resolved_at IS NOT
NULL)`), so a pending row cannot carry a ruling either. That is the right shape — one rule, no
second implementation.

**The migration seeds nothing.** ✔ Zero `INSERT` statements in `020_operator_decisions.sql` as
executable code. There is exactly one `INSERT` and it is **commented out**, as a negative-path
example — Aegis's description is accurate. Registered in `scripts/apply-db.sh`'s POST-SEED list with
its reasoning in the entry.

## Two details worth recording

**The decider is never a parameter on the operator's own path.** `cmd_list_operator_decisions`,
`cmd_count_pending_operator_decisions` and `cmd_resolve_operator_decisions` all resolve the operator
ref internally via `read_operator_ref()` and pass *that* to the repository. A caller cannot name a
decider, so it cannot read or appear to rule on someone else's obligation. The resolve path
additionally re-checks `row.decider != operator` and refuses — so a row Aegis made for itself cannot
be resolved from the operator's click. And `read_operator_ref()` has **no default**: when neither the
file nor the env states an identity it returns `NotConfigured`, and the caller must surface it. That
is the correct failure direction.

**The UI cannot fabricate either.** `OperatorDecisionSection.tsx` passes no `decider` anywhere and
offers no free-text resolution — the choices are buttons. So a ruling outside the offered options is
not reachable from the screen. And `fleet-status.sh` refuses to read a missing table as a zero:
`读不到裁定清单(operator_decisions 表还没建)—— 这一项没被检查,不是'没有'`. A silent 0 and a real 0
are different things, which is the failure mode this whole feature exists to remove.

## Gates, run by me in Nimbus's worktree at `a545a77`

`cargo build --workspace` zero warnings · `cargo test --workspace` **292 passed / 0 failed** ·
`cargo clippy --workspace --all-targets` **0 warnings** · `cargo fmt --all -- --check` clean ·
`cd ui && tsc --noEmit` clean · `vitest run` **199 passed** (14 files, including the 5 new
`OperatorDecisionSection` tests) · `vite build` green.

## Finding 1 — the hardcoded operator identity diverges from the configured one

`"human:supervisor"` is a **literal**, not a resolved identity, and it appears in four places that
must agree:

| where | value |
|---|---|
| `decisions.rs:377` — the MCP create default | `"human:supervisor"` (literal) |
| `020_operator_decisions.sql:215` — the column DEFAULT | `'human:supervisor'` (literal) |
| `fleet-status.sh` — the pending count and list, twice each | `decider='human:supervisor'` (literal) |
| `config::read_operator_ref()` — the desktop resolve path | **whatever `~/.seatloom/operator` says** |

Consequence, measured: with `~/.seatloom/operator` containing `human:zhang-san`, an Aegis-created row
whose decider was omitted lands as `human:supervisor`, and the desktop resolve then **refuses** it —
`operator decision "dec-x" is for "human:supervisor" to rule on, not "human:zhang-san"`. The
obligation exists, is counted, is listed, and **cannot be ruled on by the human it is addressed to.**
No error is raised at create time; the mismatch only appears at the moment of resolution.

**This is not a spec violation and not a regression.** `human:supervisor` is the documented default
in both the packet and the schema, and it is the correct value for this operator's current
configuration. I record it because the dispatch asked me to weigh exactly this, and because the
divergence is silent and load-bearing: the feature's headline promise is a number and a list the
operator can act on, and the one configuration that breaks it is "the operator file names someone
else" — which is precisely what `read_operator_ref()` exists to allow. The fix is to derive the
default from `read_operator_ref()` in the MCP path rather than hardcoding it, and to have
`fleet-status.sh` resolve the same identity rather than repeat the literal. That is a design
decision for the operator, not something to change inside this dispatch.

## Honest limits

- **No database was reachable.** `DATABASE_URL` is unset and there is no local postgres, so the 020
  integration tests (6 of them, including the immutability proof) **skip** here rather than pass. I
  verified the constraint definitions by reading the SQL — the IFF pairing, the creator shape, the
  status domain — but I did not execute a negative path against a live database. Nimbus's note says
  he did, on a scratch DB; I am reporting what *I* measured.
- **My probe exercised `validate_request` only**, the pre-database layer. The refusal types are the
  same ones the write path returns, but I did not drive a real write.
- **`a_single_option_is_not_a_choice`** — the struct doc says a single option is not a choice. I
  measured it: one option is **accepted** (`Ok(["只此一法"])`). This is a doc-vs-behaviour mismatch,
  not a gate failure, and it is the *safe* direction (a one-option row is honest about having one
  option, and the consequence field still states what happens). Recording it because the doc claims
  otherwise.
- **The 22→25 tool count and the vocabulary widening (28→30 / 12→13)** I did not re-derive
  element-by-element; I confirmed the widened CHECKs exist as complete guarded lists and that the
  guards check for the new value rather than assuming it.

## Bottom line

**PASS.** Only the operator and Aegis can create, and every other seat plus every near-miss is
refused outright; a missing `choices` or `consequence` is refused by name; a resolved row is
immutable at the Rust path **and** by raw SQL through the same IFF constraint; the migration seeds
nothing and its one `INSERT` is a commented counter-example. The two things Aegis asked me to weigh
are both measured: an omitted decider is **not** the caller (the packet's fear, absent), and an empty
choices array is genuinely refused. Four mutations, including defaulting the decider to the caller,
are caught — the last of them by the project's own cited test. One finding: `"human:supervisor"` is
hardcoded in four places that must agree, so an operator file naming anyone else produces an
obligation the human cannot rule on. Silent, load-bearing, pre-existing, and the operator's call.
