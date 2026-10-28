# Delivery: V5 honesty fixes — independent verification, PASS with one finding

| Field | Value |
|---|---|
| dispatch_id | disp-CCaw_71EhfQk7P1GQq2Bu |
| verified | disp-RWEf5ATPvICLkGtF__fEY (AEGIS-2026-10-20-v5-honest-surfaces-v1) |
| verdict | pass |
| verdict event | ev-1ZDgqPoVkpeEB7T7O7hgY (ReviewVerdictIssued, emitted) |
| evidence | docs/coordination/tasks/flux/FLUX-2026-10-21-v5-honesty-fixes-verdict-v1.md |
| branch | packet/flux/intent-verdict @ 2677f36 |
| tested at | packet/mira/knowledge-panel @ dbb51e7 |

## What I did

Verified **one** dispatch — `disp-RWEf5ATPvICLkGtF__fEY` — at `packet/mira/knowledge-panel`'s
**HEAD** (`dbb51e7`), not the patch commit `735da06` (8 commits earlier), because HEAD is
what a merge carries. No other dispatch's verdict was borrowed or merged into this one.

Method was **render-then-mutate**, not read. A throwaway vitest harness (13 assertions, two
probe files) drove the three real components — the real `SupervisionDashboard`, the real
`SyncFreshness`, the real `GlobalDashboard` — against a mocked store at the boundary the
components themselves read. Then I mutated the implementation to prove the assertions bite:

| mutation | result |
|---|---|
| `未知(未记录)` → a rendered clock value | 1 assertion FAILS |
| budget card `value="—"` → `value="72%"` | 1 assertion FAILS |
| `projects.map(...)` → hardcoded `p-1/p-2/p-3`; freeze `diffMs` | 3 assertions FAIL |
| whole file reverted to track's pre-patch version | 4 assertions FAIL |
| failed hydrate marked hydrated (store lie) | 0 FAIL — honest limit |

All mutations restored byte-identically; `git diff` in Mira's worktree is empty and the
project's own suite re-run green afterwards.

## Gates, run by me in Mira's worktree

`tsc --noEmit` clean · `vitest run` **239/239** (19 files) · `vite build` green (the >500 kB
advisory is pre-existing). `NODE_OPTIONS` still points at a deleted `.agentd` loader, so every
node invocation ran in child shells with only that stale preload unset — the same workaround
Mira recorded.

**Lint, measured rather than asserted**: `lint-changed.sh` reports 73 violations tree-wide, all
pre-existing. For the three files this patch touches I ran eslint+prettier side-by-side against
`/private/tmp/lwtM` (on `track/infra-foundation`):

| file | eslint track → branch | prettier track → branch |
|---|---|---|
| GlobalDashboard.tsx | 0 → 0 | fail → **clean** (improved) |
| SyncFreshness.tsx (new) | — | clean |
| SupervisionDashboard.tsx | 2 → 2 (same 2 errors, lines moved) | fail → fail (same 2 deviations) |

So "delta-zero" is true about *introduced* violations. It is **not** the same as "the files are
clean" — `SupervisionDashboard.tsx` remains prettier-dirty, which the delivery note's
"2/2 / 0/0" phrasing leaves ambiguous. Recorded in the verdict.

## The finding

**Four fabricated numbers still render, in the file the dispatch fixed** — `平均完成率 64%，处于健康区间`,
`导致关键路径（ENV-001）延期 4h`, `待审计文档共 3 份`, `影响下游 2 个席位的启动计划`. Measured by actually
hovering all five cards: on a project with zero work items the completion-rate hover still
promises 64%. Same class as the defect, same file, and absent from the delivery note's sweep
list — so criterion 5 is incomplete as delivered. A second, pre-existing one (verified present on
`track` too): `待决策事项` counts Chinese inbox types while `hydrateFromBackend` emits English ones,
so it reads 0 against real data, and `DriftDetected` is invisible to both dashboards.

Neither blocks the PASS: the packet's own six criteria are met, and both findings are outside
its "three located items" scope. Both are recorded for a follow-up.

## Boundary

No code, backend, DB, or merge change by me. I added and then **deleted** my two probe files
(`ui/src/zz_flux_v5_probe*.test.tsx`) — Mira's worktree is byte-identical to `dbb51e7`.
`git status` and `git diff` empty in both worktrees at the end. No acceptance card opened.

## Artifact index

- Verdict: `docs/coordination/tasks/flux/FLUX-2026-10-21-v5-honesty-fixes-verdict-v1.md`
- Ledger event: `ev-1ZDgqPoVkpeEB7T7O7hgY`
- This delivery note.
