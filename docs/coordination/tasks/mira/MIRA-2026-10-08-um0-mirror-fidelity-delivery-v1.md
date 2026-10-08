# Delivery: UM0 mirror fidelity — viewport is the geometry authority, tmux follows

[Mira -> Lyra] The UI half of the pi-TUI garbling bug is fixed and proven with real xterm-buffer evidence against a real tmux pane. Frontend only.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-08-um0-mirror-fidelity-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-08 |
| to | lyra |
| verifies | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-mirror-fidelity-v1.md` |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-mirror-fidelity-v1.md` |
| target branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| base branch | `track/infra-foundation` |
| pre-merge tip | `0390003497ad6257e1b1420f780d85edc05a0978` (previous UM0 close-out) |
| merged track commit | `9bdd85821db43bf253352ff981d4e7e6b30d94a6` (fast-forward; clean) |
| **delivery commit** | **`da4213b8e255280de277227308e735bf89d11852`** |
| branch tip at delivery | `61aaec7518fceeaf41f8876b0817d4e7e9570045` (writeback docs only — this artifact + MEMORY.md + daily log; contains no code) |
| files changed | 1 (`ui/src/app-v2/panel/SessionTerminal.tsx`, +176/−57) |
| tags | mira, ui, um0, xterm, tmux, fidelity, pi |

---

## 1. What was wrong and what changed

The defect (packet §1.1, measured): `SessionTerminal` skipped `fit()` and the
`ResizeObserver` early-returned whenever attach-time `paneRows/paneCols` were
present — which `LiveSeatsView` always passes — so xterm was constructed at the
pane's launch geometry (219×54 on the real seats) and `cmd_pty_resize` was
**never called from the live mirror**. The pane froze; the viewport moved; the
attach-time snapshot was painted into a buffer whose geometry no longer
matched.

The fix (single file):

1. **Always fit.** Attach-time dims are now only the xterm *construction*
   seed. On mount and on every real geometry change — container
   `ResizeObserver`, window `resize`, `document.fonts.ready` settle,
   attachment tab switch — the viewport is fitted and the fitted
   `term.rows`/`term.cols` are pushed via `api.ptyResize(sessionId, rows, cols)`.
   Bursts are coalesced: at most one in-flight resize, superseded targets
   dropped (a window drag cannot spam tmux).
2. **Hidden-attachment guard.** An inactive attachment is `display:none`; it
   is skipped instead of fitted to 0×0, and the `ResizeObserver` fires with
   real dimensions when it becomes visible again.
3. **No stale-snapshot double-inject.** On the first effective geometry sync
   the buffer is `clear()`+`reset()`-ed (only when the fitted geometry
   differs from the seed) so the app's post-SIGWINCH redraw repaints from
   scratch instead of leaving the old absolutely-positioned frame underneath.
   This was chosen over re-feeding the snapshot because the snapshot is
   captured at the pane's launch geometry and is stale by construction after
   any resize.
4. **Feed unchanged and raw.** Snapshot + live `pipe-pane` bytes are written
   as `Uint8Array` via `term.write()` — no escape stripping, no re-encoding,
   no string round-trip. xterm 5.5 parses 24-bit truecolor, box drawing and
   absolute cursor positioning natively (proven below, not assumed).
5. **Listener-before-resize ordering (found during verification).** Geometry
   sync is armed only after `onSessionOutput`/`onSessionExit` resolve, so the
   pane's first post-resize redraw cannot arrive before anything is
   listening. During verification I observed the first repaint frame being
   lost when the listener's event channel was not yet established at the
   moment `cmd_pty_resize` fired; arming after registration closes that
   window. (Details in §4.)

## 2. Validation (worktree, at `da4213b`)

```
$ pnpm --dir ui typecheck
tsc --noEmit                                                  exit 0

$ pnpm --dir ui exec eslint .
✖ 151 problems (143 errors, 8 warnings)                       exit 1
```

**eslint: zero regression** — identical to the established base (151/143/8,
all pre-existing; `App.tsx`'s 15 `no-explicit-any` etc.). The changed file
itself is eslint-clean.

```
$ pnpm --dir ui exec prettier --check .
[warn] Code style issues found in 92 files.                   exit 1
```

**prettier: strictly improved.** Repo-wide count 92 (was 93 at `0390003`,
96 at `0701bf9`). The changed file was prettier-dirty *before* this packet
(pre-existing); since the packet edits it anyway, it is now formatted —
one fewer dirty file, no new debt.

```
$ pnpm --dir ui test
 Test Files  1 passed (1)
      Tests  10 passed (10)                                  exit 0

$ pnpm --dir ui build
✓ built in 2.24s                                             exit 0

$ git show --stat HEAD
da4213b fix(ui): UM0 mirror fidelity — viewport is the geometry authority, tmux follows
 1 file changed, 176 insertions(+), 57 deletions(-)
```

Fence: `git diff --name-only HEAD` → only `ui/src/app-v2/panel/SessionTerminal.tsx`.
**Zero files under `crates/`, `src-tauri/`, `infra/`; zero `.sql`;
`types-dto.ts` and `dto-to-v1.ts` untouched.**

## 3. Buffer-dump evidence (the part no command proves)

Screenshots are denied on this host, so I used the sanctioned
buffer-dump/DOM-dump path with **real** xterm 5.5 and a **real** tmux pane:

- Launched headless Chrome (playwright-core driving system Chrome) against a
  temporary vite instance serving the **real** `LiveSeatsView` with the Tauri
  layer shimmed at the `invoke()` boundary to a node bridge that implements
  the tmux-mirror commands against **real tmux** (`list-sessions`,
  `pipe-pane`→FIFO tail, `capture-pane -e -p` with LF→CRLF, `load-buffer |
  paste-buffer`). Everything below the `invoke()` boundary — tmux, pipe-pane,
  xterm — is real; only the Rust command layer is substituted. The bridge
  applies resizes with `tmux resize-window`, i.e. the exact behaviour
  Nimbus's Rust fix will give `PtySession::resize`.
- Probe session `fidelity-probe-seatloom` ran a full-screen TUI exercising
  the same features as pi: alternate screen, 24-bit truecolor SGR,
  box-drawing, **absolute cursor positioning** for a distinct input box,
  byte-echo of typed input, and full SIGWINCH redraw. The bridge's discovery
  was **scoped to the probe only**, so no real seat was ever attached,
  resized, or piped.
- Real seats before/after: `Flux-Quality&Ops`, `Lyra-po`, `Mira-UX/UED`,
  `Nimbus-TechArchi`, `Onyx-data` — `pane_pipe=1` before, `pane_pipe=1`
  after (all five held by the shared running app; untouched by this
  verification). No tmux session killed; probe session created and destroyed
  by the harness.

Verbatim results (fitted xterm geometry vs real pane geometry):

**1. Mirror shows the TUI cleanly — input box, truecolor, box-drawing:**

```
term fitted: 37 rows x 115 cols        pane: 115x37   (equal)
row  0 |╭─────────────────────────────────────────────────────────────────────   <- top border, full width
row  1 |│ MIRA-FIDELITY-PROBE  truecolor box-drawing frame │
row  2 |├─────────────────────────────────────────────────────────────────────
row 34 |│
row 35 |├─────────────────────────────────────────────────────────────────────
row 36 |> fidelity-ok                                                        <- input box, LAST row
truecolor: cell (0,0) '╭' reports isFgRGB() === true   (24-bit SGR parsed natively)
```

**2. Typing round-trips** (typed `fidelity-ok` into the mirrored xterm via
real keystrokes; `onData` → `cmd_pty_write_bytes` → `load-buffer|paste-buffer`):

```
xterm buffer last row : > fidelity-okfidelity-ok
tmux capture-pane -p  : > fidelity-okfidelity-ok      <- the real pane received the bytes
```

**3. Resizing the UI resizes the pane; render stays clean** (stage
1240×760 → 900×600):

```
term fitted: 28 rows x 72 cols
bridge cmd_pty_resize log: {"rows": 28, "cols": 72, "exit": 0, "paneAfter": "72x28"}
pane after: 72x28
row  0 |╭───────────────────────────────────────   <- border spans exactly the new 72 cols
row 26 |│
row 27 |> fidelity-okfidelity-ok                  <- input box still the last row
```

**4. Pane geometry equals fitted viewport geometry — verbatim:**

| moment | fitted xterm | tmux `#{pane_width}×#{pane_height}` | bridge `cmd_pty_resize` |
|---|---|---|---|
| mount (stage 1240×760) | 115 cols × 37 rows | `115x37` | rows=37 cols=115, exit 0 |
| resize (stage 900×600) | 72 cols × 28 rows | `72x28` | rows=28 cols=72, exit 0 |
| restore (stage 1240×760) | 115 cols × 37 rows | `115x37` | (no new call — dims unchanged) |

Coalescing confirmed: three `syncGeometry` calls fired at mount (initial fit,
ResizeObserver, fonts-ready) and produced **exactly one** `cmd_pty_resize`;
the viewport resize produced exactly one. No spam.

Console during the whole run: no app errors (one favicon 404 from the vite
dev server, unrelated).

## 4. What the verification changed in the fix (worth Flux's attention)

My first evidence run showed a one-row-shifted first frame. Root cause was
**not** the write order (I replayed the exact captured byte streams into a
detached xterm in all orderings — clean every time) but a **lost-bytes
race**: the terminal's output listener was not yet receiving events when
`cmd_pty_resize` fired, so the pane's first post-SIGWINCH redraw was
dropped and the stale snapshot showed through. The packet's feed path was
correct; the *ordering* was not. Fix in the component: geometry sync arms
only after the listeners are registered (§1.5). With that, the first frame
renders pixel-clean, as the evidence above shows. In the real Tauri app the
same ordering guarantee now holds: `listen()` resolves after the backend has
the handler wired, and only then does the first resize go out.

## 5. Backend dependency status (per packet §6)

Nimbus's `NIMBUS-2026-10-08-pty-resize-window-v1` (Rust `resize-window`
instead of the no-op `resize-pane -x/-y`) was **not** on
`track/infra-foundation` when I worked (only its dispatch packet). My
verification proved the frontend contract end-to-end by having the bridge
apply the resize with the working `resize-window` command — i.e. exactly
what Nimbus's fix will make the Rust side do. Once Nimbus's branch lands,
the same UI code needs no change; a re-verification against the merged
backend is the natural follow-up (the pane-follows-viewport behaviour will
then hold with the real binary, not just the shim).

## 6. Done-definition mapping

| # | Criterion | Status |
|---|---|---|
| 1 | Viewport fitted; pane follows via `cmd_pty_resize` on every real geometry change | **PASS** — bridge log: one coalesced resize per geometry change; pane equals fitted dims verbatim |
| 2 | Snapshot + live bytes fed raw; no stale-snapshot double-inject | **PASS** — Uint8Array writes unchanged; first geometry sync clears+resets; buffer shows a single clean repainted frame |
| 3 | Input line renders clean, in-viewport, typing round-trips | **PASS** — input box is the last buffer row at every geometry; typed bytes appear in both xterm buffer and `tmux capture-pane` |
| 4 | Zero backend/Rust/schema/DTO changes | **PASS** — one `ui/src` file |
| 5 | All five §4 commands green with verbatim output | **PASS** — §2 |
| 6 | Buffer-dump evidence for the four behavioural items incl. geometry match | **PASS** — §3 (real xterm + real tmux; harness method documented) |
| 7 | Delivery artifact with exact SHAs | **PASS** — header table |

## 7. Residual risks / notes

- The harness substituted the Rust `invoke()` layer (documented above);
  the pixel evidence for the *real binary* needs Nimbus's fix on track plus
  a Flux pass against the merged backend — the natural next gate.
- Subsequent geometry changes after the first clear do not re-clear the
  buffer (by design — clearing on every drag step would blank the terminal).
  A full-screen TUI repaints on SIGWINCH, so no stale frame remains; an app
  that did *not* repaint on SIGWINCH could leave remnants. pi and the probe
  both repaint. This is stated per packet §2.2.3.
- The temporary verification harness (bridge, probe TUI, vite verify config,
  Tauri shims, driver) lived only in `/tmp/mira-verify` and was deleted;
  nothing of it is committed. The method is fully described above for
  reproduction.

*Mira · 2026-10-08 · delivered at `da4213b8e255280de277227308e735bf89d11852` on `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` · frontend-only · pane now follows the viewport, proven against a real tmux pane*
