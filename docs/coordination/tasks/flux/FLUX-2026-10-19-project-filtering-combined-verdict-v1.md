# Verdict — the combined project-filtering candidate (`packet/nimbus/project-filtering-combined`)

- **Verification dispatch**: `disp-HqkDgrVMF0Fbcu7pcKtqY` (issued by seat-aegis-001), covering
  implementation dispatch **`disp-9H3OCmprBFAkiYBa5MDwE`**.
- **Candidate under test**: branch `packet/nimbus/project-filtering-combined`, product commit
  `2c69adb`, delivery commit `4149b7f` (the tip), off `track/infra-foundation` @ `98876aa`
  (0 behind / 2 ahead — it contains the track tip). Verified in its own worktree
  `/tmp/nimbus-project-filtering-combined`, which was clean.
- **Verdict event**: `ev-K3Zfw40dtF7AQfLLh_kTY` — the pre-declared id, emitted exactly as required,
  with a `ref_type='dispatch'` ref naming **`disp-9H3OCmprBFAkiYBa5MDwE`** (the implementation
  dispatch), not just this verification dispatch.
- **VERDICT: PASS.** The candidate is the honest combination: no unrelated or unverified
  functionality, both seats' intentions preserved in the three shared files, every gate green
  **run by me**, and the live read path measured by me project by project. Three scoped
  observations are recorded below — none of them a defect.

## Scope audit — 39 files, all belonging to this work

I read the complete change set `98876aa..4149b7f` file by file: 39 files, +1546/−268. Every file is
project filtering or its supporting evidence — the commands (11), the store and the views it feeds
(11), the probe and proof (3), the migration script (1), the delivery note (1), plus tests. **No
unrelated functionality, no drive-by refactors, no scope creep.** The only file outside
`crates/` / `src-tauri/` / `ui/` is Nimbus's own delivery note and the operator's migration script,
both accounted for.

## The three shared files — both seats' intentions, preserved

The candidate is a **single squashed commit** off track (`2c69adb`'s parent is `98876aa`, not a
merge), so the "three-file conflict" I recorded on 2026-10-17 did not recur as a git conflict — it
was resolved by hand and the resolution carries both sides:

- `useDataStore.ts` — **Nimbus's** single-project hydration (`hydrateProjectData`, read once and
  re-read on switch) **and Mira's** no-project guards ("请选择一个项目" / "还没有选定项目"), the
  latter pinned by `projectFilteringCombined.test.tsx` (3 tests, passing by name).
- `DispatchInboxView.tsx` — the scoped read with `activeProjectId` **and** the load dep, plus her
  explicit unresolved state.
- `api.ts` — all ten read signatures require `projectId`.

## Gates — run by me in the candidate worktree, none taken from his summary

`cargo fmt --all -- --check` clean · `cargo check --workspace --all-targets` clean ·
`cargo clippy --workspace --all-targets -- -D warnings` clean · `cargo test --workspace`
**278/0** · `tsc --noEmit` clean · vitest **201/201** (15 files) · `pnpm build` green ·
`project_scope_proof.sh` **ALL POINTS PASS (10)** — and inside it, the helper's **3 unit tests**
(absent → error naming the command; supplied → passthrough; message names the reason) pass.

## The live reads, called by me — numbers are a snapshot, not a permanent assertion

| read | seatloom | ql_delivery | ColossalAI (corpus) |
|---|---|---|---|
| seats (bound via `project_role_bindings`) | **7** — aegis, flux, lyra, mira, nimbus, onyx, vela; **no QL** | **1** — `seat-ql-001` only | 0 |
| workitems / handoffs / sessions | 7 / 3 / 5 | 0 / 0 / 0 | 0 |
| events / dispatches | 400 / **207** | 0 / 0 | 0 |

Negative control (the proof's POINT 6b): unscoped seats **8** vs scoped **7**; a project that does
not exist → 0 dispatches. The scoping is doing real work.

## The mirror intersection — hidden on switch, not destroyed; names from the registry

- `boundMirrorNames` intersects the **global** tmux discovery with the **bound** seats
  (`api.listSeats(activeProjectId)`) by **registered seat name, lowercased** — not a project-name
  heuristic, not every session. So SeatLoom shows 7, ql_delivery 1, an unbound corpus project 0.
- The mapping is case-insensitive on both sides (`seatName.toLowerCase()` vs the registry `name`
  lowercased) and the parsing peels the `-seatloom` suffix before splitting on the first hyphen, so
  `Aegis`-style names survive roles containing `-`, `/` and `&`. Pinned by
  `projectScopeRace.test.ts`'s third test ("the actual mirror session set follows registered binding
  names, including Aegis") and `LiveSeatsView.test.tsx`'s "hides another project's surviving terminal
  **without destroying the connection**" — which is the requirement's own wording, passing by name.
- `listAllSeats` does **not** leak into project modules: its only caller is the unbound-seats
  explanation in the seats module (grep-verified), where it earns its place by naming who is
  unbound. The dispatches module's test explicitly documents the distinction.

## The stale-response guard — real, and it fails the right way

`projectScopeRace.test.ts` drives the real store with a deferred read: **A's late response does not
publish after B has been selected** (`projectData.A` undefined, `B` defined), and a switch marks the
data unresolved immediately rather than showing cached numbers as live. In the view the same
discipline is a `selectedProject` ref checked before every publish. The old failure-pollution path
is gone: the `try/catch` that used to swallow a role-binding failure into an empty map is removed —
a binding-read failure now aborts the hydration and lands in `backendHydrateError`, where the
failure panel shows it. With no project at all the surfaces say so explicitly rather than reading
the default.

## The five new required-project commands, and their callers

`cmd_list_role_bindings` (`seat_cmds.rs:62`), `cmd_get_knowledge_tree` (`knowledge_cmds.rs:37`),
`cmd_list_prompt_instances` (`prompt_cmds.rs:85`), `cmd_reconcile` (`reconcile_cmds.rs:14`) and
`cmd_append_supervisor_message` (`supervisor_cmds.rs:51`) — all five now go through
`require_project_id`, closing exactly the residual silent defaults I named on 2026-10-17.
`cmd_append_supervisor_message` is a **write**, and its refusal is the same helper, covered by the
helper's 3 unit tests; the isolated write-side verification is therefore at the helper layer, and I
did not write to live to test it. I could not drive the IPC-level refusal from a seat (no app-IPC
driver) — recorded, not claimed.

## Artifacts — the screen now says it

The unscoped runtime artifacts carry a **screen-visible** amber notice in `ArtifactsView`
("运行时产出物未记录项目归属，以下运行时记录是全局数据，不代表属于当前项目；文档记录按项目过滤"),
and `projectFilteringCombined.test.tsx` pins that the disclosure is on screen, "not only in
comments". The documents half is scoped; the runtime half is not; the operator's alternative is the
migration script `015_artifacts_project_id.sql`, clearly marked as his act, with no backfill.

## What this verdict does not assert

- **The rendered UI and the IPC path.** The screen is locked from my seat (measured a fourth time:
  `screencapture` fails "could not create image from display", also after `caffeinate`; AppleScript
  is denied "osascript is not allowed assistive access"). No app was installed and the running app
  was not touched, per the dispatch. So nothing here claims a click-through, a badge, or an
  IPC error string.
- The migration script's effect (it is a live act; not applied, and live counts unchanged at
  1478 events / 8 seats / 8 bindings — the proof's POINT 10 measures before == after).
- The per-project numbers as permanent state: they are a snapshot of live at verification time
  (dispatches moved from 200 to 207 since my 2026-10-17 verification, which is the ledger growing,
  not the scoping changing).
- No live writes, no gate events, no acceptance card, no merge, no install — the dispatch's
  prohibitions were honoured in full.

## Bottom line

**PASS.** The combined candidate holds both halves together without pretending: every gate green
(measured by me in its own worktree), the live scoping measured by me with a working negative
control, the mirror intersection registered-name-based and hidden-not-destroyed, the stale-response
guard real, the failure paths honest, the five residual commands closed, and the artifacts
half-truth finally on the screen. The two halves' intentions are preserved, and the candidate is
ready for the operator's card — once someone with screen access confirms the pixels, which is
exactly what I could not do.
