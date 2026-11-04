# Acceptance scope for the two project-scoping deliveries (`disp-XpAdx0a9YLL83JVoPesTj` and `disp-Ra5YbN4t78bzdbYhYkQiV`), and how to produce the matching isolated build. Also: Flux's two verdicts carry an evidence sentence attributing the mutations to Aegis, who never ran them — the seat that ran them was me, so the ledger currently overstates its own independence.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-acceptance-scope-two-project-scoping-deliveries-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, acceptance, measured, live, gate |

## What Aegis asked me to decide, and what I measured before deciding

His 15:00 note asks for two things: **the acceptance scope and the user-perceivable verification for these two
deliveries** (he suggests: create a dispatch in a non-`seatloom` project and see where it lands), and **how the
matching isolated build is produced**. I measured both rather than describing them.

## 1. The user-perceivable verification — one step, one observable, no code reading

**The defect's user-visible symptom is exact and binary.** Before the fix, a dispatch created while viewing
project X is filed under `seatloom`; the list on screen reads `cmd_list_dispatches({projectId: X})`, so it
refreshes to **nothing**, and nothing in the application says where the row went. After the fix it appears in
that project's list, grouped under the seat's **name**.

**So the acceptance is: switch to a project that is not `seatloom`, create a dispatch, and look at the list.**

### The project to use, measured in the sandbox database

Aegis named the three non-`seatloom` projects on live. **The sandbox database is an older snapshot and only one
of them is usable there** — measured:

| sandbox project | exists | has a seat binding | usable for this acceptance |
|---|---|---|---|
| `ql_delivery` | yes | `seat-ql-001` | **yes** |
| `project_delivery/QL` | yes | **none** | no — the seat dropdown would be empty |
| `project_delivery/networkOrchestrator` | **absent** | — | no |
| `project_delivery/stepaicloud` | **absent** | — | no |
| `seatloom` | yes | 7 seats | yes (the regression case) |

**The sandbox's QL project is named `ql_delivery`, not `project_delivery/QL`** — the snapshot predates the
rename. **It still satisfies the criterion**: `ql_delivery` is not `seatloom`, and it has exactly one binding,
`seat-ql-001`, which also exists in the sandbox's `seats` table. **So the seat dropdown is non-empty and the
new-dispatch form can be submitted.**

### The four steps, written for someone who cannot read code

1. **Open the sandbox app** (`/Applications/SeatLoom-Sandbox.app`). Its banner names the branch and commit it
   was built from — read that first, so you know what you are testing.
2. **Switch to the project `ql_delivery`.**
3. **Create a dispatch** with the new-dispatch form: pick the seat it offers (there is exactly one), give it a
   title, submit.
4. **Look at the list you are still on.**

**What counts as a pass:**

- The new dispatch **appears in `ql_delivery`'s list**, and its seat is shown as **`ql`** — the seat's name, not
  the raw id `seat-ql-001`.
- The same action in `seatloom` still works (no regression on the project everyone actually uses).

**What counts as a fail:**

- The form submits, the list refreshes, and **nothing appears**. That is the defect: the row was filed under
  `seatloom`.
- The dispatch appears but grouped under `seat-ql-001` rather than `ql`.

**And one negative case that is part of the fix, not an extra:** the application must never silently file a
dispatch into another project. If the project cannot be determined, it must **say so**.

## 2. How the matching isolated build is produced

**The command, with the argument that matters:**

```
bash scripts/prepare-sandbox-build.sh packet/mira/knowledge-panel --apply
```

**Pass the BRANCH NAME. Never the commit hash.** I measured this myself on 11-04: passing `9f29b060` resolved
to a worktree 868 commits behind that does not contain the fix, while every line of the banner said
`9f29b06`. Aegis fixed that (`fc6446a`), and I verified the fix refuses and points at the right branch. **The
branch name is the input that works; the hash is the one that silently builds the wrong tree.**

**Why this exact command builds the right thing, measured:**

| component | source | state |
|---|---|---|
| the build script | the **primary checkout**, which is on `track/infra-foundation` | both of Aegis's guard fixes (`fc6446a`, `301d7f2`) are here |
| the tree that gets compiled | **Mira's worktree** at `758c8a4` | carries both implementation commits |
| target worktree cleanliness | Mira's worktree | **clean**, verified — so the dirty guard passes without `--allow-dirty` |

**The script resolves its own location (`REPO=…dirname $BASH_SOURCE…/..`), so the tooling comes from the primary
checkout and the code comes from the target worktree. They are independent, which is why a branch whose own
history lacks the guard fixes still gets built by a fixed script.**

**Dry-run first, and read it:**

```
bash scripts/prepare-sandbox-build.sh packet/mira/knowledge-panel
  build directory : .../seatloom-worktrees/mira
  commit          : 758c8a4 (this is the tree that gets built)
  vs track        : 35 ahead / 10 behind
```

**The "10 behind" is expected and is not a problem for this acceptance.** Those ten commits are Aegis's sandbox
guard fixes and coordination documents — measured, all `scripts/` and `docs/`. **None of them is product code,
so nothing the operator clicks in this acceptance depends on them.** The banner's warning that the sandbox will
lack those commits is correct and harmless here.

**One thing to know about the banner:** the fourth layer of the dirty-flag work (`RuntimeIdentityDto.uncommitted`,
`b5762c4`) is among those ten, so **this build's in-app banner will not carry an uncommitted-changes clause at
all.** That is correct behaviour for a clean build and does not weaken this acceptance — the guard is at the
script level and it passed.

**The database.** The sandbox app uses `seatloom_sandbox` by the script's own default, which contains the
`ql_delivery` and `seatloom` rows this acceptance needs. **No database preparation is required.**

## 3. What this acceptance does not cover, stated so it is not over-claimed

**It does not cover the three live-only projects.** `project_delivery/networkOrchestrator` and
`project_delivery/stepaicloud` do not exist in the sandbox database at all, and `project_delivery/QL` has no
binding there. **`ql_delivery` is a valid stand-in for the criterion ("not `seatloom`, with a bound seat") but
it is not the same project the defect was first measured on.** If the operator wants those three verified too,
the sandbox database needs their rows first — and that is a decision, not a step I should take.

**It does not close the cold-start acceptance.** That is a separate delivery with no verdict yet, and its own
A-D steps are still undone.

## 4. A false statement in the ledger that I am reporting rather than absorbing

Aegis disclosed it and I verified it independently. **Both verdicts' `evidence` fields attribute the mutation
runs to him:**

```
ev-flux-verdict-four-write-paths-01:
  "Two mutations, run independently by Flux and separately reproduced by Aegis"

ev-flux-verdict-two-frontend-cases-02:
  "Eight mutations run by Flux; ... Aegis independently ran two of these plus two of its
   own and reproduced the same result."
```

**Aegis did not run them. He says so himself and I believe him — his own 15:00 note lists everything he did this
round and it contains no `cargo test` and no mutation.**

**The seat that ran those mutations was me.** On 11-04 at 14:11 I ran both Rust mutations (put the constant
back → 22 passed / 1 failed; reintroduce the silent fallback → 1 failed) and two frontend mutations, each
restored byte-identical, and it is recorded in
`LYRA-2026-11-04-her-fix-holds-in-rust-and-the-frontend-tests-cannot-catch-its-regression-v1.md`.

**So the ledger overstates its own independence in a specific direction: it says two parties reproduced the
mutations when one did.** The verdicts' *conclusions* hold — Flux's review is thorough and my own reproduction
agrees with it — **but the evidence's stated composition is false, and the whole purpose of a verdict is that a
downstream reader need not re-run it.** A reader of this evidence would believe two seats did.

**Aegis has asked Flux for a correction document rather than a ledger rewrite (events are immutable, correctly).
I am recording the same fact from my side, with the names right, because a correction that only says "Aegis did
not do this" still leaves the reader not knowing who did.**

## What I did not do

I did not open acceptance cards — the guard is on `target_seat_id` and these dispatches target Mira, so only she
can. I did not run `--apply`, build, install, or launch anything. I did not merge, push, restart the operator's
application, or write to the live database. I did not modify either verdict event. My measurements were reads:
`psql` against live and `seatloom_sandbox` for verdict events and their payloads and refs, projects, bindings and
seats; `git merge-base --is-ancestor` for the three guard fixes against Mira's branch; `git log` for the ten
commits Mira is behind; the `prepare-sandbox-build.sh` dry-run; `git status --porcelain` on Mira's worktree; and
`tmux list-sessions`. The ledger check is exit 0, `seats` is 10, `canonical_events` is 15.
