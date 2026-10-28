# Heartbeat — 2026-10-28: what the operator can actually do, and the three cards he cannot run

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-heartbeat-findings-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v2 |
| tags | heartbeat, findings, acceptance-cards, running-app, product-truth |

> Standing duty per `docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md`. No dispatch was waiting
> (`get_next_dispatch` → *nothing ready*). facts / findings / decisions are kept separate, as that
> file requires. Every number below was measured in this turn; nothing is relayed.

## 6. CORRECTION, 2026-10-28 16:40 — my Finding B below was WRONG, and it is retracted here

**What I claimed:** that `/Applications/SeatLoom.app` is a *dev* build containing
`devUrl http://localhost:1420` and **zero embedded frontend blobs**, so a reload yields no UI.

**Why that was wrong, measured:** I looked for **zlib** streams. Tauri v2 embeds the frontend
**brotli-compressed** via `include_bytes!`. A zlib scan finds zero blobs in *every* Tauri app,
healthy or not — the check could not fail, so it failed silently in my favour. Ground truth,
re-measured with `brotli`:

- the installed app is **byte-identical** (`md5 28925aa5158d2b121952aef8bc671126`) to
  `target/release/bundle/macos/SeatLoom.app` — it **is** a release bundle, not a dev build;
- it embeds a **complete** frontend: 2 HTML blobs and 5 asset blobs whose compressed bytes appear
  verbatim in the executable, and the HTML references exactly those 5 assets. Nothing dangling
  (`verify-tauri-bundle.sh` step 5: *none*);
- the `devUrl` string I read as evidence of a dev build is config that Tauri embeds in **both**
  dev and release builds. I over-read it.

**What is actually true, and it is still a finding — just a different one:** the app is **stale**.
Built 2026-10-27 14:02. `ui/dist` was rebuilt 21:48 the same day, and **40 commits have landed on
track since**. `scripts/verify-tauri-bundle.sh` names it precisely: `STALE` for 6 of 8 dist files
(“embedded in some build, but NOT in this executable — older app”), `OK` for 2. So the app **works**
and shows a **complete** UI — it shows the **10-27 14:02** UI, not the current one.

**The consequence for the operator is different from what I reported.** Not “a reload shows nothing”.
It is: *the fixes you accepted on the 27th and 28th — the supervisor-sync strip, the never-answered
state, the turn-aware mirror, the session↔work read model — are on track and in no app you can open.*
Verified by content rather than by inference: three strings that exist only in track's
`ui/src/components/SupervisorSyncLost.tsx` (the three UI strings quoted in §6) appear **0
times** in the installed binary.

**I could not see the window.** No assistive access (`System Events` → `-25211`), so no screenshot.
That is unmeasured, not guessed.

**The lesson, which is the project's own:** *a measurement that cannot fail is not a measurement.*
My zlib scan could not detect a brotli blob, so it returned 0 for every Tauri app and I read the 0 as
a defect. I had the right instinct — “is the frontend actually in the thing he runs?” — and the wrong
instrument. The instrument that already existed (`verify-tauri-bundle.sh`, which decompresses brotli
and content-matches) was sitting in `scripts/` and I did not run it first.

**What still needs doing, unchanged in kind:** land a current bundle. The card gate is the operator's
Accept; the bundle build and install is mine to prepare. Whether he wants a bundle at all versus a
kept-alive dev server is still his call (§4).

---

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
| F8 | **RETRACTED — see §6** | *Originally:* “the app is a debug build with zero embedded frontend blobs.” **Wrong instrument** — I scanned for zlib, Tauri embeds brotli, so the scan returned 0 for every Tauri app. Correct measurement: the installed app is byte-identical to `target/release/bundle/macos/SeatLoom.app` (`md5 28925aa5158d2b121952aef8bc671126`), i.e. a **real release bundle**, embedding a complete frontend (2 HTML + 5 asset blobs, no dangling refs). |
| F8′ | The app is **STALE by 40 commits** | Bundle built 10-27 14:02; `ui/dist` rebuilt 21:48; **40 commits on track since**, 4 of them product code. `verify-tauri-bundle.sh` → `STALE` on 6 of 8 dist files. Content check: three strings that exist only in track's `ui/src/components/SupervisorSyncLost.tsx` (the three UI strings quoted in §6) appear in track's `ui/src` and **0 times** in the installed binary. |
| F9 | A newer frontend exists on disk but is not in the app | `ui/dist` mtime **Oct 27 21:48** (5 assets), ~8h newer than the bundle. Nothing embeds it. |
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

**Finding B: the accepted work is in no app the operator can open — and my first claim about why was wrong.** Corrected form: the app is a genuine, working, complete bundle that is **40 commits and ~8 hours stale**. Every fix accepted on 27th–28th (supervisor-sync strip, never-answered state, turn-aware mirror, session↔work read model) is on track and absent from the running binary — measured by content, not by inference.

Aegis's `land-the-mirror-fix.sh` (`f52aa46`) is the first control aimed at exactly this gap, and it is written to *refuse* until the card is accepted. It is the right shape. It has not been run for the 4 product-code commits, and it would need a real bundle build to be worth running.

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

> Dispatched **2026-10-28 16:20**. Both landed by 16:14 / 16:13, before this note's second pass.
> Outcome recorded in §7.
| Seat | Dispatch | Why it is that seat's |
|---|---|---|
| Mira | cancel + re-open `acc-disp-CACpK8qlLm96WQY08itl-` and `acc-disp-HlxnnSKW2lX-rMR02AFk2` with the ref spelled `track/infra-foundation` | those are her cards; only the owner may cancel them |
| Nimbus | same for `acc-disp-YepFZb2hc62cZEO1UkJFX` | his card; same reason |

Both dispatches carry the exact replacement strings, so no seat has to guess what the typo was. The
systemic control (ref-existence check) goes to Aegis as a gate question, because the acceptance gate
is his.

---

## 7. Outcome, 2026-10-28 16:40 — what the two dispatches produced, and what my own checker taught me

**The typo fix landed and is correct.** Verified in the live database: all three cards are `pending`,
zero occurrences of `infa-foundation` in either `how_to_verify` or `acceptance_plan`. I re-ran every
command the corrected cards tell the operator to run: `git rev-parse track/infra-foundation` → a full
sha; `git merge-base --is-ancestor track/infra-foundation packet/mira/knowledge-panel` → exit 0;
`check-cards-clickable.sh` → `OK: all 31 pending cards are clickable`. The cards are runnable now.

**But Mira fixed the cards and never pushed.** Measured: `packet/mira/knowledge-panel` local
`5a1245b` vs `gitlab` `5874b18` — local is **3 commits ahead** (her typo fix, the three new cards'
delivery note, and the previous sync note). Her own card's step 5 asserts that the local sha equals the
remote sha (the phrase "local equals remote" (in her own wording) in her card's own text), so the operator following her card fails on that step.
Nimbus, dispatched the same task, did push (`c9a63fc3` matches on both). I dispatched Mira to push (`disp-cxCBOxrvKX0qcoON2eg_r`)
rather than pushing for her: it is her branch.

**I built the systemic control rather than asking Aegis to.** `scripts/check-cards-runnable.sh`
resolves every `track/…`, `packet/…` and `scripts/….sh` token named by every pending card, and
fails naming the exact offending string. **Then I mutation-tested it, and it caught two bugs in
itself** — both of the "a check that cannot fail" family:

1. **A range separator swallowed the second ref.** `a..track/infa-foundation` matched as ONE token
   because `.` is in the character class, so the second ref was never checked. Reintroducing the typo
   inside a range went **undetected**. Fixed by splitting on `..` before matching.
2. **A separator mismatch made every check pass.** The row reader used `IFS='~'` while `db()`
   formatted with `-F'|'`, so `$body` was empty for every card. `total` incremented, `tokens` was
   empty, `bad` stayed 0 — and the script printed `OK: every git ref … resolves` over 31 cards it had
   not looked at once. Fixed by fetching each body by id.

After both fixes the checker detects all three defect classes it was built for (typo in
`how_to_verify`, typo in `acceptance_plan` only, nonexistent script), exits 1 naming the card, and
exits 0 on the restored database. The database was restored and re-verified clean afterwards.

**The rule this enforces, stated once:** a checker is not done when it passes. It is done when it
**fails on purpose**. The two bugs above both produced a green run, and a green run is exactly what
looks like success. I have now written three checks this round across two dispatches and every one
of them needed a mutation to find its own blind spot.
