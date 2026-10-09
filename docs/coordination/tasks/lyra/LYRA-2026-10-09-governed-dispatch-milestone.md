# Milestone — governed dispatch (G19) end-to-end + backlog

**Owner:** Lyra (PO). **Status:** DELIVERED (core loop) — decision requested on backlog order.
**Date:** 2026-10-09. **Applies to:** the SeatLoom governed-dispatch capability (schema → MCP →
ledger → UI → autonomous seats). Supersedes nothing; first milestone summary.

---

## 1. What works today — the closed loop

One sentence: **操作者在 UI 里发一句话 → 目标席位被自主唤醒 → 自己拉取、执行、完成 →
全程在 UI 和事件账本里可见。**

Pieces, all landed on `track` and proven on the LIVE DB (not scratch):

| # | Piece | Evidence |
|---|-------|----------|
| 1 | Dispatch inbox as data (schema + dependency graph + 4-state status + event vocabulary) | P0 step 6 (`04e9033`, Aegis merged); 13 proof points green on a fresh scratch DB |
| 2 | MCP pull protocol (`enqueue_dispatch` / `get_next_dispatch` / `complete_dispatch`) | seatloom-mcp-emit (`9b96ee8` + `4938545`); title+payload returned (`93b716f`) |
| 3 | Capture spine on live: every transition is an event in the same transaction | step 5 first real capture: `WorkItemCreated`→`WorkItemStatusChanged`, canonical_events 34→36, projection written in-tx |
| 4 | Full lifecycle loop on live | `disp-demo-loop`: ready→in_flight→done, 3 lifecycle events, 36→42 |
| 5 | Event-driven autonomous pickup (NOT polling) | `cc92714`: `pg_notify('dispatch_ready')` on commit + `seatloom-waker` (non-LLM, LISTENs, nudges tmux pane only when the seat is idle). Validated with Mira: idle→wake→pull→done; busy→self-drain, never interrupted; idle = 0 tokens, context preserved |
| 6 | Fleet online: all 5 seats (mira/nimbus/flux/onyx/lyra) run the drain loop | `3db670b`: each seat's own worktree has `.pi/mcp.json` (own actor_ref) + AGENTS.md drain protocol + `--approve`; waker covers all 5 via `.seatloom/dispatch-seats.tsv`; "<seat> online" acks observed; Nimbus demonstrated priority ordering + correct hand-back of a design-sensitive task |
| 7 | Operator surface: UI 派单 tab + 新建派单 form | `2b70995` inbox view → `895f43a`/`cda5fb8` `cmd_enqueue_dispatch` (real DB seat ids) → `9e3d744` UX (live provenance, status filter, timestamps); timeline + inbox auto-refresh (8s/5s) |

**Honest gap:** the loop is proven *piecewise* on live (UI enqueue commits landed after the
last seat-wake validation). One operator click in the UI waking a real seat is expected to work
by construction — every seam is single-implementation — but has not been re-demonstrated since
the UI commits. Cheap to close; do it as the first smoke test of the next slice.

---

## 2. Backlog, ranked by value

### P1 — In-flight deadlock recovery (failed / abandoned / retry states)
- **What:** status domain has no failure path. A seat that crashes, is restarted, or abandons a
  dispatch stays `in_flight` forever — and the partial unique index (one in_flight per seat)
  then blocks *every* future pull for that seat. Permanent wedge, no recovery, silent.
- **Why first:** the fleet just went online; seat restarts are known to happen (they cost session
  context). This is the one defect that turns "online" back into "dead seat" with no operator
  signal. Small schema/state addition (failed/cancelled + retry = re-issue), high risk removed.
- **Cost:** small-medium (Nimbus + Aegis). No decision needed.

### P2 — Waker productionization (service / self-bootstrap)
- **What:** `seatloom-waker` is currently a manually-run foreground binary. If it dies or the
  machine reboots, idle seats are never poked — autonomy silently degrades to manual nudging and
  nobody notices until work stalls.
- **Why now:** the entire "event-driven, zero idle tokens" property rests on this one process.
- **Decision for 张先生:** run it as what — launchd service on the current host? systemd? inside
  the Tauri app? My recommendation: launchd user service on the current host, log to
  `.seatloom/`, auto-restart. (Cost: small once decided.)

### P3 — blocked→ready re-notification
- **What:** `pg_notify` fires on *enqueue*. A dispatch that unblocks (dependency appears) goes
  ready silently; if its target seat is idle, no poke comes and it sits until the seat next
  happens to run its drain loop.
- **Why P3:** cross-seat dependency graphs are rare today, so impact is latent — but the moment
  real multi-seat workflows use dep edges (e.g. verify-after-build), silent unblocking becomes a
  stall class. Fix: notify on the blocked→ready transition too (one more NOTIFY site).
- **Cost:** small (Aegis).

### P4 — Aegis-centered IM→dispatch
- **What:** the natural operator entry point is 张先生 talking to Aegis in IM; Aegis
  `enqueue_dispatch`es to the right seat. Today the operator must use the UI form (or a seat's
  MCP). `disp-sc2-im-wire` is the design-sensitive piece Nimbus correctly handed back.
- **Why P4:** UI form already covers the loop; IM is the *convenience* layer. Worth doing, but
  after reliability (P1–P3). Needs an Aegis design pass first.
- **Cost:** medium (design + wiring). No decision needed yet.

### P5 — UI polish (ongoing)
- **What:** dispatch page UX, empty states, error surfacing for failed enqueues, seat workload
  at a glance. Commits are already landing incrementally (`9e3d744`).
- **Why last:** it's continuous and low-risk; do it in the margins, not as a tracked milestone.

### P6 — GitLab reconciliation (blocked on PAT)
- **What:** reconcile SeatLoom issues/MRs against the dispatch ledger so external work state and
  internal dispatches tell one story.
- **Why last:** external dependency — **needs a GitLab PAT from 张先生** (scope decision:
  read-only `read_api` vs read+write). Can't start without it; not on the critical path of
  anything above.

---

## 3. Decisions requested from 张先生

1. **Backlog order** — confirm P1 (deadlock recovery) jumps the queue ahead of waker hardening.
2. **Waker hosting** — approve launchd user service (my recommendation) or name another target.
3. **GitLab PAT** — if P6 matters this cycle, issue a token and state the scope; otherwise defer.

Everything else can proceed without you.
