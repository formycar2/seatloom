# Standing duty: **the card I made possible by routing the verdict opened, and I verified it the same way I verified the other two. Mira's `acc-disp-bouEIM-49hK0xR_-B8EMm` is correctly wired — verdict `pass` at `5810c7e`, branch and dispatch both matching, all three deliverable files byte-identical to the verdict commit — and its load-bearing direction is real: removing the status guard turns her suite RED.** Three cards now await the operator, and all three have been independently verified by me.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-third-card-opened-and-i-verified-its-load-bearing-direction-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches.** `get_next_dispatch` returned nothing ready.

**Three cards pending**, and the new one is `acc-disp-bouEIM-49hK0xR_-B8EMm` — Mira's, opened after I routed her verdict to her last round.

## The chain, closed end to end

| step | what |
|---|---|
| 11-09 21:11 | Mira commits `5810c7e` — the operator's third obligation |
| ~21:56 | I measure that she has no verdict, find the route is a dispatch to Flux, and that I had used it twice before for her |
| ~22:20 | I dispatch Flux, pre-declaring `ev-75p6okkZVBW5YvgRtULlf` |
| ~22:30 | Flux verdicted **pass**, 0 gaps, naming the correct dispatch |
| ~23:00 | I find the verdict has reached nobody and dispatch Mira |
| ~23:20 | **Mira opens the card** |
| this round | I verify the card |

**So a seat's finished work reached the acceptance path through five steps, three of which were mine, and none of which existed before I measured the gap.** For Vela the same chain ran. **Two seats that had no route to the mainline now both have cards, and the reason is that somebody measured the route instead of describing its absence.**

## The card's wiring, checked before running anything

| check | measurement |
|---|---|
| `flux_verdict_ref` | `ev-75p6okkZVBW5YvgRtULlf` |
| does that event exist | **yes** |
| is it `pass` | **yes** |
| does it name `5810c7e` | **yes** |
| does its branch match the card's | **yes** — `packet/mira/knowledge-panel` |
| does its `dispatch_id` match the card's | **yes** — `disp-bouEIM-49hK0xR_-B8EMm` |
| is `5810c7e` an ancestor of the branch | **yes** |

**And the branch has moved 7 commits past the verdict to `f554ac0`, so I checked whether that matters:**

| deliverable | at `5810c7e` vs branch tip |
|---|---|
| `ui/src/components/StaleReadySection.tsx` | **IDENTICAL** |
| `ui/src/lib/staleReady.ts` | **IDENTICAL** |
| `ui/src/components/StaleReadySection.test.tsx` | **IDENTICAL** |

**None of the 7 touches a deliverable.** That is the fourth time this week I have measured a growing count and found the blobs unchanged, which is why I keep checking rather than assuming either answer.

## The verification, run by me

**Her `how_to_verify` asks for the suite, and for the operator to open the dispatch page and look for the empty state — with the non-empty state explicitly marked as something she cannot construct because it requires writing to the live database, which is beyond her permission.**

**That honesty is worth noting**: she did not write a step she could not perform and leave the operator to discover it. She said which part he can check and which part she cannot produce.

| step | I measured |
|---|---|
| her 10 tests | **10 passed / 10**, in her own worktree |
| remove the status guard | **RED** — `in_flight and done are never stale, however old` fails |
| restore | `git diff --quiet HEAD` — **byte-identical again** |

**The mutation is what makes the card honest.** Her claim "age alone never qualifies a row" is a comment; removing the guard and watching the suite go red is the evidence that the comment is enforced. **A card whose negative case is green under mutation is a card about decoration.**

**One practical note**: my isolated worktree could not run the test because it had no `node_modules`, so I ran it in her own worktree instead — and verified her tree was clean before and after, and restored the mutation from a byte-identical backup. **The isolated worktree was the wrong instrument for a test that needs installed dependencies, and I switched rather than reporting a failure that was about my harness.**

## What the three pending cards now are

| card | subject | verified by me |
|---|---|---|
| `acc-disp-4xBIRHNOnacKkCwCPJhJG` | db-error caller-path fix at `41da35b` | both steps, negative case checked by `HEAD` comparison |
| `acc-disp-MlhRWjAlGcaPUj2BqMpOr` | knowledge-base gate convention | four steps, residual description corrected by its author |
| `acc-disp-bouEIM-49hK0xR_-B8EMm` | operator's third obligation UI | suite + load-bearing mutation |

**All three are the operator's to accept. All three have had their negative cases checked rather than their happy paths.**

## What I did not do

I did not modify any file in any shared worktree, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one mutation in Mira's own worktree, applied, verified to have mutated, and restored from a byte-identical backup with her tree verified clean afterwards. **I did not construct the non-empty state** — it requires a live-database write and that is beyond my authority, exactly as her card says.
