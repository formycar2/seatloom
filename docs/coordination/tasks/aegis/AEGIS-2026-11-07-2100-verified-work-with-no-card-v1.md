| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-07-2100-verified-work-with-no-card |
| status | delivered |
| author | aegis |
| date | 2026-11-07 |
| version | v1 |
| tags | aegis, acceptance, backlog |

# Two verified deliverables have never been handed to you, because they have no card

## What the query found

Protocol step 2c asks for work that is verified green but was never offered for
acceptance. I ran it and got ten results. Seven are the sandbox and identity subjects that
are deliberately held back — some are blocked on the destroyed sandbox, and none of them
are ready. **Two are finished, verified, and cannot be integrated at all** because nothing
has ever been put in front of you:

| dispatch | what it delivered | verdict |
|---|---|---|
| `disp-GWV4D2pEBdr8cLzQhteTO` | `scripts/work-list.sh` — one command listing your actionable obligations from the three existing tables | `ev-HgnCjHzmOXMc75NwJkMQ0` pass |
| `disp-JLsCEk0-uugW-fJBIGyux` | `--dry-run` on `scripts/prepare-verification-build.sh` — prints the exact branch, commit, dirty state, target path, and what it *would* affect, without building or touching anything | `ev-OWnf_Qn34JRWWElo_Eb1Q` pass |

Both are independently verified. Neither has a card. **A card is the only path to
integration**, so this work has been complete and unlandable since it was verified.

I could not open the cards myself: the acceptance tool requires the dispatch to belong to
the seat making the request, and these belong to Nimbus. Dispatched
`disp-OupuAHH6tLwlzqXnswXV8` at priority 97 to have the owner do it, with explicit
instructions that each gets its own card and its own plan.

## The thing I insisted on in that dispatch

Both deliveries are **command-line tools**. The screens in the application do not show this
information, and no screen currently does. I required the acceptance plans to say that
plainly, so that if you accept, you are accepting a command you run — not a UI you look at.

This matters because the obvious failure mode is a card whose plan reads "you will see the
work list in the app", you accept it, and then find no such screen. That would be a false
promise, and it is exactly the shape of several earlier problems here.

## Not done

- No card opened by me. No merge, no push, no install, no restart, no live-database write.
- The seven held-back subjects stay held back; several are blocked on the sandbox, which
  still needs your authorisation.
- UI integration of these diagnostics remains undelivered, and I am not claiming it.
