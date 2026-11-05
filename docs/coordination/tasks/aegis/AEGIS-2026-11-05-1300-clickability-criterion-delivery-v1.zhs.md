# Delivery for disp-aNMhDN7ICyR_2_qPf50iL and disp-aAikyaqfBvfbQDH3WlpfE — the clickability gate now asks the ruling's question, proven in both directions

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-1300-clickability-criterion |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | dispatch, ruling, gate, mutation-verified |

**Dispatches**: `disp-aNMhDN7ICyR_2_qPf50iL` (the ruling's second half) and `disp-aAikyaqfBvfbQDH3WlpfE` (the deletion check made real at click time), both issued by `seat:seat-lyra-001`.
**Changed**: `scripts/check-cards-clickable.sh`, `scripts/test-clickability-criterion.sh` (new).
**Commit**: `TBD` on `track/infra-foundation`.

---

## 1. Her premise, verified before I acted

She wrote that all three packet paths she gave me this week point at files not on track. Measured with `git cat-file -e track/infra-foundation:<path>`: **all three fail**, and I proved the command works by hitting `scripts/check-cards-clickable.sh`, which does exist. So what reached me was the dispatch note, exactly as she said. **I read the ruling from her worktree** (`packet/lyra/milestone-summary`) rather than from the dispatch's paraphrase.

## 2. What the ruling actually changed

`dec-clickable-criterion-level-with-base`, resolved 03:43:40 UTC by `human:supervisor`:

> Clickability asks only whether the merge would drop the mainline's commits or delete files; it no longer requires the branch to be level with the mainline. The level-with-base requirement stays at card-open time (`request_acceptance`) and does not block your clicking.

The gate's predicate was `merge-base --is-ancestor "$base" "$branch"` — the level check itself — which refused every branch that was merely behind. **That is the ground he ruled invalid**, and it cost him five hours a day (Lyra's re-priced figure: windows of 61.5 / 58.9 / 61.1 / 119.4 min).

## 3. 🔴 What I got wrong twice, and what the ruling actually implies

**First attempt** — I replaced the level check with two predicates: the deletion preview, and `merge-base base branch == base` for "would drop commits".

**Both directions then failed**, and the failure was mine, not the gate's:

1. **The scenarios were built in the wrong order.** I cut the delivery branches from the same tip as the advanced mainline, so they did not contain the mainline at all — a *divergence*, not "behind". The test caught it ("behind-only does not contain the base's history — the scenario is wrong, not the gate"). Fixed by advancing the mainline first, cutting the branches from it, then advancing once more.

2. 🔴 **`merge-base base branch == base` refuses the merely-behind branch too.** A behind branch lacks the mainline's *newest* tip, so the merge base is an older commit. I had built a predicate that refuses behind and diverged identically — **re-implementing the exact ground he dropped, one layer down.**

**Then I looked at the ruling's own words and found the second half is not a merge property at all.** A git merge keeps both sides' history; no ordinary merge can drop a mainline commit. The only thing "would drop the mainline's commits" can map onto is "the branch does not contain the base" — which is `request_acceptance`'s `BranchNotSynced` at `dispatch.rs:1815`, **which he explicitly kept and explicitly said does not block clicking.**

So the honest implementation is **one predicate, not two**: the deletion preview, the only click-time question that is genuinely about the merge. The containment question stays where he ruled it belongs. I have written that reasoning into the script as a comment, because the next reader will otherwise "helpfully" add the second check back.

## 4. What the gate does now

| question | where it is answered | verdict |
|---|---|---|
| would the merge **delete** files the base tracks? | click time, this script, `merge-tree --write-tree` + `DELETION_EXEMPTIONS` | refuse |
| does the branch **contain** the base? | card-open, `request_acceptance` → `BranchNotSynced` | refuse there, **not** here |
| is the branch merely **behind**? | nowhere as a refusal — now a printed note | allow |

The one hardcoded exemption is `ui/src/app-v2/AppV2.tsx`, the same single entry `DELETION_EXEXMPTIONS` holds, ruled by him and audited by the compiler.

## 5. Both directions proven, and the test can fail

`scripts/test-clickability-criterion.sh`, **17 passed, 0 failed**:

- a branch that is **merely behind** → **exit 0, allowed** (the direction that cost five hours a day, and the one no test would naturally cover because the live ledger has zero pending cards)
- a branch whose merge **deletes** `README.md` → **exit 1, refused**
- **mutation 1**: remove the preview → the deleting branch is allowed through, proving the preview is load-bearing
- **mutation 2**: restore the pre-ruling level check → the behind branch is refused, proving the test can detect the ground he dropped
- scenario sanity first: each branch is proven to be what it claims **before** any answer is read from the gate

Three near-misses in my own testing, all recorded in the file so they are not repeated:
- the mutation copies ran from `/tmp`, so the gate's `cd "$(dirname "$0")/.."` landed in a non-repo and every mutation reported exit 2 — the *test* failing, not the gate. Fixed by placing them inside the repo under `.scratch/`.
- **mutation 1's assertion was backwards.** A gutted gate *must* let the deleting branch through; asserting it still refuses would demand a test that only passes when the mutation fails to apply.
- the probe branches left `README.md` untracked (`git rm --cached` in a scenario), which then blocked `checkout`. The test now restores it and verifies it is tracked.

## 6. What I did NOT do — and why it is the other dispatch

**I did not land Nimbus's guard.** `6e04dae` is on `packet/nimbus/project-scoped-reads`, at the same revision on origin and in the cached gitlab ref, and is **not an ancestor of track**. The accept path (`acceptance_cmds.rs:88`) still goes straight to `merge_branch_into_base` with no deletion guard.

**Landing it is a merge of another seat's branch into track — the operator's action, not mine and not Lyra's.** Both dispatches said this explicitly, and it is the same boundary as before. So the second dispatch's ask is only half-satisfiable by me: the clickability gate now checks the merge, and the accept path still does not.

That is not a quiet omission. It is the standing item I reported at 12:00 and it remains open: **he was told at 03:43 that the deletion guard bears the click-time check. It exists on a branch.** The merge he authorised deleted nothing (0 files, file counts unchanged), so this is a real risk that has not yet cost anything — but the next Accept runs without the guard he was told about.

## 7. Verification

| check | exit |
|---|---|
| `bash -n scripts/check-cards-clickable.sh` | 0 |
| `bash -n scripts/test-clickability-criterion.sh` | 0 |
| `scripts/test-clickability-criterion.sh` | 0 (17/17) |
| `scripts/check-cards-clickable.sh` (live, 0 pending) | 0 |
| `check-salvage-fresh` / `check-verdict-refs` / `check-verdict-defects` / `check-card-verifications` | 0 each |
| `test-notify-unclickable.sh` | 0 |
| `verify-docs.sh` (default gate) | 1 — **pre-existing**: `BUDGET_POOL_MODEL_MAP.md` missing its universal header, unrelated to this change |

After the test's git operations: on `track/infra-foundation`, no `probe/*` branches, `README.md` tracked, `.scratch/clickability-mutations` removed.

## 8. For the operator

**A** — accept Nimbus's `6e04dae` onto track so the click-time deletion check is real in the accept path too. It is already pushed to both remotes; Flux can verify it independently.

**B** — accept that the accept path stays unguarded, and have the record say so plainly rather than describing a guard that is on a branch.

I am not choosing for him. The gate I just fixed was the proxy; the guard is the thing itself.
