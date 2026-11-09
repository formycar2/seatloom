# Standing duty: **Mira is active again and I verified her newest work rather than assuming it from the ledger. Her `StaleReadySection` surfaces the operator's third obligation — dispatches that are `ready` and have sat unclaimed 30+ minutes — and her central judgment is the thing worth recording: rendering a stale dispatch as work waiting for the operator would make his number wrong, because the waker stopped and the dispatches cannot be pushed, which is a system problem and not his task. I verified every load-bearing claim in her commit, and her tests pass 10/10 including one that states what it does not prove.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-mira-s-third-obligation-and-her-judgment-that-it-is-not-his-work-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 491 → 497 events and 98 → 99 dispatches.** `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

**Mira completed `disp-bouEIM-49hK0xR_-B8EMm` at 13:12 UTC (21:12 CST)** — the first activity from her in the period I have been watching.

## Her work, and what I verified rather than relayed

**Her commit `5810c7e` "the operator's third obligation reaches the interface (stale ready dispatches)", 535 insertions across 5 files, worktree clean.**

| her claim | my measurement |
|---|---|
| `DispatchDto` already carries `issued_at` | **`src-tauri/src/dto.rs:333: pub issued_at: String`** |
| `cmd_list_dispatches` already takes `project_id` | stated in her comment; the component makes no backend call at all |
| the section is mounted on the dispatch page | **`DispatchInboxView.tsx` imports and renders it** |
| the staleness decision is UI-side | **`ui/src/lib/staleReady.ts`, 31 lines, no IPC** |
| tests cover three directions | **10 tests, 10 passed** |

**And the implementation is more careful than the commit message required:**

| case | behaviour |
|---|---|
| `ready` and older than 30 min | stale |
| `ready` and 5 min old | **not** stale |
| exactly at the threshold | **not** stale; one minute past is |
| `in_flight`, however old | never stale — somebody picked it up |
| `done` | never stale — it finished |
| `blocked` | never stale — waiting on a declared dependency, "not the same failure as nobody looking at it" |
| malformed `issued_at` | not stale |

**"Age alone never qualifies a row" is the right rule, and it is stated in the code.** A dispatch that is old because it is `blocked` on a dependency is not a stalled dispatch, and conflating them would produce a number that is wrong in the other direction.

## The judgment, which is the substantive part

**Her commit message:**

> The framing is copied from `notify-operator.sh`, not improved on: the waker stopped, the dispatches cannot be pushed, this is a system problem and not the operator's task. **Rendering a stale dispatch as work waiting for him would make his number wrong** — the same defect class 020 fixed by separating the acceptance count from the decision count.

**I verified the premise rather than taking it:**

| check | measurement |
|---|---|
| does `notify-operator.sh` frame it that way | **yes** — line 168: "waker stopped, dispatches cannot be pushed. **This is a system problem, not your thing**" |
| is the 30-minute threshold the same | **yes** — line 166 queries `status='ready'` with the same threshold |
| are there stale dispatches right now | **no — zero `ready` rows at all**, so the empty state is what a user sees |

**So the empty state is live, and its wording is the thing that matters**: "Every ready dispatch has somebody moving on it. **This is not 'no work', it is 'nothing stuck'.**" A blank section would read as a broken query; a section that says something true reads as a working instrument.

**And her judgment connects to a defect class this fleet has actually hit**: separating a count that belongs to the operator from a count that belongs to the system. **A seat that surfaces a system failure as the operator's to-do list is inflating his workload with things he cannot act on**, which is the same shape as reporting a problem that is not there.

## The test I want noted, because of what it refuses to claim

**One of her ten tests is named:**

> `a malformed issued_at is not stale — and this test does NOT prove the guard works`

**That is a test that states the limit of its own evidence.** A malformed timestamp producing "not stale" is indistinguishable from the guard silently failing to parse, and the test says so instead of letting a green check imply coverage it does not have.

**That is the fourth instrument this week I have seen refuse to over-claim** — Flux's verdict that listed what it did not cover, the fixture that named its own inability to produce the case, the hook that reports when a rule is not applying, and now this. **A fleet whose instruments state their own limits is a fleet whose green results mean something.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the fleet state and her dispatch, `git log`/`git status`/`git show` over her branch, `grep` over `dto.rs`, `DispatchInboxView.tsx`, `StaleReadySection.tsx` and `staleReady.ts` for the claims, `npx vitest run` on her test file, and `psql` for the live `ready` count and status distribution. **I did not merge her branch or open a card for it** — she has no verdict naming her dispatch, and opening one is not mine to do on her behalf.
