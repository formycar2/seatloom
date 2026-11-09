# Standing duty: **I found the same structural gap for Mira that I found for Vela, and I closed it the same way. All three verdicts naming Mira dispatches were for dispatches *I* issued — so the route to the mainline is a dispatch to Flux, and her newest work simply had nobody who dispatched a verifier for it. I verified her work independently and dispatched Flux, and I asked for a ruling on the one judgment in it that is not about code.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-route-was-open-for-mira-and-i-used-it-again-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 497 → 498 events and 99 → 100 dispatches**, and the movement is my own dispatch. `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

## The gap, found by measuring rather than assuming

**Last round I verified Mira's `StaleReadySection` at `5810c7e` and noted she has no verdict naming her dispatch. This round I measured what that actually means:**

| measurement | result |
|---|---|
| verdicts naming a Mira dispatch | **3** |
| who issued those dispatches | **`seat-lyra-001` — me, both of them** |
| which ones | `disp-XpAdx0a9YLL83JVoPesTj`, `disp-Ra5YbN4t78bzdbYhYkQiV` |
| did they reach accepted cards | **yes, both** |
| who issued her newest dispatch | `seat-aegis-001` |
| verdicts naming her newest | **0** |
| has anyone dispatched Flux for `5810c7e` | **no** |

**So this is the third instance of the same finding, and the second time it was mine to close.** For Vela the route existed and I had never used it. For Mira the route existed, I had used it twice before, and her newest work simply had an issuer who dispatched the work rather than a verifier for it.

**Aegis issuing a work dispatch is not a defect** — he was routing the work to the seat that could do it, correctly. **The gap is that nothing in the process dispatches a verifier for a seat's finished work unless somebody thinks to.** And the seats who can think to are the ones who already know the route, which is why it keeps being me.

## What I verified before dispatching, so the premise holds

**I have been burned by dispatches whose premise had moved, so I checked first:**

| check | measurement |
|---|---|
| is `5810c7e` her branch tip | **yes** |
| is her worktree clean | **yes — 0 changes** |
| does her test pass | **yes — 10/10, which I ran myself** |
| does it merge into track | **yes — `merge-tree --write-tree` exit 0, no conflicts** |
| has anyone already dispatched Flux for it | **no** |

## What I asked for, and why one item is not about code

**The dispatch asks Flux to verify the change, and additionally to rule on the judgment at its centre:**

> rendering a stale dispatch as work waiting for him would make his number wrong, because the waker stopped and the dispatches cannot be pushed, which is a system problem and not the operator's task.

**I verified the premise** — `notify-operator.sh` line 168 frames it exactly that way with the same threshold, and there are zero `ready` dispatches live, so the empty state is what a user actually sees.

**But whether that framing is *correct* is a different question from whether it is *faithful to the existing instrument*.** Mira copied it deliberately and said so. **A verifier who only checks the wiring would certify that the code matches the waker's framing without ever asking whether the waker's framing is right about the operator's workload.** So I asked for that explicitly, and named it as a claim about his workload rather than about the code.

**And I flagged the referent ambiguity rather than resolving it silently.** Her dispatch `disp-bouEIM-49hK0xR_-B8EMm` is already `done`; the card will need a `dispatch_id`, and I told Flux to copy it from the ledger and to say so if a fresh verification dispatch would be cleaner.

## The dispatch

| field | value |
|---|---|
| dispatch | `disp-Ztfe1-k78oc_wHlxJa7Ku` |
| status | **ready** |
| target | `seat-flux-001` |
| pre-declared verdict id | `ev-75p6okkZVBW5YvgRtULlf` |
| subject | Mira's third-obligation UI at `5810c7e` |

**Scope stated: no card, no merge, no install, no edit to her component, no change to the threshold, and no starting a waker.** Whether the waker should be running is a separate question and I said so.

## What I did not do

I did not modify any file, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict population and issuers, `git log`/`git status`/`git merge-tree` over her branch, `npx vitest run` on her test file, and `psql` for the live `ready` count. **The one write I made was the dispatch, which is inside my authority.**
