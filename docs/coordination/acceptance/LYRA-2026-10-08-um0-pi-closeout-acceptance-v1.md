# Acceptance: UM0 close-out on the migrated PI team

[Lyra] Verdict: **ACCEPT — PASS**. UM0 is closed. Mira's live-mirror UM0 branch and Nimbus's pipe-pane re-attach fix are both merged into `track/infra-foundation`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | acceptance |
| id | LYRA-2026-10-08-um0-pi-closeout-acceptance-v1 |
| status | accepted |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| milestone | UM0 — one screen for the whole team (real) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-closeout-on-pi-v1.md` |
| nudges | `/tmp/lyra-nudge-flux.md`, `/tmp/lyra-nudge-merge.md` |
| delivery under test | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-live-empty-state-closeout-delivery-v1.md` |
| verification under test | `docs/coordination/tasks/flux/FLUX-2026-10-08-um0-pi-reverification-delivery-v1.md` |
| target branch | `track/infra-foundation` |
| mira branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| mira verified commit | `0390003497ad6257e1b1420f780d85edc05a0978` |
| mira code commit | `1693a2345399f4fcd213bdc732bee1dec62ad3d1` |
| flux evidence branch | `packet/flux/FLUX-2026-10-08-um0-pi-reverification-v1` |
| flux evidence commit | `faec42abed23547c8eb00d124cfc27376d7677ef` |
| nimbus branch | `packet/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` |
| nimbus code commit | `abe4245` |
| tags | um0, acceptance, pi, live-mirror, pipe-pane |

---

## 1. Decisions needed

None outstanding. Aegis resolved HOLD-1 and HOLD-2 by ruling; Mira closed HOLD-3 by delivery; Flux verified commit-pinned and returned PASS. The only residual decision (whether `App.tsx`'s 15 pre-existing `no-explicit-any` findings must be cleaned before merge, or the ratchet base adjusted) is carried as a follow-up, not a UM0 blocker, per Flux's explicit recommendation and the zero-regression standard.

## 2. Decisions made

1. **UM0 is accepted as PASS.** The gate condition — the SeatLoom UI shows the live seats, and never shows a false `LIVE` state — is met on the verified commit.
2. **HOLD-1 (prettier-ratchet count instability)** — closed by `ui/.prettierignore` (`034ef1e`), independently measured by Flux: prettier count is **93 in all three dist conditions** (present pre-build, present post-build, absent) versus **96/103/96** at `0701bf9`. Zero new debt by offender-set comparison; the −3 is the ignored lockfile plus `App.tsx`/`NavRail.tsx` being formatted.
3. **HOLD-2 (render observability)** — closed by the sanctioned buffer-dump path: `pane_pipe` 0→1 under a controlled test, +42 KB/3 s transcript growth, and Flux's own session output recovered verbatim with ANSI intact from the mirrored stream. HOLD-3's badge matrix was additionally read from a real headless Chrome DOM.
4. **HOLD-3 (false LIVE badge on empty discovery)** — closed by Mira's fix at `1693a23`: empty discovery renders `NO LIVE SEATS`, failed discovery renders `DISCOVERY FAILED`, the nav-rail dot agrees, and the healthy path returns to green `LIVE` for the five canonical `-seatloom` seats.
5. **Both branches are merged into `track/infra-foundation`** — Mira's UM0 branch and Nimbus's pipe-pane fix, per Aegis's dispatch sequence step 3.

## 3. Action packets (executed)

| # | Owner | Action | Done-definition | Status |
|---|---|---|---|---|
| 1 | Mira | HOLD-3 close-out on `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` after merging `track/infra-foundation` | no false LIVE on empty/failed discovery; five canonical seats still discovered; typecheck/eslint/prettier/build green; frontend-only | **DELIVERED** @ `1693a23`, branch tip `0390003` |
| 2 | Flux | Commit-pinned re-verification against exact target commit | exact commit from clean workspace; HOLD-1/2/3 cleared; live mirror proven moving; PASS or specific HOLD | **PASS** @ `faec42a`, verified commit `0390003` |
| 3 | Lyra | Accept + merge Mira UM0 branch into `track/infra-foundation` | merge recorded, no scope leakage, writeback | **DONE** |
| 4 | Lyra | Merge Nimbus pipe-pane fix `packet/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` into `track/infra-foundation` | merge recorded, writeback | **DONE** |
| 5 | Lyra | Park merge-complete note for Aegis | daily-log entry with merged SHAs | **DONE** — §6 |

## 4. Blockers

- None open against UM0.
- Carried as non-blocking follow-ups (documented, not accepted silently):
  1. The `sessions` (会话) tab still asserts `live` provenance unconditionally because that older attach-only surface never runs `listTmuxSessions()`. Same false-live family as HOLD-3; recommended for a Mira consolidation follow-up.
  2. `scripts/lint-changed.sh` is still red on `App.tsx`'s 15 pre-existing `no-explicit-any` findings (15 at base `324e185`, 15 at `5132875`, 15 at target — delta 0) plus one pre-existing `StatusBar.tsx` prettier finding. Strictly improved versus `0701bf9`; the ratchet base decision remains a governance item.
  3. Re-attach semantics (`abe4245`) are newly reachable after merge but were **not** covered by Flux's PASS — Flux verified Mira's branch, which predates `abe4245`, and explicitly did not extend the PASS to re-attach behaviour. A Flux pass against the merged branch is a follow-up.

## 5. Stage-gate status

- **UM0: GO.** All three original HOLD parts are closed. The SeatLoom UI shows the five live PI seats (`Flux-Quality&Ops-seatloom`, `Lyra-po-seatloom`, `Mira-UX/UED-seatloom`, `Nimbus-TechArchi-seatloom`, `Onyx-data-seatloom`) and no longer presents an empty discovery as live.
- Next: Aegis reconciles `track/infra-foundation` to GitLab and restarts the shared UI onto the merged code, which completes "operate the team through the UI" for the first half of the goal.
- Operational note for whoever restarts the shared UI: Flux observed the shared app (pid 52131) and its vite (pid 35321, port 1420) closed externally at 19:28:28 — a graceful human window close, no crash report. A fresh `tauri dev` on the shared checkout is required before re-pointing it at the merged code.

## 6. Merge record

Recorded verbatim at merge time in this checkout (branch `track/infra-foundation`):

| Step | Branch | Commit | Result | Merge commit |
|---|---|---|---|---|
| Mira merge | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` | `0390003497ad6257e1b1420f780d85edc05a0978` | merged `--no-ff`; conflicts in `docs/coordination/MEMORY.md` and `docs/coordination/memory/2026-10-08.md` resolved by retaining both seats' writeback; no code conflicts | `feffcb7` |
| Nimbus merge | `packet/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` | `63cfaeb980ab92a7ca23634a087548266e03d50c` (code `abe4245`) | merged into `track/infra-foundation`, clean | `1a34007` |

Both target commits are confirmed ancestors of `track/infra-foundation` after the merges (`git merge-base --is-ancestor` for `0390003` and `abe4245` both return 0).

Scope proof on the Mira delta (`5132875..0390003`): 14 files, +2030/−71, all under `ui/` and `docs/`; zero `crates/`, `src-tauri/`, `infra/`, `.sql`, `types-dto.ts`, `dto-to-v1.ts`.

## 7. Follow-ups and owners

| Item | Owner | Note |
|---|---|---|
| Consolidate the two attach surfaces (`sessions` tab unconditional `live`) | Mira | Follow-up packet; not a UM0 blocker |
| `App.tsx` 15 `no-explicit-any` behind the ratchet | Nimbus / Aegis governance | Ratchet-base decision, unchanged from v1 |
| Flux pass on re-attach semantics against the merged branch | Flux | Follow-up; `abe4245` is now reachable |
| Pipe state close on session drop (extends the `-o` toggle bug) | Nimbus | Already recommended inside `NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` scope |
| Aegis reconcile track → GitLab + restart shared UI | Aegis | Parked in daily log per dispatch |

---

*Accepted by Lyra · 2026-10-08 · UM0 GO on the migrated PI team · Mira UM0 merged at `feffcb7` (`0390003`) + Nimbus pipe-pane merged at `1a34007` (`abe4245`) into `track/infra-foundation` · Flux PASS at `faec42a`*
