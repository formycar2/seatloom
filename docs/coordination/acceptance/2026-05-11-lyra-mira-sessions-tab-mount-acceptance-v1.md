# Lyra Acceptance — V1 Sessions Tab Mount (2026-05-11)

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance |
| id | 2026-05-11-lyra-mira-sessions-tab-mount-acceptance-v1 |
| status | UNCONDITIONAL PASS |
| author | lyra |
| date | 2026-05-11 |
| to | aegis, mira |
| verifies | MIRA-2026-05-11-v1-sessions-tab-mount-delivery-v1 (commit 83b1223) |
| flux_layer_a | 2026-05-11-flux-mira-sessions-tab-mount-layer-a-v1 (5/5 PASS) |
| tags | acceptance, sessions-tab, mira-delivery, b1-unblocked |

---

## Acceptance decision

**UNCONDITIONAL PASS.**

Mira's delivery at commit `83b1223` correctly mounts `SessionsWorkspace` as the `'sessions'` tab in V1. Flux Layer A is clean (5/5). Scope matches the packet exactly — 2 files, 9 lines added, 2 removed.

---

## What this unblocks

- **B1 Layer B** (Mr. Zhang runtime verification) is now unblocked. The 会话 tab is visible in V1 NavRail; clicking it renders `SessionsWorkspace`; the tmux attach flow (select session → Attach → bidirectional terminal) is testable end-to-end.
- **Combined Layer B checklist** for Mr. Zhang (per prior coordination):
  1. NavRail shows 会话 tab (Terminal icon, label "会话")
  2. Click → SessionsWorkspace renders with tmux session dropdown
  3. Dropdown lists real tmux sessions (Onyx-data-seatloom visible)
  4. Select Onyx-data-seatloom → Attach → bidirectional terminal opens
  5. Type text in terminal → appears in tmux pane
  6. Ctrl-C forwarded correctly
  7. Paste works (paste-buffer path)
  8. Close tab → tmux session stays alive
  9. Restart SeatLoom → Onyx-data-seatloom still running in tmux (R3 invariant)
  10. Scrollback preserved while attached

After Layer B PASS: B1 promotes from PROVISIONAL → UNCONDITIONAL.

---

## What this closes

- **R4 gap** identified in prior escalation: Sessions tab was missing from V1, blocking B1 Layer B.
- **Mira packet** `MIRA-2026-05-11-v1-sessions-tab-mount-v1.md` fully executed and accepted.

---

*Accepted by Lyra · 2026-05-11 · Unconditional*
