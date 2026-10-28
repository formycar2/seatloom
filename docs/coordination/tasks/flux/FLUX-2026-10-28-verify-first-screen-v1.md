# Flux verdict — the first screen, adversarially verified

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-28-verify-first-screen-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | verification, first-screen, adversarial, mutation-testing, negative-control, seat-flux |

Dispatch `disp-first-screen-verify`.
Contract `docs/coordination/tasks/lyra/LYRA-2026-10-16-first-screen-contract-v1` §6.2 slice 4
(on `packet/lyra/milestone-summary` @ `8be9f41`, not in this tree).
Verdict event: **`ev-verdict-first-screen`**, emitted with exactly that id.

## Verdict: PASS, narrow, with two findings

The slices under test are Mira's one-line default-tab change (`App.tsx:133` = `'dispatches'`)
and the dependency-column annotation, rendered by Mira's annotation slice. Verified at the
merged track HEAD in a detached worktree.

## The one thing above all: distinguishable, and that is the contract's only job

The dispatch said to verify this above everything else, so I built it as its own case with the
**same referent in both states** — the only shape that can actually prove the distinction,
because two different referents would produce different text for the wrong reason.

**INPUT:** `dep_value = 'ev-x'`, rendered twice — once with that event inside the newest-500
window, once with it absent.

```
landed : "ALREADY IN THE LEDGER: [document] produced an artifact — (which one not recorded)"
unread : "HAVE NOT READ WHAT IT WAITS FOR: this one is not among the newest 500 ledger rows.
          NOTE — this is not a claim that it does not exist; this screen simply cannot read that
          far; do not take 'not found' as 'not there'."
```

Two different sentences. The landed one does not contain the unread wording and the unread one
does not contain the landed wording. So **the operator can tell a live blockage from a stale
one**, which is the whole job.

The part I want on the record is *why* the unread wording is built the way it is. `api.listEvents`
is bounded — `ORDER BY occurred_at DESC LIMIT $1` — so it is a **positive detector, not a
negative one**. Absence from the window is absence of evidence. A view that said "does not
exist" there would be the lie shape this project keeps paying for, and a view that said only
"cannot tell" without saying *which direction* it cannot tell in invites the reader to supply
the wrong one. So the sentence says both, explicitly. That is the right call and it is the
reason the case passes rather than merely renders.

## Every case the dispatch named, with exact input and exact output

| # | case | input | exact output | |
|---|---|---|---|---|
| 1 | blocked on a referent that **exists** | 1 blocked dispatch, `deps=[{depKind:'event', depValue:'ev-first-screen-default-tab-landed'}]`, that id in the window | `ALREADY IN THE LEDGER: [document] produced an artifact — (which one not recorded)`; row pill reads `blocked` | ✅ |
| 2 | blocked on a referent that does **not** exist | same shape, `depValue:'ev-this-never-happened'` absent | `HAVE NOT READ WHAT IT WAITS FOR: this one is not among the newest 500 ledger rows. NOTE — this is not a claim that it does not exist…` — **no claim of non-existence anywhere** | ✅ |
| 3 | **the same referent in both states** | `'ev-x'` once in-window, once not | two different sentences, asserted `not.toBe` | ✅ |
| 4 | a dispatch with **no edges** | `deps=[]` | deps cell renders an em-dash placeholder and **no badge**; no sentence at all | ✅ |
| 5 | empty list — **nothing ever dispatched** | `listDispatches → []` | `NO DISPATCHES YET` + `click New Dispatch at the top right to hand a task to a seat` + `0 items total` | ✅ |
| 6 | empty list — **search matched nothing** | search `zzz-no-match` | `NO MATCHING DISPATCHES` + `try another keyword, or clear the search box` + `clear search` | ✅ |
| 7 | empty list — **status filter empty** | the `done` filter over a `ready` row | `NO DISPATCHES IN THIS STATE` + `switch to All to see dispatches in other states` | ✅ |
| 8 | **a read that fails** | `listDispatches` rejects `connection refused: postgres at localhost:5432` | `FAILED TO LOAD DISPATCHES: Error: connection refused: postgres at localhost:5432` — the measured text verbatim | ✅ |
| 9 | **mirror still reachable** | source assertions on `App.tsx` | `'live-seats'` in the Tab union; `activeTab === 'live-seats'` still renders `<LiveSeatsView>`; `setActiveTab('live-seats')` still exists | ✅ |
| 10 | **⌘K still opens** | synthetic keydowns | meta+k, ctrl+k, meta+shift+K each open (3 calls); a **bare `k` adds nothing** | ✅ |

On case 7 there is a subtlety worth stating, because it is where a lazy check would report a
false failure: **all three empty-list cases share the phrase `NO DISPATCHES YET`.** Case 5 reads
`NO DISPATCHES YET — click New Dispatch at the top right…` and case 7 reads
`NO DISPATCHES IN THIS STATE — switch to All…`. Asserting the absence of that phrase across
cases fails. What actually distinguishes them is the **qualifier and the next action** each adds — which is precisely the contract's requirement, so
the assertions are written on the qualifier.

## Mutations, and a negative control on my own probe

| mutation | result |
|---|---|
| the landed sentence replaced with the bare `dep_value` | **RED 2** — case 1 and case 3 |
| the unread sentence's both-directions disclaimer deleted | **RED 1** — case 2 |
| **negative control**: `App.tsx` default reverted to `'live-seats'` | **CHECK A goes RED** — so the default-landing assertion bites rather than being vacuously true |

Both mutations were reverted from a `/tmp` copy with the tree verified byte-identical to the
merged HEAD afterwards. The negative control matters more than it looks: an assertion about
`App.tsx:133` that passes whether or not the line says `'dispatches'` would have let me report
the default-landing as verified while measuring nothing.

## F1 — neither sentence carries an actionable referent id (non-blocking, not fixed)

The landed sentence renders the event **type** in words through the shared `eventToText`
translator (`produced an artifact — (which one not recorded)`) and drops the id. The unread sentence names the window
but not the id either. So a row blocked on a referent the operator wants to chase has to be
located some other way. I asserted this as the *current* behaviour rather than the desired one.

It does not make the two cases indistinguishable — which is what the contract asks — and the
DTO is Mira's fence, so I did not change it. But "which event exactly" is the natural next
question after "has it landed", and today the answer is not on screen.

## F2 — the annotation is window-based, because the measured-existence fact is not on track

This is structural, and it is the same finding Mira and Nimbus both recorded. Nimbus's
`DispatchDependencyDto.referent` (three states: `exists:true` / `false` / `null`, with the
negative control that a failed read is `null` and not `false`) is at `942da99` on
`packet/nimbus/gate-vs-announcement` and is **not on `track` and not in this tree**. I verified
that directly: `src-tauri/src/dto.rs`'s `DispatchDependencyDto` still has exactly three fields,
`dispatch_id` / `dep_kind` / `dep_value`.

So the UI reports the three states it can actually measure — **in-window / not-in-window /
no-edges** — rather than the two the contract's slice 3 asks for — **exists / does-not-exist**.
The consequence is on the record and honest: the view cannot say "does not exist", only "not in
the newest 500".

I want to be precise about why this is a PASS and not a FAIL on the contract's letter. The
dispatch's own one-thing-above-all is that a live blockage and a stale one be
distinguishable, and they are. The narrower slice-3 wording ("must say so, and must not look
like the previous case") is satisfied by "not in the newest 500, and that is not a claim it
does not exist" — which is *more* honest than "does not exist" would be, given what the code
can measure. Claiming a FAIL here would punish the seat for refusing to fake a fact, which is
the behaviour the contract exists to produce. When `942da99` reaches track the change is
additive and the finding closes itself.

## Gates

| check | result |
|---|---|
| `ui` `vitest run` | **27 files / 322 tests** passed — including my 11 probe assertions |
| `ui` `tsc --noEmit` | clean on the product tree |
| probes removed afterwards | tree verified byte-identical to the merged HEAD |

## What I did not verify

- **I did not open the built app.** Every result is from jsdom probes against the real
  component with the real `api` boundary mocked. The operator's GUI acceptance card is the
  click-level check, exactly as the contract says a verdict is not.
- **I did not verify the running binary carries any of this.** The installed app is a dev
  build and predates these changes; that is a rebuild-and-reinstall matter for the operator.
- **I did not verify Nimbus's three-state DTO itself**, only that it is absent from this tree.

## Hygiene

One detached worktree at the merged track HEAD, removed afterwards. Two mutations and one
negative control, each reverted with the tree verified byte-identical. My two probe files were
deleted before removing the worktree, so the product tree carries nothing of mine. No live-DB
writes, no files of another seat modified, no merge of anything into track.

## Files

`packet/flux/intent-verdict`. The verdict event is `ev-verdict-first-screen` in the ledger.
