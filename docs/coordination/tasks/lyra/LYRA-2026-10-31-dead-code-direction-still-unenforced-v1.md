# Standing-duty verification: the drift direction is now enforced and the dead-code direction is not, and the surviving mutation is the one that would restore the defect she just removed

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-dead-code-direction-still-unenforced-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, finding |

## What I verified

Mira's dead-code fix has landed as `ec80aae`. Her tree is clean, so mutation-testing it was safe by the rule
I adopted last round.

**The dead branch is gone as a live statement.** The one remaining occurrence of
`if (LIVE_TABS.has(tab)) return 'live';` is a comment at `provenance.ts:152`, and the comment preserves the
reasoning — that it became unreachable the moment `LIVE_TAB_BACKING` covered the same tabs, that deleting it
left 23 tests green, and **that a gate which reads as a live rule but can never run is worse than no gate**.
She wrote that last sentence herself, and it is the exact principle Flux articulated and I signed off on.

`LIVE_TABS` is derived at `:113`: `new Set(Object.keys(LIVE_TAB_BACKING))`. Suite: **380 passed across 31
files.**

## The two mutations, and one survived

| Mutation | Result |
|---|---|
| **re-insert the dead branch** above its comment | **14 passed — SURVIVES** |
| add `timeline` to `BACKING` only (the drift direction) | **4 failed — caught** |

## What the surviving mutation means, precisely

**Re-inserting the dead branch is undetected.** This is not the same class as the earlier surviving
mutations, and I want to be exact about why, because the wrong conclusion is available here.

- The earlier survivors were *behavioural*: a real operator-visible path with no test on it. Those were real
  gaps.
- **This one is a code-shape check that cannot be expressed as a test.** The branch is dead *because of what
  the surrounding code does*, so any test that asserts "this branch is absent" is asserting a property of the
  source text, not of behaviour. You cannot write a behavioural test for "there is no unreachable gate here" —
  unreachability is exactly the property that behaviour cannot observe.

**So this mutation is not a gap in her test suite. It is the limit of mutation testing itself**, and I am
recording it as such rather than asking her to chase it. **Asking for a test that catches the re-insertion of
dead code would be asking for a lint rule, not a test** — and there is a real tool for that: TypeScript's
`allowUnreachableCode: false`, or ESLint's `no-unreachable`. **Neither is currently configured.** That is the
honest ask, and it is a different ask from "add a test."

## The drift direction is genuinely closed

Adding `timeline` to `BACKING` alone produces 4 failures. That was the direction I was most worried about —
**the one where two lists could drift apart with nothing watching** — and it is now caught, because there is
only one list and it is the single source of truth. Her construction was correct.

## The standing lesson, updated

I have now run mutations in both directions against this fix twice, on two different commits, and the results
differed:

| Direction | her working tree (mid-work) | her landed commit `ec80aae` |
|---|---|---|
| neutralise the BACKING branch | 3 failed | — |
| add to BACKING only (drift) | 5 failed | 4 failed |
| re-insert the dead branch | — | **survives** |

**The drift direction is enforced in both states. The dead-code direction was never enforceable by test.** I
did not previously run the third mutation, which is why I am reporting it now rather than having reported it
as closed last round.

## The ask

**Configure `allowUnreachableCode: false` in `tsconfig.json` and/or enable `no-unreachable` in the ESLint
config.** This is the instrument that catches the surviving mutation, and it is a one-line change that
catches it for the entire codebase, not just this file. **It is Nimbus's and Onyx's surface, not Mira's** —
so this is a finding for the operator and for them, and I am not editing a tsconfig from a seat that does not
own it.

## What I did not do

I did not edit `provenance.ts`, the tsconfig, or any lint config. I ran two mutations against her clean tree,
restored from backup, and verified by `git status --porcelain` that only my files were touched and the count
is zero, then re-ran the suite to confirm 14/14 baseline. I did not grant any card.
