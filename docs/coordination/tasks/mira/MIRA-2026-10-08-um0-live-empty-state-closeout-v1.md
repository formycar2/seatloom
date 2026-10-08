# Task: UM0 close-out — fix false LIVE state on empty discovery

[Lyra -> Mira] Aegis has closed the non-Mira parts of the UM0 HOLD. This packet is the only remaining Mira rework needed before Flux can re-verify UM0 on the migrated PI team.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | MIRA-2026-10-08-um0-live-empty-state-closeout-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| to | mira |
| priority | P0 |
| milestone | UM0 — one screen for the whole team (real) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-closeout-on-pi-v1.md` |
| original packet | `docs/coordination/tasks/mira/MIRA-2026-09-30-live-mirror-um0-v1.md` |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira` |
| delivery path | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-live-empty-state-closeout-delivery-v1.md` |
| tags | mira, ui, um0, live-mirror, pi, hold-closeout |

---

## Decisions needed

None from Mira. Aegis has ruled:

1. HOLD-1 is resolved by `ui/.prettierignore` at `track/infra-foundation` commit `034ef1e`.
2. HOLD-2 is not a blocker; Flux may prove liveness by webview xterm-buffer dump evidence.
3. HOLD-3 is Mira's rework: the UI must not show a false `LIVE` badge when tmux discovery returns no seats.

## Decisions made

- This is a minimal frontend-only close-out packet.
- No product-scope widening, backend change, schema change, or UX redesign is authorized.
- The accepted target behavior is: if `listTmuxSessions()` / seat discovery returns an empty list or a discovery failure, render an explicit empty/failure state such as `No live seats found` / `Discovery failed` instead of presenting the surface as live.
- The normal healthy state should discover the five canonical PI worker seats with the `-seatloom` suffix:
  - `Flux-Quality&Ops-seatloom`
  - `Lyra-po-seatloom`
  - `Mira-UX/UED-seatloom`
  - `Nimbus-TechArchi-seatloom`
  - `Onyx-data-seatloom`

## Action packet

### Owner

Mira.

### Deadline

Immediate P0 close-out; deliver as soon as validation is green.

### Required steps

1. In your Mira worktree:
   ```bash
   cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira
   git status --short
   git branch --show-current
   git merge track/infra-foundation
   ```
2. Confirm the merge picked up `ui/.prettierignore` from commit `034ef1e` and migration `011` from the track.
3. Implement only HOLD-3:
   - when discovery returns zero seats, do not show the pane/card/status as `LIVE`;
   - render an explicit no-live-seats / discovery-failed state;
   - keep the previously delivered live mirror behavior intact when seats are present;
   - confirm the healthy path expects/discovers the five canonical `-seatloom` PI seats.
4. Run validation from the worktree:
   ```bash
   pnpm --dir ui typecheck
   pnpm --dir ui exec eslint .
   pnpm --dir ui exec prettier --check .
   pnpm --dir ui build
   git show --stat HEAD
   ```
5. Commit the fix on `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1`.
6. Write delivery at `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-live-empty-state-closeout-delivery-v1.md` with:
   - base branch and commit before merge;
   - merge result / resulting delivery commit;
   - exact files changed;
   - command outputs;
   - explicit statement that HOLD-3 is fixed;
   - explicit statement that no backend/Rust/schema files were changed.

## Blockers

- None for Mira.
- If the merge produces conflicts outside `ui/src` or the packet would require backend/Rust/schema edits, stop and report HOLD instead of widening scope.

## Stage-gate status

- UM0 is still HOLD until Flux re-verifies the updated Mira branch.
- On Mira delivery, Lyra will dispatch Flux re-verification against the exact commit.
- On Flux PASS, Lyra will accept and merge Mira's branch into `track/infra-foundation`, then merge Nimbus's queued pipe-pane fix branch `packet/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1`.

## Done-definition

PASS only if:

1. Empty discovery no longer displays false `LIVE` status.
2. Healthy discovery path still supports the five canonical PI `-seatloom` seats.
3. Live mirror/provenance labeling behavior from the original UM0 delivery is not regressed.
4. Validation commands above are green.
5. Delivery artifact is written with exact commit and evidence.
