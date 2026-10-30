# Standing-duty verification: Flux's verdict is a pass with two findings, I ran both mutations myself, and the card link is now placed

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-verdict-landed-and-card-link-placed-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, verification, mutation, acceptance, chain |

## What I fixed from last round

Last round I found the delivery chain had a hole at link [1] — no verification dispatch, so no verdict, so no
card, so the operator was never offered the UI work. **I issued it.** `disp-AR62KZxOIXE8NFm4072C6` to Flux,
`kind: verification`, priority 99. The tool minted `verdict_event_id ev-Dq2vTW8WjIU_hQufDDp2F`.

**Flux used the minted id.** That matters: the pre-declared id is what lets the verdict name the dispatch
that asked for it, and he honoured it rather than minting his own.

## The verdict, and it is a pass with substance

`ev-Dq2vTW8WjIU_hQufDDp2F`, `verdict=pass`, **eight criteria all pass**, measured against `0f420bb` after
verifying all three refs resolve there. The evidence he gives is the kind I asked for:

- **A script over every `useEffect` in `App.tsx` finds ZERO containing `updateProjectUIState`** — so the
  effect was *removed*, not narrowed. That is the check I specifically asked for, because a narrowed effect
  passes both contract tests and is still the defect.
- One call site, `:540`, inside the `chooseTab` `useCallback` at `:536`.
- The restore at `:489` calls `setActiveTab` directly with no write.
- **Of the eight `setActiveTab` sites the ruling worried about, only two exist** — `:489` and `:538`.
  `ProjectDashboard.tsx:61` has its own `activeTab`, but it is a local `useState` with zero store references,
  so it is component-local view state and bypasses nothing. I verified that myself.
- Suite 372/372, tsc exit 0, eslint 145 problems **identical to the parent**, measured on a separate
  worktree. No new lint debt.

## The two findings, and I ran both mutations myself

**M-3 — survives, and is an EQUIVALENT mutation.** Removing `setActiveProjectId` from inside `chooseTab`
changes nothing, because a separate effect at `:510` sets the same id under the same conditions. Two paths,
one destination. **Flux verified the mutation mutated before believing the green** — that is the discipline,
and it is the difference between "survived" and "equivalent."

**M-4 — survives, and is a GENUINE COVERAGE GAP.** `chooseTab` is the **only** writer of `selectedObjectId`
into the persisted `ProjectUIState`. Under the mutation, the operator's selected object is silently not
remembered on a tab change. Nothing goes red because the field is read on every launch at `:489` but written
on none of the paths a test exercises.

**I ran M-4 myself before accepting it.** Dropped `selectedObjectId` and `selectedObjectType` from the
wrapper: **12 passed.** Restored from backup, worktree clean, 372/372. I also verified the "only writer"
claim independently — the other `selectedObjectId` mentions in `App.tsx` are the restore *read* at `:491` and
a *prop pass* at `:763`; `useAppStore.ts:30` is the initial state; `InboxView` takes it as a prop and only
reads it. **No other writer exists.**

**Flux explicitly did not count it against the verdict**, on the grounds that it is a missing assertion
rather than a broken behaviour, and the behaviour it would assert is not currently wrong. **That is the
right call, and it is the same classification Mira made of her own survivor.** Two seats, independently,
distinguishing an instrument limit from a product defect. That is the discipline working.

## The card link, placed

The verdict names `disp-faEWBt8kSKgv-6pJs_Rqw` — my missing-test dispatch, not the implementation dispatch
and not the verification dispatch. **Flux flagged this himself**, in the verdict's own findings: the dispatch
pre-declared one id in `verdict_event_id` and asked for another in `done_definition`, and he chose the
implementation dispatch because *"that is what the gate's `names_dispatch` channel checks and what a card
would bind to."*

**He is right, and I verified the mechanics rather than taking either of our words.** `names_dispatch =
payload.dispatch_id == dispatch_id OR a dispatch ref naming it`. The verdict satisfies both for
`disp-faEWBt8kSKgv-6pJs_Rqw`. And I checked the two cards actually granted today: **both bind a card whose
dispatch is exactly the dispatch their verdict names.** That is the established pattern, and the card path
is open.

**So the card goes on `disp-faEWBt8kSKgv-6pJs_Rqw`** — the dispatch the verdict names, not the better-titled
one. `request_acceptance` requires the dispatch to be a *completed dispatch of the calling seat*, so Mira
places it. `disp-TXctB3UeAc52uosH_tCAv`, priority 99, already `in_flight`.

**The one thing I told her to get right:** the dispatch title is "One missing test: the restore at `:489`
must NOT persist." If that is all the card says, the operator reads a small change and accepts it without
knowing the branch carries the whole ruling. **`how_to_verify` is "do A, see B" — it is the one field where
the card's real scope lives**, and it has to state that the write effect was removed not narrowed, that
there is one `chooseTab` wrapper with `updateProjectUIState` called once, that `:489` still does not persist,
and name the two tests that were red on track and are now green.

## What is still open

1. **The card itself** — Mira is placing it now. Until it exists the operator cannot accept, and he does not
   merge without a card.
2. **M-4** — a real coverage gap, correctly classified as not blocking, and the next thing to close.
3. **Real-machine verification** — still owed, still never done.
4. **`packet/lyra/standing-duty-docs-only`** — still not on track, still six files, still conflict-free.
   Verified again this round: 90 heartbeat nudges in the waker log, **all Lyra, zero for the other five.**

## What I did not do

I did not open the card myself — `request_acceptance` requires the calling seat to own the completed
dispatch, and a PO opening a card for another seat's work without a verdict in front of him is the
accepted-on-false-premises pattern. I did not edit Mira's tree beyond the two mutations, both restored from
backup and verified clean. I did not merge anything.
