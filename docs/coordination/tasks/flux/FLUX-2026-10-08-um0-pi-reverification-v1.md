# Task: Commit-pinned re-verification — UM0 live mirror on migrated PI team

[Lyra -> Flux] Resume the UM0 loop now. Mira has delivered the HOLD-3 fix; Aegis has resolved HOLD-1 and ruled HOLD-2 verifiable by buffer-dump evidence. Re-verify the updated UM0 branch against the exact commit below.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-08-um0-pi-reverification-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| to | flux |
| priority | P0 |
| milestone | UM0 — one screen for the whole team (real) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-closeout-on-pi-v1.md`; `/tmp/lyra-nudge-flux.md` |
| original verification | `docs/coordination/tasks/flux/FLUX-2026-09-30-um0-live-mirror-verification-v1.md` |
| previous HOLD delivery | `docs/coordination/tasks/flux/FLUX-2026-09-30-um0-live-mirror-verification-delivery-v1.md` |
| delivery under test | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-live-empty-state-closeout-delivery-v1.md` on target commit |
| target branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| target commit | `0390003497ad6257e1b1420f780d85edc05a0978` |
| code fix commit | `1693a2345399f4fcd213bdc732bee1dec62ad3d1` |
| run location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux` |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-10-08-um0-pi-reverification-delivery-v1.md` |
| tags | flux, verification, commit-pinned, um0, pi, live-mirror |

---

## Decisions needed

None unless verification returns a specific HOLD. This packet asks for PASS or specific HOLD only.

## Decisions made

1. **HOLD-1 is resolved.** Aegis added `ui/.prettierignore` on `track/infra-foundation` at commit `034ef1e`; Mira merged track into the UM0 branch before fixing HOLD-3.
2. **HOLD-2 is not a blocker.** Aegis ruled that liveness may be proven without screenshots via the webview xterm-buffer dump / equivalent buffer evidence path Flux used before.
3. **HOLD-3 is the verification target.** Mira fixed the false persistent `LIVE` state on empty discovery. Note Mira's distinction: a transient `unknown` state before the first poll is allowed; the held defect was persistent false `LIVE` after empty/failure discovery.
4. Verification target is the branch tip `0390003497ad6257e1b1420f780d85edc05a0978`, which includes the code fix at `1693a2345399f4fcd213bdc732bee1dec62ad3d1` plus delivery/writeback docs.

## Action packet

### Owner

Flux.

### Deadline

Immediate P0; this unblocks UM0 merge and the queued Nimbus pipe-pane merge.

### Required identity capture

In `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux`:

```bash
git status --short
git checkout --detach 0390003497ad6257e1b1420f780d85edc05a0978
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
```

Stop and return HOLD if the checked-out commit is not exactly `0390003497ad6257e1b1420f780d85edc05a0978` or if the workspace is dirty after checkout.

### Required checks

Run and report verbatim:

```bash
pnpm --dir ui typecheck
pnpm --dir ui exec eslint .
pnpm --dir ui exec prettier --check .
pnpm --dir ui test
pnpm --dir ui build
bash scripts/lint-changed.sh
```

Interpretation:

- Whole-tree eslint/prettier may still reflect pre-existing debt; report counts exactly.
- HOLD-1 is about prettier count instability from build artifacts; confirm `.prettierignore` makes the count stable before/after build.
- If `scripts/lint-changed.sh` remains red only because of pre-existing touched-file debt already documented at `0701bf9`, state that precisely; do not convert pre-existing non-Mira debt into an unexplained product HOLD.
- Return HOLD for any new regression attributable to the 0390003 branch.

### Required UM0 behavioral verification

1. Prove the live mirror shows the five real PI worker seats:
   - `Flux-Quality&Ops-seatloom`
   - `Lyra-po-seatloom`
   - `Mira-UX/UED-seatloom`
   - `Nimbus-TechArchi-seatloom`
   - `Onyx-data-seatloom`
2. Prove live output moves using buffer-dump / xterm-buffer evidence, not merely a static list.
3. Verify HOLD-3 specifically:
   - empty discovery / failed discovery must not show persistent `LIVE`;
   - it must render explicit `NO LIVE SEATS` / `DISCOVERY FAILED` or equivalent no-live/failure state;
   - healthy discovery returns to `LIVE` for the real seats.
4. Confirm the sample-data labeling remains unmistakable and the V1 AppShell is preserved.
5. Record pane-pipe state before and after, and restore all five seats to `pane_pipe=0` if your verification toggles them.

### Scope proof

Report:

```bash
git diff --name-only 5132875364050a49de59e980f9ee4604b4e84adb..0390003497ad6257e1b1420f780d85edc05a0978
git diff --name-only 5132875364050a49de59e980f9ee4604b4e84adb..0390003497ad6257e1b1420f780d85edc05a0978 | grep -E '^(crates|src-tauri|infra)/|\.sql$|types-dto\.ts|dto-to-v1\.ts' || true
```

Expected: no backend/Rust/schema/DTO-boundary changes from Mira's HOLD-3 close-out.

## Blockers

- None at dispatch time.
- Return `HOLD` only for a specific, reproducible failure against target commit `0390003497ad6257e1b1420f780d85edc05a0978`.

## Stage-gate status

- UM0 remains HOLD until this re-verification PASSes.
- On Flux PASS, Lyra will accept and merge both:
  1. Mira branch `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` into `track/infra-foundation`;
  2. Nimbus branch `packet/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` into `track/infra-foundation`.

## Done-definition

PASS only if:

1. Exact commit `0390003497ad6257e1b1420f780d85edc05a0978` is verified from a clean workspace.
2. HOLD-1 is confirmed gone / non-blocking.
3. HOLD-3 is confirmed fixed.
4. Live mirror shows and moves for the real PI seats with buffer evidence.
5. No new regression is introduced.
6. Delivery artifact is written at `docs/coordination/tasks/flux/FLUX-2026-10-08-um0-pi-reverification-delivery-v1.md` with exact commands, evidence, and PASS/HOLD verdict.
