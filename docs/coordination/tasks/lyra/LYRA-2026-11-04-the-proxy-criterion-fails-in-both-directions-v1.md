# Standing-duty finding, and it is the serious one: the proxy criterion on the card gate fails in **both** directions, and I measured both in scratch repositories. It blocks clicks whose merges are perfectly safe (three times today), and it permits clicks whose merges delete files tracked on the base. Worse than I said last round: `merge_would_delete` runs only at **request** time, so the accept path has no deletion guard at all — the one place that could stop the dangerous merge is the one place that does not check.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-the-proxy-criterion-fails-in-both-directions-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, finding, measured, gate, live |

## Why I measured this

Last round I verified that my ruling's criterion is implementable, and said the defect was that *the right check
is wired into the merge and an unrelated proxy is wired into the gate.* **I had not measured whether the proxy can
fail.** A check that cannot fail is worse than no check, so the proxy's failure modes are the thing to measure — and
I measured them in both directions this time.

## Direction one — the proxy blocks a safe merge (measured, and it happened three times today)

Scratch repository: a base with `keep.txt`, a delivery branch adding `new.txt`, then the base moves forward with
`other.txt`.

```
merge-base --is-ancestor main delivery   ->  NO        (the gate says: behind, UNCLICKABLE)
merge-tree --write-tree main delivery    ->  exit 0, no conflict
git diff --name-status main..<tree>      ->  1 A        (one file added)
```

**Zero deletions, no conflict, nothing lost — and the gate refuses the operator's click.** That is exactly what
happened to both real cards three times today, each time because Aegis committed a delivery note.

## Direction two — the proxy permits a dangerous merge (measured)

Scratch repository: a base with `keep.txt`, a delivery branch that **removes `keep.txt`** and adds `new.txt`, and
the delivery branch contains the base.

```
merge-base --is-ancestor main delivery   ->  YES       (the gate says: CLICKABLE)
merge-tree --write-tree main delivery    ->  exit 0, no conflict
git diff --name-status main..<tree>      ->  D keep.txt
                                          A new.txt
```

**The gate permits a click whose merge deletes a file tracked on the base.** No conflict, so nothing stops it;
the branch is level with the base, so the proxy is satisfied.

**This is the direction that matters.** Direction one costs the operator his own action for an hour. **Direction
two costs him data, silently, with the system's own gate saying the click was fine.**

## And the guard that would catch it is not on the accept path

**`merge_would_delete` is called from exactly one place: `request_acceptance`, at `dispatch.rs:1822`.** Measured —
`grep -rn merge_would_delete src-tauri/ crates/` returns the definition in `git/ops.rs`, two of its tests, and that
single call site. **`acceptance_cmds.rs` contains zero occurrences.**

**So the sequence is: the deletion risk is checked when the seat opens the card, and never again when the operator
clicks.** Between those two moments the branch can change — a seat can commit to it, the base can move, and the
merge preview computed at request time describes a state that no longer exists. **The check that protects against
data loss is evaluated at the one moment when its answer is already stale.**

**What the accept path does check** (`acceptance_cmds.rs:60-95` and `:311-378`): the operator identity, the card is
pending, the verdict is green and about this dispatch **re-checked at decision time** (good — that one is re-read),
and then `merge_branch_into_base`, which verifies the repo is a work tree, the tracked tree is clean, the branch
exists, the base exists, checks out the base, and merges `--no-ff`, aborting on conflict. **No deletion preview.**

**Note the asymmetry the code itself demonstrates:** the verdict is re-checked at decision time because *a verdict
can be re-issued between the two moments.* The branch can change in exactly the same window, and nothing re-checks
it.

## What I am asking for, narrowed to one thing

**Add `merge_would_delete` to the accept path, before the merge.** It is already written, already tested
(`git/ops.rs:311,317` — a synced branch and a destructive branch), and already used one function over. **It previews
with `merge-tree` and moves no ref, so it is safe to call at any moment.**

**And that one addition is what makes my last round's ruling actionable rather than theoretical.** With the deletion
guard on the accept path, the level-with-base proxy can be dropped from the clickability gate without losing
anything — because the thing it was standing in for would finally be checked where the merge actually happens.

## What I am not doing

**I am not editing `acceptance_cmds.rs`.** It is product code in the Tauri surface and it is not mine; and the
change I am asking for touches the one path that merges into the integration branch, which is exactly the kind of
change that should not be made by the seat that keeps getting told to stop taking unsupervised infrastructure
actions. **I am recording the measurement and the ask.**

## What I did not do

I did not merge anything in the real repository — both scenarios were fresh repositories under `/tmp`, since
removed. I did not edit any script or product code. I did not accept, cancel, or re-open any card, push, restart
any application, or write to the live database. My measurements were reads plus the two scratch simulations:
`git merge-base --is-ancestor`, `git merge-tree --write-tree` and `git diff --name-status` in three scratch
repositories and on the real branches, `grep -rn merge_would_delete` across `src-tauri/` and `crates/`, and reading
`acceptance_cmds.rs:60-95, 311-378` and `git/ops.rs:124-175`. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 25.
