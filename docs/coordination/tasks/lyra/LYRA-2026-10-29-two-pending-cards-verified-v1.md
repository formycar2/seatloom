# Standing-duty verification: both pending cards are sound, and the caveat on the first is real and already resolved

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-two-pending-cards-verified-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, verification, acceptance, closure, correction |

## Why this document exists

Last round I reported that two new pending cards repeat the verdict-pairing defect I ruled on — a card
opened on an implementation dispatch, evidenced by a verdict about the verification dispatch, whose
document never mentions the implementation dispatch. I asked Aegis to tell the operator before he
clicked them.

Aegis replied at 15:07 with something I had not measured, and it changes the conclusion. **I am
recording the correction rather than leaving my earlier finding standing as the last word.**

## What Aegis found that I did not

**Card 1's verdict was issued nine hours before the fix it names landed.**

| | |
|---|---|
| `ev-sitJrfiH1K8-lLl0hmCbP` issued | **2026-10-29 04:08** |
| `e011c0f` — "The clamp is now called, and an unknown is no longer a measured zero" | **2026-10-29 13:16** |

I verified both timestamps myself. Flux's verdict is a **PASS with two named defects** — the primary fix
was unpinned (deleting the clamp left all 213 tests green, including the one named for it, because that
test re-implemented `usize::min` internally while the clamp was inline), and an unknown pane limit was
rendered as `"0 lines"` — a number presented as a measurement that is not one.

**So the verdict was correct when issued and is stale now.** A verdict records the state at the moment it
was issued; it does not update when the work improves. The caveat is real, and the code no longer has
the defect.

## What I verified, rather than taking either of their words

**1. Both defects are genuinely fixed at the branch tip.** `crates/seatloom-core/src/pty/mod.rs:133` is
now a named, public function:

```rust
pub fn clamp_backfill_to_pane(wanted: usize, pane_limit: Option<usize>) -> usize {
    match pane_limit {
        Some(limit) => wanted.min(limit),
        None => wanted,
    }
}
```

and it is **called** at `:543`, not inlined. The `None` arm returns `wanted`, not `0`. Both of Flux's
named defects are addressed structurally — which is exactly what he said Nimbus did.

**2. I mutation-tested it myself, and it is caught.** I replaced the body with `let _ = pane_limit;
wanted` — restoring the exact defect — and ran the suite:

```
test pty::tests::an_unknown_limit_is_not_the_same_as_a_limit_of_zero ... FAILED
test pty::tests::the_backfill_never_asks_for_more_than_the_pane_retains ... FAILED
test result: FAILED. 39 passed; 2 failed
```

Restored from backup and re-ran: **215 passed / 0 failed**, and `git status --short` is empty, so
Nimbus's branch is exactly as I found it. **The two tests genuinely pin the clamp now.** That was the
whole point of Flux's first finding, and it is closed.

**3. Card 2 is clean and pinned.** `ev-GcK0xPj526FQyp918Wdlc` records `fix_at 01517af, verified at
baccada`, and there are **zero** commits between that and the branch tip. Flux verified the exact state
the card names.

**4. Card 1's gap is 117 commits, which is the real number.** Flux verified `b6ef3aa`; the branch tip is
`71962fb`. That is not a small drift. **It does not invalidate the card** — the defect Flux named is
fixed at the tip and I proved it by mutation — but it means the verdict is evidence about a state that
no longer exists, which is precisely Aegis's caveat and precisely why he offered a re-verify.

## The correction to my own finding

**My "these cards repeat the verdict-pairing defect" was right about the pointer and wrong about the
conclusion.** The pointer is still wrong in the way I described: the document never mentions the
implementation dispatch. But the reason is now clear and benign — **the verdict is about the verification
unit, which is what a verdict is, and the implementation dispatch is named as a `context` ref because the
verification covered it.** That is the shape my ruling said needs the document read, and reading it
shows the substance is there.

**I should have read the verdict payload before reporting.** I checked `payload.dispatch_id`, the refs,
and the document's mention of the implementation dispatch — and stopped there. The payload carries
`verifying_dispatch`, `fix_at`, `the_honest_summary` and `what_i_did_not_verify`, and those keys would
have told me this was a verification-unit verdict with an explicit pin. **The information that resolves
the ambiguity was in the same row I queried.**

## Aegis's self-correction, which is the more useful half

He nearly reported "the fix was lost" and caught himself: he had run his measurement in
`~/Documents/GitHub/seatloom-worktrees/nimbus`, which is on `packet/nimbus/session-work-readmodel` — a
different branch. The correct worktree is `nimbus-psr`. He records it as the **third** time today he
measured the wrong code, alongside running Mira's tests in the main checkout instead of her branch, and
a false defective-merge alarm.

**I have made the same class of error four times today.** His list and mine are the same failure, and the
correlation is the finding: **every one of them was a measurement taken in a location that was not
verified to be the location the claim was about.** His new protocol entry — *the surviving mutation is
always where nobody thought to test* — is the same shape from the other side.

## Conclusion

**Both cards are sound.** Card 2 outright. Card 1 with a caveat that is real, already resolved in the
code, and correctly disclosed by Aegis before the operator decides. The operator can accept both; if he
would rather not accept on a verdict that names a defect, Aegis has offered a re-verify and that is one
more cycle.

The structural finding from last round still stands and is unaffected: **the refs channel accepts
"mentions", so a seat can satisfy the guard without the verdict being about the dispatch.** That is a
real gap and it still needs the dispatch-format rule. It just was not the explanation for these two
cards.

## What I did not do

I did not accept either card — that is the operator's action. I did not cancel or re-open them. I did not
edit Nimbus's branch; I restored my mutation from backup and verified it clean. I did not dispatch a
seat.
