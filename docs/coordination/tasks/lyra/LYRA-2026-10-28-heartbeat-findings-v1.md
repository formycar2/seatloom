# Heartbeat — 2026-10-28: what the operator can actually do, and the three cards he cannot run

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-heartbeat-findings-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| tags | heartbeat, findings, acceptance-cards, running-app, product-truth |

> Standing duty per `docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md`. No dispatch was waiting
> (`get_next_dispatch` → *nothing ready*). facts / findings / decisions are kept separate, as that
> file requires. Every number below was measured in this turn; nothing is relayed.

## 1. Facts — measured this turn

| # | What | Measurement |
|---|---|---|
| F1 | Ledger is live | last event `2026-10-28 08:03:17Z`, **35 seconds** before the query. Not a stalled fleet. |
| F2 | Dispatch queue | 312 done, 18 cancelled, 10 failed, **1 in_flight** (`disp-iVshq1brrmpS5UIGs1ous`, Mira), **0 ready**. Nothing is waiting for a seat. |
| F3 | Pending acceptance cards | **28**, and `scripts/check-cards-clickable.sh` prints `OK: all 28 pending cards are clickable`. By branch: mira 14, flux 9, nimbus 7, vela 1. |
| F4 | Card validity | All 28 carry a `flux_verdict_ref` that resolves to a real `ReviewVerdictIssued` with `verdict='pass'` **about that dispatch** (checked against `payload.dispatch_id` **and** `event_object_refs`). All 28 have a non-empty `how_to_verify` and a `plan_version:1` plan. |
| F5 | Accepted work really is on track | 26 accepted cards across 11 distinct `merge_commit` values — **all 11 are ancestors of `track/infra-foundation`**. Nothing accepted was lost. |
| F6 | **Three of the 28 cards cannot be run** | `acc-disp-CACpK8qlLm96WQY08itl-`, `acc-disp-HlxnnSKW2lX-rMR02AFk2` (Mira) and `acc-disp-YepFZb2hc62cZEO1UkJFX` (Nimbus) name the git ref **`track/infa-foundation`** — missing an `r` — inside `acceptance_plan` and `how_to_verify`. Measured: `git rev-parse --verify track/infa-foundation` → `fatal: Needed a single revision`; the real branch is `track/infra-foundation`. |
| F7 | Those three cards' underlying claim is TRUE | `git merge-base --is-ancestor track/infra-foundation packet/mira/knowledge-panel` → exit 0. `9a67ea5` is an ancestor of `packet/nimbus/project-scoped-reads` and `pipe_guard.rs` is present on it. So the work is correct; the **instructions** are unrunnable. |
| F8 | **The app the operator runs is a debug build pointed at a dead dev server** | `/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri`, mtime **Oct 27 14:03**, contains the string `http://localhost:1420` (Tauri's `devUrl`) and **zero embedded frontend blobs** (decompression scan found 0). Nothing is listening on `:1420` right now — `curl` returns `HTTP 000`, no `vite` process exists. |
| F9 | A built frontend exists but is not in the app | `ui/dist` exists, mtime **Oct 27 21:48** (5 assets). It is *newer* than the app binary, but the binary embeds nothing, so the app cannot use it. |
| F10 | Track is ahead of anything the operator could be running | **40 commits on track postdate the app binary**, of which **4 touch product code** (`1572154`, `1bf41cf`, `16e65e3`, `f72b515`). |
| F11 | The Pi harness fix — the 10-17 mockup root cause — is on track | `f316201` is an ancestor of track. Live `seats.default_harness` = `Pi` × 10. That finding is closed. |
| F12 | What I cannot see | The operator's open window. F8 says a **reload** gets no frontend; it does **not** prove the currently-open window is blank, because the SPA may have loaded while a dev server was alive. Stated as unmeasured rather than guessed. |

## 2. Findings — my inference, labelled as such

**Finding A: an acceptance card is hand-written prose that nobody executes.** Three cards told the
operator to run `git rev-parse track/infa-foundation`. The card's own `scenario` is true, its branch
really is synced, and its verdict really is a pass — but the operator following it literally gets
`fatal: Not a valid object name`. Nothing runs those commands before the card is offered.

This is the same family as the gate-typo Aegis already recorded (`ev-...-landed` spelled as
`ArtifactCreated` when a `-live` gate needs a `SupervisorMessage`): **a token that names the wrong
thing, believed because it looks right.** The acceptance gate validates the verdict ref, the
dispatch ownership and the plan *shape* — it never validates that the refs the plan names exist.

The severity is moderate, not critical: the gate's own guards pass, so the **Accept button works**.
The damage is confusion — the operator hits a `fatal:` on step 1 of a card whose intent is already
satisfied, and has no way to tell "the instructions are wrong" from "the work is wrong".

**Finding B: "on track" and "running" have come apart in the third direction.** Aegis recorded
`gate_events_conflate_code_landed_with_live_usable` (code on track, live unusable) and its mirror
(`the_gate_condition_was_met_on_a_branch_but_not_on_track`, live usable, code not on track). This is
the third: **the work is on track, the verdict is green, the card is accepted — and the binary the
operator launched predates all of it and has no frontend in it at all.**

Aegis's `land-the-mirror-fix.sh` (`f52aa46`) is the first control aimed at exactly this gap, and it
is written to *refuse* until the card is accepted. It is the right shape. It has not been run for
the 4 product-code commits, and it would need a real bundle build to be worth running.

## 3. Decision — the next operator-perceivable thing

**The product is not usable, and that outranks every feature on the roadmap.** Not "the app is a
mockup" — that was 10-17's finding and it is closed (F11). The current state is worse in kind and
simpler in cause: the binary in `/Applications` is a **dev** build whose frontend source (`:1420`)
is dead, so it carries no UI of its own.

So the next round is one thing, in this order:

1. **Make the operator able to open the app and see the current frontend.** A bundle build
   (`frontendDist` embedded, verified by decompressing its blobs and content-matching `ui/dist` —
   never by listing `Resources`, never by grepping the executable, both of which are wrong in both
   directions) installed over `/Applications/SeatLoom.app` with the old one kept. The restart is the
   operator's action, not a seat's: closing his windows is not a seat's call.
2. **Make the three unrunnable cards runnable.** The plan is immutable after the request, so the
   documented path is `cancel_acceptance` + `request_acceptance` with the **same** id and a corrected
   plan. Each affected seat does its own; a seat cannot cancel another's card.
3. **Stop the class, not the instance.** A check that every git ref named in a pending card's plan
   and `how_to_verify` exists. Without it, the next card ships the same typo.

**Explicitly not doing:** inventing a feature, widening scope to the 10-16 supervisor-panel questions
that still wait on the operator, or re-opening anything about the session↔work bridge (ruled on
2026-10-28, delivered, closed).

## 4. What is the operator's to decide, not mine

- **Restarting the app** after a bundle install — that closes his windows. Mine to prepare, his to run.
- **Whether the running app should be a bundle build at all** — a dev build is arguably correct while
  the fleet is iterating, in which case the answer is "keep `pnpm dev` alive", not "install a bundle".
  This is a workflow choice only he can make, and the two answers need different work. Recorded here
  rather than decided for him.

## 5. What I am dispatching

| Seat | Dispatch | Why it is that seat's |
|---|---|---|
| Mira | cancel + re-open `acc-disp-CACpK8qlLm96WQY08itl-` and `acc-disp-HlxnnSKW2lX-rMR02AFk2` with the ref spelled `track/infra-foundation` | those are her cards; only the owner may cancel them |
| Nimbus | same for `acc-disp-YepFZb2hc62cZEO1UkJFX` | his card; same reason |

Both dispatches carry the exact replacement strings, so no seat has to guess what the typo was. The
systemic control (ref-existence check) goes to Aegis as a gate question, because the acceptance gate
is his.
