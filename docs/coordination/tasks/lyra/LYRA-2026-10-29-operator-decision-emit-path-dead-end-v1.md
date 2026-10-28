# Standing-duty finding: `emit_event` accepts `OperatorDecisionRequested`, and the result is an obligation nobody can resolve

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-operator-decision-emit-path-dead-end-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, blocking, dispatch-ops, write-path, dead-end |

## The defect

Two obligations the operator must rule on exist **only as ledger events**. They have no
`operator_decisions` row, so they cannot be resolved and they do not appear in the surface built to show
them.

```
ev-op-decision-aegis-commit-policy   2026-10-28 06:06
ev-op-decision-023-deadlock          2026-10-28 22:04
```

Both were emitted by Aegis. Both are real questions with measured consequences. **Neither is answerable
by the person it is addressed to.**

## Root cause, measured

`request_operator_decision` is the correct path: it writes the row and the event in one transaction, and
the event carries `decision_id`, `choices`, `decider`, `consequence` and a `subject` ref of
`ref_type='decision'` (`repositories.rs:1777-1840`, whose own comment says *"one
`OperatorDecisionRequested` in ONE transaction (contract rule 2)"*).

But `emit_event`'s type enum **also** lists `OperatorDecisionRequested`
(`crates/seatloom-mcp-emit/src/protocol.rs:158`). So a seat can reach the same event type through the
generic writer, and what lands is an event with:

- **no `decision_id`** in the payload — `select payload->>'decision_id'` returns empty for both;
- **no `decider`, no `consequence`** as columns, because there is no row;
- **`subject` = the project and `actor` = the seat** — the shape `emit_event` always writes — instead of
  the `subject`/`ref_type='decision'` ref the typed path writes.

I verified this by comparing the two events' refs against what the core path writes:

```
emit_event path (what landed):
  subject|project|seatloom
  actor  |seat   |seat-aegis-001

core path (what it should be):
  subject|decision|<decision_id>
  ...
```

## Consequences, each measured

**1. They cannot be resolved.** `resolve_operator_decision` reads
`SELECT … FROM operator_decisions WHERE id = $1` (`repositories.rs:1694`). Zero rows → the ruling is
refused for a decision that does not exist in the table.

**2. They are invisible to the operator.** `OperatorDecisionSection.tsx:66` calls
`api.listOperatorDecisions(activeProjectId, 'pending')`, which runs `cmd_list_operator_decisions` —
also a table read. So the section renders nothing. I checked that the section **is** routed
(`DispatchInboxView.tsx:1275`, and the `dispatches` tab is reachable and `live`-badged), so this is not
the unrouted-view defect I found earlier. **The surface works; the rows are missing.**

**3. The ledger now claims two obligations exist that nothing can act on.** The events are real, they
are indexed, and any reader tracing them will conclude the operator was asked. He was not — not in any
way he can answer.

## Why this matters more than two stale rows

The 023 deadlock is the more serious of the two. Aegis's own measurement: 5 duplicate groups in 402
memories, the dedup guard refuses duplicates rather than upserting, and it **cannot land on live while
duplicates exist** — so the guard is written, proved (7 proof points, 211 Rust tests), and inert. Every
retry path that writes a duplicate keeps inflating the memory count, and `recall`, promotion and the
drift number all inherit the inflation. **The fix is blocked on an operator action, and the operator
cannot see the request.**

The second one — whether Aegis's non-product commits should go to track while cards are pending — is the
question behind 19 emergency sync dispatches, and it is equally unanswerable.

## The fix, and who owns it

Two candidate fixes, and they are not equivalent:

1. **Remove `OperatorDecisionRequested` (and `OperatorDecisionResolved`) from `emit_event`'s type
   enum.** The typed tool exists precisely because this event has required columns. Allowing the generic
   writer to produce a row-less version of it is the defect. This is Nimbus's file
   (`protocol.rs`), and it is a two-line change.
2. **Make `emit_event` write the row too.** That would mean the generic path growing the typed path's
   required arguments, which defeats the point of having a typed tool.

I lean (1), but it is a write-path change and not mine to make unprompted. What I can say with certainty
is that the current state — both paths accepted, one producing a dead end — is the worst of the three.

## What I did not do

I did not edit `protocol.rs`. I did not write the missing rows — that would fabricate `choices` and
`decider` that Aegis did not supply, and inventing an operator obligation's options is exactly the kind of
guess this project has paid for. I did not dispatch any seat.
