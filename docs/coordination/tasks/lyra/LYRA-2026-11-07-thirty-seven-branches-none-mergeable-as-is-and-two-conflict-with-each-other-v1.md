# Standing duty: I measured the unlanded-branch backlog properly instead of naming it, and the picture is sharper than what I published. **37 packet branches are ahead of track and not one of them is mergeable as-is — every single one needs a merge first.** But 15 of them are 2-3 commits behind and **all 15 merge cleanly into track**. The finding that matters is that two of them conflict with *each other*, on the same subject.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-thirty-seven-branches-none-mergeable-as-is-and-two-conflict-with-each-other-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, coordination |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 332 events and 67 dispatches — unchanged since last round.** All three worktrees clean, no commit anywhere in 20 minutes, `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

**So I went back to the backlog, and this time I measured it instead of naming it.**

## What I published before, and what it got wrong

**Last round I named three branches as "verified but undelivered" and corrected myself that two were 427 and 888 commits behind.** That correction was right but it was still a description of three branches. **The actual shape of the backlog, measured now:**

| behind track | branches |
|---|---|
| 2-3 | **15** |
| 13 | 1 |
| 182-315 | 3 |
| 330-429 | 5 |
| 537-751 | 8 |
| 890-1001 | 5 |
| **total** | **37** |

**And the number I should have led with: `git rev-list --left-right --count track/infra-foundation...<branch>` filtered to `behind == 0` returns nothing. Not one of the 37 branches is mergeable as a fast-forward.** Every one needs a merge, which means every one needs a decision, and 37 decisions is not a backlog — it is a queue nobody is draining.

## The actionable set, measured

**15 branches are 2-3 commits behind track, and all 15 are Nimbus's.** I tested each with `git merge-tree --write-tree track/infra-foundation <branch>`, which computes the merge without touching the working tree:

```
CLEAN    build-failure-regression        behind=2  code_files=5
CLEAN    build-git-ok-truthful           behind=2  code_files=10
CLEAN    build-identity-three-states     behind=2  code_files=10
CLEAN    deletion-guard-accept           behind=2  code_files=4
CLEAN    deletion-guard-only             behind=2  code_files=10
CLEAN    deploy-preview-dry-run          behind=3  code_files=0
CLEAN    dry-run-dirty-fixture           behind=3  code_files=0
CLEAN    git-status-honesty              behind=3  code_files=0
CLEAN    project-scoped-reads            behind=3  code_files=5
CLEAN    sandbox-flag-exclusion-pin      behind=2  code_files=10
CLEAN    sandbox-flag-guard-first        behind=2  code_files=10
CLEAN    sandbox-plan-failclosed         behind=2  code_files=10
CLEAN    verification-candidate          behind=2  code_files=5
CLEAN    work-list-isolation             behind=3  code_files=0
CLEAN    work-list-snapshot-actors       behind=3  code_files=0
```

**All 15 merge cleanly into track. `code_files` counts files that are not `docs/` or `scripts/`** — so six of them carry real Rust or UI code and nine are docs-and-scripts only.

## The finding that matters: two of them conflict with each other

**A branch that merges cleanly into track can still be unmergeable alongside another branch.** I tested the pairs, because 15 branches touching the same few files is the situation where "each one is fine" stops being true:

```
CLEAN    deletion-guard-only        + build-git-ok-truthful
CLEAN    work-list-snapshot-actors  + verification-candidate
CLEAN    git-status-honesty         + verification-candidate
CONFLICT deletion-guard-only        + verification-candidate
```

**And the conflict is not incidental — it is the same subject on both sides:**

```
CONFLICT (add/add):    scripts/replace-sandbox.sh
CONFLICT (content):    src-tauri/build.rs
CONFLICT (add/add):    src-tauri/tests/build_identity_incremental.rs
```

**`src-tauri/build.rs` is the build-identity mechanism.** `deletion-guard-only` carries it because the identity capability was built to make the deletion guard's acceptance checkable; `verification-candidate` carries it because that branch is a coherent verification candidate whose manifest enumerates its subjects. **Both contain a version of the same file, and merging them requires choosing one — which is a decision about which version of the identity mechanism is correct, not a merge mechanic.**

**This is the thing I would have missed by reporting "15 branches, all clean".** Each is clean *into track*. They are not all clean *with each other*, and the pair that conflicts is the pair that matters most, because those are the two subjects closest to acceptance.

## What I am not doing, and why

**I am not merging anything.** My rule after 10-31 is that an unsupervised infrastructure action writing shared state is not mine, and a merge into track is exactly that. **I am also not choosing between the two versions of `build.rs`** — that requires knowing which identity mechanism is the intended one, and I have verified *behaviour* of both without being the one who decides which ships.

**What I can state, and it is a real narrowing for whoever decides**: the conflict is confined to three files, all of them the identity mechanism, and the deletion guard itself is on a branch (`deletion-guard-only`) that merges cleanly into track *on its own*. **So the deletion guard is not blocked by the conflict — only the identity mechanism is, and only if both are merged together.**

## What I did not do

I did not modify any file, merge anything, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: `git rev-list --count` over every packet branch against track, `git diff --name-only` filtered to non-docs/scripts for code content, `git merge-tree --write-tree` for each branch into track and for the four pairs, and the conflict listing for the pair that failed. `git merge-tree` computes merges without writing to the working tree or the index, so nothing was staged or changed.
