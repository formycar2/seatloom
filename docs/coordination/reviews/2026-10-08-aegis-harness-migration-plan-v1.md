# SeatLoom Harness Migration — Claude Code → PI (compliance, one week)

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | review-2026-10-08-harness-migration-claude-code-to-pi |
| status | draft — pending Mr. Zhang decisions (§6) |
| author | aegis |
| date | 2026-10-08 |
| version | v1 |
| depends_on | `docs/coordination/ROADMAP.md` |
| tags | migration, harness, compliance, pi, opencode, continuity |

## 0. Why

Company security compliance: from ~2026-10-15 (one week) **Codex, Cursor, zcode, and Claude
Code are forbidden**. Allowed open-source harnesses: **PI** (`pi.dev`) and **opencode**. All six
AI seats (Lyra, Nimbus, Onyx, Flux, Mira) and the Aegis supervisor session run `stepcode claude`
(Claude Code) and must move. This is the first forced, real-world test of SeatLoom's core thesis:
**the harness is a replaceable tenant; identity, history, and obligations must outlive it.**

## 1. What is already done

PI is configured against the company gateways and verified (2026-10-08):

| provider | api | endpoint | models | status |
|---|---|---|---|---|
| `stepplan` | openai-completions | `api.stepfun.com/step_plan/v1` | step-5-preview, step-3.7-flash | ✅ ready |
| `stepplan-anthropic` | anthropic-messages | `api.stepfun.com/step_plan` | step-5-preview, step-3.7-flash | ✅ ready, PONG |
| `modelproxy` | openai-completions | `models-proxy…/v1` | gpt-5.5/5.4/codex, step-3.7-flash | ✅ ready, PONG |
| `modelproxy-anthropic` | anthropic-messages | `models-proxy…` | claude-opus-4-8/4-7, sonnet-4-6 | ✅ ready, PONG |

Config lives in `~/.pi/agent/models.json` (provider-level `api` + `apiKey`). A `~/.zshrc` `pi()`
wrapper defaults bare `pi` to `stepplan-anthropic/step-5-preview` (unlimited) because PI's
settings-default path 401s for custom providers; **seat launch uses explicit `--provider/--model`,
never the settings default.** opencode is already wired to the same gateway as a fallback.

## 2. The continuity question (the thesis in practice)

Claude Code session history lives in `~/.claude` jsonl; PI stores sessions in
`~/.pi/agent/sessions`. A harness switch does **not** carry that history across. But the seats'
**durable** context is already harness-independent: it is the coordination markdown under
`docs/coordination/` and the git history — not the transcript. So the migration is a **clean cut**:
seats start fresh PI sessions whose context is re-established from their role doc + the track state,
not from a ported transcript. Deep transcript preservation is P2 ingestion and out of scope for the
deadline. This is exactly why SeatLoom files truth in Postgres+git, not in `~/.claude`.

## 3. Parity map — Claude Code → PI

| Capability | Claude Code (stepcode) | PI | Action |
|---|---|---|---|
| launch | `stepcode claude --model claude-opus-5[1m] --session-id X` | `pi --provider <p> --model <m> --session-id X` | new `seat-launch.sh` |
| tmux dispatch (send-keys) | works | works (PI is a CLI in tmux) | unchanged — `seat-dispatch.sh` is harness-agnostic |
| pane mirror (pipe-pane) | works | works | unchanged |
| role / system prompt | via stepcode context | `--append-system-prompt <role-doc>` | inject each seat's role doc |
| sessions | `--session-id` | `--session-id` (same flag) | pin per seat |
| MCP (future P0 emit) | `--mcp-config` | `pi mcp` / models.json MCP | re-point when P0 lands |
| hooks (STEPCODE_HOOK_*) | stepcode-specific capture | none | drop; SeatLoom capture is separate (P0) |

Coordination infrastructure (`-seatloom` tmux naming, file-based `seat-dispatch.sh`, per-seat
worktrees) is **harness-agnostic and carries over unchanged** — it drives keystrokes into a pane,
regardless of which CLI runs there.

## 4. Model strategy

- **Default workhorse: `stepplan/step-5-preview`** (unlimited — no credit burn). Routine seat work.
- **Faster routine: `stepplan/step-3.7-flash`** (unlimited, less reasoning latency) where speed
  matters more than depth.
- **Hard tasks: `modelproxy-anthropic/claude-opus-4-8`** (metered credit — reserve, escalate
  per-packet, not as a default).
- Note: step-5-preview reasoning latency is high; if seat iteration feels slow, default seats to
  step-3.7-flash and reserve step-5-preview for harder packets. (Logged against ROADMAP §4b.)

## 5. Staged plan (fits the one-week window)

| Phase | Days | Work | Done when |
|---|---|---|---|
| **P1 Pilot** | 1–2 | migrate ONE low-risk seat (recommend **Mira**, UX, fewest live dependencies) to PI; run a full loop: Lyra dispatch → Mira(PI) works → delivers → Flux gate | one seat completes a real packet entirely on PI |
| **P2 Workers** | 2–4 | migrate the remaining workers (Nimbus, Onyx, Flux, Lyra); updated `seat-launch.sh`; each seat's role doc as `--append-system-prompt`; per-seat worktree retained | all 5 worker seats on PI; a dispatch→deliver→gate→merge cycle runs end-to-end on PI |
| **P3 Aegis** | 4–6 | re-host the Aegis supervisor session on PI (role = this session's charter; context from ROADMAP + coordination docs); retire `stepcode claude` for Aegis | Aegis drives a dispatch from a PI session |
| **P4 Cutover** | 6–7 | decommission Claude Code; audit that no seat/Aegis runs a banned harness; confirm opencode fallback | zero banned-harness processes; compliance met before 2026-10-15 |

## 6. Decisions for Mr. Zhang

1. **Default seat model** — `step-5-preview` (unlimited, slower) or `step-3.7-flash` (unlimited,
   faster) as the standing default? (opus reserved for hard packets either way.)
2. **History** — clean cut (rely on docs+git, recommended) or attempt to preserve specific
   Claude Code session transcripts before cutover?
3. **Pilot seat** — Mira (recommended), or another?
4. **PI vs opencode per seat** — all-PI, or keep opencode as the default for any seat where PI
   latency/parity is a problem?

## 7. Acceptance

Compliance is met when, before 2026-10-15, every seat and the Aegis session runs on PI (or
opencode), a full dispatch→deliver→Flux-gate→merge cycle has completed on the new harness, and no
process uses Codex / Cursor / zcode / Claude Code. SeatLoom's record (coordination docs + git +,
when P0 lands, the event ledger) is unchanged by the switch — the thesis, demonstrated.
