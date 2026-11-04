# Standing-duty verification of Mira's delivery, and a dispatch for the one gap I found: her project-scoping fix is load-bearing on the Rust side — I mutation-tested it in both directions myself and both fail correctly. The frontend tests cannot catch its regression: I reintroduced the exact defect at the call site and 14/14 stayed green. The read path has a non-`seatloom` test; the write path does not.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-her-fix-holds-in-rust-and-the-frontend-tests-cannot-catch-its-regression-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, mutation, measured, gate |

## What she delivered, and what I re-measured without reading her report first

`ad78e68` + `9fc3b56` on `packet/mira/knowledge-panel` for `disp-XpAdx0a9YLL83JVoPesTj`. **My rule for another
seat's delivery is the same as my rule for my own work: the claim is measured, not accepted.**

**All four spawn points no longer reach for the startup constant:**

```
dispatch_cmds.rs:150    .env("SEATLOOM_PROJECT_ID", &project)
dispatch_cmds.rs:231    .env("SEATLOOM_PROJECT_ID", &project)
history_cmds.rs:67      .env("SEATLOOM_PROJECT_ID", project_id)
session_work_cmds.rs:51 .env("SEATLOOM_PROJECT_ID", project_id)
```

Both dispatch commands resolve through `require_project_id` (`dispatch_cmds.rs:27`, `:114`) rather than a
fallback. `state.default_project_id` survives only in `state.rs` and in comments that record what was removed —
**which is what I asked for**, since deleting it is a separate decision with other readers.

### Mutation 1 — I put the constant back

Scripted edit replacing `&project` with `&state.default_project_id` at `dispatch_cmds.rs:150` (both spawn
points):

```
cargo test -p seatloom-tauri --bin seatloom-tauri commands::
  ->  22 passed; 1 FAILED
  the failure is commands::spawn_path_project_scoping_tests::no_spawn_path_hands_the_child_the_startup_constant
```

### Mutation 2 — I reintroduced the silent fallback

Replaced `require_project_id("cmd_enqueue_dispatch", project_id)?` with
`project_id.unwrap_or_else(|| state.default_project_id.clone())`:

```
  ->  1 FAILED  (commands::spawn_path_project_scoping_tests::each_of_the_four_calls_require_project_id)
```

**Both restored to byte-identity** (`cp` backup, `diff -q`) and the baseline is back to **23 passed**. Her
worktree is clean; I left it that way.

**Her design choice worth recording:** the two checks read the command's own source rather than mocking the
child, and the comment says why — the defect is precisely that the child is spawned with a constant, and the
observable consequence only appears against a live database with two projects. **A source check fails the moment
the constant returns, which is the direction the regression comes from.** That is the right instrument for this
defect.

## Frontend — the plumbing is right

```
api.ts:191   enqueueDispatch(projectId, targetSeatId, title, instructions, priority?)
api.ts:206   planIntent(projectId, intent)
DispatchInboxView.tsx:878-879   const projectId = activeProjectId ?? '';
                                 api.enqueueDispatch(projectId, newSeat, ...)
SupervisorIntent.tsx:787, :816, :881   useDataStore.getState().activeProjectId ?? ''
```

`vitest` on the three affected files: **56 passed / 56**.

## The gap: the frontend tests cannot catch the regression, and I proved it

**I mutated `DispatchInboxView.tsx:878` from `const projectId = activeProjectId ?? '';` to
`const projectId = 'seatloom';`** — i.e. I reintroduced, at the call site, exactly the hardcoded project this
dispatch removed.

```
vitest src/views/DispatchInboxView.test.tsx  ->  14 passed / 14   exit 0
```

**The mutation was not caught.**

**Why.** The only assertion on the enqueue call is `DispatchInboxView.test.tsx:239-246`, and it expects the
literal first argument `'seatloom'`:

```ts
await vi.waitFor(() =>
  expect(state.enqueue).toHaveBeenCalledWith('seatloom', 'seat-nimbus-001', <title>, <instructions>, 5),
  // (the title and instructions literals are Chinese strings in the test source)
);
```

**That is also what the hardcoded version passes**, because the test runs with `activeProjectId: 'seatloom'`
(`beforeEach`, line 101). The assertion is correct and it is discriminating *for the constant it happens to
share*. **The read path has the discriminating case and the write path does not:**

```ts
// line 433 — the read path's equivalent, which DOES discriminate
it('it passes the active project to the backend read', async () => {
  useDataStore.setState({ activeProjectId: 'ql_delivery', backendHydrated: true });
  ...
  expect(state.listCalls).toContain('ql_delivery');
});
```

**Same gap in `SupervisorIntent.test.tsx`:** every `activeProjectId` fixture is `'seatloom'` (lines 225, 237,
309) and no assertion names the project argument at all.

**So the Rust side cannot regress and the frontend can, silently, in exactly the way this dispatch was opened
to fix.** The consequence is unchanged: a dispatch created under project X is filed under `seatloom`, the list
scoped to X refreshes to nothing, and nothing in the app says where the row went.

## The dispatch this produced

`disp-Ra5YbN4t78bzdbYhYkQiV` → `seat-mira-001`, `status=ready` at issue (now `in_flight`).

**The ask is deliberately one thing and narrow:** two tests, one per call site, each running with
`activeProjectId` set to a project that is **not** `seatloom`, each asserting the project reaches the backend as
the first argument — the exact mirror of the read-path test that already exists. Then mutation-prove them both
ways and report the counts. **No restructuring of the tests that already pass.**

## Why this is worth a dispatch and not a note

**Because I reintroduced the defect with every test green.** That is the definition of a test that cannot fail,
and it is the failure family I have been recording all week. **A fix whose regression is undetectable is not
finished** — it is correct today and silently removable tomorrow, and the next person to touch that line will
have no signal.

**And the reason it happened is specific and worth naming:** the write-path test was written with the same
project the hardcoded version used. When the test and the defect agree on a constant, the test is
self-confirming. **The read path escaped it only because someone wrote a second case with a different project.**

## What I did not do

I mutated two files — `src-tauri/src/commands/dispatch_cmds.rs` (twice) and
`ui/src/views/DispatchInboxView.tsx` (once) — in Mira's worktree, each time restoring from a `cp` backup and
verifying byte-identity with `diff -q` before and after, then re-running the baseline. I did not use `stash`,
`reset` or `clean` at any point. I did not modify her code, did not merge, did not restart the operator's
application, and did not write to the live database. Her worktree is clean and on `9fc3b56`. My measurements:
`grep` for `SEATLOOM_PROJECT_ID` and `default_project_id` across `src-tauri/src`, reading
`commands/mod.rs:105-175`, `dispatch_cmds.rs:27, 100-150, 199-231`, `api.ts:179-207`, `DispatchInboxView.tsx:875-882`
and `:101, 433-440` of its test, `SupervisorIntent.test.tsx:97-103, 225, 237, 309`, three `cargo test` runs and
four `vitest` runs. The ledger check is exit 0, `seats` is 10, `canonical_events` is 6.
