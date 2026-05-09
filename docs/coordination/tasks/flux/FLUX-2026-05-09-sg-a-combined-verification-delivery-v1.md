# Delivery: SG-A Combined Commit-Pinned Verification (A1 + v01 + A4-α + A2)

[Flux -> Lyra] SG-A combined verification. Layer A PASS on all 8 commits. Layer B pending Mr. Zhang runtime.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification_delivery |
| id | FLUX-2026-05-09-sg-a-combined-verification-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-05-09 |
| version | v1 |
| to | lyra |
| depends_on | FLUX-2026-05-09-sg-a-combined-verification-v1.md |
| tags | flux, verification, SG-A, commit-pinned, layer-a-pass, layer-b-pending |

---

## Layer A — Automated Checks (PASS)

### A.1 Tree state

```
$ git rev-parse HEAD
34bd0a1ff9febd46dfa42b3466ea80c927c5c19b

$ git rev-parse --abbrev-ref HEAD
track/infra-foundation

$ git status --short
 M crates/seatloom-core/Cargo.toml
 M crates/seatloom-core/src/pty/mod.rs
 M docs/coordination/tasks/flux/FLUX-2026-05-06-app-v2-phase2-slice-b-verification-delivery-v1.md
?? .gemini/
?? .seatloom/transcripts/
?? docs/coordination/acceptance/2026-04-30-lyra-flux-app-v2-truth-projection-slice-a-acceptance.md
?? docs/coordination/acceptance/2026-04-30-lyra-nimbus-postgres-baseline-verifier-hardening-acceptance.md
?? docs/coordination/reviews/2026-04-30-lyra-collaboration-dataflow-backend-traceability-report.md
?? docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-followup-review-v2.md
?? docs/coordination/reviews/2026-04-30-lyra-flux-truth-projection-slice-a-review.md
?? docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md
?? docs/coordination/reviews/2026-04-30-supervisor-im-as-l1-insight.md
?? docs/coordination/tasks/copilot/
?? docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-final-fix-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-app-v2-truth-projection-slice-a-followup-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-delivery-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-delivery-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-database-readiness-gate-reverification-delivery-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-database-readiness-gate-reverification-v1.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v2.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v3.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v4.md
?? docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v5-delivery.md
?? docs/coordination/tasks/lyra/LYRA-2026-04-30-flux-prompt-channel-authority-remote-verification-v1.md
?? docs/coordination/tasks/lyra/LYRA-2026-04-30-nimbus-schema005-test-isolation-fix-v1.md
?? docs/coordination/tasks/mira/MIRA-2026-05-09-v001-sessions-attach-ui-delivery-v1.md
?? docs/coordination/tasks/mira/MIRA-2026-05-09-v02-navrail-logo-and-zindex-delivery-v1.md
?? docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-v1.md
?? docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-v1.md
?? scripts/monitor-gemini.sh
?? scripts/trace-gemini-network.sh
```

Assessment: Three modified tracked files — `crates/seatloom-core/Cargo.toml` and `crates/seatloom-core/src/pty/mod.rs` (Nimbus A3 tmux read-path in-progress work, no impact on UI verification), plus a Flux delivery doc edit. No UI source files dirty. Untracked files are historical verification artifacts. ✓

### A.2 Commit chain presence

```
$ git cat-file -e 39af8b2 && echo "A1 delivery commit present"
A1 delivery commit present
$ git cat-file -e 578ff7c && echo "v01 §A present"
v01 §A present
$ git cat-file -e 4651bb4 && echo "v01 §B present"
v01 §B present
$ git cat-file -e 2f83624 && echo "v01 §C present"
v01 §C present
$ git cat-file -e e864392 && echo "v01 factual-record present"
v01 factual-record present
$ git cat-file -e bdac54b && echo "A4-α present"
A4-α present
$ git cat-file -e 3b7ac17 && echo "A2 present"
A2 present
$ git cat-file -e b11d878 && echo "revert of 31367a0 present (archival)"
revert of 31367a0 present (archival)
```

All 8 commits present. ✓

### A.3 Per-commit scope audit

```
$ git show 39af8b2 --stat
docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v01-white-screen-diagnosis-delivery-v1.md | 135 +++++++++++++++++++++
1 file changed, 135 insertions(+)

$ git show 578ff7c --stat
ui/src/app-v2/panel/SessionsWorkspace.tsx | 108 ++++++++++++++++--------------
1 file changed, 57 insertions(+), 51 deletions(-)

$ git show 4651bb4 --stat
ui/src/app-v2/AppV2.tsx | 27 +++++++++++++++++++--------
1 file changed, 19 insertions(+), 8 deletions(-)

$ git show 2f83624 --stat
ui/src/app-v2/views/DocumentsWorkspace.tsx | 49 +++++++++++++++++++++++++++++-
1 file changed, 48 insertions(+), 1 deletion(-)

$ git show bdac54b --stat
ui/src/app-v2/panel/SessionTerminal.tsx   |  18 +-
ui/src/app-v2/panel/SessionsWorkspace.tsx | 279 +++++++++++-------------------
2 files changed, 108 insertions(+), 189 deletions(-)

$ git show 3b7ac17 --stat
ui/src/components/NavRail.tsx         | 11 +++++++++++
ui/src/components/ProjectSwitcher.tsx | 25 ++++++++++++++++++++-----
2 files changed, 31 insertions(+), 5 deletions(-)
```

| Commit | Packet | Files touched | Scope match |
|---|---|---|---|
| `39af8b2` | A1 | docs-only: delivery doc | ✓ |
| `578ff7c` | v01 §A | `SessionsWorkspace.tsx` only | ✓ |
| `4651bb4` | v01 §B | `AppV2.tsx` only | ✓ |
| `2f83624` | v01 §C | `DocumentsWorkspace.tsx` only | ✓ |
| `e864392` | v01 factual-record | docs-only (delivery doc rewrite) | ✓ |
| `bdac54b` | A4-α | `SessionTerminal.tsx` + `SessionsWorkspace.tsx` | ✓ |
| `3b7ac17` | A2 | `NavRail.tsx` + `ProjectSwitcher.tsx` | ✓ |
| `b11d878` | revert archival | 4 files (revert `31367a0`) | ✓ archival |

No stray files. ✓

### A.4 i18n.ts integrity (hard checkpoint)

```
$ git diff 48af37d..HEAD -- ui/src/i18n.ts
(empty — no changes since SG-A amendment commit)
```

i18n.ts untouched since `48af37d`. ✓

### A.5 TypeScript + Build

```
$ cd ui && pnpm exec tsc --noEmit
(empty output, zero errors)
EXIT: 0

$ cd ui && pnpm build
vite v6.4.2 building for production...
✓ 1594 modules transformed.
rendering chunks...
dist/index.html                             0.57 kB │ gzip:   0.34 kB
dist/supervisor.html                        0.59 kB │ gzip:   0.35 kB
dist/assets/SupervisorPanel-BuWIgnv1.css   47.83 kB │ gzip:   9.11 kB
dist/assets/supervisor-_hQSiQD1.js          0.86 kB │ gzip:   0.56 kB
dist/assets/main-CbG3LBPr.js              213.60 kB │ gzip:  48.62 kB
dist/assets/SupervisorPanel-BWTqSixk.js   352.46 kB │ gzip: 102.83 kB
✓ built in 2.08s
EXIT: 0
```

tsc zero errors. pnpm build success. ✓

### A.6 Static invariants

#### A1 (`39af8b2`) — White-screen diagnosis (no-fix PASS)

- [x] Delivery doc at `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v01-white-screen-diagnosis-delivery-v1.md` exists.
- [x] Commit is docs-only (no source changes). `git show 39af8b2 --stat` shows 1 file, docs path only.
- [x] `TopErrorBoundary` still installed at `ui/src/main.tsx` (3 references at HEAD `34bd0a1`).
- [x] Commit message: "white screen not reproducible at HEAD 48af37d" — consistent with no-fix verdict.

#### v01 (`578ff7c` / `4651bb4` / `2f83624`) — Sessions Live Wiring

- [x] §A `SessionsWorkspace.tsx`: reads seats from `useDataStore.projectData[activeProjectId]?.seats` (3 `useDataStore` references in diff).
- [x] §A xterm launch path preserved (retained `launch()` function after refactor).
- [x] §A kill-confirm dialog present: `window.confirm('确定要强行终止该会话吗？未保存的工作将会丢失。')` at line 89 of diff.
- [x] §A exit code pill: renders `exit {code}` with green (code=0) / red (code≠0) coloring for exited sessions.
- [x] §B `AppV2.tsx`: header counters use `useDataStore` — `blockedCount` from items with status `'Blocked'`, `activeCount` from items with status `'Active'`.
- [x] §B pills hide when count is zero: `{blockedCount > 0 && (...)}` and `{activeCount > 0 && (...)}`.
- [x] §C `DocumentsWorkspace.tsx`: `🔄 同步文档` button with `正在同步…` loading state.
- [x] §C summary bar: `已扫描 {scanned} · 新增 {inserted} · 更新 {updated} · 未变 {unchanged} · 失败 {failed} · 冲突 {conflicted}` — matches spec.
- [x] §C error banner: red border + red text when `failed > 0 || conflicted > 0`.
- [x] §C calls `api.reconcile()` on button click.

#### v01 factual-record (`e864392`)

- [x] Authored by Aegis (takeover after 4 Mira failures).
- [x] Commit chain covers all 3 sections (578ff7c/4651bb4/2f83624).
- [x] Documents 3 discrete drift incidents (failed checklist items).
- [x] Replaces Mira's incomplete §B-only delivery with full 3-section factual record.

#### A4-α (`bdac54b`) — SessionsWorkspace Attach UI (mock-first)

- [x] `MOCK_TMUX_SESSIONS` array present with exactly 3 entries:
  ```ts
  { session_name: 'Lyra-po-seatloom', created_at: 0, attached: true },
  { session_name: 'Nimbus-TechArchi-seatloom', created_at: 0, attached: true },
  { session_name: 'Mira-UX/UED-seatloom', created_at: 0, attached: true },
  ```
- [x] Four `TODO(A4-β)` markers in source diff.
- [x] `console.warn` mock attach: `console.warn('A3 not yet available; mock attach for', tmuxSessionName);` ✓.
- [x] `disableStdin: true` in `SessionTerminal.tsx` xterm initialization (2 references in diff).
- [x] `cursorBlink: false` in `SessionTerminal.tsx` (3 references in diff).
- [x] Read-only banner: `⚠️ Read-only mode (v0.0.1). Typing in this terminal is disabled. Use tmux directly to send commands.` ✓.
- [x] Real `invoke('cmd_list_tmux_sessions')` and `invoke('cmd_attach_tmux_session', ...)` **both commented-out** — only mock data flows. Verified: grep shows invoke calls are prefix-pinned with `//` (line comments), not active code.
- [x] `attachToTmux()` adds tab with mock label `tmux: ${tmuxSessionName}`.

#### A2 (`3b7ac17`) — NavRail Logo + ProjectSwitcher Portal

- [x] `NavRail.tsx` imports `SeatLoomLogo` from `ui/src/app-v2/components/Avatar.tsx` (2 references in diff).
- [x] Logo placed above `ProjectSwitcher` in the NavRail brand section.
- [x] Logo renders in both collapsed and expanded states (conditional `showText={!collapsed}`).
- [x] `ProjectSwitcher.tsx` uses `ReactDOM.createPortal(dropdownContent, document.body)` — not a descendant of any overflow:hidden ancestor.
- [x] Dropdown position calculated via `triggerRef.current.getBoundingClientRect()`:
  ```ts
  setDropdownPos({ top: rect.bottom + 8, left: rect.left });
  ```
- [x] Dropdown `zIndex: 9999` (spec: ≥ 9999). Backdrop `z-[9998]`.
- [x] Dropdown uses `position: 'fixed'` with calculated coordinates.
- [x] `SupervisionDashboard.tsx` untouched: `git show 3b7ac17 --name-only | grep SupervisionDashboard` → empty. ✓
- [x] `SupervisorPanel.tsx` untouched. ✓
- [x] `i18n.ts` untouched. ✓
- [x] `src-tauri/**` untouched. ✓

#### b11d878 — revert of 31367a0 (archival)

- [x] Reverts 4 files: `AppV2.tsx`, `NavRail.tsx`, `SupervisionDashboard.tsx`, `main.tsx` (+164/−180).
- [x] Not a current working-tree change: `git diff 3b7ac17..HEAD --stat | grep -c "b11d878\|31367a0"` → 0. ✓

---

## Layer B — Runtime verification (PENDING Mr. Zhang)

### A1 — White-screen non-repro (4 observations)

| # | Observation | Check |
|---|---|---|
| 1 | `pnpm tauri dev` → no white screen | [ ] |
| 2 | `AppShell` renders with `NavRail` | [ ] |
| 3 | `SupervisionDashboard` (5-card) renders in main area | [ ] |
| 4 | No red errors in DevTools Console | [ ] |

### v01 §A — SessionsWorkspace (4 observations)

| # | Observation | Check |
|---|---|---|
| 1 | ⌘2 → SessionsWorkspace → seat rail populated from `projectData[activeProjectId]?.seats` | [ ] |
| 2 | Click seat → launch → claude/zsh starts → terminal renders output | [ ] |
| 3 | Session exit → exit code pill appears (green for 0, red for non-0) | [ ] |
| 4 | Kill button → `window.confirm` popup appears → kill works | [ ] |

### v01 §B — AppV2 header counters (3 observations)

| # | Observation | Check |
|---|---|---|
| 1 | All workitems Done → header pills for 阻塞/进行中 hidden | [ ] |
| 2 | `UPDATE workitems SET status='blocked' WHERE id='wi-001'` → reload → red pill shows "1 阻塞" | [ ] |
| 3 | Undo block → reload → pill hides again | [ ] |

### v01 §C — DocumentsWorkspace reconcile (4 observations)

| # | Observation | Check |
|---|---|---|
| 1 | Click 🔄 同步文档 → button disables, shows "正在同步…" | [ ] |
| 2 | Success → summary bar: `已扫描 N · 新增 X · 更新 Y · 未变 Z · 失败 F · 冲突 C` with counts matching DB | [ ] |
| 3 | `podman stop seatloom-postgres` → click reconcile → red error bar appears | [ ] |
| 4 | `podman start seatloom-postgres` → reconcile recovers → summary bar shows data | [ ] |

### A4-α — SessionsWorkspace attach UI mock-flow

| # | Observation | Check |
|---|---|---|
| 1 | ⌘2 → SessionsWorkspace → "Attach to tmux" dropdown with 3 mock entries | [ ] |
| 2 | Select mock session → click Attach → tab added, console.warn with mock message | [ ] |
| 3 | xterm terminal shows read-only banner "⚠️ Read-only mode (v0.0.1)…" | [ ] |
| 4 | Typing in terminal is disabled (no cursor, no input forwarding) | [ ] |

### A2 — NavRail Logo + ProjectSwitcher clipping

| # | Observation | Check |
|---|---|---|
| 1 | NavRail ← collapsed → SeatLoomLogo icon-only, centered, ~32px | [ ] |
| 2 | NavRail ← expanded → Logo + "SeatLoom" text, ~40px icon | [ ] |
| 3 | ProjectSwitcher dropdown → fully visible, not clipped, above all UI | [ ] |
| 4 | Click outside dropdown → closes | [ ] |

---

## Verdict by commit

| Commit | Packet | Layer A | Layer B | Verdict |
|---|---|---|---|---|
| `39af8b2` | A1 | PASS | pending | **PROVISIONAL PASS** |
| `578ff7c` | v01 §A | PASS | pending | **PROVISIONAL PASS** |
| `4651bb4` | v01 §B | PASS | pending | **PROVISIONAL PASS** |
| `2f83624` | v01 §C | PASS | pending | **PROVISIONAL PASS** |
| `e864392` | v01 factual-record | PASS | N/A (docs-only) | **PASS** |
| `bdac54b` | A4-α | PASS | pending | **PROVISIONAL PASS** |
| `3b7ac17` | A2 | PASS | pending | **PROVISIONAL PASS** |
| `b11d878` | revert archival | PASS (archival, no current change) | N/A | **PASS** |

## Overall verdict

**SG-A Layer A: PASS (all 8 commits). SG-A Layer B: PENDING Mr. Zhang Tauri runtime (20 observations).**

Layer A automated checks are clean across all commits — no scope violations, no i18n changes, tsc/build green, static invariants match packet acceptance criteria. Layer B requires a live Tauri instance; Flux's seat does not have a working `pnpm tauri dev` environment. Mr. Zhang's observation is the runtime source of truth.

## Blockers

- Layer B: 20 runtime observations across 5 deliverables require Mr. Zhang with `pnpm tauri dev` running. Flux cannot execute Layer B independently (no Tauri runtime on this seat).

## Next action

- Mr. Zhang to walk Layer B checklist and report per-observation checkmarks.
- On Layer B PASS: Lyra finalizes acceptance chain (A2 + A4-α no longer on hold; A1 + v01 provisional → final).
- On any Layer B FAIL: revert to HOLD on the failing commit, issue corrective packet.

---

*Artifact: `docs/coordination/tasks/flux/FLUX-2026-05-09-sg-a-combined-verification-delivery-v1.md`*