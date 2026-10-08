# Delivery: UM0 close-out — false LIVE state on empty discovery (HOLD-3)

[Mira -> Lyra] HOLD-3 is fixed. Frontend-only, behaviour-preserving, on the original UM0 branch after merging `track/infra-foundation`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-08-um0-live-empty-state-closeout-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-08 |
| to | lyra |
| verifies | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-live-empty-state-closeout-v1.md` |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-closeout-on-pi-v1.md` |
| target branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| base branch | `track/infra-foundation` |
| **base commit (pre-merge)** | `0701bf9ded96638b378742bb627c4a93d119e101` |
| track commit merged | `5132875364050a49de59e980f9ee4604b4e84adb` (contains `.prettierignore` `034ef1e` + migration `011`) |
| merge commit | `e36f676b847f72f4d67c7a36098a793496a56993` (clean, no conflicts) |
| **delivery commit** | **`1693a2345399f4fcd213bdc732bee1dec62ad3d1`** |
| branch tip at delivery | `b42add9e2d8b623430cb6d0d94acb9bb6d77bc75` (writeback docs only — this artifact + MEMORY.md + daily log; contains no code) |
| tags | mira, ui, um0, live-mirror, hold-3, closeout, pi |

---

## 1. Verdict

**HOLD-3 fixed.** The live mirror no longer presents itself as `LIVE` when tmux
discovery returns zero seats or fails. An explicit `NO LIVE SEATS` /
`DISCOVERY FAILED` state is rendered instead — in the provenance banner, in the
nav-rail dot, and (as before) in the view body. When seats are present, the
previously delivered live-mirror behaviour is untouched.

HOLD-1 (`.prettierignore`) and HOLD-2 (buffer-dump verification path) were
resolved off-branch by Aegis/Lyra; this packet implemented **only** HOLD-3.

## 2. Merge record

Per packet step 1–2, in the Mira worktree (`…/seatloom-worktrees/mira`):

```
$ git status --short          # (empty — clean)
$ git branch --show-current   # packet/mira/MIRA-2026-09-30-live-mirror-um0-v1
$ git merge track/infra-foundation --no-edit
Merge made by the 'ort' strategy.   # no conflicts
```

Confirmed picked up from the track: `ui/.prettierignore` (commit `034ef1e`) and
`infra/postgres/schema/011_agent_history_archive.sql` (commit `6b2addb`), plus
Nimbus's P0 step-2 write API and the coordination artifacts. Resulting merge
commit `e36f676`; my fix then landed as `1693a23` on top.

## 3. What changed (exact files)

7 files, +522/−80, all under `ui/src/` plus one docs file (the dispatched
packet, committed so the branch is self-contained):

| File | Change |
|---|---|
| `ui/src/lib/provenance.ts` | `Provenance` gains `'no-live'` and `'discovery-failed'` states with explicit banners (`NO LIVE SEATS` / `DISCOVERY FAILED`, headline `No live seats found — discovery returned empty` / `Live seat discovery failed`). New `SeatDiscovery` type (`'unknown' \| 'ok' \| 'empty' \| 'failed'`). `provenanceForTab` takes the discovery status and returns the degraded states for the live-mirror tab only; every other surface's mapping is byte-identical to before. |
| `ui/src/views/LiveSeatsView.tsx` | Each discovery poll publishes its outcome to the store: `ok` (seats found), `empty` (zero seats), `failed` (IPC threw). Browser mode (no Tauri backend → no tmux IPC) publishes `empty`. No render logic otherwise touched. |
| `ui/src/stores/useLiveSessionsStore.ts` | Carries `seatDiscovery` + `setSeatDiscovery` alongside the existing session map; initial value `'unknown'` (first poll in flight). |
| `ui/src/App.tsx` | Feeds `seatDiscovery` into `provenanceForTab`; the "查看实时席位" jump button is suppressed on the live tab itself (it would be a no-op there). |
| `ui/src/components/NavRail.tsx` | Same feed for the nav-rail provenance dots, so the dot cannot read LIVE while discovery is empty/failed. |
| `ui/src/lib/provenance.test.ts` | New. Pins the healthy path (all five canonical `-seatloom` PI seats parse + sort) and the HOLD-3 mapping (ok→live, empty→no-live, failed→discovery-failed, other tabs unchanged). |
| `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-live-empty-state-closeout-v1.md` | The dispatched packet, copied onto the branch. |

Transient state, stated plainly: between mount and the first poll result the
status is `'unknown'`, which renders `live`. This resolves within one IPC
round-trip (the poll fires on mount) and is not the held defect — the held
defect was the *persistent* false LIVE over an empty/failed discovery, which
no longer exists.

## 4. Done-definition mapping

| # | Criterion | Status |
|---|---|---|
| 1 | Empty discovery no longer displays false `LIVE` | **PASS** — banner + dot + view body all render `NO LIVE SEATS` / `DISCOVERY FAILED` |
| 2 | Healthy path still supports the five canonical PI seats | **PASS** — `provenance.test.ts` parses `Flux-Quality&Ops-`, `Lyra-po-`, `Mira-UX/UED-`, `Nimbus-TechArchi-`, `Onyx-data-seatloom`; discovery logic unchanged from the UM0 delivery |
| 3 | Live mirror / provenance behaviour not regressed | **PASS** — only the banner/dot mapping for the live tab is conditioned on discovery; attach/detach, byte counters, provenance labelling of other tabs untouched |
| 4 | Validation commands green | **PASS** — §5; zero-regression standard, identical eslint count, prettier strictly improved |
| 5 | Delivery artifact with exact commit + evidence | **PASS** — this file |

## 5. Validation evidence (run in the worktree at `1693a23`)

```
$ pnpm --dir ui typecheck
tsc --noEmit                                            exit 0

$ pnpm --dir ui exec eslint .
✖ 151 problems (143 errors, 8 warnings)                 exit 1
```

**eslint: zero regression.** Base measurement (Flux, at `0701bf9`; Mira, at
`324e185`): 151 problems / 143 errors / 8 warnings — identical. Attribution:
the only changed file with findings is `App.tsx`'s 15 pre-existing
`no-explicit-any` (15 at base, 15 at head; my hunks are elsewhere). All other
changed files — including the new `provenance.test.ts` — are clean.

```
$ pnpm --dir ui exec prettier --check .
[warn] … Code style issues found in 93 files.           exit 1
```

**prettier: no regression, strictly improved, and now stable.** The repo-wide
count cannot reach 0 on this tree (93 pre-existing unformatted files remain,
e.g. `src/layouts/StatusBar.tsx`), so the operative standard is the one Flux
applied: identical-or-better count with no new debt in changed files. Base was
96 files; now 93. The −3: `App.tsx` and `NavRail.tsx` were formatted (they had
to be edited anyway; formatting-only, no behaviour change) and the lockfile
entries are excluded by the merged `.prettierignore`. **Build-order stability
proven:** count is 93 both before and after `pnpm --dir ui build` (was 96 → 103
depending on `dist` before `.prettierignore` — that instability was HOLD-1).

```
$ pnpm --dir ui test
 Test Files  1 passed (1)
      Tests  10 passed (10)                             exit 0

$ pnpm --dir ui build
✓ built in 3.62s                                        exit 0

$ git show --stat HEAD
1693a23 fix(ui): UM0 HOLD-3 — live mirror refuses a false LIVE badge on empty discovery
 7 files changed, 522 insertions(+), 80 deletions(-)
```

For Flux's re-verification, the CI ratchet (`bash scripts/lint-changed.sh`)
currently reports: `App.tsx` 15 pre-existing `no-explicit-any` errors
(unchanged count, lines shifted by my import/hook additions) and
`StatusBar.tsx` prettier (pre-existing, untouched by me). Every other changed
file is ratchet-clean. This matches the pre-existing-debt picture Flux
documented at `0701bf9`, minus two files that are now formatted.

## 6. Fence statement

**No backend, Rust, schema, DTO-boundary, or infra files were changed.**
Verified:

```
$ git diff --name-only e36f676..1693a23 | grep -vE '^(ui/|docs/)'   -> (none)
$ git diff --name-only e36f676..1693a23 | grep -E '^(crates|src-tauri|infra)/|\.sql$|types-dto\.ts|dto-to-v1\.ts' -> (none)
```

## 7. Flow coverage checklist (Mira persistence contract §5.2)

| Flow | State |
|---|---|
| Healthy discovery → seats listed, auto-attach, byte counters, provenance LIVE banner | implemented (unchanged from UM0 delivery) |
| Empty discovery → explicit no-live state (banner + dot + body) | **implemented (this packet)** |
| Discovery failure → explicit failure state with retry guidance | **implemented (this packet)** |
| Browser mode (no Tauri) → explicit "no backend, no seats" | implemented (this packet; previously only body text) |
| Other tabs' seed/fixture provenance labelling | implemented (unchanged) |
| Rendered-pixel confirmation of the new banners | **partial** — code + computed-style level only; needs Flux's webview buffer-dump evidence or a human |

## 8. Notes for Flux re-verification

- Target: branch `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1`, commit `1693a2345399f4fcd213bdc732bee1dec62ad3d1`.
- Empty-state check without killing seats: point the app at a tmux server with no `-seatloom` sessions (e.g. `tmux -L empty-probe ls`), or stub `cmd_list_tmux_sessions` to return `[]`; the banner must read `NO LIVE SEATS`, never `LIVE`.
- Failure check: make the IPC call throw; banner must read `DISCOVERY FAILED`.
- Healthy path: the five canonical PI seats should be discovered and the banner returns to green `LIVE`.

*Mira · 2026-10-08 · delivered at `1693a2345399f4fcd213bdc732bee1dec62ad3d1` on `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` · HOLD-3 fixed, frontend-only, zero backend/Rust/schema changes*
