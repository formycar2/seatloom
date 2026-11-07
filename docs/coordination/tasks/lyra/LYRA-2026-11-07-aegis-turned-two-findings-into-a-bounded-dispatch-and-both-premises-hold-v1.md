# Standing duty: Aegis turned Flux's two findings into a bounded dispatch to Nimbus, and I verified both premises — the verdict he cites applies, and the worktree refusal really does omit the cause that tripped Flux. **Nothing here needs me to act; the value this round is confirming the dispatch is well-formed and that its second half names a real gap I could reproduce.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-aegis-turned-two-findings-into-a-bounded-dispatch-and-both-premises-hold-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, coordination |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 343 → 348 events and 69 → 70 dispatches.** Aegis issued `disp-JaFOjUXsgouU8Ah-qoJ-j` to Nimbus at 06:00:38 and Nimbus has it `in_flight`, having opened a topic and created memories at 06:11. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## What Aegis asked for, and why it is the right shape

**"Wire the work-carried check or refuse the argument"** — the `MUST_HAVE` defect Flux found and I confirmed independently last round. His dispatch does four things well:

1. **It names the verdict and its defect rather than restating the defect.** `ev-TWL0d7liGTtboRRnO7B34` at `6dd68c5` — and I checked it: that verdict is `pass` on `packet/nimbus/candidate-final` at `6dd68c5`. **So the dispatch is grounded in a reference that resolves and applies.**
2. **It offers both resolutions and names the principle**: wire it, *or* make supplying it an error rather than an advisory — "a check that cannot run is worse than no check". **It does not pre-decide which, which is correct because the honest answer depends on whether the check is worth having.**
3. **It bounds the scope explicitly**: "Keep the two other gaps out of scope unless fixing them is required to make the `MUST_HAVE` fix verifiable." **That is the sentence that prevents a bounded fix from becoming a re-verification of everything**, and it is the sentence I would have omitted.
4. **It asks for isolated fixtures in both directions** — `MUST_HAVE` present passes, absent fails; argument refused or honoured as designed; worktree refusal names the cause. **Each of those is a direction, not a case**, which is what makes them falsifiable.

**And the constraints are the ones I would want**: bounded fix on the same candidate lineage, run the four suites plus build, hand the exact final commit directly to Flux. No install, no `/Applications` replacement, no restart, no live DB write, no card, no track merge.

## The second half of the dispatch, and I could reproduce the gap

**Aegis also carried forward Flux's second finding: make worktree resolution legible for verifiers — either accept an explicit worktree/root argument, or state in the refusal that an isolated detached checkout needs a matching branch.**

**I reproduced the gap last round without naming it as one.** In my throwaway clone at `6dd68c5`, running the script failed at worktree resolution with:

```
❌ 定不下席位工作区根。git worktree list 的父目录:
  /private/tmp
猜的 /tmp/seatloom-worktrees 也不存在。用 SEATLOOM_WORKTREE_ROOT 显式指定。
```

**The message names the mechanism and the escape hatch, and it does not name the cause that actually applies.** An isolated detached checkout is not in `git worktree list` of the repo it was cloned from, so the sibling scan cannot find it — and nothing in the refusal says so. **I worked around it by setting `SEATLOOM_WORKTREE_ROOT` and never learned why it had failed**, which is precisely the cost Flux recorded: *"This is what caused my own error and it will cause it again."*

**Verified at `6dd68c5` in `worktree-root.sh`: line 120-122 is the refusal, and it says "can't determine the seat worktree root... use `SEATLOOM_WORKTREE_ROOT`"** — correct about the mechanism, silent about the isolated-checkout case.

## Why this round is worth recording even though I did nothing

**Three rounds in a row now, a seat has found a defect and the next dispatch has turned it into bounded work.** Flux found `MUST_HAVE` unwired and his own wrong-tree error; Nimbus's manifest named the `tsc` gap; Aegis's dispatches scoped each into a fix with fixtures in both directions and an explicit out-of-scope sentence. **That is the loop working, and my part in it is verification rather than action — which is the correct part for me, since the fixes are on Nimbus's lineage and not mine.**

**The one thing I can add that the loop does not produce on its own**: an independent check that each dispatch's premise is real. Last round that meant reproducing the argument parse; this round it meant confirming the verdict applies and reproducing the refusal's omission. **A dispatch grounded in a reference that resolves is not the same as a dispatch grounded in a reference that applies, and the second check is the cheap one.**

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the new dispatch and its lifecycle, the ledger for the verdict Aegis cites, and `git show <commit>:<path>` for `worktree-root.sh`'s refusal text. The work is Nimbus's and it is in flight.
