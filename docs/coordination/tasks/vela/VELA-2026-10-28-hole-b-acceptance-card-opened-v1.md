# VELA-2026-10-28-hole-b-acceptance-card-opened-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-28-hole-b-acceptance-card-opened-v1 |
| status | delivered (card opened, status=pending) |
| author | vela |
| date | 2026-10-28 |
| version | v1 |
| dispatch | disp-dKYfFbORGRmtNcZ9_YGCD |
| depends_on | VELA-2026-10-26-message-count-hole-b-not-mine-v1 |
| tags | acceptance, card, hole-b, message_count, verdict |

Aegis 2026-10-28 15:00: one completed dispatch of mine has a verdict but no card
— open one. One card per dispatch, dispatch ids copied from the ledger, branch =
my packet branch, base = `track/infra-foundation`.

## The card

| field | value |
|---|---|
| acceptance id | `acc-disp-2o_BDY1tb3b92ZQwaYXEC` |
| dispatch id | `disp-2o_BDY1tb3b92ZQwaYXEC` (copied from the ledger, not typed) |
| branch | `packet/vela/washing-pipeline` |
| base | `track/infra-foundation` |
| verdict ref | `ev-verdict-disp-2o_BDY1tb3b92ZQwaYXEC` |
| status | **pending** (criterion met) |

Verified from the ledger before calling the tool: the verdict exists
(`ReviewVerdictIssued`, `verdict: pass`, `payload.dispatch_id:
disp-2o_BDY1tb3b92ZQwaYXEC`, actor `seat:seat-flux-001`, issued 2026-10-28T06:27Z),
and the lineage now shows `AcceptanceRequested` as its last event.

## The verdict id is not the one I pre-declared — worth recording

On 2026-10-27 I issued a verification dispatch to Flux with pre-declared verdict
id **`ev-verdict-vela-message-count-hole-b`**. Flux never used it; it emitted
**`ev-verdict-disp-2o_BDY1tb3b92ZQwaYXEC`** instead. I probed for mine first and
confirmed it does not exist, then used the one that does.

The mechanism is not broken — the tool validates a supplied id and the verifier
*must* emit under exactly that id — but this is the second time a pre-declared id
and the emitted id have diverged, and it means **a gated self-dispatch hung on a
pre-declared id will wait forever** if the verifier names its own. The safe
pattern is to look up the verdict by `payload.dispatch_id`, not by the id you
asked for. Recorded so the next seat does not block on a string it invented.

## What Flux actually verified, and the one thing it added

All five of my claims reproduced against live and against shipped code. The
valuable part is claim 5, which it **strengthened with something I did not have**:

> The shipped proof `crates/seatloom-mcp-emit/proof/history_read_path_proof.sh`
> **FAILS** unless `CORPUS=183 AND INVENTORY>0 AND LEAK>0`. Driving the 89 to 0
> would break the test that exists specifically to catch a database where the
> inventory was lost.

So the sentinel is not merely documented convention — there is an executable
guard that fails if someone "fixes" it. My delivery argued from the column
comment and the grouped-read filter; Flux found the test. That is a stronger form
of the same conclusion and I have cited it rather than restating mine.

It also corrected the dispatch's premise, independently of me: the summary said
"NULL indicates the corpus migration was not applied". The first half is wrong —
NULL means no extracted conversation exists, and 021 *is* applied (the corpus rows
carry counts summing to 36680).

And a second reason to escalate rather than write, beyond my red line: all 89 rows
have non-NULL `lines` and `bytes`, so a plausible proxy number exists for every
one — which is exactly what makes fabrication dangerous, since it would be
indistinguishable from real data to a reader who does not know the column's
definition.

## The race, fifth occurrence, and why the card nearly failed

The branch-contains-base guard reads the local `track` tip **at request time**.
Between my last push and this call, track advanced two more commits (`f52aa46`,
`dd7ba34`) — I measured **2 behind**, and the guard would have refused.

So I synced first: scratch-worktree dry-run (automatic merge, zero unmerged
paths), `--no-ff` merge (`e31df00`), push, then **re-measured after the push** —
0 behind, is-ancestor SUCCESS. Only then did I open the card.

This is the fifth sync in one day and the second one caused by the race rather
than by staleness. The control that actually closes it is the one Aegis added as
a pre-commit hook; it cannot cover the window between merge and push, which is
where this one bit.

## What the operator is being asked to confirm

Plain-language plan, shape v1, written for a read-only investigation rather than
a code change — four checks, all of them things visible in the app:

1. Project list shows **38 projects / 183 sessions**, not 272 and not 261.
2. `sf-code/storage` shows **2 sessions** (Flux's original 58-vs-2 probe).
3. Any session's message count matches the `message count` header (the fourth line of each by-session .md's fenced block, after harness / home path / time range) in its
   `by-session/*.md`, summing to 36680.
4. A session with no extracted conversation does **not** appear as a "0 messages"
   session — showing 0 would be worse than showing nothing, because it looks
   counted.

All four test the same distinction from different angles: the 89 NULL rows mean
"never extracted", not "number missing".

## What I did not do

Did not merge anything into track — that is the operator's Accept. Did not open a
card for a unit whose verdict does not exist (Aegis's note about
`disp-tVzSMjxpWpBQrW3dToQej`, still under Mira's verification, was correctly
excluded). Did not invent or reuse a verdict ref. Did not touch the live
database.
