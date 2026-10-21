# Delivery: the operator-decision object — PASS, the default decider measured not assumed

| Field | Value |
|---|---|
| dispatch_id | disp-ewhjQhrVFlgpcrNp11ViZ |
| verified | disp-dwSmYkULRU4HdSDJCDWH9 (AEGIS-2026-10-22-operator-decision-object-v1) |
| verdict | pass |
| verdict event | ev-qGwjjmysUqELKo2haQP5c (ReviewVerdictIssued, emitted) |
| evidence | docs/coordination/tasks/flux/FLUX-2026-10-22-operator-decision-object-verdict-v1.md |
| branch | packet/flux/intent-verdict @ 5f590dd |
| tested at | packet/nimbus/project-scoped-reads @ a545a77 (checked out and the tip) |

## Verdict: PASS

Only the operator and Aegis can create, and every other seat plus every near-miss is refused
outright; a missing `choices` or `consequence` is refused by name; a resolved row is immutable at the
Rust path **and** by raw SQL through the same IFF constraint; and the migration seeds nothing.

## The two things Aegis asked me to weigh — measured, not assumed

I wrote a throwaway probe inside the crate that **printed** the values instead of asserting my
expectations:

```
caller=seat:seat-aegis-001  decider="human:supervisor"
decider == the caller?  false
```

**1. The default decider is NOT the caller** — so the packet's feared failure is absent. Aegis cannot
create a row only Aegis must rule on by omitting the field, and no seat can direct an obligation to
itself by omission. The hardcoded value is the **operator**, which is what Nimbus's comment quotes
the packet as saying, so the dispatch's phrasing reads *opposite* to the packet. I treated the packet
as authoritative and recorded the discrepancy rather than silently choosing one.

**2. An empty choices array is genuinely refused**, by name (`NoChoices`), and a blank option as
`Empty { field: "choices" }` — for both an Aegis and a `human:*` caller.

## Four mutations, all caught

| mutation | result |
|---|---|
| any `seat:*` may create | 3 tests fail |
| the `NoChoices` guard removed | 2 tests fail |
| the decider defaults to the **caller** | 2 tests fail — **including the project's own cited test** |
| a near-miss aegis ref allowed | 1 test fails |

Mutation 3 is the notable one: defaulting the decider to the caller breaks the project's own test, so
the cited test already defends against the exact regression the dispatch worried about. Aegis's read
of it is accurate.

All mutations restored; Nimbus's tree is byte-identical to `a545a77` and the probe is gone.

## Two details worth recording

**The decider is never a parameter on the operator's own path.** All three desktop commands resolve
the operator ref internally through `read_operator_ref()` and pass *that*, so a caller cannot read or
appear to rule on someone else's obligation. The resolve path re-checks `row.decider != operator`,
and `read_operator_ref()` has **no default** — with neither file nor env it refuses rather than
guessing. The UI likewise offers no free-text resolution: the choices are buttons.

**A silent 0 and a real 0 are different things.** `fleet-status.sh` refuses to read a missing table
as a zero: `读不到裁定清单 —— 这一项没被检查,不是'没有'`. That is the failure mode this whole feature
exists to remove.

## Gates, run by me in Nimbus's worktree at `a545a77`

`cargo build --workspace` 0 warnings · `cargo test --workspace` **292 passed / 0 failed** · `clippy
--all-targets` **0 warnings** · `fmt --check` clean · `tsc --noEmit` clean · `vitest run` **199
passed** (incl. the 5 new UI tests) · `vite build` green.

## One finding

`"human:supervisor"` is a **literal** in four places that must agree: `decisions.rs:377`, the column
DEFAULT, `fleet-status.sh` (twice each), while `read_operator_ref()` resolves whatever
`~/.seatloom/operator` says. With an operator file naming anyone else, an Aegis-created row whose
decider was omitted **cannot be ruled on by the human it is addressed to** — it exists, is counted,
is listed, and the resolve refuses. No error at create time; the mismatch only appears at resolution.

Not a spec violation and not a regression: the literal is the documented default in both packet and
schema and is correct for the current configuration. Recorded because Aegis asked me to weigh exactly
this, and because the one configuration that breaks it is the one `read_operator_ref()` exists to
allow. The fix is to derive the default from `read_operator_ref()` in the MCP path — the operator's
call, not a seat's.

## Boundary

No code, DB, or merge change by me. My probe was appended to the crate, never committed, and is
deleted; Nimbus's tree is clean. **No database was reachable** (`DATABASE_URL` unset, no local
postgres), so the 6 integration tests including the immutability proof **skip** here — I verified the
constraint IFFs by reading the SQL rather than executing a negative path. Nimbus's note says he ran
them on a scratch DB; I report only what I measured. No acceptance card opened.

## Artifact index

- Verdict: `docs/coordination/tasks/flux/FLUX-2026-10-22-operator-decision-object-verdict-v1.md`
- Ledger event: `ev-qGwjjmysUqELKo2haQP5c`
- This delivery note.
