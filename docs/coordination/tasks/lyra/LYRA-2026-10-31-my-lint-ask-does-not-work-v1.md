# Standing-duty self-correction: the lint instrument I recommended last round does not catch the mutation I recommended it for, and I verified that before recommending it — after

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-my-lint-ask-does-not-work-v1 |
| status | retracted |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, self-correction, instrument, mutation, retraction |

## What I said last round, and why it was wrong

Last round I reported a surviving mutation — **re-inserting the dead `LIVE_TABS` branch is undetected** — and
I closed it with an ask:

> "Configure `allowUnreachableCode: false` in `tsconfig.json` and/or enable `no-unreachable` in the ESLint
> config. This is the instrument that catches the surviving mutation."

**I did not test that claim before I made it.** I verified the *preconditions* (neither flag is set) and then
asserted the *conclusion* (the flags would catch the mutation). Those are different measurements, and I made
the second one from memory of what linters do rather than from running one.

**I have now run it. The instrument does not work.**

## The measurements, in a scratch directory with the project's real toolchain

| Case | Code shape | `allowUnreachableCode: false` | `no-unreachable` |
|---|---|---|---|
| 1 | `if (t === ('x' as T))` — a statically-decidable dead gate | exit 0 — **not caught** | — |
| 2 | `return 1; const y = 2; return y;` — control-flow dead code | **TS7027 caught** | **caught** |
| 3 | `if (S.has(tab)) return 'live';` — **the actual mutation shape** | exit 0 — **not caught** | exit 0 — **not caught** |

**Case 3 is the one that matters, and it is the one neither instrument sees.**

## Why, and this is the real finding

**Both instruments work exactly as designed, on code whose reachability is decidable from the syntax.** Case
2 is caught by both, which proves the instruments are live and correctly configured in my test — so this is
not a broken setup.

**But the mutation I asked them to catch is not a reachability question at all.** The branch

```ts
if (LIVE_TABS.has(tab)) return 'live';
```

is unreachable *because of what the surrounding code does at runtime* — `LIVE_TABS` is now derived from
`LIVE_TAB_BACKING`, and every member of it is caught by an earlier branch. **No static analyser can know
that.** It requires reasoning across three branches and a derived constant's runtime contents, which is
precisely the property I said a test could not express.

**So my ask was the wrong shape.** I diagnosed "this is not testable" and then prescribed a linter as if
linters could reach what tests cannot. **They reach less.** A linter sees strictly the local syntax; a test
at least sees behaviour. I substituted a second, weaker instrument for the first and called it a fix.

## What I am retracting, and what I am not

**Retracted:** the ask to configure `allowUnreachableCode` / `no-unreachable` as a way to catch the surviving
mutation. **It would not catch it.**

**Not retracted:** the measurement itself. Re-inserting the dead branch is genuinely undetected by any
instrument currently in this project. That remains true and I am not softening it.

**Not retracted, and newly stated:** the two flags are still worth having, for case 2. `allowUnreachableCode`
and `no-unreachable` both catch `return 1; const y = 2; return y;`, and **this codebase currently has no
instrument for that class at all.** That is a real, separate, smaller finding — and it is the honest version
of what I tried to claim last round.

## The pattern, and it is the eighth time

**I verified the precondition and asserted the conclusion.** Both flags absent — measured. Both flags
effective — assumed. The second is the one that carries the ask, and it is the one I did not run.

This is the same family as everything else this stretch: a conclusion drawn from an instrument not first
verified to measure the thing the claim was about. Here the instrument was *my own memory of linter
semantics*, standing in for a linter. **The corrective is not "be more careful"; it is that an ask is a
claim, and a claim gets measured like any other.**

## What this means for the actual defect

The dead-code direction is **not enforceable** by test, by `tsc`, or by ESLint as currently understood. The
only instruments that could catch it are semantic ones — a type-level encoding where the derived set and the
branch cannot both exist, or a coverage tool reporting an unreachable line. **I am not going to prescribe
either without measuring them first**, which is the whole lesson of this document.

**So the honest status of that mutation is: open, with no known instrument in this project that catches it,
and the burden of finding one has not been met by me.**

## What I did not do

I did not edit `tsconfig.json` or `eslint.config.js`. All measurements ran in `/tmp` against a scratch file
using the project's own `tsc` and `eslint` binaries, with the project's real flat config plus one added rule
to isolate the effect. I did not grant any card.
