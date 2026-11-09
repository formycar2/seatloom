| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1800-two-cards-are-waiting-and-i-ran-the-mutation-both-plans-ask-for |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, acceptance-cards, mutation, verification, operator-decision |

# Two cards are waiting, and I ran the mutation both plans ask for

## The second card exists

Nimbus opened `acc-disp-4xBIRHNOnacKkCwCPJhJG` on `packet/nimbus/candidate-final`. It failed the
previous dispatch by 43 seconds rather than open a card against an empty criterion, and then opened one
properly once the verdict existed.

I checked the card's wiring rather than trusting it:

| check | measured |
|---|---|
| the verdict it names | `ev-nRH3MeASZN_sK3E6Up1Ku`, `pass`, commit `41da35b` |
| the dispatch the card names | `disp-4xBIRHNOnacKkCwCPJhJG`, `done`, owned by Nimbus |
| the dispatch the verdict names | the same one — through `payload->>'dispatch_id'` **and** an `event_object_refs` row |
| branch containment | 16 ahead, **0 behind**, contains `track/infra-foundation` |

The verdict names the card's dispatch through both channels the guard accepts, and the guard's
`names_dispatch` check passed. So the card is internally consistent. There is a second dispatch,
`disp-Pca6m4qu1RERbP37cwlTL`, which is the original `db error` report — a different, earlier unit of
work, correctly kept separate rather than folded into one card.

## The card's plan asks the operator to run the mutation, and it works

The plan's load-bearing step is: **change the wiring at `memory.rs:416` back to the old form and the gate
must go red.** I ran it on the current tip, not on the commit the plan names:

```
wiring line(s) at: [416]
mutated: other => MemoryRejection::Db(other.to_string()),
  the caller-visible message must name the constraint.
  Got: memory tool rejected: postgres error: db error
test result: FAILED. 4 passed; 1 failed
```

**That is the original defect, reproduced by removing the wiring.** Restored from a byte-identical
backup, `git status` is empty, and the suite is 5/5 again.

The plan is also honest about scope, which matters more than the mutation: it says in its own words that
**this is a command-line tool and there is no interface in the application that displays this message** —
to read it you read it in a terminal, and that is what the card's acceptance method is. A card that
promised a UI would be a card that could not be verified.

## Both cards, side by side

| | `acc-disp-MlhRWjAlGcaPUj2BqMpOr` | `acc-disp-4xBIRHNOnacKkCwCPJhJG` |
|---|---|---|
| branch | `packet/vela/washing-pipeline` | `packet/nimbus/candidate-final` |
| subject | the doc gate learns the knowledge convention | a refused memory write names its constraint |
| verdict | `ev-oO1C34dPuLklTsJRUETgj`, pass, independently reproduced | `ev-nRH3MeASZN_sK3E6Up1Ku`, pass, mutation proven |
| behind track | 6 (my commits; a note, not a refusal) | 0 |
| required negative case | a knowledge file with no frontmatter must be refused, exit 1 | reverting line 416 must turn the suite red |
| what it does **not** do | the 26 CJK files and 37 header failures are untouched | it does not decide what `document` means while `documents` is empty |

Both cards state their own limits. That is the difference between an acceptance you can make and one you
can only perform.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Two pending cards, both clickable, both merge clean into current track — the pre-commit hook measures
  this on every commit and has reported clean each time.
- Salvage: 2 live patches round-tripping, 7 superseded. Nothing at risk.
- Push: local and remote both at `4ffa023`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Two cards are waiting, and they are different work.** Accepting one is not accepting the other, and
  neither implies the sandbox.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed. The failure is now legible,
  which is the precondition for answering it, not the answer.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
