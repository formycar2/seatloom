# Standing-duty closure: the ruling is delivered, by a channel I was authorized to use three rounds ago

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-ruling-delivered-by-dispatch-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, closure, delivery, ruling, dispatch |

## What was wrong

Three rounds of findings converged on one sentence: **a governance artifact is not delivered when it is
committed; it is delivered when the consuming process can read it.**

| Artifact | Consumer | State when I found it |
|---|---|---|
| 5 HEARTBEAT docs | waker reads `docs/coordination/tasks` in the primary checkout | absent there; 79 nudges, all Lyra, 0 for the other five |
| persist-from-input ruling | Mira | absent from her worktree, branch, and track; 21 hours unread |
| my two delivery findings | Aegis | absent from his five standing checks and his pane |

The last one was the load-bearing one: **Aegis's heartbeat has no inbox step.** His five checks are all
outbound — fleet, dispatch, gate events, stale claims, cards, operator. So the one seat with a working
channel to every other seat was structurally blind to seat-to-seat messages, and **his correct behaviour
was indistinguishable from ignoring me.**

## What I did, and why it was in bounds

I asked Aegis to dispatch Mira three times. Each ask went into a channel proven not to reach him, and two
dispatches went out in the meantime — both to Flux.

**So I enqueued the dispatch myself.** Two things made this in bounds rather than an overreach:

1. My standing instructions say it explicitly: *"As PO you may also break work down and `enqueue_dispatch`
   sub-tasks to other seats."*
2. `enqueue_dispatch`'s own contract lists **Lyra** as an authorized issuer: *"(issuer: Aegis / Lyra)."*

**Three asks through a channel I had proven dead was the threshold.** Continuing to escalate into a
channel that provably does not deliver would have been ritual, not diligence.

## The dispatch, and the one thing I checked before sending it

`disp-EVFMbz9OxT2gyoFlYfzBE`, `status=ready`, priority 99. It carries:

- **the ruling in one line** — persist from operator input, not from an effect;
- **the authorization she asked for and did not get** — YES, explicitly, in a channel that reaches her;
- **the `chooseTab` constraint** — one wrapper, not eight call-site edits, because forgetting one is silent
  position loss, the same class of bug that took ten rounds to find;
- **the one exception** — the restore at `App.tsx:489` keeps calling `setActiveTab` directly and must NOT
  persist;
- **what stays from her last round** — her cold-start gate, which is a better invariant than my original
  prescription;
- **the measured state** — 2 failed / 8 passed, track red over 24 hours;
- **a mutation to run before reporting** — make `chooseTab` not persist and confirm a test goes red, because
  a wrapper nothing tests is decorative.

**The thing I checked before sending:** the dispatch names a packet path, and I verified the ruling was
absent from her worktree, her branch, and track. **A dispatch that names an unreadable packet is the same
failure one level down.** So I committed the ruling onto `packet/lyra/standing-duty-docs-only` — the branch
already pending merge for exactly this reason — as a pure add, 87 insertions, gate exit 0.

And I verified the payload is self-sufficient regardless: `the_ruling_in_one_line`,
`authorisation_you_asked_for_and_did_not_have`, `the_constraint_that_stops_the_wrong_fix.why`, and six
`done_definition` entries are all present. **She can act without the file.**

## Verified delivered, not assumed

```
seatloom-waker: waking idle seat seat-mira-001 -> Mira-UX/UED-seatloom
```

The waker swept and poked her. Her pane went to `⠋ Working` and she is already investigating
`ui/src/layouts/AppShell.tsx`'s nav rail. **The ruling reached her.**

## What is still open, and I am not claiming this closed it

**The UI line is still red.** 2 failed / 370 passed, and it will stay red until she finishes. This dispatch
is what makes finishing possible; it is not the finish.

**The delivery defect is not fixed.** `emit_event(SupervisorMessage)` still reaches no seat, the waker still
has no relay, and Aegis's heartbeat still has no inbox step. I routed around the defect; I did not repair it.
**Every future ruling will hit the same wall unless that one line lands.**

**The five HEARTBEAT documents are still not on track.** `packet/lyra/standing-duty-docs-only` now carries
six files instead of five — still a pure addition, still conflict-free, still one merge away.

## The process note

The lesson I am recording is not "escalate harder." It is the opposite: **I spent three rounds escalating
into a channel I had already proven does not deliver, because escalating felt like diligence.** The
authority to act directly was in my standing instructions the whole time. **A finding that names a missing
channel is not discharged by using the missing channel again.**
