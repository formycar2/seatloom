# Standing-duty verification: the UI card is open and correct, and the third card discloses its own unverifiability

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-ui-card-open-and-correct-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, verification, acceptance, card |

## The card I asked for exists, and it is right

`acc-disp-faEWBt8kSGgv-6pJs_Rqw`, **pending**, opened on `disp-faEWBt8kSKgv-6pJs_Rqw` with verdict
`ev-Dq2vTW8WjIU_hQufDDp2F`. Every guard condition verified independently:

```
verdict value            : pass
payload.dispatch_id      : disp-faEWBt8kSKgv-6pJs_Rqw   <- names it, primary channel
event_object_refs        : subject|dispatch disp-faEWBt8kSKgv-6pJs_Rqw  <- names it, second channel
dispatch status          : done, owned by seat-mira-001
existing pending acceptance : none before this
```

**And the `how_to_verify` is the thing I specifically warned her about, and she got it right.** It opens by
telling the operator this is not a small change:

> "This is not just one test — it carries the whole 'persist from operator input' ruling onto the branch.
> Look at this scope when you accept it, not at the dispatch title."

Then three verification steps in the running app — quit and reopen (must land on Dispatch), switch projects
(that project's own remembered tab), switch back (**that tab must still be there**, and it names step 3 as
the key one, because switching away and back is where the memory must not be lost). Then what the change
actually did, at the level of the ruling: the write effect deleted whole, one `chooseTab` wrapper, `:489`
still not persisting.

**She was told the dispatch title would mislead him if it were all the card said, and she wrote the card so
that it does not.** That is the whole job.

## The second card, and it closes a debt from 10-29

`acc-disp-E06O6IAaWefF-BOW-CNJd` — the card I measured on 2026-10-29 as **opened on a dispatch with no
verdict naming it**, and therefore un-openable. It now exists, with verdict
`ev-JPEis2MT47OXUnHW1yE5b`, `pass`, naming it in both channels. Its `how_to_verify` is the first-screen
behaviour: cold start must land on Dispatch, not on the page you were last on. **That is the ruling I
issued, and the card now matches it.** Whatever was blocking it has cleared.

## The third card is the interesting one

`acc-disp-TbGRGr5cvpJFgCeaayu7S` — topics can be closed. Its verdict `ev-U9LHdjblV_bkLtnfjuNzh` is `pass`,
but:

| | |
|---|---|
| verdict `payload.dispatch_id` | `disp-QbxkqowgN3bALg_89qY9j` — **the verification dispatch** |
| card's dispatch | `disp-TbGRGr5cvpJFgCeaayu7S` — the implementation dispatch |
| verdict refs | `subject|dispatch` = the verification dispatch; `context|dispatch` = the card's dispatch |
| verdict document | names **zero** dispatches, including the card's |
| `payload.verifying_dispatch` | `disp-QbxkqowgN3bALg_89qY9j` |
| `payload.fix_at` | `57bda95, verified at baccada` on `packet/nimbus/project-scoped-reads` |

**That is the exact shape I ruled on 2026-10-29**: card opened on the implementation dispatch, evidenced by
a verdict about the verification dispatch, whose document never mentions the implementation dispatch. The
guard accepts it through the `context` ref channel, which accepts "mentions" rather than "about."

**And it is still true that the guard is doing what it was built to do.** But this time the outcome is
different from the two cards I flagged yesterday, and the difference is the card's own text:

> "This work cannot be clicked in the current SeatLoom, so I must state plainly: the four steps below are
> not verification you can perform today — they are the three things missing before this feature is truly
> acceptable."

That is: *this work cannot be clicked in the current SeatLoom, so I must state plainly: the four steps below
are not verification you can perform today, they are the three things missing before this feature is truly
acceptable.* It then gives the operator a **refusal condition** — if the feature ships but no operator can
execute it, do not accept; verify when it is actually clickable — and lists the three measured things
needed, including that `close_topic` appears **zero times** in `src-tauri` and `ui`.

**So the operator is not being misled.** The card that satisfies the guard through the weakest channel is
also the card that tells him he cannot verify it. That is the opposite of the two cards I flagged, where
the pointer was wrong and nothing said so.

## What this changes about my standing finding

My structural finding stands and is unaffected: **the refs channel accepts "mentions", so a seat can
satisfy the guard without the verdict being about the dispatch.** That is a real gap and it still needs the
dispatch-format rule.

**But "a card through the weak channel" is not by itself a defect.** Yesterday I read it as one. This
round the same shape produced a card that is *more* honest than the mechanically-clean ones, because the
seat that opened it knew the operator could not verify it and said so in the field where he would look.
**The variable is not which channel the verdict used. It is whether the card's own text tells the operator
what he is actually being asked to accept.** I should have separated those two things yesterday instead of
treating the channel as the finding.

## What is still open

1. **The operator has not yet granted the UI card.** It has been pending for about 18 minutes. He granted
   two cards this morning within minutes of them appearing, so this is normal, not a signal.
2. **M-4** — `chooseTab` is the only writer of `selectedObjectId` and nothing asserts those fields. Real,
   correctly classified as not blocking, still unclosed.
3. **Real-machine verification** — still owed, still never done.
4. **`packet/lyra/standing-duty-docs-only`** — still not on track. Verified again this round: 90 heartbeat
   nudges in the waker log, **all Lyra, zero for the other five.**

## What I did not do

I did not grant, cancel, or re-open any card — that is the operator's action. I did not edit the third
card's verdict or its dispatch. I did not merge anything.
