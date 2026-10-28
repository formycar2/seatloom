# Heartbeat — 2026-10-28 (third pass): the last pending card is verifiable right now, and it passes

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-heartbeat-pass3-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| tags | heartbeat, last-card, verification, staleness, prepared-not-decided |

> Third heartbeat pass. No dispatch was waiting. Supersedes nothing; corrects nothing in
> `LYRA-2026-10-28-heartbeat-findings-v1.md` (that note's §6 retraction still stands). facts /
> findings / decisions kept separate.

## 1. Facts — measured this turn

| # | What | Measurement |
|---|---|---|
| F1 | The operator cleared the backlog | **30 `AcceptanceGranted` events between 10:19:08 and 10:19:26** on 2026-10-28, across `packet/mira/knowledge-panel`, `packet/nimbus/project-scoped-reads` and `packet/flux/intent-verdict`. `accepted` 26 → **56**; `pending` 31 → **1**. |
| F2 | Ledger is live | last event **18 minutes** before the query. |
| F3 | Queue | 316 done, 18 cancelled, 10 failed, **0 ready, 0 in_flight**. |
| F4 | **The one remaining card is `acc-disp-2o_BDY1tb3b92ZQwaYXEC`** (Vela, `packet/vela/washing-pipeline`), verdict `ev-verdict-disp-2o_BDY1tb3b92ZQwaYXEC` — a valid pass about that dispatch. |
| F5 | **Its four criteria are all TRUE, measured through the live read path** | `list_history_projects` over the real MCP process returns **38 project rows**, **183 sessions total**, **36680 messages total**, and `sf-code/storage` = **2**. The card expects exactly 38 / 183 / 36680 / 2. |
| F6 | The unguarded read is measurably different | the same query **without** the `message_count IS NOT NULL` filter returns **261 sessions** and `sf-code/storage` = **58** — the exact two numbers the card names as the failure mode. So the guard is live, not just present in source. |
| F7 | The card's step-4 probe resolves correctly | only 2 of 40 `projects` rows have no sessions at all (`project_delivery/networkOrchestrator`, `project_delivery/stepaicloud`); `HAVING count(a.id) > 0` excludes them, so they read as "not in the list" rather than "0 messages". **Zero** projects show a non-zero session count made of inventory-only rows. |
| F8 | The inventory guard predates the running app | guard introduced by `6d9e126` (2026-10-16); app binary built 2026-10-27 14:03. So **this card is not blocked by app staleness.** |
| F9 | The app's read path is `seatloom-mcp-emit` over stdio, not embedded SQL | the tauri binaries contain **zero** occurrences of `LEFT JOIN archived_sessions` / `message_count IS NOT NULL`; `target/debug/seatloom-mcp-emit` contains them (`×3` / `×1`). The running process maps that same file (inode `86071711` matches on-disk). |
| F10 | **A real defect in an existing script, fixed in my worktree** | `scripts/prepare-sandbox-for-card.sh` — the documented card-verification path — **exits 1 in its default dry-run mode**: `APPLY` is only assigned inside the `--apply` case, then read under `set -u` at line 159 → `APPLY: unbound variable`. It printed 109 useful lines first, so the crash is silent unless you check the exit code. Fixed by initialising `APPLY=0`; dry-run now exits 0. |
| F11 | The sandbox app is older than the main app | `/Applications/SeatLoom-Sandbox.app` binary built **Oct 21 13:21**; **282 commits** on track since, **36 cards accepted** since. It embeds a complete frontend (2 HTML + assets, references resolve) but of the 21st, not the 28th. |
| F12 | What I could not measure | whether the operator's open window currently renders — no assistive access (`System Events` → `-25211`), no screenshot. Unmeasured, not guessed. |

## 2. Findings

**Finding A: the backlog is not blocked; it is waiting for one click.** F5–F8 together mean the last
card can be run **now**, in the app the operator already has open, and every criterion passes. Nothing
needs building first. This is the opposite of a deadlock, and it is worth saying plainly so nobody
spends a round "unblocking" it.

**Finding B: the card-verification path was broken in its default mode.** F10. `prepare-sandbox-for-card.sh`
is the script built for *"can I verify this card without guessing?"* — and in dry-run, the mode a seat
runs to answer that question, it exits 1. `set -u` + a variable only assigned in one branch. The
output is still correct, which is what makes it dangerous: the failure is in the exit code, and
anything that checks `$?` concludes the card is unverifiable.

**Finding C: the "stale app" problem is per-fix, not global.** My earlier framing ("40 commits stale,
nothing accepted is usable") was too coarse. Measured properly: the inventory guard is from Oct 16 and
**is** in the running path (F8, F9). The supervisor-sync strip and never-answered state are from Oct
27–28 and are **not**. So "is the app current?" has to be asked per feature, and the answer for the
card that is actually pending is **yes**.

## 3. Decision

**Nothing is dispatched.** There is no work a seat should be given: the last card is verifiable and
passes, the queue is empty, and inventing a dispatch would consume a seat's whole attention for
nothing — which `LYRA-HEARTBEAT.md` forbids explicitly.

What I did instead: **prepared** the operator's answer in the three-part shape the protocol asks for.

**What changed:** 30 cards accepted in 18 seconds at 18:19. One card left.

**What you will see:** in the app you already have open, the history-sessions project list shows
**38 projects / 183 sessions**, `sf-code/storage` shows **2 sessions**, and the two adopted-project
rows are absent from the list rather than shown as zero. That is exactly what the card asks for.

**What you need to do:** one thing — accept or reject `acc-disp-2o_BDY1tb3b92ZQwaYXEC`. Nothing has
to be built, installed or restarted first. I have not accepted it: the Accept is yours, and the card's
own plan says so.

## 4. Escalations, and one thing I am not doing

- **`prepare-sandbox-for-card.sh` is fixed only in my worktree.** It is not on track and I did not push
  it there — pushing to track is Aegis's action, and `check-cards-clickable.sh`'s own rule says not to
  merge another seat's branch. Aegis: please take the one-line `APPLY=0` fix.
- **The sandbox app is 282 commits and 36 cards behind.** If the operator wants a side-by-side for a
  future card, `prepare-sandbox-build.sh track/infra-foundation <required-string>` dry-runs clean and
  takes a few minutes. Not run: `--apply` installs an app, which is not mine to do unprompted.
- **Not doing:** accepting the card, building a bundle, restarting anything, or re-litigating the
  app-staleness finding — F12 says I cannot see his window, and F8 says it does not block this card.
