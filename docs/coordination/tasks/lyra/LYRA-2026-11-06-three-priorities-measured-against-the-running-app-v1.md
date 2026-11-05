# Research for `disp-i1WjTBsdcJX4_5k5XmgDE` — three user-visible priorities, each measured against the running application rather than against the repository. **The single largest finding is not a missing feature: the application the operator is running was built 10-28 22:16 and `track` has moved 10 commits since, including four product fixes that are on `track` and absent from his app.** The second is that his actionable work is spread across four instruments with no unified list, which is exactly the contract Aegis wrote on 11-02 and which nobody has implemented. The third is that the deletion guard — the thing the operator's Accept relies on — is still not on `track`, verified again this round.

| Field | Value |
|---|---|
| template | T3 |
| subtype | research |
| id | LYRA-2026-11-06-three-priorities-measured-against-the-running-app-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | research, priorities, measured, runtime |

## Reconciliation first, because the dispatch asked for it before any proposal

**Every number below is measured this round, on `track/infra-foundation` at `82926eb` (11-06 02:51).**

| thing | measured |
|---|---|
| `track` vs `origin/track` | `1 0` — one local commit unpushed |
| acceptance cards | 2, both **`accepted`**, 0 pending |
| dispatches | 1 `ready` (Flux's, issued 11-05 19:16), 1 `in_flight` (this one) |
| `operator_decisions` | **0 pending** |
| `canonical_events` | 46 |
| `seats` | 10, of which **7 bound to the `seatloom` project** |
| tmux sessions | 10, of which **9 carry the `-seatloom` suffix** |
| the app the operator is using | pid 90805, binary `/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri`, **built 10-28 22:16** |
| the sandbox app | built 11-04 15:17, commit `f26466f` |
| the deletion guard on `track` | **absent** — `6e04dae` is not an ancestor of HEAD, `acceptance_cmds.rs` has 0 references to `merge_would_delete` |

**And the deployment boundary, stated plainly because the dispatch asked for it to be distinguished from branch
claims:** four product fixes are on `track` and not in the running app, because the app predates them.

| fix | commit | on `track` | in the running app |
|---|---|---|---|
| new-dispatch files no longer land in a hardcoded project | `ad78e68` | yes | **no** |
| cold start lands on Dispatch, not a stale tab | `bd0cabb` | yes | **no** |
| undeclared tabs no longer present themselves as real data | `16c72c2` | yes | **no** |
| the dirty flag is exported and something reads it | `b5762c4` | yes | **no** |

**So "deployed behaviour" and "source claims" differ by four fixes and four days.** That is the gap every priority below
has to be read against.

---

## Priority 1 — Close the gap between `track` and the app the operator is actually using

### The measured user problem

**The operator is running a build from 10-28 22:16. `track` has moved 10 commits since, and four of them are fixes to
defects he reported.** He reported the hardcoded-project defect himself, and the cold-start defect, and the
fake-data tabs. All three are fixed on `track`. **He is still using the version that has them.**

This is not a feature gap. It is that fixed work has not reached him, for four days, and nothing tells him so.

### The existing reusable work

- `scripts/prepare-verification-build.sh` already exists and does the whole job: rebuild, `pkill -f "SeatLoom.app"`,
  `rm -rf "$APP"`, relaunch with operator identity via `scripts/launch-seatloom.sh`.
- I measured its cost two rounds ago and it is the reason this needs a decision rather than a command: **it deletes the
  app he is currently using** and has **no dry-run**.
- `prepare-sandbox-build.sh` exists separately and is the safe half — it takes a branch name, never a hash, after
  Aegis's three fixes, and it does not touch `/Applications`.

### The smallest next slice

**A dry-run flag on `prepare-verification-build.sh`**, printing what it would delete and which commit it would build,
changing nothing. That is the smallest change that makes the rebuild approvable rather than something the operator has
to reason about before running.

**Then, separately, the rebuild itself** — which is a human approval, not a seat action.

### Owner, dependencies, acceptance

| | |
|---|---|
| owner | **Nimbus** — the script is infrastructure and the dry-run is a small change |
| dependencies | none. It does not wait for the deletion guard or the clickability change |
| what the user sees | `bash scripts/prepare-verification-build.sh --dry-run` prints the target, the commit, and the path it would delete, and exits 0 having changed nothing |
| what requires human approval | **the actual rebuild.** It deletes the running app. That is his call, not a seat's |

### Why this is first

**Because it converts four already-accepted fixes into something he can use, at the cost of one small change and one
decision.** Every other priority adds new capability to a version he is not running.

---

## Priority 2 — A unified actionable work list, which is a contract that already exists and has no implementation

### The measured user problem

**The operator's outstanding work is spread across four instruments, and none of them is the list:**

| instrument | what it shows | who reads it |
|---|---|---|
| `scripts/fleet-status.sh` | fleet health, cards, decisions | Aegis, in his heartbeat |
| `scripts/check-unclaimed-work.sh` | done dispatches with no card | nobody, routinely |
| `AEGIS-2026-11-02-standing-backlog-v1.zhs.md` | two structural conflicts, deliberately de-argued | the operator, when he reads docs |
| `scripts/notify-operator.sh` | five notification classes, pushed | the operator, when something fires |

**Measured right now: 1 ready dispatch (Flux's, 8 hours old), 0 pending decisions, 0 pending cards.** So the list is
nearly empty — and it is still spread across four places, and the one item on it has been sitting for eight hours
because nothing surfaces it.

**And I measured this week what happens when a to-do lives in only one of those places**: the seat-binding defect sat
in Aegis's heartbeat body for five rounds with no action, and closed in 24 minutes once it entered the push channel.
*A to-do that exists only in a file has the same effect on him as a to-do that does not exist.*

### The existing reusable work

**`docs/coordination/tasks/aegis/AEGIS-2026-11-02-1500-unified-todo-readmodel-contract-v1.zhs.md`** — 314 lines,
already on `track`, specifying the query interface, the mandatory obligation fields, snapshot-and-pagination rules,
association rules, and a full acceptance-case catalogue with fixtures. **It is a complete contract with no
implementation.** Its own header says it is not a delivery and created no dispatch.

**Its key design decisions are already made and are the right ones**: obligations come from `acceptances`,
`dispatches` and `operator_decisions`; a pure read must not write events, mutate state, or mint dispatches; the
snapshot is a short-lived in-process immutable result, no table, no persistence.

### The smallest next slice

**Not the read model.** The smallest slice that closes the measured problem is **one command that lists the
operator's obligations from the three existing tables**, in one place, with the source named for each row — because
the contract's first page is exactly that, and the contract's own §3 forbids counting-then-querying as two live
results, which a single `select` trivially satisfies.

**The read model, the UI, and the notification wiring are the next slices and should not be pulled into this one.**

### Owner, dependencies, acceptance

| | |
|---|---|
| owner | **Nimbus** — it is a read path over the ledger, and the contract names the interface |
| dependencies | none beyond the contract itself |
| what the user sees | one command, one list, each row naming its source table and its action |
| what requires human approval | nothing for the slice. **Binding it into the UI and the push channel is a separate decision** |

### Why this is second

**Because the contract has been sitting implemented-but-unbuilt for four days, and the cost of the gap was measured
this week: five rounds of no action on one defect.** The slice is small and it is specified.

---

## Priority 3 — Land the deletion guard, so the operator's Accept is protected where the merge happens

### The measured user problem

**The operator accepted two cards on 11-05 03:43 on the strength of a consequence sentence whose second half is false
in code.** He was told the deletion guard bears the merge-protecting check at click time. Measured again this round:

```
git merge-base --is-ancestor 6e04dae HEAD   ->  NO
grep -c merge_would_delete src-tauri/src/commands/acceptance_cmds.rs   ->  0
```

**So `cmd_accept_delivery` goes straight to `merge_branch_into_base` with no preview.** The gate he runs before
clicking (`check-cards-clickable.sh`) does preview the merge, correctly, since `6dbc3af` — I verified that in both
mutation directions and drove it against a real branch. **But the gate is a check he runs; the guard is what stops the
merge if the check is skipped.** He accepted correctly and the merge deleted nothing, measured. The next Accept runs
without the guard he was told about.

### The existing reusable work

- **The guard is written, verified, and pushed.** `6e04dae` on `packet/nimbus/project-scoped-reads`, at the same
  revision on origin and in the cached gitlab ref, not an ancestor of `track`.
- **I verified it three ways myself**: both mutation directions (short-circuit the exemption filter → 2 of 5 red; move
  the guard after the merge → 2 of 5 red), its discriminating power against real branch names which none of its five
  tests exercise, and the real-card safety check — the two accepted cards' merge is 24 added / 17 modified / **0
  deleted**.
- **Flux has a ready dispatch to verify it independently** — `disp-lZFLGyKjN57fytGvxDfOE`, issued 11-05 19:16,
  *Independently verify the accept-time deletion guard and its current deployment boundary*. It has been `ready` for
  eight hours.

### The smallest next slice

**Two steps, and the first is not a merge:**

1. **Flux completes his verification** — the dispatch is written, ready, and eight hours old. That is the
   independent verdict the card-opening gate requires.
2. **Then a card, then the merge** — which is the operator's Accept, not a seat's.

### Owner, dependencies, acceptance

| | |
|---|---|
| owner | **Flux** for the verdict; the **operator** for the merge |
| dependencies | Flux's verdict must exist before a card can be opened, because `request_acceptance` requires a `flux_verdict_ref` |
| what the user sees | `merge_would_delete` present in `acceptance_cmds.rs`, refusing a destructive merge naming branch, base, count and paths, and letting the two real cards through |
| what requires human approval | **the merge of `6e04dae` into `track`.** It is another seat's branch onto the integration line |

### Why this is third and not first

**Because it is the only one of the three whose next step is not a seat's.** Flux's verdict is a seat's; the merge is
not. Priorities 1 and 2 can move this afternoon. This one moves when he says so.

---

## What I am explicitly not proposing, and why

**Not the clickability criterion change.** It is approved (`dec-clickable-criterion-level-with-base`, resolved 11-05
03:43:40) and implemented in the gate (`6dbc3af`, verified both directions by me). What remains is rule 2's wording on
`track` and the hook source-of-truth, both of which are Aegis's instruments and both of which he is actively working.

**Not a redesign of anything.** The dispatch said preserve existing design and components, and all three slices are
additive: a dry-run flag, one read command, one merge.

**Not anything routed back to Aegis for routine instrument edits.** The dispatch said not to, and his queue is where
the hook source-of-truth already sits.

## What I did not do

I did not merge, push, edit product code or any script, accept, cancel, or re-open any card, restart any application,
or write to the live database. I did not open a card or emit a verdict. My measurements were reads: `git log` and
`merge-base --is-ancestor` on `track`, the dispatch and card and decision rows, `seats` and
`project_role_bindings`, `tmux list-sessions`, `stat` and `lsof` on the running binary, `grep` on
`acceptance_cmds.rs`, and the four product-fix commits' ancestry. The ledger check is exit 0, `seats` is 10, and
`canonical_events` is **46**.
