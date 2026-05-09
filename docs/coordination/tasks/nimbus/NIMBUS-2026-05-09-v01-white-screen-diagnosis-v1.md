# Task: V1 White-Screen Diagnosis (SG-A §A1)

| Field | Value |
|---|---|
| template | T3 |
| subtype | diagnostic |
| id | NIMBUS-2026-05-09-v01-white-screen-diagnosis-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-09 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-05-09 |
| depends_on | `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (commit a5998c1), `docs/coordination/reviews/2026-05-09-aegis-mvp-gap-to-tmux-replacement.md` (commit 0cc401b, §2 superseded but §4 packet list baseline), `ui/src/main.tsx` (TopErrorBoundary installed at commit 935e77a), `ui/src/App.tsx` (V1 shell entry point) |
| tags | nimbus, diagnostic, white-screen, v0.0.1, SG-A |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Blocking. A2/A3/A4 may proceed in parallel, but SG-B packets (B1/B2/B3) wait for full SG-A acceptance (A1+A2+A3+A4 all PASS). |

## Context

Mr. Zhang reports that when running `pnpm tauri dev`, the V1 main window renders a **white screen** instead of the expected `AppShell` + `SupervisionDashboard` layout.

`TopErrorBoundary` was installed in `ui/src/main.tsx` at commit `935e77a` specifically to catch render-time crashes and surface the error stack. If the white screen is caused by a React render error, `TopErrorBoundary` should display a red error card with:
- Error message
- Full stack trace
- Reload / Dismiss buttons

If `TopErrorBoundary` is **not** rendering (still white screen), the crash may be occurring before React mounts, or the error boundary itself has a bug.

## Your Task

**Diagnose and fix the white-screen issue.** You need Mr. Zhang to provide the error evidence first, then locate and repair the root cause.

### Step 1 — Request error evidence from Mr. Zhang

Ask Mr. Zhang to:

1. Run `pnpm tauri dev` in the repo root.
2. When the white screen appears, check:
   - **Does `TopErrorBoundary` render a red error card?**
     - If YES: screenshot the card (or copy the "Message" + "Stack" text) and paste it here.
     - If NO (still blank white): open DevTools Console (⌘⌥I on macOS) and screenshot any red error messages.
3. Also check the **Tauri backend logs** in the terminal where `pnpm tauri dev` is running — look for Rust panics or `[ERROR]` lines.

**Do not proceed to Step 2 until you have the error evidence.**

### Step 2 — Locate the root cause

Once you have the stack trace or console error:

1. Identify the **file:line** where the error originates.
2. Read that file using the `Read` tool.
3. Determine the root cause:
   - **Hydration mismatch** (backend data shape ≠ frontend expectation)?
   - **Missing import** or **undefined variable**?
   - **Type error** in a component render path?
   - **Tauri command failure** during `hydrateFromBackend()` in `App.tsx` line 42?

### Step 3 — Fix and verify

1. Apply the minimal fix to resolve the error.
2. Run `cd ui && npx tsc --noEmit` — must exit 0 with zero errors.
3. Run `cd ui && pnpm build` — must succeed.
4. Run `pnpm tauri dev` again and confirm:
   - No white screen.
   - `AppShell` renders with `NavRail` on the left.
   - `SupervisionDashboard` (5-card layout) renders in the main content area.
   - No red errors in DevTools Console.
   - No Rust panics in backend logs.

### Step 4 — Commit and report

Commit your fix with a message like:
```
fix(ui): resolve V1 white-screen caused by <root-cause>
```

## Delivery Format

Write your delivery to:
```
docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v01-white-screen-diagnosis-delivery-v1.md
```

Required sections:

```markdown
# Delivery: V1 White-Screen Diagnosis (SG-A §A1)

[Nimbus -> Lyra] V1 White-Screen Diagnosis

commit:
- <your-fix-commit-hash>

completed:
- Step 1: Received error evidence from Mr. Zhang (paste evidence below)
- Step 2: Root cause identified at <file>:<line>
- Step 3: Fix applied and verified
- Step 4: Committed

## Error evidence (Step 1)

<paste Mr. Zhang's screenshot or error text here — DO NOT SUMMARIZE>

## Root cause (Step 2)

File: <file-path>
Line: <line-number>
Issue: <one-sentence description>

## Fix applied (Step 3)

<paste the git diff of your fix — use `git show <commit> --stat` + `git show <commit>` output>

## Verification (Step 3)

```
$ cd ui && npx tsc --noEmit
<paste full output>

$ cd ui && pnpm build
<paste last 20 lines of output>

$ pnpm tauri dev
<describe what you see: AppShell renders? SupervisionDashboard visible? Any console errors?>
```

blockers:
- none / <describe any blockers>

verdict:
- PASS / HOLD / FAIL

next action:
- wait for Lyra acceptance

artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v01-white-screen-diagnosis-delivery-v1.md
```

## Scope Constraints

- **Only fix the white-screen issue.** Do not refactor unrelated code.
- **Do not touch**:
  - `ui/src/app-v2/panel/SupervisorPanel.tsx` (chan-03/chan-09 verified)
  - `ui/src/app-v2/dashboard/GlobalDashboard.tsx` (chan-03 verified)
  - `ui/src/i18n.ts` (Mira's v01 scope)
  - `src-tauri/**` (unless the error is a Rust panic)
- **TypeScript must remain clean** (`npx tsc --noEmit` zero errors).
- **Build must succeed** (`pnpm build` exit 0).

## Coordination Rules (COORDINATION_RULES.md §3)

- **Delivery must include tool output verbatim** (not summaries). Paste `tsc`, `pnpm build`, `git show` output in full.
- **Commit hash required**. Lyra will verify the exact commit during acceptance.
- **No scope drift**. If you discover additional bugs while diagnosing, report them separately — do not fix them in this packet.

---

*Task issued by Lyra · 2026-05-09 · SG-A §A1 · Blocking for SG-B*
