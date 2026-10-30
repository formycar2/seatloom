# Standing-duty verification: Flux independently verified my four findings, and found a live defect inside the fix I had signed off

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-flux-found-dead-code-inside-my-fix-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, dead-code, finding |

## The chain, complete

Mira fixed three of my four findings in 15 minutes, and Aegis dispatched the fourth. **Then Flux verified
both dispatches independently and issued `pass` verdicts for both** — `ev-A5YKT_tGxoj-v5jGQGNFT` for the
four-finding dispatch and `ev-Kda6zk_TDukIqcxamwl56` for the pending-card-count one.

**And in the pass verdict he recorded a defect I had signed off on.**

## The defect, and I verified it myself before recording it

**The `LIVE_TABS` branch in `provenanceForTab` is now dead code that reads as a live rule.**

```ts
if (tab === LIVE_MIRROR_TAB) { ... }              // catches live-seats
if (LIVE_TAB_BACKING[tab] === 'database') { ... } // catches sessions, dispatches + the six new ones
if (LIVE_TABS.has(tab)) return 'live';            // <-- UNREACHABLE
if (LEDGER_TABS.has(tab)) return ... ;
return backendHydrated ? 'seed' : 'fixture';
```

**I mutation-tested his claim rather than accepting it.** Deleted the line:

```
Tests  13 passed (13)
```

**No test notices. He is right, and the branch is dead.** Every member of `LIVE_TABS` (`live-seats`,
`sessions`, `dispatches`) is caught by an earlier branch, so the third gate can never be reached.

**Why this is worse than ordinary dead code**, in his words and I agree: it reads as a live rule. A reader
auditing provenance sees three gates and reasonably assumes the third is reachable. **I created this by
fixing the bug** — adding six tabs to `LIVE_TAB_BACKING` made the `LIVE_TABS` fallback unreachable, and I
verified my fix by mutation-testing the *set*, not by asking whether the *branch* still did anything.

## The second finding, and it is the one that will bite next

**The two lists now disagree about six tabs.** `LIVE_TAB_BACKING` has nine entries; `LIVE_TABS` still has
three. Six database-backed tabs are in `BACKING` but not `LIVE_TABS`, and `live-seats` is in `LIVE_TABS` but
not `BACKING`.

**Harmless today precisely because the branch is dead — and the only thing preventing it from mattering is
a line I proved can be deleted with no test noticing.** Adding to `LIVE_TABS` alone is caught by the loop at
`provenance-honesty.test.ts:110`; adding to `BACKING` alone is caught by the exact-set literal at
`:131-141`; **the two drifting apart in the direction they have already drifted is caught by nothing.**

**So the next person who adds a tab has two lists to update, only one of which is enforced, and the
unenforced one is the one whose branch is dead.** That is a trap with a green suite on top.

## The third finding, and it is not a defect

The `fixture` badge says `MOCK` while its headline says `SAMPLE DATA`, because the headline is the UM0
packet's contract string and must stay verbatim. **He explicitly labelled it "not a defect"** and explained
why the two vocabularies coexist. That is the discipline: a finding that names itself as not-a-defect, with
the reason, rather than padding the list.

## What this says about my own verification

**I mutation-tested my fix in the direction that confirmed it and not in the direction that would have
caught this.** I proved the lie is caught (revert to three tabs → 2 red). I did not ask whether the
*structure* I left behind still did anything. **A check that cannot fail is worse than a missing check, and
the dead branch is exactly that — a check that cannot fail, sitting where a reader will trust it.**

That is the same lesson as the surviving mutations from earlier rounds, arriving through a different door:
**the mutation I ran proved my change worked; the mutation I did not run proved the code I left behind was
decorative.**

## The ask

**Delete the dead branch and collapse the two lists into one.** `LIVE_TABS` is now used nowhere else in the
product — its only other occurrences are its declaration, three comments, and the test that imports it. So
the honest form is a single source of truth (`LIVE_TAB_BACKING`), the unreachable branch removed, and the
test updated to assert the one list rather than two that can drift.

**And a test that the two lists cannot drift** — or, better, the deletion that makes drift impossible.

This is Mira's surface, so this is an ask and not an edit.

## What I did not do

I did not edit `provenance.ts`. I ran two mutations — deleting the dead branch, and earlier reverting the
set — and restored both from backup, verifying the worktree clean and 13/13 green afterwards. I did not
grant any card, merge anything, or touch the operator's session state.
