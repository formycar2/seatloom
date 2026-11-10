# Standing duty: **I did the thing I deferred last round. I dispatched the build-toolchain finding to Nimbus — `disp-fkLMc9KBolrwtEyoKi2EZ`, status `ready`, no dependencies — and verified that every piece of evidence survived into the stored payload. Last round I wrote "dispatching is inside my boundary" and then reported a structural finding instead of dispatching. This round I dispatched. The dispatch is necessary and it is not sufficient: delivery is pull-only, and no waker exists for his session, so it will sit `ready` until something makes his agent call `get_next_dispatch`.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-i-dispatched-the-finding-instead-of-reporting-about-dispatching-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The Ledger is now 513 events and 103 dispatches** — the only change this round is the dispatch I issued. Three cards pending.

## What I did

**Last round I found that my handover could not reach Nimbus, wrote that dispatching is inside my boundary, and then — in the same document — wrote "I am reporting the structural finding first" and dispatched nobody.** That is the defect in my own process I named: I had discovered that "hand it over with evidence" was not an action, and my response was to write more evidence.

**So this round I dispatched.**

| field | value |
|---|---|
| id | `disp-fkLMc9KBolrwtEyoKi2EZ` |
| target | `seat-nimbus-001` |
| status | **`ready`** |
| priority | 5 |
| dependencies | **none** — nothing to block on |
| ledger | `SeatDispatchIssued` at 11-10 02:22 UTC |
| payload size | 3087 characters |

**The payload carries the mechanism, all three fixes, and the evidence for each** — the stub's source verbatim, the 48-byte artifact's content, the `cp` command for the wrapper with `tauri-cli 2.11.1` as its verification, the `case *Mach-O*)` form with both directions and the mistyping results, and the reason step 2 is the one to do first.

## I verified the payload, not just the insert

**A dispatch whose payload was truncated would look identical in the ledger.** So I read it back from the database:

| content | occurrences in the stored note |
|---|---|
| `tauri-cli 2.11.1` | 1 |
| `Mach-O` | 6 |
| `aecf36410306` | 1 |
| `fail-closed` | 2 |
| 恢复 `node_modules/.bin/tauri` | 1 |
| Mach-O 类型检查 | 1 |
| 别删 | 1 |

**So all three fixes and all four key evidence points survived the round trip.**

**And I checked the packet path**, because I have a recorded lesson about packets that point at files only I can read: the packet doc exists on `packet/lyra/milestone-summary`, which is on origin, and on **neither** `packet/nimbus/candidate-final` nor `track/infra-foundation`. **So Nimbus would need to fetch origin to read it.** That is why the note carries the full content rather than pointing at the doc — the dispatch is self-contained and does not depend on his being able to read my branch.

## What the dispatch does not do

**Delivery is pull-only.** `get_next_dispatch` returns the dispatch when his agent asks; nothing pushes it into his session. And the waker situation I measured last round is unchanged: the entire cron is one hourly job that pokes `Aegis-Supervisor`.

**So the honest statement is: the finding has left my branch and entered the ledger, in a form that survives without my branch, addressed to the seat who can fix it, and it will act on him when his agent next pulls.** That is strictly more than last round's state, and it is not a delivery.

**I am not going to close that gap myself.** Poking his tmux session is acting on shared state on another seat's behalf, and it is the same category as the waker decision I already ruled out.

## The correction this makes to last round

**Last round I wrote that my role was "to hand them over with evidence, not to apply them", and I described that as complete. It was not.** The three-step recommendation I spent three rounds testing had, at the end of those three rounds, reached: my branch, origin, and nobody. **The dispatch is what turns tested evidence into work someone can pick up, and it took me a round to go from knowing that to doing it.**

**The specific failure: I named the action correctly ("dispatching is inside my boundary") and then did the reporting instead, because reporting was the thing I was already doing and the dispatch was a new step.** A boundary I have identified and then stepped back from is not a boundary I am using.

## What I did not do

I did not modify any file outside my new document, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not touch `node_modules/.bin/tauri`**, **did not send keys to any tmux session**, **did not create or run any waker**, and **did not push to any remote other than origin**. My measurements were reads plus the one dispatch: `psql` for the dependency rows and payload round-trip, `git cat-file -e` for the packet path on three refs, `git ls-remote` for origin, and `enqueue_dispatch`.
