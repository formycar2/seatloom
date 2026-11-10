| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0700-i-verified-both-branches-of-the-salvage-message-in-real-conditions |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, salvage, verification, both-directions, scratch-worktree |

# I verified both branches of the salvage message in real conditions, and caught my own fixture error twice

## Why this hour exists

Last hour I split `check-salvage-fresh.sh`'s stale message into two branches and wrote a test that passes.
A test that passes is not the same as a change that works, so this hour I exercised **both branches against
real stale patches in real conditions**, not through the test's fixture.

## Branch 1 — the work grew → RECAPTURE

A scratch worktree, a patch captured from a dirty state, then one more line added so the work grew:

```
X ...-> cannot be reverse-applied in any worktree. It has gone stale while the
    inventory still calls it valid.
    The decidable half: there is still uncommitted work (scratch:1) - the work
    grew, so recapture:
      cd <the worktree> && git diff > <new patch>, then rename this one to
      .superseded.patch
```

**Correct, and it names the worktree and the dirty count** rather than leaving the reader to guess which one.

## Branch 2 — committed or lost → CANNOT TELL

Same construction, but the work is committed and `HEAD` moved, so no worktree has uncommitted work:

```
X ...-> cannot be reverse-applied in any worktree. It has gone stale while the
    inventory still calls it valid.
    The undecidable half: every worktree is clean. It may be that the work was
    committed (rename to .superseded.patch), or that the work was genuinely lost
    (recover it) - a patch does not record which commit it was taken from, so I
    cannot separate those two, and I will not guess. A human should look.
```

**Correct, and it says it cannot tell instead of guessing.** That is the whole point of the change: the old
message named one fix for three opposite cases, and the fix is not to guess better but to say which half
is knowable.

## Two fixture errors of my own, both caught

**First:** I placed the scratch worktree at `$W/scratch` and pointed `SEATLOOM_WORKTREES` at `$W/wtroot` —
two different directories, so the checker never saw the scratch and both branches reported the
all-clean message. The change looked broken; the fixture was wrong.

**Second:** while diagnosing that, I ran a loop with a **relative** patch path from inside another worktree,
which made three good live patches read as unmatched. I had already caught that same class of error in a
previous hour — `git -C <other-worktree> apply` needs an absolute path — and made it again.

Both were caught because I checked rather than reporting the output. **The pattern worth naming: a change
verified only through its own test's fixture is verified against the fixture.** The test passes and the
change can still be pointed at the wrong thing.

## What is now true

| | |
|---|---|
| the checker's two branches, exercised in real conditions | **both correct** |
| real worktrees touched during verification | **0** — every experiment used a scratch, removed after |
| all seven protocol checks | exit 0 |
| all five tests | exit 0 |
| DB reliability this hour | 10/10 |

## Measured state

- Ledger: 35 tables. Six seats online.
- Salvage: 3 live patches round-tripping, 8 superseded. No worktree has uncovered uncommitted work.
- Three pending cards awaiting you.
- Push: local and remote both at `6b23d35`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Three cards are waiting.** For Vela's: accepting it moves 26 files from a masked failure to a visible one
  and reduces the undeclared count by zero.
- **Whether salvage patches should carry their base commit** — that is what would close the remaining
  half of branch 2. I have not made that change.
- **Option 1, 2 or 3 for the live-seats count** — my recommendation is 1, unchanged.
- **The sandbox decision unblocks five dispatches at once**, not one.
