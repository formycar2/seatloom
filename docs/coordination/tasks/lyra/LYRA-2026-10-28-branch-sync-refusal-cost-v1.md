# Finding: the acceptance gate refuses a behind branch for a reason git does not have — 16 emergency syncs today are the cost

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-branch-sync-refusal-cost-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | aegis |
| tags | acceptance-gate, branch-sync, false-refusal, cost, finding |

> Sixth heartbeat pass, no dispatch waiting. This is a **finding with a measured root cause and a
> proposed change**, not a decision I am executing. facts / findings / decisions separated.

## 1. Facts — measured this turn

| # | What | Measurement |
|---|---|---|
| F1 | Sixteen emergency sync dispatches went out **today** | 04:17 ×3, 05:06 ×2, 06:03 ×3, 06:06 ×1, 07:05 ×3, 07:12 ×1, 11:01 ×1, 11:06 ×1, 14:06 ×1. Across all four worker seats. |
| F2 | Each one says the same thing | *"your N cards are unclickable because your branch is behind track"*. Vela's latest (14:06) is titled with the literal word **EMERGENCY (ninth time)** — its own numbering counts nine rounds for that seat. |
| F3 | **The refusal is not a git failure.** `git merge` of a behind branch **succeeds and loses nothing** when the two sides touch different files. Proven on a scratch repo: trunk adds `other` then `third`; packet adds `mine`; packet ends 2 commits behind trunk; `git merge --no-ff packet` into trunk exits 0 and **all three files are present afterwards**. |
| F4 | When files DO overlap, git refuses by itself | the same scratch test with both sides editing the same file gives `CONFLICT (content)` and exits non-zero, with the conflict text. `merge_branch_into_base` already handles this correctly — it aborts and returns the git output verbatim. |
| F5 | So the sync requirement is not load-bearing for correctness | the merge would succeed; nothing would be reverted; git's own conflict check covers the one real risk. |
| F6 | The sync check lives in `request_acceptance`, not in Accept | `DispatchRejection::BranchNotSynced` is raised by `crates/seatloom-mcp-emit/src/dispatch.rs` (~line 1808, `branch_contains`). The operator-facing Accept path — `src-tauri/src/commands/acceptance_cmds.rs` `merge_branch_into_base` — **does not re-check sync at all**. |
| F7 | So the refusal fires at **request time**, when the card is created | a card created while its branch is behind is refused outright. Once created, Accept merges fine. |
| F8 | The pre-commit hook is correct and now measures the right thing | `.git/hooks/pre-commit` (installed 14:05 today) builds the would-be commit with `git commit-tree` and measures every pending card against **that** tip. The earlier version measured the pre-commit state, which is why it printed OK and then stranded 12 cards — that defect is fixed and documented in the hook's own header. |
| F9 | But the hook only guards `track/infra-foundation` in the main checkout | by design (`[ "$branch" != "track/infra-foundation" ] && exit 0`). It cannot stop a seat from being left behind by a **different** seat's commit. |
| F10 | Vela's branch is currently synced and the one card is clickable | `git rev-list --count packet/vela/washing-pipeline..track/infra-foundation` = **0**; `check-cards-clickable.sh` → `OK: all 1 pending cards are clickable`. Vela merged within a minute of the 14:06 dispatch, again. |

## 2. Finding

**The check prevents a failure that git already prevents, and costs 16 emergency dispatches a day to do it.**

`BranchNotSynced` exists to stop *"accepting an unsynced branch would merge work that never saw the
base's later commits"* — i.e. silently reverting whatever landed on track since the fork. F3 shows git
does not do that: a merge brings both sides' work together, and overlapping edits produce a conflict
that `merge_branch_into_base` already refuses rather than forces. So the protection the check provides
over git's own behaviour is, in the non-overlapping case, **nothing**.

The cost is not nothing: 16 dispatches today, each consuming a seat's whole attention, each with the
word EMERGENCY in the title, to perform one `git merge` that takes seconds. Vela has done it nine times.
And the emergency framing is the part that should worry us most — it is the same shape as the
`-seatloom` broadcast Aegis already recorded: **urgency that skips the scope check.**

Two structural points make it worse:

1. **The check is at the wrong end of the lifecycle.** It fires when a card is *requested*, so a seat
   can be current, get a card created, and be told an hour later that its card is unclickable because
   someone else committed. The information it needs is "is this branch's tip contained in its base at
   Accept time" — which is a question about the *present*, asked by the *person clicking*, not about
   the past asked by the *person requesting*.
2. **It cannot be satisfied by the person it blames.** A seat behind track must merge track into its
   branch. Track keeps moving. So the state it demands is a moving target that only stops moving when
   the whole fleet stops committing — which is the opposite of the fleet's job.

## 3. What I propose, for Aegis to rule on

**Option 1 — downgrade `BranchNotSynced` from a refusal to a warning carried on the card.** The card
records `synced_at_request: true|false` and, when false, the exact `git rev-list --count` figure. The
Accept path already merges safely (F3/F4). The operator then sees *"this branch was 2 commits behind
when the card was opened"* and can decide — which is a decision he is entitled to make, with a number,
rather than a refusal he cannot see the reason for.

**Option 2 — make it an Accept-time check with an auto-sync offer.** At Accept, if the branch is
behind, offer *"merge track into the branch first"* as a single action. This keeps the correctness
property (never merge a branch that never saw the base) and moves the work to the moment it is needed,
by the person doing it, once.

**Option 3 — keep it as is.** Defensible: the current behaviour is safe, and the cost is seat
attention rather than correctness. If this is chosen, the cheap mitigation is to **stop using EMERGENCY in
the title** and to batch: one dispatch per seat per round, not one per card.

I lean **Option 1**, with Option 2's auto-sync as the follow-up. My reason: the check's stated purpose
(never revert the base's later work) is already guaranteed by git plus the existing conflict refusal,
so what remains of it is a **disclosure** duty, and disclosure belongs on the card, not in a refusal.
But this is a gate-criterion change and the criterion is Aegis's, so I am not touching it.

## 4. What I am not doing

- Not editing `dispatch.rs` or the hook. The gate criterion is Aegis's; the hook is his and is correct.
- Not dispatching anything to any seat. The queue is empty and the seats are idle; manufacturing work
  from a finding would be exactly what `LYRA-HEARTBEAT.md` forbids.
- Not proposing this as urgent. It is a recurring cost, not an outage, and the product's current state
  (1 pending card, clickable, criteria true) is fine.
