# Standing-duty report: my four product findings reached nobody, so I stopped writing findings into a channel that demonstrably does not deliver

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-findings-reached-nobody-so-dispatched-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, delivery, dispatch, product-finding, closure |

## What I found when I checked my own output

I asked the question I should have asked days ago: **can the seat who would fix my findings see them?**

```
on track/infra-foundation:            none of the four
in Mira's own worktree:               none of the four
newest Lyra doc in her tree:          October 16
Lyra docs I have written since 10-29: 57
```

**She can see 35 Lyra documents, the newest from October 16. I have written 57 since 10-29.** Four product
findings, each with a measured fix, each committed and pushed to two remotes, and **every one of them
invisible to the only seat who can act on them.**

That is the fifth proof of the same sentence this week: **a governance artifact is not delivered when it is
committed; it is delivered when the consuming process can read it.** I have now proved it for the HEARTBEAT
docs, the persist ruling, my own two delivery findings to Aegis, and now these four product findings. **And I
kept producing the artifact anyway.**

## What I did instead

**Dispatched them.** `disp-OQyilLqRjR1fDSWsWE-5n`, `kind: implementation`, priority 99, status `in_flight`
within a minute — she pulled it and her pane is at `⠋ Working`.

The dispatch carries all four findings with their substance inline, because the documents are unreachable:

1. **The SAMPLE-badge lie, which a test enforces in both directions.** The `store-backed` test asserts
   `inbox` hydrated is `seed`; the enumeration pins the database-backed set to exactly
   `['dispatches','sessions']`. I proved both directions: removing entries → 2 red; **adding the six
   live-reading tabs → 3 red, the test refuses the fix.** The premise is false — `hydrateProjectData`
   fills the store from seven real Postgres reads, and live holds 10 seats, 7 workitems, 15 artifacts,
   2946 events. The fix is three edits, and the third is rewriting a test that asserts a distinction which
   does not exist.
2. **His three pending cards are invisible**, two independent reasons: the panel is closed
   (`supervisor.open = false`) with no badge anywhere else, and the panel scopes to his active project
   which has zero acceptances — so opening it shows "nothing to accept."
3. **The installed app predates the fix by 43 hours** — 10-28 22:16 versus 10-30 17:50 — so the first
   card's own step 1 fails on it. The project already wrote `prepare-verification-build.sh` for exactly
   this, with the reason in its header in plain language, and it was not run.
4. **His active project does not exist.** `project_delivery` is not a row in `projects`; four rows start
   with it. Every project-scoped read returns zero, so every surface is empty, and the inbox's empty state
   tells him his work is all done when it is in `seatloom` with 7 workitems and all three pending cards.
   **This is the fourth independent reason and it sits underneath the other three** — it explains the empty
   list in reason 2 of finding 2, which I had been reporting as its own problem.

## Why this is the right move and not a workaround

The alternative was to keep writing findings as documents. **I have done that five times this week and
measured, this round, that it delivers nothing.** A finding nobody can read is not a finding; it is a note
to myself with a commit hash.

My standing instructions authorise it explicitly — *"As PO you may also break work down and
`enqueue_dispatch` sub-tasks to other seats"* — and the tool lists Lyra as an issuer. I used that authority
twice earlier today for the UI work, both times with a verified result. **This is the third, and it is the
one that carries the most substance.**

## The lesson, stated against myself

For several rounds I reported "this is Mira's surface, so this is an ask and not an edit" and stopped
there. **That was true and it was also an excuse.** An ask that cannot be read is not an ask. The correct
form of "this is her surface" is not "write a document she cannot see" — it is "put the finding where she
will actually encounter it," which for a governed seat is a dispatch.

**I knew this on 10-30 when I documented that a ruling not carried by a dispatch is not delivered. I then
wrote four more findings as documents.** The knowledge did not change the behaviour, which is the same
thing I recorded about myself twice already this week: knowing a failure mode does not immunize against it.
What changed it this round was checking the delivery surface *of my own output* before calling it delivered.

## What is still open

The four findings are now in front of the seat who can fix them, and the first two are the ones that
unblock the operator. **Nothing else has changed:** he has been absent from the product for 17.5 hours,
the landing script is still unrun, and the three cards are still pending. Those are his, and they are
measured, documented, and — for the first time — actually reachable by the seats who would act on them.
