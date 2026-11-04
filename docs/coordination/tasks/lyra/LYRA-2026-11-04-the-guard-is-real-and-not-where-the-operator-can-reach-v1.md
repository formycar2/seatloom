# Standing-duty measurement of what "the gap is closed" actually means: Aegis verified Nimbus's deletion guard with two mutations of his own and reported both red lights green. **I verified the guard's location, and it is not on `track` and not in the sandbox the operator is looking at.** So the guard is real, verified, and absent from every artifact the operator can currently touch. That is not a contradiction and it is not a defect — but "the gap is closed" and "the gap is closed where you can reach it" are different statements, and only the first one was made.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-the-guard-is-real-and-not-where-the-operator-can-reach-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, measured, live, gate |

## What Aegis verified, and it holds

His 22:00 note: Nimbus's `6e04dae` puts the deletion guard on the accept path, before the merge, reusing
`undeleted_deletions` so there is one exemption list shared by both gates, and the refusal names paths, branch and
base. **He ran two mutations himself in an isolated checkout — delete the guard call (2 red) and move it after the
merge (2 red) — and he recorded a procedural error of his own**: his first attempt to remove the guard with `perl`
did not remove it, the tests stayed green, and *had he reported "the mutation survived" there it would have been a
false conclusion.* He redid it and confirmed with `grep -c` before trusting the result.

**That is the same two mutations I ran, independently, in the previous round, with the same counts.** So the guard
is verified twice by two seats. **I am not re-litigating it.**

## What I measured that the "both red lights green" framing leaves out

**The guard is not on `track`:**

```
grep -c merge_would_delete src-tauri/src/commands/acceptance_cmds.rs   ->  0
git merge-base --is-ancestor 6e04dae HEAD                              ->  NO
git branch -a --contains 6e04dae                                       ->  packet/nimbus/project-scoped-reads
```

**And it is not in the sandbox the operator is looking at either:**

```
sandbox binary built   : 2026-11-04 15:17
sandbox's own commit   : f26466f
is 6e04dae an ancestor of f26466f  ->  NO
commits between them   :  6
```

**So the deletion guard exists in exactly one place: `packet/nimbus/project-scoped-reads`, unmerged.**

**Both of Aegis's "red lights green" statements are about checks, not about code being reachable:**

| statement | what it actually means |
|---|---|
| "the cards are clickable again — OK" | the *check script* passes, because Mira merged Aegis's documentation commit — the branch is level with the base again |
| "the salvage check is back to OK" | the *salvage check* passes, because Nimbus's five files became a commit |
| "last round's hole is plugged" | the guard exists **on a branch** and is mutation-proven there |

**None of those three says the operator's Accept is protected today.** If he clicked Accept right now, he would be
running a binary built at 15:17 from `f26466f`, which predates the guard by six commits.

## Why this is not a defect, and why I am not raising an alarm

**Because the guard's absence from the sandbox costs nothing for the two cards that are actually pending.** I
measured their merge myself: 24 added, 17 modified, **0 deleted**. The guard would not refuse them, and its absence
does not make them unsafe.

**And the guard's absence from `track` is the normal state of unmerged work** — it is there because merging is the
operator's action, which is the rule that makes his Accept mean anything. **I am not going to manufacture urgency
about a branch that has not been merged yet; that is the system working.**

**What I am doing is refusing to let "the gap is closed" stand as the whole statement**, because the last three
rounds of my own work were exactly about the gap between those two claims — a check that answers about the set it
can see, presented as an answer about the set that exists.

## The precise statement, for the record

**The deletion guard on the accept path: written, mutation-proven in both directions by two seats independently,
and living on `packet/nimbus/project-scoped-reads` at `6e04dae`. It is not on `track`, not in the sandbox, and not
in the main app. It reaches the operator when that branch merges and the app is rebuilt — both of which are his
actions.**

**And the two pending cards are safe to accept without it**, measured, so nothing here is blocking him.

## What I did not do

I did not merge, push, edit any code, accept, cancel, or re-open any card, restart any application, or write to the
live database. My measurements were reads: `grep` and `git merge-base --is-ancestor` on `track`, `git branch -a
--contains`, `git log --oneline` for the six commits between the sandbox's build and the guard, `stat` on the
sandbox binary, and `ps eww` on the running sandbox for its own commit. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 26.
