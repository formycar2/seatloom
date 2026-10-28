# Verdict — state memory + timeline filter + two legacy fakes: items 2 and 3 PASS, item 1 FAILS its own headline criterion

- **Verification dispatch**: `disp-W5jdVVRsWqpc2H3qY6egg`.
- **Implementation dispatch**: **`disp-DFeH6Ut_aNfzIbuuR21V5`** (`verify_dispatch_id` in the payload);
  packet `AEGIS-2026-10-20-ui-memory-and-desync-v1`.
- **Branch under test**: `packet/mira/knowledge-panel` @ `dbb51e7` (HEAD; the patch landed in
  `21c9974`, 29 commits earlier — a merge carries the head, so the head is what I tested).
- **Verdict event**: `ev-mkwI3idecZKlZBGXOoC31` — emitted as required.
- **VERDICT: FAIL** — with item 2 (all five sub-claims) and item 3 (both 3a and 3b) verified PASS
  on evidence, and **item 1 failing the packet's own first acceptance criterion by measurement, not
  by opinion**.

## Scope

One dispatch, its own verdict. Nothing was merged in from the V5 verdict
(`disp-RWEf5ATPvICLkGtF__fEY`, `ev-1ZDgqPoVkpeEB7T7O7hgY`). The packet's title correction note is
honoured: this is three items, not the "项目脱节" work named in the dispatch's title field.

## Method — render the real things, then mutate them

23 assertions across four throwaway harnesses (all deleted; `git status`/`git diff` empty at the
end, so Mira's worktree is byte-identical to `dbb51e7`). The harnesses drove the **real**
`TimelineView` with a real jsdom window and a real localStorage double, the **real**
`AttachSessionDialog`, and — for item 1 — the **real `App`**, mocked only at the `api` boundary,
with `useAppStore.persist.rehydrate()` standing in for "a new process reading the last one's
storage". Then I mutated the implementation to prove the assertions bite:

| mutation | result |
|---|---|
| `matchesVisibility = true` (dead switch restored) | **6 assertions FAIL** |
| file reverted to `track`'s pre-patch `TimelineView.tsx` | **10 of 12 FAIL** |
| `COLLAPSED_BY_DEFAULT` emptied (default = show all) | **8 assertions FAIL** |
| the visibility key made global (project dropped) | **3 assertions FAIL** |
| `AttachSessionDialog` renders a plausible `node · pid 4242 · …` row | **1 assertion FAIL** |
| a new file imports `MOCK_GLOBAL_SUMMARY` | **tsc FAILS** — `TS2305: has no exported member` |
| `App.tsx`'s existence guard removed | **1 assertion FAIL** (the finding assertion) |

Every mutation was restored and re-verified byte-identical.

## Item 1 — FAIL: the packet's first acceptance criterion, measured against the real App

The packet's criterion is unambiguous: *"打开 SeatLoom,切到项目 A 的某个标签,选中一条;**关掉应用,
重开** —— 应该回到项目 A 的那个标签、那条选中项"*. A restart **does not** land back on the last
project. Measured, not reasoned about:

```
state.projects = [seatloom, ql_delivery]
recorded activeProjectId = 'ql_delivery'
→ appStore: ql_delivery → p-1 → p-1 → seatloom
→ dataStore.activeProjectId = 'seatloom'   (the DEFAULT, not the recorded one)
```

**The mechanism**, in two parts, both measured:

1. `useDataStore`'s initial `activeProjectId` is **`'p-1'`** — the first row of
   `SEED_PROJECTS_FOR_TESTS`, a project that does not exist in any real database. Its own comment
   says these exist for tests explicitly.
2. The **write** effect (`App.tsx:260-271`) runs on every `activeProjectId` change and calls
   `setActiveProjectId(activeProjectId)` with `useDataStore`'s value. On launch the store is still
   at its `'p-1'` default, so the write effect **overwrites the recorded `'ql_delivery'` with
   `'p-1'`** before the restore effect (`App.tsx:101-116`) reads `persistedProjectId`. The restore
   then reads a value that is no longer the operator's project, and both effects settle on whatever
   hydration chose. The record left on disk is `seatloom`, so the next launch starts there again —
   the position is lost *permanently*, not just once.

**Materiality.** This is not a cosmetic failure. The operator's ruled symptom was that a restart
"总是回到默认" — exactly what still happens; the patch moved the persistence but not the outcome.
And the clobber is one-directional: the recorded id is overwritten with a value that is neither the
recorded project nor the eventual project (`'p-1'`), so a storage read can no longer be trusted to
describe the operator's last position even if the ordering were fixed.

**Pre-existing or introduced?** The `'p-1'` initial value is pre-existing (present on `track` and
on `21c9974^`), but the *patch is what promised to make the record work*, and it does not. That is
the standard the verdict holds it to — the packet's criterion 1 is unmet.

**Not verified either way:** no actual application restart. I cannot open, click or close the app
(the screen is locked from a seat shell; `screencapture` fails and assistive UI scripting is
denied). What I did instead is simulate a relaunch exactly at the layer that decides it — a fresh
rehydration from the same storage, driving the real `App` — which is where the failure is
observable.

## Item 2 — PASS, on all five sub-claims

**(2a) The dead switch is real.** `templateFilter` is gone; `hiddenTypes` is consulted by the
memo (`matchesVisibility = !hiddenTypes.has(event.event_type)`), and `clearFilters`/the reset path
clears it through the same setter. With 5 rows (3 of them memory/topic) only 2 render. Mutating the
memo back to `matchesVisibility = true` kills 6 assertions, and reverting to `track`'s file kills 10.

**(2b) The default collapses the three named types, and it says so on screen.** The collapsed strip
renders `已收起 3 类共 3 条` with the real counts, one chip per type, and `全部展开`. A memory-only
project renders the strip rather than a bare "没有找到匹配的事件" — collapsing is not hiding.
Emptying the default kills 8 assertions.

**(2c) Remembered, per project.** Written to
`seatloom.timeline.visibility.<projectId>`; a stored `[]` restores show-all on the next run, and
project A's choice does not leak into project B (asserted by inspecting **both** keys). Dropping
the project from the key kills 3 assertions.

**(2c-robustness) The storage cannot break the view.** Verified by making `localStorage` itself
throw: a throwing `getItem` still renders the default-collapsed timeline, and a throwing `setItem`
still flips the toggle and re-filters. A corrupt value, and a value of non-strings, both fall back
to the collapsed default rather than throwing or coercing. (These last two are beyond the packet's
stated criteria and are correct behavior.)

## Item 3 — PASS

**3a.** No `MOCK_PROCESSES` survives — grep finds only a comment. The dialog renders
`暂无数据` plus `后端没有可列出的本地进程注册表…`, twice states that no placeholder stands in, names
the tmux mirror as the real capability, and the confirm button is `disabled` with
`接入并同步 (未实现)` and an explanatory title. Reintroducing a plausible fake row kills my
assertion — I had to add that assertion after a first mutation attempt passed, which is the same
gap the V5 verdict recorded; noted so the record shows the probe was corrected rather than the code.

**3b.** `AppV2.tsx` is deleted, no reference to it exists in `ui/src` or `src-tauri/src` outside
three explanatory comments, and `mock-data.ts` exports exactly one constant (`MOCK_CHANNEL_DATA`) —
`MOCK_CONTACTS`, `MOCK_MESSAGES` and `MOCK_GLOBAL_SUMMARY` are gone. Strongest evidence: a probe
that imports the module and asserts the keys are absent, plus a probe that re-adds an importer and
is stopped by `tsc` (`TS2305`). The remaining `MOCK_CHANNEL_DATA` read is a no-op for every real
project (`channelId` is `ch-<real id>`, the keys are `ch-p1/p2/p3`), verified against its producer.

## Gates, run by me in Mira's worktree

`tsc --noEmit` clean · `vitest run` **239/239**, 19 files · `vite build` green (the >500 kB advisory
is pre-existing). `NODE_OPTIONS` still points at a deleted `.agentd` loader, so every node
invocation ran in child shells with only that stale preload unset.

**Lint, measured file-by-file against `/private/tmp/lwtM` (on `track/infra-foundation`):**

| file | eslint track → branch | prettier track → branch |
|---|---|---|
| `App.tsx` | 15 → 15 | clean → clean |
| `useAppStore.ts` | 0 → 0 | **FAIL → clean** |
| `AttachSessionDialog.tsx` | 0 → 0 | **FAIL → clean** |
| `app-v2/mock-data.ts` | 0 → 0 | **FAIL → clean** |
| `TimelineView.tsx` | 10 → 9 | clean → clean |
| `TimelineVisibility.test.tsx` (new) | — | clean |

So the delivery note's table is accurate — including its `TimelineView.tsx 10/9` line, which I
reproduced exactly — and four files got *better*. Nothing regressed.

## Honest limits

- **No restart, no click, no screenshot.** The screen is locked from a seat shell. Item 1 is
  verified at the persistence/effect layer with the real `App`, which is where the defect lives —
  but "close the app, reopen, and *see* it land on the tab" is not something I observed.
- **My `App` harness mocks `api` and forces `isTauri() => true`.** The real app's behavior against
  the real Tauri commands is not covered by it. The mechanism I measured (`useDataStore`'s initial
  value, then the write effect's ordering) is independent of the mock — it is the store's own
  initial value and App.tsx's own effect order — so the finding's mechanism holds.
- **I did not verify the packet's criterion-2 line "点某个类型能真的只剩那一类"** by clicking; I
  asserted the filter memo's behavior through rendered output, which is the same fact at the layer
  the code runs at.

## Bottom line

**FAIL, for one reason: item 1 does not do what the packet asked, and I measured it rather than
inferred it.** Items 2 and 3 pass with mutation evidence behind every claim — the timeline filter
is genuinely live (6-10 assertions die when it is reverted to the dead-switch state), the collapsed
default is the operator's ruling implemented and remembered per project, and both legacy fakes are
gone in a way `tsc` will refuse to undo. The delivery note's own "not verified" honesty about the
restart is correct and was the only thing standing between this and a silent pass; the finding is
that the thing it said it had not verified does not work when it is verified.

Fix for the record: the restore must win the race against the write effect — restore *before* the
write effect is allowed to run, and `useDataStore`'s `activeProjectId` must not start at a
test-only seed id. Either change alone would probably pass a naive test; both together make a
restart land where the operator left it.
