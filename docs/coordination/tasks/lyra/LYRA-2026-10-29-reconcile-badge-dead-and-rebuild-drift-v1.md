# Standing-duty finding: the reconcile badge is a dead control counting the wrong thing, and a fresh rebuild still is not live

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-reconcile-badge-dead-and-rebuild-drift-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, product-truth, ui, rebuild, blocking |

## Facts

### 1. The reconcile notification can never appear, and it counts the wrong thing anyway

`ui/src/App.tsx:385` declares `showReconcileSummary` and `setShowReconcileSummary`. The state is
**never set to `true` anywhere in the file**:

```
$ grep -n 'setShowReconcileSummary(true)' ui/src/App.tsx
exit=1     # no match
```

The only two writes are `setShowReconcileSummary(false)` at lines 886 and 889, both inside the
`onClose`/`onViewInbox` handlers of the component itself. So the guard at line 884 —
`showReconcileSummary && activeTab !== 'all-projects'` — is a conjunction whose first conjunct is
permanently `false`. **`ReconcileNotification` is unreachable.** It cannot render for any input.

And the number it would have shown is not a reconcile count. `App.tsx:387`:

```ts
const reconcileIssueCount = useMemo(
  () => currentProjectData?.workItems.filter((w) => w.status === 'Blocked').length ?? 0,
```

That is the count of **Blocked workitems** — an unrelated quantity wearing a reconcile-shaped name.
Live currently has `done|7` and no `Blocked` rows at all, so even if the control were reachable it
would render `count={0}`.

**This is Mira's standing-duty check #2** (every enabled control does something) **and check #3**
(an empty state that says why it is empty) both failing on one element, in the product the operator
opens every day. It is the defect class he named himself on 2026-10-16 — *does clicking it do what it
says* — except this one does not even need clicking to be wrong.

### 2. `run_reconcile` is wired and reachable; the document corpus it maintains is invisible

I first measured this wrong and am recording the correction. I grepped `crates/*/src/bin/` and
`crates/seatloom-mcp-emit/` only, found no call site, and concluded `run_reconcile` was dead code.
**It is not.** Grepping the workspace:

```
./src-cli/src/main.rs:69:            match run_reconcile(&pool, ReconcileTrigger::Manual, ...)
./src-tauri/src/commands/reconcile_cmds.rs:16:    run_reconcile(&pool, ReconcileTrigger::Manual, ...)
```

It is reachable from the CLI and from a registered Tauri command. What is actually missing is the
**route from the UI to it**: `ui/src/app-v2/views/DocumentsWorkspace.tsx` exists and has a
`reconcileResult` state, but it is imported by nothing except its own test file:

```
$ grep -rn "DocumentsWorkspace" ui/src --include=*.tsx | grep -v test
# (no output — only its own definition)
```

And `ui/src/App.tsx` has no `documents` nav entry. So the document surface is built, tested, and
unreachable, while the one control that mentions reconcile is dead. **The product has a document
feature that no user can open and a reconcile badge that no user can see.**

### 3. A fresh rebuild with the 021→011 fix converges on live's schema, and diverges on data

With 021 moved after 011, all 14 post-seed migrations apply cleanly and the table count matches live
exactly:

```
tables: rebuild=32  live=32     (identical set)
v_memory_session_join: builds, 0 rows
```

Against that, the row counts:

```
  SAME  sessions=5          artifacts=15     operator_decisions=0
  DIFF  seats:        rebuild=6   live=10
  DIFF  projects:      rebuild=38  live=40
  DIFF  workitems:     rebuild=6   live=7
  DIFF  archived_sessions: rebuild=183 live=272
  DIFF  documents:     rebuild=14  live=0
  DIFF  canonical_events: rebuild=34 live=2497
  DIFF  dispatches: rebuild=0 live=352    acceptances: rebuild=0 live=62
  DIFF  memories:   rebuild=0 live=391    topics:       rebuild=0 live=51
  DIFF  project_role_bindings: rebuild=6 live=10
```

Two of these are expected and I am labelling them as such rather than as defects: `canonical_events`,
`dispatches`, `acceptances`, `memories` and `topics` are **the work itself** — a rebuild cannot
reproduce 2497 events that happened after the seed was written, and it should not. `archived_sessions`
is 183 vs 272 because migration 021 inserts the 183-row corpus and the other 89 are inventory-only
rows created by a later path.

Two are **not** explained by that, and both are user-visible in principle:

- **`documents`: rebuild=14, live=0.** No migration deletes documents — `grep -rn "TRUNCATE\|DELETE
  FROM documents"` over `infra/postgres/` returns nothing, and the only `DELETE FROM documents` in the
  workspace is test-fixture teardown in `dispatch.rs:3485`. All 14 files the seed names **exist on
  disk today** (I checked each path). So the seed is truthful and live has no rows: the document
  corpus was never applied to live, or was applied and lost, and **nothing in the product notices**
  because no UI reads `documents`. `list_documents` exists in `repositories.rs:2445` and has no
  caller outside its own definition.
- **`seats`: rebuild=6, live=10**, with every live seat at `Pi` /
  `stepplan-wash-anthropic/step-5-preview` and every rebuilt seat at `Stepcode` /
  `claude-opus-5[1m]`. This is the drift Onyx's 005 was written to close. **I verified it is not
  user-visible**: `defaultHarness` is declared in `ui/src/lib/types-dto.ts:29` and crosses the wire,
  but `grep -rn "defaultHarness" ui/src` outside `types-dto` returns only test fixtures. Nothing
  renders it. The `live-seats` tab reads `cmd_list_tmux_sessions`, not the `seats` table. So the seat
  row count and harness values are **schema truth, not product truth** — which is exactly why nobody
  has noticed.

## Decisions

1. **The dead reconcile badge is Mira's, not mine.** It is her standing-duty check #2 catching a real
   defect on a real surface. I am recording the measurement and handing it over rather than editing
   `App.tsx`, because Mira owns the interface and I do not.
2. **`documents` 14-vs-0 is the more serious one** and it needs a decision about *what the document
   corpus is for* before anyone fixes the count. If the document surface is meant to be reachable,
   then `DocumentsWorkspace` needs a route and `list_documents` needs a caller, and live's zero rows
   become a visible hole. If it is not meant to be reachable, then it should be said plainly rather
   than left as a built, tested, unrouted view. **That is a product decision and it is mine to bring
   to the operator, not to make quietly.**
3. **The rebuild's data divergence is expected for work tables and is NOT expected for `documents`.**
   I am separating those two explicitly, because reporting "rebuild differs from live" as a single
   finding would be the same mistake Aegis made in `disp-4fvDHLSSpZthngAsr6kZM` — he measured his own
   imagined version of a claim instead of the claim.

## Correction I owe the record

In §2 I state that my first measurement of `run_reconcile` was wrong. I am keeping that in the
document rather than deleting it, because the error came from grepping two directories and
concluding absence — the same "looked right, was actually a guess" family this project has already
paid for, and the record is more useful with the miss in it.
