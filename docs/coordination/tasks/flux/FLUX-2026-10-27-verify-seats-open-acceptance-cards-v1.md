# Flux verdict — opening the two missing acceptance cards, and why his self-correction holds

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-seats-open-acceptance-cards-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | reverification, acceptance-cards, self-correction, branch-containment, seat-flux |

Dispatch `disp-06a95xPk7h6E2pj6FDo4O`, request 3 of 3.
Subject: `disp-lu24NPShI-80fqWuvNZeq` — Nimbus opens the two missing acceptance cards, commit `7e0b60c`.
Verdict event: `ev-RZkE0mWIuSNWp0FmpJ8nU` (this dispatch's own id, payload naming `disp-lu24NPShI-80fqWuvNZeq`).

## Verdict: PASS

Both cards exist in the live DB, are `pending` as requested, carry the right verdict ref,
and have 3 steps / 3 expected / 3 preconditions each. The branch-contains-base criterion
held at the moment it was checked. Gates reproduce exactly.

## Criterion: the two rows

```
        dispatch_id         | status  |               branch               |          base          |     flux_verdict_ref
---------------------------+---------+------------------------------------+------------------------+--------------------------
 disp-NMcTvureVxwPh_3bFAGxx | pending | packet/nimbus/project-scoped-reads | track/infra-foundation | ev-hh9xak8wlN5fJst6107cy
 disp-VWK5z1k7Jcu8-iQFrGVrb | pending | packet/nimbus/project-scoped-reads | track/infra-foundation | ev-hh9xak8wlN5fJst6107cy
```

Both rows now read `accepted` in the live DB — they were created `pending` at
`2026-10-26 17:05:34` and `17:05:47` UTC and have since been granted. That is the
lifecycle working, not a defect: the card's job was to exist so a human could act on it.

`jsonb_array_length` on both plans returns **3 / 3 / 3** (steps / expected /
preconditions), matching his claim. I read `disp-VWK5z1k7Jcu8-iQFrGVrb`'s preconditions
in full; they are real and specific (`SEATLOOM_ACTOR_REF` points at the seat, brotli and
python3 must be on PATH, and the app must reach the local postgres or
`build_app_state`'s `expect` panics — with that last one explicitly scoped out as
another problem, which is the honest way to write it).

## He verified the verdict before trusting the packet — and he was right to

Rather than take the packet's word, he queried `ev-hh9xak8wlN5fJst6107cy` himself and
found `verdict=pass` with refs carrying **both** `dispatch:disp-VWK5z1k7Jcu8-iQFrGVrb`
and `dispatch:disp-NMcTvureVxwPh_3bFAGxx` plus `dispatch:disp-i_ZLKc4E2S0Bm78K-Zmdy`.
I re-ran the same queries: `ReviewVerdictIssued`, `pass`, subject
`disp-NMcTvureVxwPh_3bFAGxx`, and three dispatch refs exactly as he listed. So one
verdict genuinely covers both dispatches and pointing both cards at it is correct — not
a workaround.

This matters more than it looks. The dispatch rules forbid collapsing two deliveries into
one verdict precisely because a card can only point at one verdict, so the *other* card
could never open. Here the collapse already happened upstream (my own verdict covered
two dispatches), and the right response is to point both cards at that one verdict rather
than to invent a second verdict event. He did the former.

## The branch-contains-base criterion held when checked

He reports `git merge-base --is-ancestor` failing before the merge, and a 65-commit gap
he classified by path rather than by number. I re-measured now:

- `git merge-base --is-ancestor track/infra-foundation packet/nimbus/project-scoped-reads`
  **fails now**.
- The 9 commits in track but not in his branch are all dated **2026-10-27 16:48** through
  **2026-10-28 00:05** (+0800).
- His merge of track into his branch (`44723e5`) is **2026-10-27 01:02** (+0800), i.e.
  `2026-10-26 17:02` UTC.
- The cards were created **2026-10-26 17:05:34 / 17:05:47** UTC.

So the criterion was satisfied by about three minutes at the time it was evaluated, and
track simply moved on afterwards. This is the honest reading and I am recording it rather
than reporting a false failure: **the branch did contain the base when the criterion ran.**
The path-based classification he used (118 docs / 100 `ui/src` / 26 `src-tauri/src` /
11 core / 9 mcp-emit over the 65 commits) is what justified merging at all — a
documentation-only gap would have been skippable, and this one was not.

## His self-correction on `isComposing` is correct and verified

He had written that `isComposing` and `proposeDimensions` both come from
`@xterm/addon-fit` and are not SeatLoom code. I corrected that on 10-27; he accepted the
correction, reproduced it himself, and took my version of the conflicting hunk in
`scripts/verify-tauri-bundle.sh` rather than preserving his own wrong comment.

Verified at `7e0b60c`:

- `ui/src/hooks/useGlobalShortcuts.ts:31` contains `if (event.isComposing) return true;`
  — **SeatLoom's own source**, exactly as the corrected comment says.
- `proposeDimensions` appears in `ui/src` **only** in
  `ui/src/app-v2/panel/SessionTerminal.geometry.test.tsx` — a test, never product code.
- `scripts/verify-tauri-bundle.sh:33,37,139` carries the corrected attribution, including
  the line `isComposing      -> ui/src/hooks/useGlobalShortcuts.ts:31 (SeatLoom's own);`
  and a grep-key list naming it `SeatLoom useGlobalShortcuts.ts:31 (was mislabelled xterm-only)`.

The thing worth recording is not the mistake, it is his own diagnosis of it: he wrote the
claim after checking `ui/node_modules` but **not** `ui/src`. A conclusion reached without
looking at the source is worse than no conclusion, and he said so in his own delivery.

## Both merged fixes survived the merge

| Fix | Verified at `7e0b60c` |
|---|---|
| `path: d.homePath ?? null` | present, `ui/src/lib/dto-to-v1.ts:158` |
| `homePath: string \| null` | present, `ui/src/lib/types-dto.ts:22` and `:430` |
| `home_path: Option<String>` | present, `src-tauri/src/dto.rs:33` |
| favicon `ui/public/vite.svg` | present |

Each checked individually, not assumed from the merge succeeding.

## Gate figures — reproduced

| Check | His claim | My measurement at `7e0b60c` |
|---|---|---|
| UI full suite | 23 files / 278 tests | **23 files / 278 tests** |
| `ui` `tsc --noEmit` | clean | **clean** |
| `cargo test --workspace` | 0 failed | **297 passed, 0 failed** |
| `cargo clippy --workspace --all-targets` | 0 issue | **0** |
| `verify-tauri-bundle.sh` | PASS 8/8 | **PASS: all 8 frontend files embedded, byte-for-byte** |

The Rust count differs from his "0 failed" only in that I counted it: 297 passed, 0 failed
at this commit, versus 305 at the later `3b52ea2`. Both are correct for their own commit.

## The `dompurify` finding is real and is a worktree-hygiene trap

He reports 2 test files failing after the merge with
`Failed to resolve import "dompurify" from "src/lib/sanitizeHtml.ts"`, because
`ui/package.json` declares `^3.4.17` but the worktree's `node_modules` never got a
`pnpm install`. I hit the same class of problem from the other side — a detached
worktree failing to compile `seatloom-tauri` because `ui/dist` is absent, needing a
symlink. Both are the same rule: **a worktree is an independent checkout and does not
inherit the main checkout's install artifacts.** He is right that this will keep
presenting as "tests won't start". I reproduced it, so I can confirm the failure mode
exists independently of his note.

I also confirmed his dismissal of the 14 "Unhandled Errors": `App.restartRestore.test.tsx`
deliberately throws an unmocked Tauri `invoke` to test an error path, and the count is
identical with his changes stashed. Attributing that to his own work would have been
wrong, and he checked.

## He declined to open Flux's card, correctly

The packet asked whether Flux wanted a card for `disp-i_ZLKc4E2S0Bm78K-Zmdy`. He declined
because `request_acceptance` only permits the seat that completed the dispatch, so either
he or Aegis would be refused by the `NotCompleted` guard.

He is right on the mechanics, and the reasoning is the general one: **a seat cannot open
an acceptance card for another seat's dispatch, including its own verifier's.** I am
answering his question here, as he asked: I do not need a card for
`disp-i_ZLKc4E2S0Bm78K-Zmdy`, because that dispatch produced no code change — it was an
audit, and its verdict event plus delivery note are the complete artifact. The rule a
card enforces (merge only what was verified) has nothing to gate when there is nothing
to merge.

## Findings

**F1 (hygiene, non-blocking).** He committed three files under `.scratch/`
(`013_dispatch_states.sql`, `014_acceptance_gate.sql`, `017_acceptance_cancel.sql`,
394 insertions). These are scratch copies of already-tracked migrations. They add
duplicated DDL to the tree, which is a real drift risk: two copies of a migration that
can be edited independently. Nothing in the rules I checked forbids `.scratch/`, so this
is not a violation, but it is the kind of thing that should either be gitignored or
deleted before merge.

## What this does not prove

- **The cards' content is verified for shape, not for sufficiency.** I confirmed 3/3/3 and
  read one precondition list. Whether those steps are *enough* for a human to grant or
  reject the acceptance is a judgement the human makes, not one I can certify.
- **I did not re-run the merge.** His two merge commits (`44723e5`, `0fffd86`) and their
  conflict resolutions are taken as delivered; I verified their *outcome* (both fixes
  present, bundle script PASS, gates green) rather than replaying the merge.
- **The 65-commit gap's per-path classification is his.** I verified the dates and the
  ancestry arithmetic, which is what determines whether the criterion held. I did not
  re-count the 118/100/26/11/9 breakdown.

## Live-machine hygiene

Read-only queries against the live DB (`acceptances`, `canonical_events`,
`event_object_refs`). One detached worktree at `7e0b60c`, removed afterwards and pruned.
No writes to any seat's worktree, no live DB mutation.

## Files

No files of mine changed for this request; the verdict and this note are on
`packet/flux/intent-verdict`.
