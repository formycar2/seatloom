# Delivery: Mount SessionsWorkspace in V1 main shell (Sessions tab)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | MIRA-2026-05-11-v1-sessions-tab-mount-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-05-11 |
| commit | 83b1223d8bb51a1a9e1590a60e10bbe19c83f0c3 |
| packet_ref | MIRA-2026-05-11-v1-sessions-tab-mount-v1 |
| branch | track/infra-foundation |

---

[Mira -> Lyra] V1 Sessions tab mounted. B1 Layer B unblocked on UI side.

---

## §1 — git show --stat

```
$ git show 83b1223 --stat
commit 83b1223d8bb51a1a9e1590a60e10bbe19c83f0c3
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Mon May 11 12:29:06 2026 +0800

    feat(ui): mount SessionsWorkspace as 会话 tab in V1 NavRail (MIRA-2026-05-11-v1-sessions-tab-mount)

 ui/src/App.tsx                | 6 +++++-
 ui/src/components/NavRail.tsx | 5 ++++-
 2 files changed, 9 insertions(+), 2 deletions(-)
```

Two files touched, zero others. `SessionsWorkspace.tsx` / `SessionTerminal.tsx` / backend / DTO / i18n all untouched.

---

## §2 — tsc + pnpm build

### tsc

```
$ cd ui && pnpm exec tsc --noEmit
(empty — zero errors)
```

### pnpm build

```
$ cd ui && pnpm build
> tsc && vite build

vite v6.4.2 building for production...
transforming...
✓ 1606 modules transformed.
rendering chunks...
computing gzip size...
dist/supervisor.html                        0.59 kB │ gzip:   0.35 kB
dist/index.html                             0.64 kB │ gzip:   0.36 kB
dist/assets/main-CFbL2ovg.css               4.64 kB │ gzip:   1.72 kB
dist/assets/SupervisorPanel-BYlfx6MH.css   47.96 kB │ gzip:   9.13 kB
dist/assets/supervisor-BYn6Jcyo.js          0.86 kB │ gzip:   0.56 kB
dist/assets/SupervisorPanel-Sih_hy8v.js   353.04 kB │ gzip: 103.04 kB
dist/assets/main-BzyQ3Qfu.js              516.85 kB │ gzip: 125.61 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 1.78s
```

Build exit 0. Chunk-size warning is pre-existing (not introduced by this packet — `SessionsWorkspace` was already reachable from app-v2 and pulls the same deps).

---

## §3 — Static invariant self-check

| # | Invariant | Status | Line evidence |
|---|-----------|--------|---------------|
| 1 | Scope held to 2 files (App.tsx + NavRail.tsx); optional types not needed | PASS | `git show --stat` above — 2 files |
| 2 | `'sessions'` added to `Tab` union | PASS | `ui/src/App.tsx:36` — `\| 'sessions' \|` inserted before `'all-projects'` |
| 3 | NavRail items array has `{ id: 'sessions', icon: Terminal, label: '会话' }` entry | PASS | `ui/src/components/NavRail.tsx:65` — inserted after `playbook` row |
| 4 | App.tsx render switch has `if (activeTab === 'sessions') return <SessionsWorkspace />` early-return | PASS | `ui/src/App.tsx` render fn — placed after `'all-projects'` early-return, before MasterDetail block, matching the packet's "full-bleed, not master-detail" directive |
| 5 | No regressions in existing 8 tabs | PASS | tsc zero errors; pnpm build exit 0; no changes to existing tab case handlers |

Supporting changes:
- `NavRail.tsx` `NavTab` union extended with `'sessions'` (same file as items array, so counted in the 2-file scope).
- `NavRail.tsx` `lucide-react` import extended with `Terminal`.
- `App.tsx` import added: `import { SessionsWorkspace } from './app-v2/panel/SessionsWorkspace';`.

---

## §4 — Verdict

Layer A self-check PASS.

---

blockers:
- none

next action:
- Flux Layer A verify → Lyra acceptance → Mr. Zhang Layer B (B1 6-step bidirectional smoke now runnable end-to-end from V1 main shell)

artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-05-11-v1-sessions-tab-mount-delivery-v1.md
