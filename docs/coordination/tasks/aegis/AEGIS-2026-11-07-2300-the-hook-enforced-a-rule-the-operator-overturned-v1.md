| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-07-2300-the-hook-enforced-a-rule-the-operator-overturned |
| status | delivered |
| author | aegis |
| date | 2026-11-07 |
| version | v1 |
| tags | aegis, hooks, acceptance, correction |

# The commit guard was enforcing a rule the operator overturned, and its stated reason was false

## What happened

Two acceptance cards appeared while I was committing my 22:00 delivery note. The pre-commit
hook refused, saying the branches "will not contain this commit" and that "the acceptance
gate refuses a branch that does not contain its base."

I did not override it. I checked whether it was right.

## It was not, twice over

**The criterion is the one the operator overturned.** You ruled on 2026-11-05 03:43:40 UTC
that clickability asks only whether the merge would drop mainline commits or delete files,
and that being level with the mainline is no longer a ground to refuse. The hook was asking
"does the branch contain the future tip" — exactly the question you ruled out.

**Its stated justification was false about the accept path.** I traced every caller of
`branch_contains` in the codebase. There is exactly one: `dispatch.rs:1808`, inside
`request_acceptance` — the moment a card is *created*. The accept path
(`acceptance_cmds.rs:311`, `merge_branch_into_base`) never checks it; it runs
`git merge --no-ff`, and `--no-ff` cannot drop commits. So containment genuinely lives at
card creation, which is what your ruling said. The hook was quoting a rule that the code
does not apply, at the place it does not apply it.

**I measured the real state rather than trusting the hook.** For both cards: merge into the
future tip is clean, 0 files deleted, 0 mainline commits dropped. Those cards are clickable
right now. `check-cards-clickable.sh` agrees.

## The fix, and how it was proven

The hook now asks the ruling's question: `git merge-tree --write-tree` against the tip that
*will* exist. A conflict is refused — that is what fails on your click. Being behind is not.

I also fixed two things about the guard itself:

- **A pass now prints its evidence.** It used to exit 0 silently. A pass with no evidence is
  indistinguishable from a pass that never checked, which is the failure mode this hook was
  built to prevent.
- **It can now be fed a must-fail sample.** The database query was hardcoded, so I could not
  test the guard without writing a fake card into the live ledger, which I am not permitted
  to do. A guard that cannot be fed a failing case is not a tested guard.
  `SEATLOOM_HOOK_CARDS_CMD` redirects it; the default is unchanged.

`scripts/test-precommit-card-check.sh` proves four directions, and passes repeatably:

| direction | result |
|---|---|
| the two real cards are allowed, and report `merges clean` | pass |
| a genuinely conflicting branch is refused, and the reason says `CONFLICTS` | pass |
| a missing branch is reported separately, not folded into a generic failure | pass |
| a clean branch in the same fixture is allowed — so the refusal above really comes from conflict, not from "there are cards" | pass |

## Three mistakes of mine worth recording

1. **My first must-fail fixture did not conflict.** I appended a line the future commit did
   not have, which is a clean merge, not a conflict. The guard correctly said "clean" and my
   test reported "the guard is broken." A fixture that cannot fail makes you suspect the
   thing you are testing.
2. **My anchor check ran after the modification.** When it failed, the staged change was
   already in place, so the next run read the previous run's leftovers and the same guard
   misfired twice in a row.
3. **My cleanup restored from the index before resetting it.** So it "restored" the
   modification back into the working tree, and only the exit trap actually cleaned up. A
   cleanup that only works after the test has exited is not a cleanup.

None of these touched the real applications: fingerprint before and after all of it is
`90fcd5ecd2107bd7782aec850158fd89`.

## Not done

- No card, no merge, no install, no restart, no live-database write.
- The hook change is in `.git/hooks/`, which is untracked operational state by design and
  is not part of a commit. The test script is committed; the hook itself needs your decision
  on whether it should be tracked.
- The sandbox is still the 95-byte fake and still needs your authorisation.
