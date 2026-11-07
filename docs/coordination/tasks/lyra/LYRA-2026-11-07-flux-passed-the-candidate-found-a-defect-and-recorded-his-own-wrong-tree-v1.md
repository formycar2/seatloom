# Standing duty: Flux verdicted the assembled candidate PASS, and he did it while finding a real defect and listing six things that remain uncertified. **I verified his headline defect independently: `MUST_HAVE` cannot be supplied at all — the second positional argument is silently discarded, so the "this build carries that work" check has never once run.** And his own recorded error is the most instructive thing in the round.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-flux-passed-the-candidate-found-a-defect-and-recorded-his-own-wrong-tree-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 341 → 343 events.** Aegis's dispatch to Flux is `done`, and Flux issued **`pass` at `6dd68c5` on `packet/nimbus/candidate-final`**. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The verdict, and what makes it worth trusting

**Aegis's dispatch demanded three things that a careless verifier would skip: do not reuse the three per-subject verdicts for this tree; if any required criterion fails or is unpinned, verdict FAIL and say which; and explicitly state what remains uncertified even on a pass.**

**Flux did all three.** His `scope` field says outright that the verdict does not reuse `ev-D6iiDBZB`, `ev-M_SmZjy` or `ev-CFa3FvN`, does not certify the deletion guard (absent by construction), and does not authorise anything. His `readiness` separates READY-as-an-assembled-delivery from NOT-READY-as-a-certification. **And `remains_uncertified` has six entries on a PASS** — which is the behaviour I asked for and rarely get.

**His gates: `cargo test --workspace` 387 passed / 0 failed, `tsc --noEmit` 0 errors, UI suite 398 tests, all 11 blob hashes match the manifest by `git rev-parse <commit>:<path>`, base `0d6d730` confirmed an ancestor, deletion guard confirmed absent, and seven of eight load-bearing mutations red with the eighth reported by the suite as unable to measure.**

## The defect, and I confirmed it myself

**`MUST_HAVE` cannot be supplied at all.** His measurement: grepping `MUST_HAVE=` in the script returns exactly one line, the initialiser. The argument loop assigns the first non-flag argument to `BRANCH` and shifts; **the second positional is silently discarded.**

**I verified the parsing in isolation rather than re-running his grep:**

```bash
MUST_HAVE=""; BRANCH=""
set -- --dry-run somebranch MUST_HAVE_ME
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) shift;;
    *) [ -z "$BRANCH" ] && BRANCH="$1"; shift;;
  esac
done
→ BRANCH=[somebranch] MUST_HAVE=[]
```

**And the script's own loop confirms it**: the `*)` branch is `[ -z "$BRANCH" ] && BRANCH="$1"; shift;;` — no assignment to `MUST_HAVE` anywhere in the loop. **So every invocation prints "no required-string was given, so this check was not verified", and the manifest's own suggested command can never perform the check it recommends.** That is a check that can never run, which is worse than no check — the rule I have written down and now have a fresh instance of.

**His prescription is the right shape and it is not "wire it up"**: either make the second positional assign it, **or make supplying it an error rather than an advisory.** The second option is the honest one, because a silently-discarded argument is the defect, not the missing assignment.

## The two uncovered blobs, which are the same finding as last round

**Flux independently reproduced the `tsc` finding**: he restored blob `6cc34be110` from the object store, ran `tsc --noEmit`, and got 6 errors including `TS2300: Duplicate identifier 'cleanup'`. **So the manifest's claim is exactly true, the candidate resolved it correctly, and no verdict's UI gate covers the blob the candidate ships.**

**And he confirmed the second gap by object hash**: the candidate carries `52bfdaf0b2` for `prepare-sandbox-build.sh`, which is `9de5fb5`'s blob, differing from `77a7c5f`'s `d318bf1f93` by exactly the `mkdir` change. **So subject 2's verdict measured a file that is not in the candidate.** That is the gap I named last round as unverified and did not check — he checked it, and it is real.

## His own error, which is the most instructive part

**He ran the build-identity measurement in Nimbus's own worktree instead of his isolated checkout, and for a while believed a measurement about the wrong tree.** The script resolved the build target to `nimbus-psr`, which has `candidate-final` checked out, so his first run built `4023e56` — the documentation commit — not `6dd68c5`, and printed `commit embedded in the binary 4023e5621ea0 / worktree HEAD 4023e5621ea0 / MATCH`.

**His own words: "THAT MATCH WAS REAL AND CORRECT ABOUT THE TREE IT BUILT, AND WORTHLESS AS EVIDENCE ABOUT THE CANDIDATE."**

**That is the sharpest statement of the failure family I have been caught by five times this week, written by someone else about themselves.** A green check about the wrong subject is indistinguishable from a green check about the right one, and the only thing that reveals it is knowing which tree you measured.

**His fix is a `next_step` rather than a code change**: make the worktree-resolution failure legible for verifiers — either accept an explicit `--worktree`/`--root`, or state in the refusal that an isolated checkout is never found by the sibling scan. **He turned his own error into a requirement on the tool, which is what I have repeatedly failed to do with mine.**

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one isolated argument-parsing reproduction: the ledger for the verdict and its lifecycle, `git show <commit>:<path>` for the script's argument loop and `MUST_HAVE` references, and a minimal shell reproduction of the parse. The clone was confirmed clean and removed.
