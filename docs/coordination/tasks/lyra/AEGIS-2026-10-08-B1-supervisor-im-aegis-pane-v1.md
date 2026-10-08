# Dispatch: B1 — UI Supervisor-IM pane bound to the Aegis session (minimal "operate via Aegis")

[Aegis -> Lyra] Mr. Zhang ratified the UI supervisor-IM design (minimal-first). This is **B1**: the
smallest packet that delivers "operate the team through the UI via Aegis." **Dispatch only after UM0
is merged to track** (B1 reuses UM0's mirror infra; don't run it in parallel on the same UI files).

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-10-08-B1-supervisor-im-aegis-pane |
| status | queued (behind UM0) |
| author | aegis |
| date | 2026-10-08 |
| version | v1 |
| to | lyra |
| priority | P0 |
| milestone | operate the team via Aegis in the UI (minimal) |
| depends_on | `docs/coordination/reviews/2026-10-08-aegis-ui-supervisor-im-loop-design-v1.md`, UM0 merged |
| tags | p0, dispatch, ui, supervisor-im, aegis |

## Goal
A **Supervisor-IM surface in the UI bound to the Aegis session**: Mr. Zhang types to Aegis in the
UI and sees Aegis's replies + its seat-dispatches; combined with the UM0 live-seat mirror, he
operates the team through the UI without attaching tmux by hand.

## Mira (PI/step-5-preview) — UI, frontend-first
- Add a **Supervisor-IM view/panel** that binds to the **Aegis supervisor tmux session** (reuse the
  UM0 live-mirror machinery: `cmd_attach_tmux_session` + pipe-pane mirror to READ Aegis's output,
  `cmd_pty_write` to SEND). Free-form text box → Aegis; mirror renders Aegis's stream.
- The Aegis session is bound **by name, configurable** — do not hardcode; default to the current
  supervisor session name (confirm with Aegis; today it is tmux `archive`, which may be renamed to a
  stable canonical name — treat the name as config so a rename doesn't break the binding).
- Detachable into its own window per the V1 reference (memory: supervisor IM is L1 + detachable).
- Keep the seat mirror (UM0 `LiveSeatsView`) as the adjacent "watch the team" surface.
- Frontend-only where possible (the IPC commands already exist); `pnpm --dir ui typecheck`+eslint+
  prettier zero-regression; work in your worktree.

## Nimbus (PI/opus) — IPC gap check only
Confirm the existing commands cover "attach to + pty_write a session bound by an arbitrary name"
(seats use this already, so likely no change). Only if a real gap exists (e.g. the supervisor
session name needs special handling), add the minimal Rust in your worktree. Do NOT expand scope —
B2 (step 4 MCP emit) is a separate later packet.

## Acceptance (Flux, PI/step-5-preview)
In the running UI: (a) the Supervisor-IM panel shows Aegis's live output; (b) typing in it reaches
Aegis (Aegis's session receives the message); (c) when Aegis is told to ping/dispatch a seat, that
seat's mirrored pane visibly reacts — all without leaving the UI. Prove via the webview buffer-dump
evidence method (screenshots denied). Zero-regression gates.

## After
Lyra accepts + merges; Aegis reconciles to GitLab and restarts the shared UI. Then the minimal
"operate via Aegis in the UI" loop is live. Next: B2 (step 4 MCP emit) → B3 (step 6 governed
dispatch) → B4 (UI timeline / work-graph) for the structured/safe form.
