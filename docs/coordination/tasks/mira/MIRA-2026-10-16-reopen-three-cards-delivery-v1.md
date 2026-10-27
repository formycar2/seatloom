# Delivery: three acceptance cards re-opened on a legitimate verdict

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-reopen-three-cards-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-ehsQayg3PH7wgdwsMua6C |
| tags | acceptance, reissue, verdict, delivery, mira |

---

## 0. The one-paragraph version

Three acceptance cards re-opened on a legitimate green basis. The dispatch's two
criteria were checked **per dispatch, by me**, and because last time's accident came
from checking only the shape, I also read the original verdict the re-issue rests on
and confirmed it was a real measurement rather than a signature. The acceptance plans
are the voided cards' wording, verbatim. No merge, no edit to the basis, no batching.

## 1. The two criteria, checked per dispatch

| dispatch | `verdict == pass` | names this dispatch via a `dispatch` ref |
|---|---|---|
| `disp-cEyS0pTJRZaC3b64vFUdR` (live-seats reconnect) | ✅ | ✅ |
| `disp-QBVu3LeruaTavYLIN5BCI` (supervisor feed human text) | ✅ | ✅ |
| `disp-CEa11raWfi8W_UHiXGKBw` (pending-acceptance count) | ✅ | ✅ |

Read from `ev-NFPr-zN36by-i9SE-0FPH`, not taken from the dispatch's prose. That event
carries six refs: the five slice dispatches as `ref_type='dispatch'` / `role='context'`,
plus its evidence document as subject.

## 2. The extra check, and why it was the one that mattered

A re-issued verdict is only as good as the verdict it re-states. So I read
`ev-0idwakl17abEU75LNED24` — the original — and confirmed it is a measurement, not a
signature:

- Flux verified the **background first**, not on trust: all five cards cited
  `ev-verdict-history-sessions`, a FAIL about the 2026-10-14 historical-session corpus,
  unrelated to an icon, a reconnect fix, a feed translation or a count.
- Flux ran the gates **itself**, on `track`: `pnpm typecheck` clean, `vitest` 93/93,
  `scripts/lint-changed.sh` clean, `vite build` green, `cargo test --workspace` 251/0,
  `clippy -D warnings` clean, and one pre-existing `cargo fmt` red at
  `src-tauri/src/main.rs:88` verified byte-identical before and after my work. None of
  it taken from my delivery note.
- Flux measured the operator's arithmetic itself: 38 projects / 183 sessions / 36,680
  messages / OfficeBench 49 / seatloom 14 / `(root)` 8 / cursor 14 with all 14 undated,
  cross-checked against `resume-contexts/` independently of the database.
- It verified my **reasoning** in the code, not as a matter of taste — e.g. that
  `cmd_list_live_sessions` returns `Vec<String>` so it cannot rebuild the seat mapping,
  and that `cmd_attach_tmux_session` inserts unconditionally so re-attaching would leak
  a second `pipe-pane`.
- The re-issue says plainly what it is: `not_a_second_verification` — the same
  measurement, re-stated with the per-dispatch association added, because
  `event_object_refs` is written only at emission time and no tool can retro-add a ref.

One thing I did **not** treat as a rejection: the re-issue's `payload.dispatch_id` is
`disp-65NYElf3D-Ha7tXOTSguG` (the dispatch that asked for the re-issue), not any of the
five. `payload.dispatch_id` is a single value and cannot carry five dispatches; the
per-dispatch refs are the channel that does. Rejecting on the payload alone would have
been the same shallow shape-check that caused the original accident.

## 3. What was opened

Three cards, each under its original `acceptance_id` (a cancelled row is terminal and
keeps its primary key, so re-opening requires the same id), branch
`packet/mira/knowledge-panel`, base `track/infra-foundation`, verdict
`ev-NFPr-zN36by-i9SE-0FPH`. The acceptance plans are the voided cards' wording
**verbatim** — the scenario, every step with its what-to-do / what-passes / what-is-rejected
text, and the preconditions —
per the dispatch's instruction not to re-invent what Mr. Zhang could already follow.

Verified by reading each card back: all three `status: pending`, all three citing the
new verdict, plan step/expected counts intact (6/6, 5/5, 5/5).

## 4. A correction to my own method, recorded

While verifying I re-derived the pending set as *requested − granted − cancelled* and
got **0** — while all three cards were `pending`. The derivation is broken by re-opening:
a cancelled-then-re-opened id appears in both `requested` and `cancelled`, so it
subtracts itself out. It was sound only while cancellation really was terminal, an
assumption never written down, so nothing failed when it stopped being true. A card's
own `status` is the only authority; recorded as a memory so the next count is not
derived this way.

## 5. Boundaries

No merge. No edit to the verdict basis to make a card clickable. No batching — the three
dispatches were confirmed individually, and the basis was not "one event, therefore
fine". No live-database write. No code change in this dispatch.

## 6. Artifact index

- Branch: `packet/mira/knowledge-panel` (no code change from this dispatch).
- Ledger: `AcceptanceRequested` × 3 (re-open), `SupervisorMessage
  ev-eSY54xbMquNN4ove2tKTV`.
- The three cards: `acc-disp-cEyS0pTJRZaC3b64vFUdR`,
  `acc-disp-QBVu3LeruaTavYLIN5BCI`, `acc-disp-CEa11raWfi8W_UHiXGKBw`.
