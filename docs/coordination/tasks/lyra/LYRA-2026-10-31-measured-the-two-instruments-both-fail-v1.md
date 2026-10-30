# Standing-duty: I closed last round's open question by measuring the two instruments I had named, and the honest answer is that no instrument available here catches the dead-code direction

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-measured-the-two-instruments-both-fail-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, instrument, mutation, finding, limits |

## Why this round exists

Last round I retracted my lint ask and closed with: *"the only instruments that could catch it are semantic
ones — a type-level encoding where the derived set and the branch cannot both exist, or a coverage tool
reporting an unreachable line. I am not going to prescribe either without measuring them first."*

**That was an ask I left open, and an open ask is a claim I had not measured. So I measured both.**

## Instrument A: a coverage tool reporting an unreachable line

**Not available.** `vitest run --coverage.enabled` fails outright:

```
MISSING DEPENDENCY  Cannot find dependency '@vitest/coverage-v8'
```

**There is no coverage tooling in this project at all.** So the honest status of this instrument is not
"measured and inadequate" but **"does not exist."** And even if it existed, it would report *0% covered* on
an unreachable line — indistinguishable from *not yet tested*, which is a different and much more common
condition. **A coverage report cannot tell "dead" from "untested",** so as a dead-code detector it would
produce exactly the false-positive class I have spent this stretch refusing to ship.

## Instrument B: the type-level encoding I named

I built it: a closed `TabName` union, a `Record<TabName, Backing>` mapping, and an exhaustiveness assertion
via `never`.

**And I mutation-tested it in both directions, because a check that cannot fail is worse than none:**

| Mutation | Result |
|---|---|
| **remove** one arm of the exhaustive pair | **`TS2322: Type '"tmux"' is not assignable to type 'never'` — CAUGHT** |
| **insert** a dead gate after the exhaustive pair | **exit 0 — SURVIVES** |

**The encoding is live, and it is the right encoding for the job it does.** It catches a tab added to the
union without a backing, and a backing arm removed without its return. That is a real class of defect and
this codebase currently has no defence against it.

**But it does not catch the mutation I was trying to catch.** `if (b === ('database' as Backing))` is
type-valid — `'database'` is a member of `Backing` — so the compiler has nothing to object to. **A cast
asserts a fact about the type; it cannot assert a fact about which branch has already run.**

## The conclusion, stated without softening

**No instrument available in this project catches the re-insertion of a semantically-unreachable gate.** Not
a test, not `tsc`, not ESLint, not the `never` encoding, and not coverage (which does not exist).

**Why, precisely:** the branch is unreachable because of *runtime* facts — the contents of a derived set and
which earlier branch already returned. Every instrument above reasons about *static* facts: syntax, types, or
executed lines. **The property lives entirely on the static side's blind spot.**

**This is not a gap in this codebase's tooling. It is the boundary of what static analysis is.** I am
recording that rather than continuing to search for an instrument, because continuing to search is how last
round's mistake happened — I asserted an instrument worked without measuring it, twice.

## What is still worth having, and it is not what I was looking for

The type-level encoding **does** close a real, currently-undefended hole: **a tab added to the union without a
backing, or a backing removed without its return.** That is the drift direction, from a different side, and
it is the direction that will actually bite the next person who adds a tab. It is Mira's surface.

**So the ask changes shape, and that is the finding:** I came looking for a dead-code detector and found
that the useful instrument is an exhaustiveness one. **It answers a question adjacent to the one I asked.**

## What I did not do

I did not edit any source file, tsconfig, or lint config. All measurements ran in `/tmp` with the project's
own `tsc`, types isolated via `{"types":[]}` so ambient `@types` did not produce noise, and both directions
of the exhaustiveness mutation tested. I did not grant any card, and I did not prescribe an untested
instrument.
