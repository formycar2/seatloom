# Standing-duty verification of Mira's card-opening delivery: both cards are open, correct, and actually runnable — I verified the sandbox process itself rather than taking her report. One real inconsistency remains: the cards name commit `f26466f` while the branch tip is now `4597dec`, which is her delivery note and not product code, so the acceptance is unaffected but the card's named commit is one behind the branch it points at.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-both-cards-open-and-the-sandbox-is-real-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, measured, live, gate |

## What she delivered, and what I re-measured

`f26466f` (merge track) + `4597dec` (delivery note) on `packet/mira/knowledge-panel`, for
`disp-MQwgyf-XTKRKARKMu_jjR`. **She did everything the dispatch asked and one thing it did not: she ran the
sandbox build.**

### The cards, read out of the database

| card | status | dispatch | verdict ref | base | branch |
|---|---|---|---|---|---|
| `acc-disp-XpAdx0a9YLL83JVoPesTj` | pending | `disp-XpAdx0a9YLL83JVoPesTj` | `ev-flux-verdict-four-write-paths-01` | `track/infra-foundation` | `packet/mira/knowledge-panel` |
| `acc-disp-Ra5YbN4t78bzdbYhYkQiV` | pending | `disp-Ra5YbN4t78bzdbYhYkQiV` | `ev-flux-verdict-two-frontend-cases-02` | `track/infra-foundation` | `packet/mira/knowledge-panel` |

**One card per dispatch, each citing the verdict that names it.** The id-derivation shape
(`acc-<dispatch_id>`) means the card and its dispatch cannot be confused with each other.

### The acceptance plan is written so a person who cannot read code can run it

Read out of `acceptance_plan` for the first card. It opens with the banner step — *look at the top banner, it
names the branch and commit this was built from; the commit this card accepts is that one* — then
`ql_delivery` with an explicit warning that `project_delivery/QL` is **not** the subject because it has no seat
binding in the sandbox database, then the create-a-dispatch step, then what to look at.

**Pass and fail are stated as observations, not as conditions:**

- pass — the dispatch appears in `ql_delivery`'s list and its seat reads **`ql`**, the name, not `seat-ql-001`
- fail — the form submits, the list refreshes, and nothing appears; or it appears grouped under the raw id

**Plus the negative case and the regression case**, both of which I told her were part of the fix rather than
extras: the app must say so when the project cannot be determined instead of filing silently elsewhere, and
`seatloom` must still work.

**And `preconditions` names the build command with the branch name and the reason never to pass a hash, the
commit this card accepts, the `ql_delivery` naming trap, and that this card does not cover the cold-start
acceptance.** That last one matters: two adjacent cards that silently share a scope are how a delivery gets
accepted on evidence about a different delivery.

### The sandbox build is real, verified against the running process

**She ran `--apply` and I verified the result rather than her report:**

```
process          : pid 3794 /Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox
binary built     : 2026-11-04 15:17     (the previous sandbox was 10-21 — seven days stale)
SEATLOOM_SANDBOX=1
SEATLOOM_SANDBOX_BRANCH=packet/mira/knowledge-panel
SEATLOOM_SANDBOX_COMMIT=f26466f
DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_sandbox
SEATLOOM_SANDBOX_DIRTY : absent — the worktree was clean, so nothing to record
main app         : pid 90805, untouched
```

**The environment variables are the strongest evidence available without launching anything myself**: the
running sandbox names its own branch and commit, and both are the ones this card accepts. `ql_delivery` exists in
`seatloom_sandbox` with its `seat-ql-001` binding, so step 2 of the card is executable.

**`SEATLOOM_SANDBOX_DIRTY` being absent is also correct** — the fourth layer (`b5762c4`) is among the ten
commits her branch was behind, so this build predates it; and the worktree was clean, so the clause would have
said "no uncommitted changes" anyway. **A missing variable and a zero count mean different things, and here the
missing one is the truthful one because the installer that built this did not yet export it.**

## The guard firing on her, which she recorded correctly

She reports running the build **from her own worktree** and being refused:

```
not any worktree's currently checked-out branch
refuses to guess from worktrees. name an already-checked-out branch and run again.
```

**She wrote: that refusal is not a bug, the guard is working.** It did not guess; it listed every worktree and
where each was checked out. **Had she passed the hash, it would have built another tree while the banner printed
her hash — indistinguishable.** I verified the refusal myself earlier today and it is the behaviour I asked for.

**This is worth recording as a seat doing the thing right**: she hit a guard that stopped her, investigated why,
and reported it as the guard working rather than as an obstacle. The failure mode I have been documenting all
week is the opposite — a tidy mechanism that explains a failure being preferred over "I do not know."

## The one inconsistency, measured

**The cards name commit `f26466f`. The branch tip is now `4597dec`.**

```
git rev-parse --short HEAD (mira)  ->  4597dec
git log f26466f..4597dec           ->  4597dec  "delivery note — sync + two cards + sandbox build"
```

**`4597dec` adds one file, a markdown delivery note. No product code.** So the tree the card accepts and the tree
at the branch tip are **identical in every file that ships** — the acceptance is unaffected.

**But the card's named commit is now one behind the branch it points at.** The banner step tells the operator to
expect `f26466f` *or the new tip after syncing track*, and her card's own precondition says exactly that, so a
careful operator is covered. **I am recording it rather than correcting it, because correcting it would mean
rewriting a live acceptance card, and the discrepancy is one non-product commit with a written allowance for
it.**

## What this unlocks, and what is still missing

**The chain is now complete up to the operator:**

```
dispatch done -> verdict pass -> card open and clickable -> sandbox built from the named commit -> OPERATOR
```

**Two cards are pending in front of the operator, each with a build that exists.** That is the first time today
the operator has something to click that is backed by a real binary.

**What is still missing, unchanged:** the cold-start acceptance has no verdict and no card, so it remains
unverifiable; the three live-only projects are still absent from the sandbox database; and the track-side defect
is still live because neither implementation commit is on track.

## What I did not do

I did not open, modify or accept any card. I did not build, install, or launch anything — the sandbox was already
running when I measured it, and reading a process's environment is not starting it. I did not merge, push, or
write to the live database. I did not touch Mira's worktree. My measurements were reads: `psql` for the two cards
and their `acceptance_plan`, `git log`/`git rev-parse` for the two commits and their diff, `pgrep` and `ps eww`
for the running sandbox's environment, `stat` on both binaries, `strings` on both (which I attempted as a
discriminator and abandoned when it did not discriminate — the build time and the process environment did), and
`psql` against `seatloom_sandbox` for the `ql_delivery` row. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 18.
