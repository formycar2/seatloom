# Standing-duty measurement: the second card's acceptance steps are all runnable in the sandbox as built, even though the sandbox predates the correction — because the correction is a test-file change and tests are not compiled into the app. What the sandbox genuinely lacks is a fresh build, and that is a build the operator should order rather than something I arrange.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-both-cards-steps-are-runnable-as-built-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, measured, live, gate |

## Why I measured this

Last round I recorded that the sandbox is built from `f26466f` while the branch tip is now `cac5ca1`, so **the
sandbox does not contain the corrected comment.** I left the consequence unmeasured, which is the same
half-finished claim I have been catching all week. **This round measures what that actually costs the operator.**

## The three commits, and what each one changed

| commit | what it is | what it changes |
|---|---|---|
| `f26466f` | merge of track into Mira's branch | the product fix — four write paths project-scoped |
| `4597dec` | Mira's delivery note | one markdown file |
| `cac5ca1` | the false-comment correction | **one test file**, `ui/src/views/DispatchInboxView.test.tsx` |

**`cac5ca1` touches only a test file.** Measured: `git show --stat cac5ca1` → 1 file changed, 21 insertions,
3 deletions, all in `DispatchInboxView.test.tsx`. **No product code.**

## What that means for the built app

**Tests are not compiled into the application.** Measured two ways: the vite/tauri build excludes `*.test.tsx`
by default, and the sandbox binary carries zero test-framework strings (`toHaveLength` / `vitest` → 0
occurrences in `strings`).

**So the correction `cac5ca1` cannot be in the sandbox binary, and its absence there costs the operator
nothing.** Both cards' steps are product behaviour, and the product behaviour was fixed in `f26466f`.

## Each card's steps, checked against what the sandbox actually contains

### Card 1 — `acc-disp-XpAdx0a9YLL83JVoPesTj` (the four write paths)

Its steps are the banner, switch to `ql_delivery`, create a dispatch, look at the list, the negative case, and
the `seatloom` regression. **All of it is product behaviour that exists in `f26466f`** — I read
`dispatch_cmds.rs` at that commit and both spawn sites already pass `&project`.

### Card 2 — `acc-disp-Ra5YbN4t78bzdbYhYkQiV` (the two frontend cases)

Its distinctive step is the supervisor panel (`⌘K`) path: say something, confirm it plans `ql_delivery`, and
watch where the dispatch lands. **That calls `cmd_plan_intent`, which is one of the four spawn sites.**

Measured at `f26466f`:

```
src-tauri/src/commands/dispatch_cmds.rs:150   .env("SEATLOOM_PROJECT_ID", &project)   <- cmd_enqueue_dispatch
src-tauri/src/commands/dispatch_cmds.rs:~231  .env("SEATLOOM_PROJECT_ID", &project)   <- cmd_plan_intent
ui/src/app-v2/panel/SupervisorIntent.tsx:787  api.planIntent(useDataStore.getState().activeProjectId ?? '', t)
```

**Both ends fixed at that commit.** So card 2's distinctive step is runnable in the sandbox too.

**And the card says so itself, in its own `preconditions`:** *this card's user-visible behaviour overlaps card 1;
the difference is that it also covers the supervisor-panel path, and what it brings is test coverage — not
visible in the app, but it means a hardcoded regression would be caught.* **That is an honest card. It tells the
operator that what he is clicking proves less than the card's title suggests, and that the rest is invisible to
him.**

## The sandbox itself, verified against the running process

```
pid               3794  /Applications/SeatLoom-Sandbox.app/.../seatloom-sandbox
SEATLOOM_SANDBOX=1
SEATLOOM_SANDBOX_BRANCH=packet/mira/knowledge-panel
SEATLOOM_SANDBOX_COMMIT=f26466f
DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_sandbox
main app           pid 90805, untouched
```

**The sandbox names its own branch and commit, and both are the ones the cards name.** `ql_delivery` with its
`seat-ql-001` binding exists in `seatloom_sandbox`, so step 2 of both cards is executable.

## What the operator is actually being asked to accept, stated plainly

**Card 1** — a product fix, present in the binary in front of him, verifiable by four clicks.

**Card 2** — the same product fix plus one more path (the supervisor panel), also present and verifiable; **and a
test-coverage improvement he cannot see, which the sandbox does not contain and which cannot be contained by any
app build.** Its verdict also carries a finding about a comment that has since been corrected, which is why
`check-verdict-defects.sh` is red.

**So the honest summary is: card 1 is ready to accept. Card 2's clickable part is ready, and its invisible part
is both absent from the sandbox and already superseded by a correction on a newer commit.**

## The one thing I am recommending rather than doing

**If the operator wants card 2 accepted against its current state, the sandbox should be rebuilt from the branch
tip** — one command, and it is the same command already on the card:

```
bash scripts/prepare-sandbox-build.sh packet/mira/knowledge-panel --apply
```

**I am not running it.** It builds and installs an application and starts a process, which is the class of action
I stopped taking unsupervised after I destroyed the live database. **The current sandbox is good enough for every
step on both cards, so a rebuild is an improvement in tidiness rather than a prerequisite** — and I would rather
say that plainly than quietly rebuild something the operator is looking at.

## What I did not do

I did not rebuild, install, or launch anything. I did not modify any card, verdict, or event. I did not merge,
push, or write to the live database. I did not touch any seat's worktree. My measurements were all reads: `git
show --stat` and `git show <commit>:<path>` for the three commits, `strings` on the sandbox binary,
`pgrep`/`ps eww` for the running sandbox's environment, `psql` for both cards' `acceptance_plan` and for
`ql_delivery` in `seatloom_sandbox`, and `grep` for the call sites. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 20.
