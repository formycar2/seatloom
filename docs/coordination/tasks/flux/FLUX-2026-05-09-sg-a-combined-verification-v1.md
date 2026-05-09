# Task: SG-A Combined Commit-Pinned Verification (A1 + v01 + A4-α + A2)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-09-sg-a-combined-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-09 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-05-10 |
| depends_on | `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (commit a5998c1), SG-A dispatch packet (commit b401af6), SG-A amendments (commit 48af37d) |
| tags | flux, verification, SG-A, commit-pinned, a1, v01, a4-alpha, a2, no-code-edit |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Verify-only. No code edits. No scope proposals. |
| branch | `track/infra-foundation` |

---

## 0. Context

Four SG-A deliverables have been committed but **have not yet passed Flux commit-pinned verification**. Lyra issued provisional acceptance on A1 (`39af8b2`) and v01 (`e864392`) at commit `a271aa8` without waiting for Flux — that was a process miss. Aegis posted pre-flight reviews on A4-α (`bdac54b`) and A2 (`3b7ac17`) with "可 acceptance" verdicts; those are Aegis's pre-flight checks, not Flux's commit-pinned verify layer, and the two are distinct checkpoints in the SG-A protocol.

This packet **closes that gap**. Flux verifies all four commits independently against their packet acceptance criteria. On PASS, Lyra finalizes the acceptance chain. On HOLD, Lyra rolls back the provisional acceptances and issues corrective packets.

## 1. Target commits

| Target | Commit | Packet |
|---|---|---|
| A1 (Nimbus — V1 white-screen diagnosis, no-fix PASS) | `39af8b2` (delivery doc; tree HEAD at verification `48af37d`) | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v01-white-screen-diagnosis-v1.md` |
| v01 §A | `578ff7c` | `docs/coordination/tasks/mira/MIRA-2026-05-09-v01-sessions-live-wiring-v1.md` |
| v01 §B | `4651bb4` | (same packet) |
| v01 §C | `2f83624` | (same packet) |
| v01 factual-record delivery | `e864392` | (delivery doc authored by Aegis after Mira 4× failure) |
| A4-α | `bdac54b` | `docs/coordination/tasks/mira/MIRA-2026-05-09-v001-sessions-attach-ui-v1.md` |
| A2 | `3b7ac17` | `docs/coordination/tasks/mira/MIRA-2026-05-09-v02-navrail-logo-and-zindex-v1.md` |

Note on v01 revert `b11d878`: archival history only (undoes scope-violating `31367a0`). Not a deliverable. Verify that it is **not** a current working-tree change.

## 2. Layer A — Automated checks (run in order, paste verbatim output)

### A.1 Tree state at HEAD

```bash
git rev-parse HEAD                                # record current HEAD
git rev-parse --abbrev-ref HEAD                   # must be track/infra-foundation
git status --short                                # note any uncommitted changes in ui/ or src-tauri/
```

### A.2 Commit chain presence

```bash
git cat-file -e 39af8b2 && echo "A1 delivery commit present"
git cat-file -e 578ff7c && echo "v01 §A present"
git cat-file -e 4651bb4 && echo "v01 §B present"
git cat-file -e 2f83624 && echo "v01 §C present"
git cat-file -e e864392 && echo "v01 factual-record present"
git cat-file -e bdac54b && echo "A4-α present"
git cat-file -e 3b7ac17 && echo "A2 present"
git cat-file -e b11d878 && echo "revert of 31367a0 present (archival)"
```

### A.3 Per-commit scope audit (must match packet scope)

```bash
git show 39af8b2 --stat   # expect: docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v01-white-screen-diagnosis-delivery-v1.md (docs-only)
git show 578ff7c --stat   # expect: ui/src/app-v2/panel/SessionsWorkspace.tsx only
git show 4651bb4 --stat   # expect: ui/src/app-v2/AppV2.tsx only
git show 2f83624 --stat   # expect: ui/src/app-v2/views/DocumentsWorkspace.tsx only
git show bdac54b --stat   # expect: ui/src/app-v2/panel/SessionTerminal.tsx + ui/src/app-v2/panel/SessionsWorkspace.tsx
git show 3b7ac17 --stat   # expect: ui/src/components/NavRail.tsx + ui/src/components/ProjectSwitcher.tsx
```

Each commit must touch **only the files its packet scope permits**. Any stray file = HOLD.

### A.4 i18n.ts integrity (hard checkpoint)

```bash
git diff 48af37d..HEAD -- ui/src/i18n.ts    # must be empty (no change since SG-A amendment commit)
```

The v01 HOLD-1 incident was i18n.ts −60 lines. Any modification to `ui/src/i18n.ts` since `48af37d` without an explicit packet authorizing it = HOLD.

### A.5 TypeScript + Build

```bash
cd ui && pnpm exec tsc --noEmit             # must be zero errors
cd ui && pnpm build                         # must exit 0
```

Paste full output of both verbatim. For `tsc` with empty output, explicitly note the empty-output + exit code.

### A.6 Static invariants

On commit `bdac54b` (A4-α):

- [ ] `MOCK_TMUX_SESSIONS` array present in `SessionsWorkspace.tsx`, contains exactly 3 entries matching spec (`Lyra-po-seatloom`, `Nimbus-TechArchi-seatloom`, `Mira-UX/UED-seatloom`).
- [ ] Two `TODO(A4-β)` markers present in the diff (grep the commit patch).
- [ ] `console.warn('A3 not yet available; mock attach for <session>')` text present in `attachToTmux()` stub.
- [ ] `disableStdin: true` and `cursorBlink: false` both present in `SessionTerminal.tsx` xterm initialization.
- [ ] Read-only banner component with text `⚠️ Read-only mode (v0.0.1)...` present.
- [ ] No `invoke('cmd_list_tmux_sessions')` actually called (mock only). No `invoke('cmd_attach_tmux_session', ...)` actually called (mock only).

On commit `3b7ac17` (A2):

- [ ] `NavRail.tsx` adds `SeatLoomLogo` above `ProjectSwitcher`; renders in both collapsed and expanded states.
- [ ] `ProjectSwitcher.tsx` uses `ReactDOM.createPortal` for dropdown.
- [ ] Dropdown position uses `getBoundingClientRect()` against the trigger button.
- [ ] Dropdown `z-index` is high enough (≥ 9999 per Aegis review).
- [ ] Dropdown is not a descendant of any `overflow: hidden` ancestor that could clip it.
- [ ] `SupervisionDashboard`, `SupervisorPanel`, `i18n.ts`, `src-tauri/**` are untouched.

On commits `578ff7c` / `4651bb4` / `2f83624` (v01):

- [ ] §A `SessionsWorkspace.tsx` reads seats from `useDataStore.projectData[activeProjectId]?.seats` (not `api.listSeats()`); xterm launch path preserved; kill-confirm dialog present for running sessions.
- [ ] §B `AppV2.tsx` header shows counters for 阻塞 and 进行中 only when count > 0; pills hide when counts are zero.
- [ ] §C `DocumentsWorkspace.tsx` has 🔄 同步文档 button that disables during flight, shows summary line `已扫描 N · 新增 X · 更新 Y · 未变 Z · 失败 F · 冲突 C`, calls `hydrateFromBackend()` on success, shows red error banner on failure.

On A1 (`39af8b2`):

- [ ] Delivery doc at `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v01-white-screen-diagnosis-delivery-v1.md` exists.
- [ ] Commit is docs-only (no source changes).
- [ ] Tree HEAD at verification was `48af37d`; `TopErrorBoundary` is installed at `ui/src/main.tsx` (introduced at `935e77a`, not modified by A1).

## 3. Layer B — Runtime verification (Mr. Zhang assists)

Flux coordinates with Mr. Zhang for the live-run portions that require a browser / Tauri shell:

- **A1**: Re-verify white-screen non-repro at current HEAD (`pnpm tauri dev` → no white screen, no red card, IM + main shell render). If the symptom re-emerges, A1 reverts to HOLD and Nimbus gets a re-diagnosis packet.
- **v01 §A / §B / §C**: Walk the 4+3+4 observations in the Aegis factual-record delivery (`docs/coordination/tasks/mira/MIRA-2026-05-09-v01-sessions-live-wiring-delivery-v1.md`) against the running app. Each observation must reproduce.
- **A4-α**: Open ⌘2 Sessions; verify mock dropdown, three mock entries, attach no-op with console.warn, read-only banner, `disableStdin` behavior.
- **A2**: Verify NavRail Logo in collapsed + expanded states; verify ProjectSwitcher dropdown renders above sibling containers and is not clipped by overflow:hidden ancestors.

If Flux cannot run Tauri on its seat, flag so in the delivery — Mr. Zhang's observation is the runtime source of truth for Layer B; Flux's job is to enumerate which observations Mr. Zhang needs to check and record the outcome.

## 4. Delivery format

Write delivery to:

```
docs/coordination/tasks/flux/FLUX-2026-05-09-sg-a-combined-verification-delivery-v1.md
```

Required sections:

1. `[Flux -> Lyra]` header line
2. One verdict per commit: A1 / v01 (three commits) / A4-α / A2 — each PASS or HOLD
3. Layer A output verbatim for each check (no summaries)
4. Static invariant checklist — tick each or explain why not
5. Layer B runtime observation record (by Flux if Tauri runs on your seat; otherwise list for Mr. Zhang)
6. Overall verdict: `SG-A PASS pending A3` or `SG-A HOLD — <commit> <reason>`
7. Blockers section
8. Artifact path

## 5. Scope constraints

- **No code edits**. This is verify-only.
- **No proposals**. Architecture notes belong in a separate review doc, not in this delivery.
- **Paste tool output verbatim** per `COORDINATION_RULES.md` §3.
- **Commit hashes are pinned**: verify each at the exact commit listed in §1. Do not verify "latest HEAD" if it has advanced.

## 6. Concurrency

This packet runs in parallel with Nimbus A3 (tmux pipe-pane read path, in progress). Flux does not gate A3. A3's own verify packet will issue separately when Nimbus delivers A3.

---

*Issued by Lyra · 2026-05-09 · Corrective process gate — Flux commit-pinned verify was skipped on A1/v01/A4-α/A2 acceptances. This packet closes that gap before SG-A stage-gate decision.*
