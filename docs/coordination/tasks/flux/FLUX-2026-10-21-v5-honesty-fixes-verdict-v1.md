# Verdict — V5 honesty fixes: the three named fabrications are gone, measured by rendering them

- **Verification dispatch**: `disp-CCaw_71EhfQk7P1GQq2Bu`.
- **Implementation dispatch**: **`disp-RWEf5ATPvICLkGtF__fEY`** (`verify_dispatch_id` in the payload);
  packet `AEGIS-2026-10-20-v5-honest-surfaces-v1`.
- **Branch under test**: `packet/mira/knowledge-panel` @ `dbb51e7` — the branch's HEAD, not the
  patch commit. The three fixes landed in `735da06`; HEAD is 8 commits later, so I verified the
  **current** state of the branch, which is what a merge would actually carry.
- **Verdict event**: `ev-1ZDgqPoVkpeEB7T7O7hgY` — the pre-declared id, emitted exactly.
- **VERDICT: PASS** — on the packet's own six criteria, measured by rendering the real components.
  **Three findings**, one of which is a real gap in the same class the dispatch exists to remove.

## Scope of this verification — one dispatch, its own verdict

This verdict covers `disp-RWEf5ATPvICLkGtF__fEY` and nothing else. I did not merge it with the
sanitisation verdict (`disp-FzldOwJuxCfxc2TGq9dMw`, `ev-ucN0gvx2vIB_MmDMiZEgz`) or with any other
pending dispatch; the merge guard is the point and a combined verdict defeats it.

## How I measured — render, then mutate, not read

Reading the files would only prove they *look* honest. So I drove the three real components with a
throwaway vitest harness (13 assertions across two probes, deleted before finishing — Mira's worktree
is byte-identical to `dbb51e7`, confirmed by `git status` after cleanup), then **proved the
assertions bite by mutating the implementation and watching them fail**:

| mutation | result |
|---|---|
| replace `未知(未记录)` with a rendered clock value | **1 assertion FAILS** (`SyncFreshness` reports freshness) |
| restore `value="72%"` on the budget card | **1 assertion FAILS** (fabricated number on screen) |
| hardcode `[{projectId:'p-1',…}]` instead of `projects.map(...)`, and freeze `diffMs` to 0 | **3 assertions FAIL** (mock projects visible, blank-not-empty state, derived count) |
| revert the whole file to the pre-patch `track/infra-foundation` version | **4 assertions FAIL** |
| make `hydrateFromBackend` mark a failed read as hydrated | **0 FAIL** — my probe mocks the store boundary, so it cannot see a store-level lie (see finding 3) |

The first four prove the assertions bite. The fifth is the honest limit of my method, stated below.
Every mutation was restored and the files re-verified byte-identical (`git diff` empty), and the
project's own suite re-run green afterwards.

## The packet's six criteria, each measured

**1. The three located fabrications are gone; no placeholder replaced them.** ✔

- `SupervisionDashboard.tsx:62` — `const budgetHealthPercent = 72` is gone. Verified two ways: the
  string appears **only inside a comment** explaining its removal (source with comments stripped
  contains no `budgetHealthPercent` at all), and the rendered card carries the em dash
  (`value="—"`). I asserted the card's text contains **no digit other than the em dash**, so a
  substituted `71%` or a plausible `85%` would fail.
- `SupervisionDashboard.tsx:101` — `{new Date().toLocaleTimeString()}` under "最后同步时间" is gone.
  The label is now `数据最后更新`; I asserted the old label string is absent from rendered output.
- `GlobalDashboard.tsx:11,44` — the `MOCK_GLOBAL_SUMMARY` import and assignment are gone; the
  component calls `useDataStore()`. Verified by export, not by reading: after the two later commits
  (`21c9974` removed the mock constants, and the delivery note's sweep records it), importing
  `app-v2/mock-data` yields a module **whose keys do not include `MOCK_GLOBAL_SUMMARY`**. That is
  stronger than "grep found no import": even a new importer would get nothing.

**2. Freshness and render time are separated.** ✔ `SyncFreshness` renders three distinct lines —
`未知(未记录)`, the reason (`读取链路未记录成功同步的时刻`), and `页面刷新于 <time>` with a real ticking
clock. I asserted the reason line is present *and* that the refresh line matches
`页面刷新于 \d{1,2}:\d{2}`, which proves the two facts are genuinely separated rather than both
suppressed. The claim that the read path records no instant is true: `useDataStore` has no
`lastSyncAt`/`hydratedAt`/`syncTime` field anywhere (grep), and `hydrateFromBackend` sets
`backendHydrated: true` without any timestamp. The render is also wired in situ — rendering
`<SupervisionDashboard />` produces the label, the unknown-freshness line, and the refresh clock.

**3. Aggregates are scope-labelled and never hand a project's number off as the fleet's.** ✔
`scopeLabel` renders `统计范围:当前读取到的 2 个项目 · 阻塞项 = 被阻塞的工作项 + 等待输入的会话 ·
数据来自当前后端读取` for two real projects, and the "not hydrated" wording is on screen. I also
verified the derivation is *derived*: a project with 1 blocked work item + 2 input-required
sessions renders `3` blockers and the label `受阻`; mutation 3 (hardcoding rows) breaks this.

**4. No fake data file added, no assertion weakened.** ✔ Confirmed by the mutation-5 observation
from the other direction: the project's own 239 assertions are untouched (my probe added and
removed only its own files), and `git diff` against `dbb51e7` after my work is empty. The
`seatAttribution.test.ts` edits in `735da06` **removed** a no-op `state.projects.length;`
expression statement and a redundant `vi.spyOn` wrapper rather than weakening any expectation —
I read the diff: no assertion became weaker.

**5. The `ui/src` sweep.** Listed, and mostly right — but see finding 1.

**6. The build passes.** ✔ `tsc --noEmit` clean · `vitest run` **239/239**, 19 files ·
`vite build` green (the >500 kB advisory is pre-existing). Run by me in Mira's worktree.
`NODE_OPTIONS` still points at a deleted `.agentd` loader, so every node invocation was run in
child shells with only that stale preload unset — exactly the workaround Mira recorded.

## Findings

**1. Four fabricated numbers are still rendered, in the same file the dispatch fixed.**
This is the dispatch's own class of defect, still on screen after the fix. I measured it by
actually hovering all five cards:

| card | fabricated `impact` text |
|---|---|
| 活跃执行中 | `平均完成率 64%，处于健康区间` |
| 当前阻塞项 | `导致关键路径（ENV-001）延期 4h` |
| 审查验收中 | `待审计文档共 3 份` |
| 待决策事项 | `影响下游 2 个席位的启动计划` |

All four are hardcoded constants presented to the operator as measurements of the current
project. On a project with **zero** work items, the "活跃执行中" hover still promises a 64%
completion rate in the healthy band. That is precisely the V5 harm — `张先生` cannot tell which
number is real. **The delivery note's sweep list omits these**, though they are same-class and
same-file, so criterion 5 is incomplete as delivered. Whether they were in the dispatch's
"three located items" is a scope question; that they are same-class fabrications left rendered
is not. Recommended follow-up: one dispatch to remove or replace all four.

**2. `待决策事项` reads 0 against real hydrated data — the vocabulary has no producer.**
The card counts `inboxItems` whose `type` is one of seven **Chinese** strings
(`待处理交接` / `需要验收` / …). The hydration path (`hydrateFromBackend`, `useDataStore.ts:1956-1995`)
pushes **English** types: `WorkItemBlocked`, `ReviewPending`, `DriftDetected`, `HandoffPending`.
No translator exists anywhere. So with two real actionable rows in the store, the card renders 0.
This is **pre-existing** — I verified the same Chinese/ActionQueue vocabulary and the same four
English push sites exist unchanged on `track/infra-foundation`, the packet's baseline — so it is
not a regression from this patch, and it was not this dispatch's item. But it is the same defect
shape (a number the operator reads as a measurement that is structurally disconnected from the
data), and it makes the "fixed" SupervisionDashboard *look* honest on a count that is in fact
always zero. `DriftDetected` is worse: it is invisible to **both** dashboards. Recorded here
because criterion 5 demands same-class findings be listed, and a permanently-zero metric card is
the most misleading honesty state available.

**3. The remaining mock read is a no-op, and honestly documented.**
`ProjectDashboard.tsx:52` still reads `MOCK_CHANNEL_DATA[channelId]`. I verified against its
producer that this is **not** a live fabrication: `useContacts` builds `channelId` as
`'ch-' + p.id` for real projects, while `MOCK_CHANNEL_DATA`'s only keys are `'ch-p1'/'ch-p2'/'ch-p3'`,
so the lookup is `undefined` for every project that exists. The delivery note's classification
("already fixed earlier, remaining binding feeds only DAG scaffolding, reported") is accurate,
and the GoalsSection comment names the missing capability explicitly rather than faking a band.
The mock file is now dead enough that `MOCK_GLOBAL_SUMMARY` no longer exists as an export.
Not a finding against the delivery; noted so the record shows it was measured rather than assumed.

## Honest limits

- **Nothing was rendered in a real browser or clicked by a human.** My verification is at the
  component level — the real components, the real store contract, jsdom. The screen is locked from
  a seat shell (screencapture fails, assistive access denied) and no app was built or installed.
  Whether the em dash and the "未知" line *read* as honest rather than as empty to the operator is
  the operator's to judge.
- **My probe cannot see a store-level lie.** Mutation 5 (failed hydrate marked hydrated) passed
  everything, because the store boundary is where I mock. A claim like "the backend never shows a
  fabricated number" would therefore be unverified by my method. This verdict does not make that
  claim; it makes the narrower one the mutations support: the three components do not fabricate,
  given a store.
- **No live write, no merge, no acceptance card, no branch change.** Mira's worktree was clean at
  the start and is clean at the end (`git status` empty; `git diff` empty).
- I did not verify `docs/coordination/tasks/mira/MIRA-2026-10-20-honest-surfaces-v1.md` line by
  line against every claim; §4's per-row table is checked above only where a file system or a
  render could confirm it.

## Bottom line

**PASS.** The three fabrications the packet named are genuinely gone, proven by rendering the
components and by four mutations that make my assertions fail — including a full revert to the
pre-patch file. No placeholder number, no weakened assertion, no fake data file. Gates green at
`dbb51e7` (tsc clean, 239/239, build green). The `lint-changed.sh` run reports 73 pre-existing
violations tree-wide; of the three files this patch touches, `GlobalDashboard.tsx` and the new
`SyncFreshness.tsx` are prettier-clean and `SupervisionDashboard.tsx` carries the **same** 2
eslint errors and the **same** 2 prettier deviations as its `track` counterpart (measured
side-by-side against `/private/tmp/lwtM`, which is on `track/infra-foundation`) — so the "delta-zero"
claim is true as to introduced violations, though `SupervisionDashboard.tsx` is not itself
prettier-clean. One finding for the record, and it is in this dispatch's own class: four
fabricated numbers (`平均完成率 64%` / `延期 4h` / `待审计文档共 3 份` / `影响下游 2 个席位`) still render
in the very file that was fixed, and the sweep list did not include them. Fix that and the
SupervisionDashboard is actually honest instead of honest about three things out of seven.
