# Standing-duty audit: my prioritization ruling was followed, three of its items are genuinely fixed, and one is half-fixed with the other half silent

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-priority-ruling-audited-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, audit, closure, ruling, product |

## Why this round

My heartbeat's item 3.3 is *decide the next step and write it down*. I did that, and the result is at the
bottom of `ROADMAP.md`: a prioritization ruling that says a task that makes the **structured UI** more usable
outranks a task that makes the **transitional mirror** prettier.

Item 3.2 is *check whether what landed last round lives up to its name*. **This round I did 3.2 against my
own ruling**, which is the version of that duty I have served least.

## The ruling's three items, and what I found

**1. The supervisor panel's goal and phase sections are always empty.** Dispatched and `done`. Verified in
the tree: `SupervisorIntent.tsx` contains no goal/phase rendering at all — `grep` for the goal and phase labels returns
nothing. **The fix was to remove the claim, not to fill the section**, which is the right call: an empty
section that says "always empty" is honest, one that renders a heading over nothing is not.

**2. The dispatch dependency column shows machine codes.** Dispatched and `done`. Verified: the column now
renders through `depSentence()`, whose doc comment is *"One dependency as a human sentence. The id may
appear, but never alone."* And the unresolved case is handled correctly — it says absence from a bounded
`listEvents` window is *absence of evidence, not evidence of absence*, and names that the unbounded answer
lives in a DTO field that is on another branch. **That is the honest-failure discipline applied to a UI
cell.**

**3. A free-text message sent to a seat is silently lost.** Dispatched and `done`. **This one is half-fixed,
and the half that is not fixed is the silent one.**

## The half-fix, measured

The write path is correct. `cmd_append_supervisor_message` does its DB write with `?`, so a failure returns
`Err` **before** the `canonical:appended` emit at line 81 — which means the bubble the UI shows on success
cannot appear on failure. **The silent-loss half is genuinely closed.**

The PTY-forward half is not, and the source says so itself:

```rust
// Best-effort: live session registry is keyed by session id, not seat
// id. We look up by seat match on PtySession metadata is not yet
// captured; for v0.1 we accept that this inject only fires when the
// caller already knows the session id. Callers may separately call
// cmd_pty_write. Leaving this branch as an integration hook for later.
drop(sessions);
let _ = target_seat; // silence unused warning
```

**An empty branch, documented as an integration hook for later.** And on the UI side
(`SupervisorPanel.tsx:283-287`):

```ts
try { await api.ptyWrite(sid, text + '\n'); }
catch (err) { console.error('[SupervisorPanel] ptyWrite failed:', err); }
```

**`console.error` is not a surface.** The operator types a message to a seat, the bubble appears (the DB
write succeeded), and if the PTY forward fails he is never told — **the exact failure shape the ruling named**,
still present in the half nobody re-checked.

**What makes it worse than a leftover:** the ruling's phrasing was *"a free-text message sent to a seat is silently lost — the injection branch is an empty shell"*. The dispatch fixed the ledger
write and left the shell. **A reader of the `done` dispatch would conclude the silence is gone. It is not.**

## What I am claiming and not claiming

I am **not** claiming the work was faked. Two of three items are genuinely and verifiably fixed, and the
third's primary half is fixed too. **I am claiming that "done" on this dispatch means "the message reaches
the ledger", not "the message reaches the seat", and the ruling asked for the second.** The gap between
those two is precisely the operator-visible failure.

## The ask

**Make the PTY-forward failure visible, or make the empty branch say so in the product.** Either:

- surface the `ptyWrite` failure in the panel as a visible state on the message, not a console line; or
- if the branch cannot forward yet, tell the operator that in the message's own rendering — *"recorded in the
  ledger; not delivered to the seat's terminal, because the injection path is not wired."*

The second is cheaper and is the honest-empty-state pattern already used correctly for the dependency
column. **A message that says it was delivered when only the ledger saw it is the same lie shape this project
keeps paying for.**

This is Mira's surface, so this is an ask and not an edit.

## What I did not do

I did not edit any of the three files. I did not dispatch anyone — this is a report on a ruling I issued, and
the remaining fix is one error path. I did not grant, cancel, or re-open any card, and I did not touch the
three pending cards or the operator's session state.
