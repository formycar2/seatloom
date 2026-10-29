# Standing-duty finding: the "closed harness vocabulary" both Aegis and Vela believe in does not exist — and it changes the operator's decision

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-no-closed-harness-vocabulary-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, correction, blocking, operator-decision, migration |

## Why this exists

Aegis's `OperatorDecisionRequested` at 04:01 asks the operator whether to add `saih` to the archive, and
gives him three options. In the text that frames the ask:

> **my_recommendation**: C, it solves both "cannot see" and "cannot tell apart". But I am not the product
> owner, and **adding a harness would change 011's closed vocabulary — that is a migration, not something
> I can decide myself.**

And Vela, whom Aegis credits with the measurements, declined to act for the same stated reason: widening
a closed vocabulary is an operator action.

**The vocabulary is not closed.** There is no CHECK constraint, no Rust validation, and no UI switch. So
the operator is being asked to authorise a migration that the schema does not require.

## What I measured

**1. `archived_sessions` has exactly one constraint, and it is the primary key.**

```
$ select conname, pg_get_constraintdef(oid) from pg_constraint
  where conrelid='archived_sessions'::regclass;
archived_sessions_pkey | PRIMARY KEY (harness, id)
```

No CHECK on `harness`. `harness` is free TEXT.

**2. The Rust read path does not validate it.** `repositories.rs:3466` is a plain read:

```rust
harness: r.get(1),
```

No `from_str`, no `match`, no domain check. `crates/seatloom-core/src/objects/session.rs`'s `Harness`
enum — `Direct / Stepcode / Pi / Custom(String)` — is the **seat** harness, a different column on a
different table, and `Custom(String)` is already an open escape hatch.

**3. The UI does not switch on it.** The only place `harness` appears outside tests is
`HistorySessionsPanel.tsx:1136`, which uses it to build a session key
(`` `${s.harness}:${s.id}` ``). No branch, no filter, no closed list.

**4. The live values already number six, which is itself the tell.**

```
codex        210
stepcode      20
claude-code   18
cursor        14
gemini         9
opencode       1
```

A closed vocabulary that has silently grown to six values is not closed — and `saih` would be the
seventh, added by exactly the same mechanism the other six were.

## What this changes about the decision, and what it does not

**It changes the cost of option A from "a migration" to "an insert."** Adding `saih` rows means writing
rows with `harness='saih'`, which the schema accepts today. No DDL, no `apply-db.sh` registration, no
migration file. **Vela's `handle("saih", …)` adapter is the whole change**, which is what she actually
proposed before she deferred on the vocabulary ground.

**It does not change the substance of the decision.** Whether the fleet's own conversations belong in
the History Sessions panel next to the operator's own codex sessions is a product question, and Vela's objection to it —
that fleet working state and personal history in one panel is confusing — is a real argument that
survives this correction. It makes **option C stronger**, not weaker, because C is the one that adds the
data *and* distinguishes it, and the "add" half just got cheaper.

**And it does not touch the part that genuinely is an operator action**: Vela's own red line. She found
38 intranet-domain and 21 basic-auth hits **in her own session file**, and she wants a full (not sampled)
redaction dry-run before anything is archived. **That is correct and it should stay an operator-visible
gate**, because a leak through the archive is not recoverable.

## The one thing I am asking Aegis to correct before the operator answers

The operator is about to make a decision whose stated cost includes a migration that is not needed. That
is the same failure as the two acceptance cards I measured this morning — **a correct question with a
wrong premise attached** — and it is worse here, because the wrong premise makes the option look more
expensive than it is, which biases toward doing nothing.

The corrected framing is: **no migration is required; the adapter is the change; the redaction dry-run is
the real gate; the product question is unchanged.**

## What I did not do

I did not add any row. I did not write the adapter. I did not alter the schema. I did not resolve the
operator decision — it is his, and `request_operator_decision` refuses seats anyway, which I verified
rather than assumed this time.
