# Flux delivery — branch synced to track, all 8 pending cards now pass the containment gate

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | FLUX-2026-10-28-sync-branch-for-8-pending-cards-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | branch-sync, merge, gates, urgent, seat-flux |

Dispatch `disp-JsytVHwj3V8Vb1YuM0WeY`.
Packet `docs/coordination/tasks/aegis/AEGIS-2026-10-27-open-missing-acceptance-cards-v1.md`.

## The three numbers the dispatch asked for

| | |
|---|---|
| merged HEAD | **`afa9648`** — `Merge branch 'track/infra-foundation' into packet/flux/intent-verdict` |
| gates | UI **25 files / 311 tests**, `tsc` clean; Rust **305 passed / 0 failed**; clippy **0**; fmt clean |
| `select count(*) from acceptances where status='pending'` | **23** total, **8** of them mine |

All 8 of my pending cards now satisfy the gate's containment condition, checked one card at
a time rather than inferred from the branch tip:

```
✅ disp-06a95xPk7h6E2pj6FDo4O   ✅ disp-IgwyJqKM_ef9T3j63sD-i
✅ disp-4zePyIT3GboLBELuobHAS   ✅ disp-VH6l1HixlUVYuy4m0NEp-
✅ disp-E57bNvMK5uTMgrH7tg2cK   ✅ disp-i_ZLKc4E2S0Bm78K-Zmdy
✅ disp-l-5ODXaSdIQcrcxTurFe_   ✅ disp-xyvPfhtGxQo5x_2u4VGPU
```

No card uses any base other than `track/infra-foundation`, so there is no second base to
satisfy.

## It took two merges, not one — and the first check I ran was wrong

The dispatch said 2 commits behind. I merged those 2 and ran the gate's containment check
across all 8 cards — and got **8 ❌**. That looked alarming, and it was worth not glossing
over: `track/infra-foundation` had moved **again** while I was running the gates. A third
commit, `d977704` ("I pushed to track after the cards opened and broke the operator's
Accept"), landed in the window between my merge and my verification.

So I merged that too, and re-ran the per-card check. Then all 8 pass.

The lesson I am recording rather than skipping: **a containment check is a statement about
an instant, and the branch tip is a moving target while other seats are working.** "I
merged track" and "my cards can be accepted" are two different claims, and only the second
one matters. I had verified the first and nearly reported it as the second. The per-card
check is what actually answers the question, and it is what caught the drift.

## What the two merges contained

Nothing that conflicts with anything of mine. I confirmed before merging that my side has
**zero** changes in either file:

- `ui/src/stores/useConversationStore.test.ts` — 162 lines, the 11-test file that Mira's
  audit found missing and that I escalated in the previous dispatch. Aegis restored it with
  `16e65e3`, verified as load-bearing.
- `docs/coordination/knowledge/a_path_limited_merge_is_not_a_whole_branch_merge.md` and
  `i_pushed_to_track_after_cards_opened_and_broke_the_operators_accept.md` — knowledge docs,
  plus the `INDEX.md` row for the second.

## The restored test file closes the gap I published

This is the direct payoff of the previous dispatch's escalation. Track's UI suite went from
24 files / 300 tests to **25 files / 311 tests** once the file came back. So the figure I
published in the dual-render verdict — 25 files / 311 tests, which was not reproducible on
track when Mira caught it — **is now reproducible on track**, at the merged HEAD, and I
re-measured it to confirm rather than assuming.

## What I did not do

- **No cards re-opened.** The dispatch said they were already open and only the containment
  condition was failing; re-opening would have collided with `acceptances_pkey` for no gain.
- **No code changes.** Both merges are docs and one test file, none of them mine.
- **I did not touch the live database.** The `acceptances` reads are read-only.

## Note on the cause

Aegis says this was his omission — he pushed to track after the cards were opened without
telling me to sync. That is accurate as far as it goes, and his knowledge doc records it.
But the reason it bit me twice in a row is that my sync was a one-shot act: I merged,
believed it, and moved on. A card's acceptability decays the moment anyone pushes to its
base, and the only durable answer is to re-run the per-card check immediately before
handing work over — not to have merged once.

## Files

`packet/flux/intent-verdict` at `afa9648`. No files of mine changed.
