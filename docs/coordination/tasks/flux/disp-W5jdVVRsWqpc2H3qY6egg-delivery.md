# Delivery: state memory + timeline filter + two fakes — FAIL verdict, item 1 does not restore

| Field | Value |
|---|---|
| dispatch_id | disp-W5jdVVRsWqpc2H3qY6egg |
| verified | disp-DFeH6Ut_aNfzIbuuR21V5 (AEGIS-2026-10-20-ui-memory-and-desync-v1) |
| verdict | **fail** |
| verdict event | ev-mkwI3idecZKlZBGXOoC31 (ReviewVerdictIssued, emitted) |
| evidence | docs/coordination/tasks/flux/FLUX-2026-10-21-ui-memory-desync-verdict-v1.md |
| branch | packet/flux/intent-verdict @ 1a5bd05 |
| tested at | packet/mira/knowledge-panel @ dbb51e7 |

## Verdict: FAIL — one item, and it is the item the dispatch is named after

Item 2 and item 3 pass on mutation evidence. **Item 1 does not restart into the last project,**
which is the packet's first acceptance criterion and the headline of the dispatch. Measured against
the real `App`, not inferred.

## The finding

```
recorded 'ql_delivery'  →  appStore: ql_delivery → p-1 → p-1 → seatloom
```

1. `useDataStore`'s initial `activeProjectId` is **`'p-1'`** — the first row of
   `SEED_PROJECTS_FOR_TESTS`, a project in no database.
2. `App.tsx`'s **write** effect (lines 260-271) runs first and persists that value, overwriting the
   recorded project **before** the **restore** effect (lines 101-116) reads `persistedProjectId`.
3. The launch lands on the default, and the record left on disk is `seatloom`, so the next launch
   starts there again — the position is lost **permanently**, not just once.

The `'p-1'` default is pre-existing on `track` (verified at track and at `21c9974^`), but this
dispatch is the one that promised to make the record work, and it does not.

## What did verifiably pass

- **Item 2a** — the dead switch is real. `hiddenTypes` is consulted by the filter memo and cleared
  through the same setter by `clearFilters`/reset. `matchesVisibility = true` kills 6 assertions; a
  full revert to `track`'s `TimelineView.tsx` kills 10 of 12.
- **Item 2b** — default collapses the three named types and names what it hid on screen
  (`已收起 3 类共 3 条` + per-type chips + `全部展开`). Emptying the default kills 8 assertions.
- **Item 2c** — remembered per project (`seatloom.timeline.visibility.<projectId>`), with A's key
  and B's key inspected separately. Dropping the project from the key kills 3 assertions.
- **Item 2 robustness** (beyond the packet): a throwing `localStorage.getItem` still renders the
  collapsed default; a throwing `setItem` still toggles and re-filters; corrupt and non-string
  values fall back rather than throwing or coercing.
- **Item 3a** — `MOCK_PROCESSES` gone, the dialog says `暂无数据` and names the missing capability
  and the tmux alternative, and the confirm button is disabled with `接入并同步 (未实现)`.
- **Item 3b** — `AppV2.tsx` deleted with no reference left; `mock-data.ts` exports exactly one
  constant; a re-added importer of `MOCK_GLOBAL_SUMMARY` is refused by `tsc` (`TS2305`).

## Gates, run by me in Mira's worktree

`tsc --noEmit` clean · `vitest run` **239/239** (19 files) · `vite build` green. `NODE_OPTIONS`
still points at a deleted `.agentd` loader, so every node invocation ran with only that stale
preload unset.

Lint measured file-by-file against `/private/tmp/lwtM` (on `track/infra-foundation`):

| file | eslint track → branch | prettier track → branch |
|---|---|---|
| App.tsx | 15 → 15 | clean → clean |
| useAppStore.ts | 0 → 0 | FAIL → **clean** |
| AttachSessionDialog.tsx | 0 → 0 | FAIL → **clean** |
| app-v2/mock-data.ts | 0 → 0 | FAIL → **clean** |
| TimelineView.tsx | 10 → 9 | clean → clean |
| TimelineVisibility.test.tsx (new) | — | clean |

The delivery note's table reproduces exactly, including `TimelineView.tsx 10/9`. Nothing regressed;
four files improved.

## Boundary

No code, backend, DB, or merge change by me. All 23 probe assertions were in throwaway files that
I deleted; `git status` and `git diff` are empty in Mira's worktree, which is byte-identical to
`dbb51e7`. One probe correction is recorded rather than hidden: my first `AttachSessionDialog`
mutation passed, so I added the assertion that was missing before drawing any conclusion.

## Artifact index

- Verdict: `docs/coordination/tasks/flux/FLUX-2026-10-21-ui-memory-desync-verdict-v1.md`
- Ledger event: `ev-mkwI3idecZKlZBGXOoC31`
- This delivery note.
