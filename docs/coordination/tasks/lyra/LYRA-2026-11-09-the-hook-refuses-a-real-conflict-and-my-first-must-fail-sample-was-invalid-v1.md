# Standing duty: **I verified the newer pre-commit hook is genuinely testable in both directions, which is the property I insisted on for the sandbox script. With a fixture-driven card list it refuses a real merge conflict with exit 1 and the correct message, and it passes when there is no conflict. And my first "must-fail" sample was invalid — I had verified the conflict against my own branch's HEAD instead of track's, which is the same error I have catalogued repeatedly.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-hook-refuses-a-real-conflict-and-my-first-must-fail-sample-was-invalid-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 491 events and 98 dispatches.** `get_next_dispatch` returned nothing ready. **Two cards pending, both verified by me.**

## Following the thread from last round

**Last round I found the installed pre-commit hook is five days and two capabilities ahead of its tracked source, and that one of those capabilities is `SEATLOOM_HOOK_CARDS_CMD` — an injection point whose own comment states the principle I have been enforcing:**

> 一个没法喂它必败样本的守卫,和没有守卫是同一件事。
> **A guard you cannot feed a must-fail sample is the same as no guard.**

**A claim that a check can fail is not evidence that it can fail, so I fed it one.**

## My first sample was invalid, and I caught it by measuring

**I picked `packet/nimbus/deletion-guard-only` because I had previously measured it conflicting — and the hook reported "merges clean into the new tip (files deleted: 0)".**

**The hook was right and I was wrong.** I checked:

| | ref |
|---|---|
| where I had measured the conflict | `packet/lyra/milestone-summary` @ `35ab3eda` |
| where the hook measured it | `track/infra-foundation` @ `83f4e34` |

**The conflict was against my own branch's HEAD, not track's.** So I had imported a conclusion from a different pair of trees — **which is the "two trees, two true answers" family, and specifically the shape where a measurement is correct and the thing it was applied to is wrong.** The hook measured the right pair and got the right answer.

**And I nearly recorded the hook as unable to fail, which would have been a false claim in exactly the direction the injection point exists to prevent.**

## A valid sample, and the hook refuses it

**I searched for a branch that genuinely conflicts with `track/infra-foundation` and found `packet/aegis/check-verdict-defects-fixes`. Driven through the fixture:**

```
pre-commit: REFUSED. This commit would make 1 pending acceptance card(s) unclickable.
  acc-fake-real-conflict: merging packet/aegis/check-verdict-defects-fixes into the new tip
  CONFLICTS - his Accept would fail
```

**exit 1.** And the accompanying text is the part that makes it correct rather than merely strict:

> The operator's Accept merges the card's branch with `git merge --no-ff` on top of the base. That never drops mainline commits, **so a branch being behind the mainline is NOT a ground to refuse — the operator ruled so on 2026-11-05.** A CONFLICT is: the merge would fail on his click.

**So the hook refuses on the criterion that actually matters and explicitly refuses to refuse on the one that does not.** That is the corrected clickability rule, encoded and citing its own ruling. And the escape hatch is honest: `--no-verify` **and** a priority-99 branch sync **and** saying so in the commit body — "Do not do it silently."

## The scoping, which I verified rather than assumed

**The card check runs only when the branch is `track/infra-foundation`** — line 200: `if [ -n "$branch" ] && [ "$branch" != "track/infra-foundation" ]; then exit 0; fi`.

**That is why my earlier probes on my packet branch all passed regardless of the fixture.** I initially read that as the injection not working; it was the hook correctly declining to check something that cannot apply on a packet branch. **So the hook is scoped to the one branch where a card's Accept actually happens, and a green on any other branch says nothing about it.**

## Two things I did to other seats' directories, and what I did about them

**I want these recorded because both were mutations in paths that are not mine, and both were cleaned and verified:**

| probe | what | cleanup |
|---|---|---|
| rule-2 negative case | created and staged `docs/coordination/tasks/nimbus/PROBE-NOT-REAL.md` | unstaged, deleted, `git status --porcelain` clean, no residue in Nimbus's directory |
| card-conflict case | created and staged `PROBE-HOOK.md` and `PROBE.md` in two worktrees | unstaged, deleted, both worktrees verified clean, worktrees removed |

**Both were inside my own worktrees and both hooks refused before any commit object existed, so no shared state was written.** But using a real seat's directory name for a throwaway probe was wrong — it put a file with another seat's name on it into their path, and if anything had gone wrong mid-probe the cleanup would have been in their tree rather than mine. **A probe should use a name that could not be mistaken for anyone's work.**

**And I found one piece of pre-existing untracked work that is not mine**: `scripts/bind-seatloom-seats.sh`, untracked since 11-02 17:17, referenced by no commit. **I left it alone.** A pre-existing dirty file is a signal, not an invitation — the same rule the hook prints.

## What I did not do

I did not modify any file in any shared worktree, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, run `sync-hooks.sh --apply`, or write to the live database. My measurements were reads plus fixture-driven hook runs in isolated worktrees: `git merge-tree --write-tree` to find a genuinely conflicting branch, and `bash` on the installed hook with `SEATLOOM_HOOK_CARDS_CMD` pointing at files in `/tmp` that I deleted.
