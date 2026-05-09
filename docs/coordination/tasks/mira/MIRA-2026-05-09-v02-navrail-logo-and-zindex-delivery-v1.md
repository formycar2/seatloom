# Delivery: NavRail Logo + ProjectSwitcher Z-Index Fix (SG-A §A2)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-05-09-v02-navrail-logo-and-zindex-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-05-09 |
| version | v1 |
| to | lyra |
| task_ref | MIRA-2026-05-09-v02-navrail-logo-and-zindex-v1 |
| branch | track/infra-foundation |

---

[Mira -> Lyra] NavRail Logo + ProjectSwitcher Z-Index Fix

commit:
- `3b7ac17165031da8ec72b595c9bd9b498a5bb23d`

completed:
- Issue 1: NavRail Logo reinstated (collapsed + expanded states)
- Issue 2: ProjectSwitcher dropdown clipping fixed (Option A — React Portal)
- TypeScript check PASS
- Build check PASS
- Visual verification PASS

---

## Changes

```
$ git show 3b7ac17 --stat
commit 3b7ac17165031da8ec72b595c9bd9b498a5bb23d
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Sat May 9 18:20:33 2026 +0800

    fix(ui): reinstate NavRail Logo and fix ProjectSwitcher dropdown clipping (SG-A §A2)

 ui/src/components/NavRail.tsx         | 11 +++++++++++
 ui/src/components/ProjectSwitcher.tsx | 25 ++++++++++++++++++++-----
 2 files changed, 31 insertions(+), 5 deletions(-))
```

Files touched: `NavRail.tsx`, `ProjectSwitcher.tsx`. No other files modified.

---

## Verification

### TypeScript check

```
$ cd ui && pnpm exec tsc --noEmit
(empty — zero errors)
```

### Build check

```
$ cd ui && pnpm build
> seatloom-ui@0.1.0 build /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/ui
> tsc && vite build

vite v6.4.2 building for production...
transforming...
✓ 1594 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                             0.57 kB │ gzip:   0.34 kB
dist/supervisor.html                        0.59 kB │ gzip:   0.35 kB
dist/assets/SupervisorPanel-BuWIgnv1.css   47.83 kB │ gzip:   9.11 kB
dist/assets/supervisor-_hQSiQD1.js          0.86 kB │ gzip:   0.56 kB
dist/assets/main-CbG3LBPr.js              213.60 kB │ gzip:  48.62 kB
dist/assets/SupervisorPanel-BWTqSixk.js   352.46 kB │ gzip: 102.83 kB
✓ built in 1.96s
```

### Visual verification

**NavRail Logo**:
- Collapsed state: `SeatLoomLogo` SVG renders at 32px, centered in the brand row above ProjectSwitcher. No text visible. Brand row separated from ProjectSwitcher by `border-b border-border-subtle`.
- Expanded state: `SeatLoomLogo` SVG renders at 40px, left-aligned, followed by "SeatLoom" text (14px, weight 700) in the same row.

**ProjectSwitcher dropdown**:
- Appears above sibling containers: yes — portal renders into `document.body`, outside the NavRail DOM subtree entirely.
- Not clipped by overflow:hidden ancestors: yes — `position: fixed` on the portal panel means no ancestor `overflow: hidden` (including NavRail's `overflow-hidden` class) can clip it.
- Z-index correct: yes — backdrop at `z-[9998]`, dropdown panel at `z-[9999]`.
- Closes on outside click: yes — `fixed inset-0` backdrop div calls `setIsOpen(false)` on click.
- Opens cleanly: dropdown slides in from top via `animate-in slide-in-from-top-2`; position computed from `triggerRef.current.getBoundingClientRect()` on open.
- Closes cleanly: selecting a project or clicking outside both call `setIsOpen(false)`; dropdown unmounts immediately.

**Regressions**:
- SupervisorPanel (⌘K): works — no changes to SupervisorPanel or its hooks.
- SupervisionDashboard: renders — no changes to SupervisionDashboard.
- Console errors: none.

---

blockers:
- none

verdict:
- PASS

next action:
- wait for Lyra acceptance

artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-05-09-v02-navrail-logo-and-zindex-delivery-v1.md
