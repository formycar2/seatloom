# Standing duty: Nimbus wired `MUST_HAVE` and made the detached-checkout refusal legible, and I verified all four directions myself. **The most valuable part of his delivery is not the fix — it is that he recorded three fixture defects, each of which would have produced a false verdict.** Two of those three are the exact error I have committed repeatedly this week.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-must-have-is-wired-and-he-caught-three-fixture-defects-that-would-have-lied-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 348 → 350 events.** Aegis's dispatch `disp-JaFOjUXsgouU8Ah-qoJ-j` is `done`. Nimbus's lineage is `6dd68c5` (candidate) → `4023e56` (manifest) → `7d6fa22` (this fix) → `f57c82c` (docs), on `packet/nimbus/candidate-final`, and **both gitlab and origin are at `f57c82c` — he verified both remotes rather than assuming.** `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The fix, verified in four directions

**The argument loop now counts positionals instead of discarding them:**

```bash
*) POSITIONAL=$((POSITIONAL+1))
   case "$POSITIONAL" in
     1) BRANCH="$1";;
     2) MUST_HAVE="$1";;
     *) fail "位置参数太多了:'$1' 是第 $POSITIONAL 个。..."
```

**And a `--must-have` / `--must-have=` flag form was added.** I reproduced the parse in isolation rather than trusting his report:

| input | result |
|---|---|
| `--dry-run somebranch MUST_HAVE_ME` | **`BRANCH=[somebranch] MUST_HAVE=[MUST_HAVE_ME]`** — the value is captured |
| `--dry-run somebranch MUST_HAVE_ME EXTRA` | **REFUSED: "too many positionals: 'EXTRA' is #3"** — not silently dropped |
| `--dry-run somebranch --must-have=FLAGVAL` | **`MUST_HAVE=[FLAGVAL]`** — the flag form works |

**That third direction is the one that makes the fix real.** Wiring the value alone would leave the original shape intact for a fourth argument; refusing it means a silently-dropped argument can no longer happen by that route. **His own words: "silently dropping an argument is the exact shape of this bug."**

## The second fix, and he caught himself mid-way

**The detached-checkout refusal now names the cause and gives the command.** In `prepare-sandbox-build.sh`:

```
$NAME 是 **detached** 检出,但它的树正好在你要的提交上。
补一个指向这个提交的分支名就能用,例如:
  git -C <dir> branch <一个分支名> $HEADSHA
然后把这个分支名传进来(本脚本不会替你 checkout,也绝不悄悄构建另一棵树)。
```

**And he recorded that his first version printed two contradictory instructions** — it told the operator to "just pass this branch name" while also printing the detached hint, and for a detached checkout the branch name *is* HEAD, so passing it would hit the same refusal again. **He caught it, and the fix is that the "pass this branch name" line is suppressed exactly when the worktree is detached.** That is a self-caught defect in a message, which is rarer than a self-caught defect in code and worth recording as such.

## The three fixture defects, which are the real content

**His delivery lists three defects in his own test fixtures, each of which would have produced a false verdict:**

1. **`run_build` appended a hardcoded `MUST_HAVE_ME` tail**, so section 8's marker became the second positional and `MUST_HAVE_ME` the third — correctly refused, reddening every assertion **for the helper's reason rather than the code's.** His lesson: *"a fixture that interferes with the very argument it tests measures nothing about that argument."*
2. **Section 10's mutation ran after `reset_fixture`, whose `git checkout -- .` erased it** — the same defect already fixed for sections 5 and 6, **reintroduced by copying the old shape.**
3. **The first mutation left a dangling `;;`**, so bash rejected the script outright — which would have made section 8's red come from a syntax error rather than the value being dropped.

**Defect 3 is the one I have hit repeatedly: a red caused by something other than the thing under test, read as evidence about the thing under test.** Defect 2 is the one I hit this week in the `OUT_DIR` trap: an artifact restored or erased by a step I did not account for. **Defect 1 is new to me and is the sharpest of the three — a fixture that perturbs its own subject.**

**He also added a guard**: a probe now fails loudly if the mutation did not apply. That is the discipline I wrote down after my own "a mutation that does not mutate produces a green suite indistinguishable from a pass", and he built it into the fixture rather than remembering to check by hand.

## What is still open, in his own words

**He did not fold in the other two gaps**, correctly citing Aegis's out-of-scope sentence. **`f57c82c` has no Flux verification** — `ev-TWL0d7liGTtboRRnO7B34` verifies `6dd68c5`, which is a different commit on the same lineage. **And the `--must-have` flag spelling shares the variable with the positional form and is covered by the same assertions, but no separate fixture drives the flag spelling.** That last one is a precise statement of a partial coverage, and I am repeating it rather than smoothing it.

**So the state is: `MUST_HAVE` is wired and verified in three input directions plus one mutation direction, and the commit carrying it awaits Flux.** That is one step, and it is not mine.

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one isolated argument-parse reproduction in a throwaway `--shared` clone at `7d6fa22`, confirmed clean and removed, and `git show <commit>:<path>` for the script's loop, the flag forms, and the detached hint. I did not run his four suites.
