# Flux delivery — six acceptance cards opened, and the merge that had to happen first

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | FLUX-2026-10-27-open-six-acceptance-cards-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | acceptance-cards, merge, own-work, seat-flux |

Dispatch `disp-E1T0biUI2dMz1Vu7fiDqX`.
Packet `docs/coordination/tasks/aegis/AEGIS-2026-10-27-open-missing-acceptance-cards-v1.md`.

## Result: six cards, all `pending`, each with its own verdict

```
        dispatch_id         | status  |     flux_verdict_ref
---------------------------+---------+--------------------------
 disp-4zePyIT3GboLBELuobHAS | pending | ev-mira-audit-flux-4zePyI03
 disp-E57bNvMK5uTMgrH7tg2cK | pending | ev-nimbus-audit-flux-E57bNv05
 disp-IgwyJqKM_ef9T3j63sD-i | pending | ev-nimbus-audit-flux-IgwyJq04
 disp-VH6l1HixlUVYuy4m0NEp- | pending | ev-mira-audit-flux-VH6l1H02
 disp-i_ZLKc4E2S0Bm78K-Zmdy | pending | ev-mira-audit-flux-i_ZLKc01
 disp-xyvPfhtGxQo5x_2u4VGPU | pending | ev-nimbus-audit-flux-xyvPfh03
```

Six dispatches, six cards, one each — no merging, as the dispatch required. The ids
are the deterministic `acc-<dispatch_id>` form the tool derives by default.

I did not take the packet's verdict list on trust. Before calling `request_acceptance`
I queried each of the six event ids from the ledger directly and confirmed three
things about each: it exists, it is a `ReviewVerdictIssued`, its `payload->>'verdict'`
is `pass`, and its `payload->>'dispatch_id'` names that dispatch. All six passed. Then
after opening them I re-queried the join, so each card's verdict is confirmed to name
its own dispatch rather than someone else's.

## What actually blocked this: the branch was 362 commits behind

The dispatch's rule 1 said to use `track/infra-foundation` as base and to confirm the
branch contains it first. It did not: `git merge-base --is-ancestor` failed, with my
branch **362 behind and 80 ahead**. `request_acceptance` would have refused, and
correctly — handing an operator a branch that branched from a commit he has not seen
is exactly the wrong thing.

So the merge had to happen first, and it was the larger part of this dispatch.

### How I resolved the 15 conflicts

Not by taking a side wholesale. For each file I asked whether track's version is a
**strict superset** of mine, and took track's only when it was, with the check recorded
in the merge commit:

| File | My side | Track | Check before taking track's |
|---|---|---|---|
| `db/event_domain.rs` | 28 event types / 12 ref types | 30 / 13 | confirmed track's file contains my 015/018/019 values |
| `db/repositories.rs` | 93 pub fns | 104 | shared queries diffed and verified byte-identical |
| `db/models.rs` | — | +`VerdictFacts` | pure addition, no counterpart on my side |
| mcp-emit `main/dispatch/protocol/waker.rs` | 21 tools | 27 | my tool set minus track's was **empty** |
| `knowledge_cmds.rs` | silent `default_project_id` fallback | `require_project_id` fails | the fallback **was** the defect |
| `main.rs`, `types-dto.ts`, `api.ts`, `mod.rs`, `lib.rs`, `rollout-ack.txt` | — | +020, +history | additive only; kept both sides where I had content |

The one that needed judgement was `knowledge_cmds.rs`. My side had
`project_id.unwrap_or_else(|| state.default_project_id.clone())` and track replaced it
with `require_project_id(...)`, which **errors** instead. That is not a regression to be
preserved — the silent fallback is precisely the bug the change closes, because it made
every project read every other project's rows. I verified the reasoning is written into
track's `acceptance_cmds.rs` ("the pending-acceptance area showed seatloom's 18 cards under every
project, measured on live: all 18 rows have project_id='seatloom'") before taking it.

### Two deletions, both correct

- **`AppV2.tsx`** — track deletes it in `21c9974` ("two legacy fakes removed") and my
  side never touched it, so the deletion is right. I checked for dangling references:
  the only remaining mentions are two comments explaining the removal.
- **`AGENTS.md`** — this is the one cross-seat collision and it deserves its own note.

### The `AGENTS.md` collision

My branch carried **Nimbus's** seat protocol file. It got there via `37b46f2`, which
committed it deliberately so the forward-recording snippet would survive the branch.
Track's version is **Aegis's** seat file, written by Aegis for himself.

I took track's, and checked that nothing was lost first: track's `AGENTS.md` contains
the same §5 forward-recording snippet at line 127 that Nimbus's commit was carrying.
So the reason my branch had the file is satisfied by track's version.

This is a structural fact worth recording, not just a merge detail: **the repo root's
`AGENTS.md` is a single shared path that every seat's branch wants to own, and every
merge will fight over it.** It is currently Aegis's file and a seat merging track will
have its own seat protocol overwritten. That is survivable as long as the content is
duplicated elsewhere, which it happens to be here. It is not a good design and I am
reporting it rather than fixing it, because which seat owns the repo-root file is
Aegis's call.

### The `dompurify` trap — the one Nimbus reported from the other side

After the merge, `tsc` errored and 2 test files failed with
`Failed to resolve import "dompurify"`. Nimbus hit this same thing and reported it as
"every worktree needs its own `pnpm install`". I hit it independently, which confirms
his finding rather than relying on his word: `ui/package.json` declares
`dompurify ^3.4.17`, this worktree's `node_modules` never had it, and `pnpm install`
fixed it. Before: 2 files failed, tsc errored. After: 24 files / 300 tests pass, tsc
clean. A worktree is an independent checkout and does not inherit the main checkout's
install artifacts.

## Gates, measured on the merged tree

| Check | Result |
|---|---|
| `cargo build --workspace` | 0 issues |
| `cargo test --workspace` | **305 passed, 0 failed** |
| `cargo clippy --workspace --all-targets` | **0 warnings** |
| `cargo fmt --all -- --check` | clean |
| `ui` `tsc --noEmit` | clean (after `pnpm install`) |
| `ui` `vitest run` | **24 files / 300 tests** |
| branch contains `track/infra-foundation` | yes (behind 0, ahead 81) |

I built before committing the merge, which is the specific rule Aegis recorded after
path-limited merges broke track.

## What I did not do

- **No code changes.** This dispatch was a process act: merge to satisfy the gate,
  open six cards. The only files changed are the six delivery-note docs already on the
  branch, plus the merge itself.
- **No merge of my own work into track.** The cards are `pending`; the merge into
  `track/infra-foundation` is the operator's act on accept, never mine.
- **I did not pick which seat owns `AGENTS.md`.** Reported, not decided.
- **I did not open a card for `disp-i_ZLKc4E2S0Bm78K-Zmdy` on Aegis's behalf or vice
  versa.** Nimbus asked me this question directly in his card-opening delivery and I am
  answering it here: no card is needed for that dispatch, because it produced no code
  to gate. A card enforces "merge only what was verified" and there was nothing to
  merge.

## Escalation

1. **`AGENTS.md` is a single shared path contested by every seat.** Reported above; the
   ownership decision is Aegis's.
2. **Worktrees do not inherit install artifacts.** Confirmed independently. Worth
   adding to the worktree-creation checklist so it stops presenting as "tests won't
   start".

## Files

`packet/flux/intent-verdict` at the merge commit. The merge message records the
per-file conflict reasoning in full.
