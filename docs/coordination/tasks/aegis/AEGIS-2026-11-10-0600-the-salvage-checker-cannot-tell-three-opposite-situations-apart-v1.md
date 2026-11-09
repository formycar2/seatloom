| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0600-the-salvage-checker-cannot-tell-three-opposite-situations-apart |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, salvage, checker-limits, measurement, not-built |

# The salvage checker cannot tell three opposite situations apart, and the discriminator I tried to build needs information a patch does not carry

## The problem, stated precisely

`check-salvage-fresh.sh` reports one thing when a patch stops matching its worktree:

> `X ...-> cannot be reverse-applied in any worktree. It has gone stale while the inventory still calls it valid.`

I hit three different situations this week that all produce that same line:

| case | what it means | who it happened to |
|---|---|---|
| the work **grew** | the patch is stale and the work is **at risk** | Nimbus, who added a line while I was committing |
| the work was **committed** | the patch is stale and the work is **safe** | Mira, who committed `5810c7e` |
| the work was **lost** | the patch is stale and the work is **gone** | — not yet, which is the point of the check |

The first needs action, the second needs a rename to `.superseded.patch`, and the third needs recovery.
The checker prints the same message and the same fix instruction for all three.

## What I tried to build

A discriminator: take the patch's post-image for each file, apply the patch to a scratch checkout of the
worktree's `HEAD`, and see whether anything changes. If the result equals `HEAD`, the work is in a commit.

It failed on the one case I could test it against. Mira's superseded patch is work I **know** is committed,
and every version of the discriminator said "does not apply" or "differs".

## Why it cannot work, and this is the finding

A patch records a **diff between two filesystem states**. It does not record **which commit** the state it
was taken from corresponds to. So to reconstruct "is this already committed" you need the capture-time
commit — and that is exactly the information that is missing.

Concretely: Mira's patch was captured when her worktree was at `5810c7e`. Her branch has since moved to
`f554ac0`. Applying the patch to today's `HEAD` fails; applying it to `5810c7e` also fails, because the
patch's tracked half is a diff against the **pre-capture** state of `DispatchInboxView.tsx`, which is not
any commit's content. The only state it reconstructs is the one it was taken from, and that state is not
addressable.

**So the discriminator would need the patch to carry its base commit, and it does not.**

## What this means, and what I did not do

The check's failure mode is a false alarm of a specific kind: it says "your snapshot is stale" when the
truth may be "your work is safe". A false alarm of that shape is expensive — it is the thing that gets
ignored after being ignored once too often, and this week I corrected it by hand **three times**.

I did not add a base-commit field to the patch format. That is a change to a data format that other seats
write to, and the honest fix has a cost I cannot assess from here: either the capture records its base
commit (which means every future patch carries it, and old ones remain ambiguous), or the checker asks
`git log` whether the content appeared in a recent commit (which is a heuristic — content can be
committed, amended, and recommitted).

Both are real options and both are someone's decision. What I can say with certainty is that the current
message is wrong in one of the three cases, and that the case it is wrong in is the one that has now
happened twice.

## What is measurably true right now

| | |
|---|---|
| live patches | **3**, all round-tripping against their worktrees |
| superseded patches | **8** |
| worktrees with uncommitted work and no coverage | **0** |
| how many times I corrected a stale patch by hand this week | **3** |

So the store is currently accurate. The finding is about the message, not the state.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0, plus `check-unclaimed-work` at 0.
- Three pending cards awaiting you.
- Push: local and remote both at `00a38ea`, confirmed. No new commits this hour.
- DB reliability: 10/10 this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Whether salvage patches should carry their base commit.** That is the change that would let the checker
  tell the three cases apart. I have not made it.
- **Three cards are waiting.** For Vela's: accepting it moves 26 files from a masked failure to a visible
  one and reduces the undeclared count by zero.
- **Option 1, 2 or 3 for the live-seats count** — my recommendation is 1, unchanged.
- **The sandbox decision unblocks five dispatches at once**, not one.
