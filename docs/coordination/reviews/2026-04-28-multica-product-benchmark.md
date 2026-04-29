# Multica Product Benchmark for SeatLoom

| Field | Value |
|---|---|
| Owner | Lyra |
| Date | 2026-04-28 |
| Status | Issued |
| Scope | External product benchmark of `https://multica.ai/` for SeatLoom product decisions |
| SeatLoom contract reference | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md` |

## 1. Benchmark snapshot

Multica is not "another coding agent CLI." It is a task collaboration layer where humans and AI agents operate in one shared workspace, with issues as the main work object and tasks as the execution object.

Three defining product traits stand out:

1. Agents are treated as first-class teammates, not hidden automation.
2. Runtime execution stays on the user's machine via a daemon, while the server coordinates tasks, history, and UI.
3. The product is explicit about what each runtime can and cannot do, especially session resumption, MCP, skill injection, and parallelism.

## 2. What overlaps with SeatLoom, and what does not

### Product overlap

- Both products care about human + agent collaboration, not single-agent prompting.
- Both products care about runtime reality: local execution, resume behavior, failures, and coordination state.
- Both products need a persistent, inspectable trail of work, not ephemeral chat alone.

### Product difference

- Multica is issue-centric and workspace/server-centric.
- SeatLoom is project-local and ledger-centric, with `Seat / Session / WorkItem / Handoff / Artifact / Inbox` as the primary truth model.
- Multica's inbox is a human notification center driven by subscriptions.
- SeatLoom's inbox is an action queue driven by product rules and should stay that way.

Conclusion: SeatLoom should borrow Multica's execution rigor, runtime observability, and agent-management clarity, but should not drift into a generic issue tracker.

## 3. Borrow recommendations

### P0 — Strongly recommended now

| Borrow | Why Multica proves the value | SeatLoom adaptation |
|---|---|---|
| Runtime capability matrix | Multica documents per-tool truth for session resume, MCP, skills, and model selection instead of pretending all runtimes are equal. | Add a visible capability layer to SeatLoom runtime selection and session detail: resume support, MCP support, skill-pack support, approval model, and known caveats per runtime. |
| Runtime health and concurrency surface | Multica exposes daemon heartbeat, online/offline state, concurrency limits, and failure recovery semantics. | Add a runtime health panel for SeatLoom with online state, last heartbeat, running sessions, queued actions, and per-runtime limits. This will make `wrap`, `attach`, `switch runtime`, and `rehydrate` much more trustworthy. |
| Human-only inbox semantics | Multica clearly states that agents do not consume inbox notifications; inbox is for humans, while agent work is triggered directly. | Keep SeatLoom Inbox as a human action queue only. Make this explicit in UX copy and docs: agents act through WorkItem/Handoff/runtime triggers, not by "reading Inbox." |
| Execution-history drilldown | Multica shows live execution output and task execution history as part of the work surface, not as hidden logs. | Strengthen SeatLoom `TerminalPanel` / session detail with a compact "what happened in this run" layer: start, tool steps, retries, artifacts, blocker reason, completion evidence. |
| Chinese mixed-script readability rules | Multica recently shipped Inter with CJK fallback and automatic CJK+Latin spacing. This is a direct response to the exact readability problem we hit today. | Introduce a Chinese-first typography policy for SeatLoom: font pairing, CJK/Latin spacing, title scale, dense-label rules, and terminal monospace pairing. |

### P1 — Worth planning after baseline stabilization

| Borrow | Why Multica proves the value | SeatLoom adaptation |
|---|---|---|
| Shared skill library | Multica turns `SKILL.md` packs into reusable team capability and syncs them to runtimes. | Add SeatLoom "Skill Packs" or "Seat Packs" as reusable execution context modules attached to seats or runtime profiles. Keep this tied to real work surfaces, not a generic marketplace first. |
| Scheduled automations with visible history | Multica's Autopilots provide recurring runs with history and an explicit execution mode. | Add scheduled reconcile, daily digest, branch health scan, or stale-session sweeps — but every scheduled run must leave a visible SeatLoom artifact, inbox item, or timeline event. Avoid invisible automation. |
| Plug-and-play runtime discovery | Multica detects installed tools on `PATH` and registers available runtimes automatically. | Add onboarding/runtime setup that detects Codex, OpenCode, Gemini CLI, etc., then labels each as usable / degraded / unsupported for resume or MCP. |
| Fullscreen transcript view | Multica added a dedicated transcript surface for deep inspection. | Add an expanded session evidence view for review and debugging, especially for acceptance, incident replay, and handoff verification. |
| Command palette and pinned navigation | Multica added `Cmd+K`, pinned issues/projects, and search improvements. | Add a command palette for `open workitem`, `jump to seat`, `launch wrap`, `trigger reconcile`, `switch project`, and `open recent artifact`. |

### P2 — Useful later, but risky if pulled forward

| Borrow | Why it is interesting | SeatLoom caution |
|---|---|---|
| Private agent chat outside tracked work | Multica's chat has strong privacy and isolation boundaries. | Only consider this later as a "scratch conversation" mode. Do not let it bypass the WorkItem/Handoff/Artifact ledger. |
| Project lead can be an agent | Multica lets agents lead projects. | SeatLoom can eventually allow a seat to act as a project steward, but only after governance, accountability, and stage-gate semantics are stable. |
| Sub-issue hierarchy | Multica is adding deeper issue/project structure. | SeatLoom should not copy generic issue hierarchy too early; the current WorkItem dependency model is enough for now. |

## 4. What SeatLoom should explicitly not borrow

| Do not borrow | Why |
|---|---|
| Generic issue-board center of gravity | It would dilute SeatLoom's differentiator: project-local operational truth across seats, sessions, handoffs, and recovery. |
| Open-ended status transitions | Multica allows any issue status to jump to any other. SeatLoom needs stricter lifecycle gates because it is also an execution ledger. |
| Notification-driven inbox semantics | Multica inbox is subscription-driven; SeatLoom Inbox is intentionally a filtered action queue. Keep this distinction. |
| Server-first execution mental model | Multica server coordination makes sense for its workspace product. SeatLoom should preserve local-first project authority and avoid cloud dependency in the core story. |
| "Hire AI employees" positioning | Strong marketing, but wrong for SeatLoom. SeatLoom should stay focused on coordination, recovery, traceability, and execution continuity. |

## 5. Most actionable synthesis for SeatLoom

If SeatLoom borrows only five things from Multica in the next cycle, they should be:

1. A runtime capability matrix with hard truth about resume/MCP/skills.
2. A runtime health panel with heartbeat, online state, and concurrency visibility.
3. A richer session execution-history view that turns raw terminal output into reviewable evidence.
4. A Chinese-first typography and mixed-script spacing system.
5. A future automation model where scheduled actions always create visible ledger outcomes.

## 6. Recommended next actions

| Priority | Action | Owner |
|---|---|---|
| P0 | Convert runtime capability truth into SeatLoom UI requirements for `Wrap`, `Attach`, `Switch Runtime`, and session detail. | Lyra + Nimbus |
| P0 | Start a dedicated typography/readability pass informed by mixed CJK/Latin usage. | Lyra + Flux |
| P1 | Draft a SeatLoom runtime-ops panel spec. | Nimbus |
| P1 | Draft a SeatLoom automation model that emits Inbox/Timeline/Artifact outputs instead of invisible background jobs. | Lyra |

## Sources

- Multica landing page: https://multica.ai/
- Multica docs home: https://multica.ai/docs
- Daemon and runtimes: https://multica.ai/docs/daemon-runtimes
- Tasks: https://multica.ai/docs/tasks
- Assign issues to agents: https://multica.ai/docs/assigning-issues
- Inbox and subscriptions: https://multica.ai/docs/inbox
- Issues and projects: https://multica.ai/docs/issues
- Agents: https://multica.ai/docs/agents
- Skills: https://multica.ai/docs/skills
- Chat: https://multica.ai/docs/chat
- Autopilots: https://multica.ai/docs/autopilots
- AI coding tools matrix: https://multica.ai/docs/providers
- Changelog: https://multica.ai/changelog
- About: https://multica.ai/about
