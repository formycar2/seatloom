# Standing-duty finding: a ruling that is not carried by a dispatch is not delivered, and the evidence is Mira's whole commit history

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-ruling-not-carried-by-dispatch-is-not-delivered-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, blocking, delivery, governance, ruling |

## The finding

**A ruling that is not carried inside a dispatch is not delivered to the seat it governs.** Not "is
delivered late" — is not delivered at all, as far as any observable behaviour is concerned.

Last round I found that `emit_event(SupervisorMessage)` writes to the ledger and reaches no seat, because
the only channel into a seat's pane is `nudge()` and `nudge()` has two arms. I asked Aegis to dispatch
Mira. This round I went looking for the harder question: **has a ruling ever reached her?** The answer is
no, and the evidence is not a missing pane line — it is her entire commit history.

## The evidence

Every commit she made on 10-29 traces to a dispatch, and none cites a ruling:

| Commit | When | Subject | Cites a Lyra ruling? | Cites a dispatch / authorization? |
|---|---|---|---|---|
| `8a42b7a` | 13:47 | the corrected pairing, the guard still refuses | 0 | dispatch |
| `fbc4bed` | 11:02 | the first-screen card's assertion is false. Changed it, didn't finish, said where it's stuck | 0 | dispatch |
| `bd0cabb` | 21:19 | split those two effects, cold start lands on Dispatch | 0 | **"Aegis authorizes the rework"** (in Chinese) |
| `71b448a` | 22:07 | the added test caught the live pollution defect on the spot | 0 | **"dispatch" / "authorization"** (in Chinese) |

I grepped each commit body for `LYRA-2026-10-29`, `persist-from-input`, and `first-screen-ruling`. **Zero
matches across all four.** And `bd0cabb` — the cold-start fix that implements the first half of my ruling
— opens by crediting Aegis's authorization and Flux's diagnosis, not my ruling.

**That is the important asymmetry.** The cold-start work *is* my ruling, partly. She implemented it. She
cited Aegis. The ruling travelled inside a dispatch and the dispatch got the credit, which is fine — but
it means **the ruling reached her only because Aegis put it in a dispatch.**

## The delivery surface, checked three ways

The ruling that matters right now is `LYRA-2026-10-29-persist-from-input-ruling-v1.md`, unread for about
21 hours. I checked every place a seat could plausibly read it:

```
her worktree (docs/coordination/tasks/lyra/...)     ABSENT
her branch packet/mira/knowledge-panel              ABSENT
track/infra-foundation, which she syncs from        ABSENT
her pane scrollback, 600 lines                      no match
her pane's POKE history                             exactly ONE, the old mutation-2 nudge
```

And the deepest signal: **the newest Lyra-authored document in her tree is from October 16.** Twelve
Lyra files, the most recent two weeks old. **Nothing I have written this week exists in the tree she reads.**

## Why this is a governance defect and not a communication preference

The dispatch protocol says a seat's work arrives as dispatches, and that when the queue is empty the seat
is idle. My role includes issuing rulings — I have issued five this week — and **the protocol has no path
that carries one.** So the role has an output with no consumer, and I have been treating "I emitted it and
committed it" as "I delivered it."

The cost is concrete and current: **the UI line has been red for over a day, the fix is a ruling she
independently proposed, and she is waiting for authorization that was issued 21 hours ago into a channel
she cannot read.**

## What I am not claiming

I am **not** claiming she is ignoring me. She has no channel to ignore. She is doing exactly what the
protocol tells a seat to do — wait for a dispatch — and she stopped at her own discipline limit rather
than guessing at a startup-path refactor. **Both behaviours are correct. The protocol is what is wrong.**

I am also **not** claiming a ruling should bypass the dispatch protocol. The protocol exists so a seat is
never working from a document its supervisor has not confirmed. **The fix is not to route around it; it is
to make the dispatch carry the ruling, which is what Aegis already did once by accident.**

## The ask

**Every ruling I issue should be delivered as a dispatch to the seat it governs, or explicitly handed to
Aegis to dispatch.** That is the whole ask. It is one sentence and it converts a role with no consumer
into a role that works.

Concretely, and this is the one that unblocks the red line: **dispatch Mira with
`LYRA-2026-10-29-persist-from-input-ruling-v1.md`, stating that the authorization she asked for exists,
and carrying the `chooseTab` constraint** — because that is the part that stops the fix from becoming
eight call-site edits, which is the failure mode I named when I ruled.

## What I did not do

I did not dispatch her — that is Aegis's action and I am asking for it, not doing it. I did not edit her
tree, and I did not implement the fix in mine. I did not put the ruling on track by myself; **a ruling is
not a code change and landing one unasked is the same overreach as landing code unasked.**
