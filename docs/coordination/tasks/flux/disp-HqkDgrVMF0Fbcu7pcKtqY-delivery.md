# Delivery — `disp-HqkDgrVMF0Fbcu7pcKtqY` (Flux): independent verification of the combined project-filtering candidate

- **Dispatch**: `disp-HqkDgrVMF0Fbcu7pcKtqY`, issued by seat-aegis-001, kind `verification`.
- **Covers implementation dispatch**: **`disp-9H3OCmprBFAkiYBa5MDwE`** — named in the verdict's
  `event_object_refs` (`ref_type='dispatch'`), alongside this verification dispatch.
- **Candidate**: `packet/nimbus/project-filtering-combined` — product `2c69adb`, delivery `4149b7f`
  (tip), off `track/infra-foundation` @ `98876aa`, verified in its own worktree
  `/tmp/nimbus-project-filtering-combined` (clean).
- **Verdict**: **PASS.**
- **Verdict event**: `ev-K3Zfw40dtF7AQfLLh_kTY` — the pre-declared id, emitted exactly as required.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-19-project-filtering-combined-verdict-v1.md` | new — the verdict, per requirement |
| `docs/coordination/tasks/flux/disp-HqkDgrVMF0Fbcu7pcKtqY-delivery.md` | this note |

No source file was changed. **No app installed, no running app touched, no track merge, no live
write, no gate event, no acceptance card** — the dispatch's prohibitions were honoured in full. The
candidate's own worktree was read and built in; nothing in it was modified except the gitignored
`ui/node_modules` (already present).

## Scope audit and the shared files

Read the entire 39-file change set (+1546/−268): every file belongs to project filtering or its
evidence — the 11 commands, the 11 store/views they feed, the probe and proof, the operator's
migration script, the delivery note, the tests. No unrelated or unverified functionality. The
"three-file conflict" from 2026-10-17 did not recur as a git conflict (the candidate is one squashed
commit off track, `2c69adb`'s parent is `98876aa`), and the hand resolution preserves both seats:
Nimbus's single-project hydration **and** Mira's no-project guards in `useDataStore.ts`; the scoped
read **and** the explicit unresolved state in `DispatchInboxView.tsx`; ten `projectId`-requiring
signatures in `api.ts`.

## Gates — run by me in the candidate worktree

fmt clean · `cargo check --workspace --all-targets` clean · `clippy --workspace --all-targets -D
warnings` clean · `cargo test --workspace` **278/0** · `tsc --noEmit` clean · vitest **201/201** (15
files) · `pnpm build` green · `project_scope_proof.sh` **ALL POINTS PASS** (10) · the
`require_project_id` unit tests **3/3**.

## Live, called by me (a snapshot, not a permanent assertion)

SeatLoom: **7 seats without QL** / 7 workitems / 3 handoffs / 5 sessions / 400 events / 207
dispatches. ql_delivery: **exactly 1** (`seat-ql-001`) and zeros. ColossalAI: 0 everywhere.
Negative control: unscoped 8 vs scoped 7; an unknown project → 0.

## The behaviors the dispatch named, by name in the tests

- **Mirror intersection**: `boundMirrorNames` — global discovery ∩ bound seats, by **registered
  name lowercased** (no project-name heuristic); the `-seatloom` suffix is peeled before the first
  hyphen so roles containing `-`, `/`, `&` survive. "hides another project's surviving terminal
  without destroying the connection" passes by name, as does "the actual mirror session set follows
  registered binding names, including Aegis".
- **`listAllSeats` does not leak**: its only caller is the seats module's unbound-seats explanation
  (grep-verified).
- **Stale response**: A's late response does not publish after B is selected; a switch marks data
  unresolved immediately. Old failure pollution removed — the role-binding `catch {}` is gone, so a
  binding failure surfaces in `backendHydrateError` instead of a silent empty map.
- **The five commands**: `list_role_bindings`, `get_knowledge_tree`, `list_prompt_instances`,
  `reconcile`, `append_supervisor_message` all go through `require_project_id` (the last is a write,
  refused by the same helper). The isolated write-side check is at the helper layer; I did not write
  to live.
- **Artifacts**: an amber on-screen notice states the runtime half is unscoped, pinned by a test
  that the disclosure is visible, "not only in comments".

## Not verified, stated plainly

The rendered UI and the IPC path: the screen is locked from my seat (fourth measurement —
`screencapture` fails, assistive access denied), and the dispatch forbids installing or touching a
running app. So no click-through, badge, or IPC error string is claimed. Also unverified: the
migration script's effect (a live act, not applied; live counts unchanged before/after).
