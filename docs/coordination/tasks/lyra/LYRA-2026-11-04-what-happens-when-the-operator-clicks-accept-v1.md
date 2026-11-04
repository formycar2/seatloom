# Standing-duty measurement of the path the operator is about to take: I measured what actually happens when he clicks Accept on either card. The merge runs in the primary checkout, not the sandbox — which is correct and is the only reason accepting these cards would bring the fix to the app he uses. I also simulated the two-cards-one-branch case in a scratch repository and found that `BranchNotSynced` cannot block the second click, because that guard lives only in `request_acceptance` and not in the accept path.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-what-happens-when-the-operator-clicks-accept-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, measured, live, gate |

## Why I measured this

Both cards are pending, both gates are green, and I have spent four rounds verifying that the cards are
*runnable*. **I had not measured the step after the operator's click** — what the application actually does with
his Accept. That is the step that determines whether accepting is safe, and it was the one thing I had left as
"the operator's action" without knowing what the action is.

## Where the merge happens, and why it is the right place

`cmd_accept_delivery` (`src-tauri/src/commands/acceptance_cmds.rs:57`) calls `merge_branch_into_base(&state.repo_root,
&row.acceptance.branch, &row.acceptance.base)`.

**`repo_root` for the sandbox resolves to the primary checkout.** Measured:

```
the sandbox process has no SEATLOOM_REPO_ROOT in its environment
its cwd is /            (launched from /Applications)
main.rs:75-107          env var absent -> cwd is / -> parent heuristic fails -> falls back to
                        ~/Documents/GitHub/seatloom, which exists with Cargo.toml and docs/
```

**So an Accept clicked inside the sandbox performs `git merge --no-ff --no-edit packet/mira/knowledge-panel` in
`~/Documents/GitHub/seatloom` — the primary checkout, on `track/infra-foundation`.**

**That is the correct design and it is the whole point.** The sandbox is where he verifies; the primary checkout
is where the code lives; accepting is the act that moves the fix from the first to the second. **It is also why
the fix is not in his running app yet** — the main app (pid 90805) was built 10-28 and nothing has merged since.

**Preconditions for the merge, all measured satisfied:**

| guard | measured |
|---|---|
| repo is a real git work tree | yes |
| tracked tree clean (`??` does not block) | yes — only the long-standing untracked `scripts/bind-seatloom-seats.sh` |
| branch exists locally | yes — `refs/heads/packet/mira/knowledge-panel` |
| verdict is green and about this dispatch, checked **at decision time** | yes — `flux_verdict_facts` is called again in the accept path, not only at request time |

## The two-cards-one-branch case, simulated rather than assumed

**Both cards name the same `branch` and the same `base`.** So the obvious worry is: after the first Accept merges
the branch into track, does the second card still work?

**I simulated it in a throwaway repository at `/tmp/lyra-merge-sim`** — a base branch, a delivery branch with one
commit, then `git merge --no-ff --no-edit delivery` on the base:

```
after the first merge:  git merge-base --is-ancestor base delivery  ->  NO
```

**So the delivery branch no longer contains the base, and `BranchNotSynced` would refuse it — if the accept path
checked that.**

**It does not.** Measured: `BranchNotSynced` is returned only from `request_acceptance`
(`dispatch.rs:1815`, inside the function beginning at 1760). **`acceptance_cmds.rs` contains no sync guard at
all** — its only `contains` calls are test assertions on error strings. The accept path checks the verdict, the
clean tree, and the branch's existence, then merges.

**So the second card's Accept will also succeed**, producing a second `--no-ff` merge commit whose content is
already present. **That is a redundant merge commit on track, not a failure** — and git will say so rather than
erroring.

## What I am flagging rather than fixing

**A redundant merge commit is harmless but it is noise on the integration branch, and the operator has no way to
know that clicking the second card will not do what the first did.** The card text does not say "this merges the
same branch again."

**I am not changing the card text or the code.** This is a UI-wording and product-behaviour question about a path
that has never been exercised twice, and the honest thing is to record it for whoever owns that surface rather
than to quietly make the second card look like the first.

**The practical guidance, which is what the operator actually needs: accept either card, and the fix lands. The
second click adds a merge commit and nothing else. If tidiness matters, one card is enough to get the code.**

## What the operator gets, stated in his three sections

**Changed:** both cards are clickable, both gates green, and I have now measured the click itself end to end.

**What you will see:** clicking Accept in the sandbox merges `packet/mira/knowledge-panel` into
`track/infra-foundation` in the primary checkout with `--no-ff`. **Your running app will not change** — it was
built 10-28 and only a rebuild picks up merged code, which is a separate step and yours to order.

**What needs you:** accept a card to land the fix. If you accept both, the second produces a redundant merge
commit. And the rebuild is still outstanding — accepting is not the same as having the fix in the app you use.

## What I did not do

I did not click, accept, cancel, or re-open any card. I did not merge anything in the real repository — the
simulation was a fresh repository at `/tmp/lyra-merge-sim`, since removed. I did not push, restart any
application, or write to the live database. My measurements were reads: `acceptance_cmds.rs:57-95` and `311-345`,
`main.rs:75-107`, `dispatch.rs:1760,1815`; `pgrep`/`ps eww`/`lsof` for the sandbox process's environment and cwd;
`git rev-parse`/`show-ref`/`status` on the primary checkout; `psql` for both cards' `base` and `branch`; and the
scratch-repository simulation. The ledger check is exit 0, `seats` is 10, `canonical_events` is 25.
