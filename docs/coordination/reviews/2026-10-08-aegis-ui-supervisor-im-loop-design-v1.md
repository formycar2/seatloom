# SeatLoom UI Supervisor-IM Loop — operate the team through the UI via Aegis

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | review-2026-10-08-ui-supervisor-im-loop |
| status | draft — pending Mr. Zhang review |
| author | aegis |
| date | 2026-10-08 |
| version | v1 |
| depends_on | `docs/coordination/ROADMAP.md`, `docs/coordination/reviews/2026-09-29-aegis-platform-design-macro-to-micro-v1.md` |
| tags | design, ui, supervisor-im, aegis, dispatch, l1, um2 |

## 0. Goal

Mr. Zhang operates the standing team **through the SeatLoom UI**: he talks to **Aegis** in the UI,
Aegis commands the worker seats, and the whole loop is visible in the UI. This is the union of
user-milestones **UM0** (see the team) and **UM2** (progress + dispatch at a glance), plus the L1
supervisor-IM write loop.

## 1. Key decision — reuse the tmux mirror, don't build an Aegis↔DB daemon

Aegis is a conversational session (Claude Code now, PI later), not a service, and stays that way.
The UI connects to Aegis the same way it connects to any seat: **treat the Aegis session as a
mirrored + driven tmux pane.** The UI already has the machinery — `cmd_attach_tmux_session` +
`pipe-pane` mirror (UM0) to read, `cmd_pty_write` to send. So:

- The UI's **Supervisor-IM surface is a view bound to the Aegis tmux session**: typing sends to
  Aegis via `pty_write`; Aegis's replies and its seat-dispatches stream back via the mirror.
- Aegis commands seats exactly as today — `seat-dispatch.sh` / tmux `send-keys` to each seat's pane
  — and those seat panes are themselves mirrored in the UI (UM0), so the user watches them react.

**Rationale:** reuses existing, working infrastructure; keeps Aegis a normal conversational session;
no new bridge/daemon; harness-agnostic (works whether Aegis is Claude Code or PI — the UI just binds
to whatever tmux session Aegis runs in). This is "mirror, not replacement" extended to the supervisor.

Rejected: an Aegis-polls-PostgreSQL bridge (makes Aegis a daemon, awkward for a conversational
session) and a bespoke routing service (heavier, redundant with the mirror).

## 2. The loop — minimal (now-ish) vs rich (structured)

**Minimal loop (mostly free once UM0 lands):** UI Supervisor-IM = mirror(Aegis pane) + a send box.
User types intent → Aegis → Aegis dispatches seats (tmux) → seat panes mirrored → user watches.
This already delivers "operate the team through the UI via Aegis" at terminal fidelity.

**Rich loop (the durable form):** the dispatches/deliveries/verdicts also flow as **structured
events** into a UI **timeline / work-graph** (who's doing what, who's blocked on whom, what's
queued) — not just raw terminal bytes. This needs the capture + governed-dispatch layer:
- **write API (A1)** — DONE (`append(event)→projections`).
- **step 4 MCP emit** — seats emit `DispatchSent` / `DeliveryMade` / `ReviewVerdict` / `ArtifactCreated`
  events through a governed MCP server.
- **step 6 governed dispatch (G19)** — per-seat inbox, ordered/one-in-flight delivery, so UI-issued
  commands don't interleave (the problem Mr. Zhang flagged early).
- **UI timeline** — renders the event stream from `canonical_events` (the read side already exists).

## 3. Build packets

| # | Packet | Owner | Delivers |
|---|---|---|---|
| B1 | **Supervisor-IM pane** — a UI surface bound to the Aegis tmux session (mirror + `pty_write` send box), detachable per the V1 reference | Mira (UI) + Nimbus (any IPC gap) | the **minimal loop**: talk to Aegis in the UI, watch seats |
| B2 | **step 4 MCP emit server** — `emit_event(type,payload,refs)`, schema-validated, seats' `--mcp-config` points at it | Nimbus | seats' actions become structured events |
| B3 | **step 6 governed dispatch (G19)** — per-seat inbox + `depends_on` + ready-aware ordered delivery + cycle/stall detect | Nimbus | UI/Aegis commands delivered safely, no interleave |
| B4 | **UI timeline / work-graph** — render the event stream (dispatches/deliveries/verdicts, who's-blocked-on-whom) | Mira | the **rich loop**: UM2 progress-at-a-glance |

B1 rides on UM0 (same mirror infra) — smallest, do first. B2→B3 are the P0 capture backbone
(already on the ROADMAP critical path). B4 is the read-model view over events.

## 4. Sequencing

1. **UM0 close-out** (in flight) → UI shows the 5 live PI seats.
2. **B1** → Supervisor-IM pane bound to Aegis → the minimal "operate via Aegis" loop works.
3. **B2 (step 4 MCP)** → seats emit events.
4. **B3 (step 6 dispatch)** → ordered, safe UI-issued commands.
5. **B4** → work-graph timeline → UM2.

After step 2 the stated goal is met at terminal fidelity; steps 3–5 make it structured, safe, and
glanceable.

## 5. Open questions for Mr. Zhang

1. **Minimal-first?** Accept B1 (mirror+send Aegis pane) as the v1 of "operate via Aegis" — a
   terminal-fidelity supervisor IM — with the polished intent→proposal→confirm L1 loop (per the PRD
   `US-P0-02`) as the target form layered on later? (Recommended: yes — fastest to a usable loop.)
2. **Aegis pane home.** Aegis should run in a stable, UI-attachable tmux session (today it is the
   supervisor session). Confirm the UI binds its Supervisor-IM to that session by name.
3. **Command granularity in the UI.** Does Mr. Zhang issue free-form intent to Aegis (Aegis decides
   how to dispatch), or also want direct "dispatch seat X" buttons? (Recommended: free-form to Aegis
   first; structured buttons after B4.)
