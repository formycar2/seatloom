# Standing duty: **I routed a verdict to the seat it belonged to, rather than leaving it in the ledger. Flux verdicted Mira's `5810c7e` `pass` last round and named her dispatch; her branch had not moved since 21:11 and no dispatch had told her, so a completed verification was acting on nobody. I verified that only she can open the card, and dispatched her with everything she needs including the two things the plan must state honestly.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-i-routed-a-verdict-to-the-seat-it-belonged-to-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 501 → 503 events and 100 → 101 dispatches**, and the movement is my own dispatch. `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

## The gap, and it was the last link in a chain I had started

**Last round I dispatched Flux to verify Mira's work. He verdicted `pass` and named `disp-bouEIM-49hK0xR_-B8EMm` correctly.**

**What I did not check was whether that reached her.**

| check | measurement |
|---|---|
| any dispatch to Mira telling her | **none** — her last dispatch is `disp-bouEIM-49hK0xR_-B8EMm`, `done` |
| her branch tip | **`5810c7e`, unchanged since 21:11** |
| her worktree | **clean, 0 changes** |
| a card naming her dispatch | **none** |

**So a complete, passing, zero-gap verification existed in the ledger and the seat whose work it verified had no way to know.** The chain was: I found she had no verdict → I dispatched Flux → Flux verdicted → and then nothing, because a verdict is a record and not a notification.

**And this is the same failure I recorded about my own corrections three rounds ago: a finding that reaches nobody acts on nobody.** I wrote that sentence about my own arithmetic corrections and then produced the identical gap one round later.

## Why I dispatched rather than opening it

**I verified the guard before deciding:**

| check | source |
|---|---|
| does `request_acceptance` require the caller to own the dispatch | **yes** — `WHERE id = $1 AND target_seat_id = $2 AND status = 'done'` |
| does the verdict ref have to exist and match | **yes** — `flux_verdict_facts(&args.flux_verdict_ref, &args.dispatch_id)` |
| is the card id derived from the dispatch | **yes** — both her prior cards are `acc-<dispatch_id>` |

**So `disp-bouEIM-49hK0xR_-B8EMm` targets `seat-mira-001` and only she can open the card. Nor should I be able to** — a card is its author's statement about its own work, and a third party writing it would make the statement unattributable. **That is the same boundary I held when I declined to fix Vela's card number myself, and it produced a better result there too.**

## What I put in the dispatch

**Everything she needs to act without re-deriving anything, and the two things that would make a card misleading if omitted:**

| content | why |
|---|---|
| the verdict id, commit, branch, gap count, and the dispatch it names | so she does not have to search |
| the three mutations and their outcomes | the status guard RED and the threshold RED are what make it load-bearing; the NaN guard staying green is the interesting one |
| the framing ruling and its structural reason | a ready dispatch is some seat's work, there is no operator-side claim entry point, and the waker is what should move it — **that belongs in the plan because it is why the section is not a to-do list for him** |
| the referent instruction | copy `disp-bouEIM-49hK0xR_-B8EMm` from the ledger, not typed |
| **the three-place threshold coupling** | `notify-operator.sh`, `work-list.sh`'s `STALL_MINUTES`, and `STALE_AFTER_MINUTES` — only the middle is configurable. Flux correctly did not call it a defect in her change, **but the plan should say it so the operator is not surprised later** |
| **what the change does not do** | it does not start a waker and does not decide whether one should run |

**The last two are the "an operator who accepts on a false premise is worse than one who rejects" requirement, applied to a card I did not write.** I have no stake in her card, which is exactly why I can see what it should not omit.

## The dispatch

| field | value |
|---|---|
| dispatch | `disp-vDUTp_4rWlPjqD8TwRoAy` |
| status | **in_flight** — she has picked it up |
| target | `seat-mira-001` |

**Scope: open the card. No merge, no install, no edit to her component, and no invented dispatch id.**

## What I did not do

I did not modify any file, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict and the absence of any dispatch to Mira, `git log`/`git status` over her branch, and `grep` over `dispatch.rs` for the ownership guard, the verdict-facts check, and the card-id derivation. **The one write I made was the dispatch, which is inside my authority.**
