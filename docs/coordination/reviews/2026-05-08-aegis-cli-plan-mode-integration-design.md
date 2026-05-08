# CLI Plan Mode + Structured Event Integration — Design Review

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | 2026-05-08-aegis-cli-plan-mode-integration-design |
| status | issued |
| author | aegis |
| date | 2026-05-08 |
| tags | adapter, claude-code, gemini-cli, codex-cli, plan-mode, prompt-authority, phase-4, phase-5 |
| depends_on | `docs/architecture-decisions.md` AD-012 (PromptState), `docs/architecture-design.md` §3 adapter, `infra/postgres/schema/005_prompt_and_channel_action_authority.sql`, `crates/seatloom-core/src/adapter/traits.rs` |

---

## 1. Problem statement

SeatLoom's v0.1 PTY-wrapping strategy (Phase 4) captures the **rendered terminal byte stream** of a wrapped CLI agent (Claude Code / Gemini CLI / Codex CLI / …). That byte stream is sufficient for:

- Replaying what a seat's screen looked like
- Sending keystrokes back in (approve / keypress / text input)
- Writing a `.seatloom/transcripts/<session_id>.raw.log` evidence file

It is **not sufficient** for any of the following, which are load-bearing for replacing tmux-based coordination:

1. Detecting that a seat has entered a structured decision surface such as Claude Code's **plan mode** and is waiting for user approval of a formally written plan.
2. Extracting the plan body (so Supervisor can review it without the user having to re-read scrolling terminal output).
3. Knowing which tool the CLI invoked, with what arguments, and what the result was.
4. Correlating a user's approval keystroke (`1`, `2`, `enter`) with the specific plan/tool-call it resolved, across replay.
5. Persisting structured events (plan proposed → plan approved → plan edited → plan rejected) into the canonical event ledger.

Concretely: if Lyra's Claude Code session enters plan mode and writes a plan to `~/.claude/plans/some-slug.md`, a naïve PTY capture will record ~300 rendered lines of ANSI-escaped markdown + a one-line prompt. Supervisor IM cannot render that as a card, cannot link it to an Artifact, and cannot reliably detect "approved vs. rejected" without fragile regex over terminal bytes.

**PTY is a transport, not a protocol. We need a second layer that sees the protocol.**

## 2. Evidence landscape per CLI

What each target runtime actually writes to disk. All paths verified on the author's machine on 2026-05-08.

### 2.1 Claude Code

- **Plan files (plan mode output)**: `~/.claude/plans/<random-slug>.md` — one file per plan mode enter. Human-readable markdown. Written immediately when the agent calls `ExitPlanMode` (the tool fires after the plan body is already saved).
- **Session transcript (append-only JSONL)**: `~/.claude/projects/<repo-slug>/<session-id>.jsonl` where `<repo-slug>` is the project path with `/` replaced by `-` and leading `-` preserved (e.g. `-Users-jyxc-dz-0100609-Documents-GitHub-seatloom`).
- **JSONL record shape** (confirmed from live sample):
  - `{"type":"permission-mode","permissionMode":"bypassPermissions","sessionId":"..."}` — mode marker
  - `{"type":"user","message":{"role":"user","content":"..."},"promptId":"...","parentUuid":"..."}` — user input
  - `{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","name":"ExitPlanMode","input":{"plan":"<path>"}},...]}}` — tool call
  - `{"type":"tool-result","toolUseId":"...","content":[...]}` — tool outcome
  - `{"type":"file-history-snapshot",...}` — file edit tracking
- **Current process state**: `~/.claude/sessions/<pid>.json` — one file per live Claude Code PID.
- **Prompt history (global, cross-session)**: `~/.claude/history.jsonl`.
- **Hooks configuration**: `~/.claude/settings.json` (or per-project `<repo>/.claude/settings.json`).
- **Hook events available** (as of knowledge cutoff): `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Stop`, `Notification`, `SubagentStop`, etc. Each hook runs an arbitrary shell command and receives the tool name + input JSON on stdin.
- **Plan-mode signal**: the `ExitPlanMode` tool call is the canonical "I have a plan, please approve" signal. The tool's `input.plan` field is the markdown body. After approval the transcript continues with a `tool-result` block followed by normal tool use.

### 2.2 Gemini CLI

- **State directory**: `~/.gemini/` (presence confirmed in repo via `.gemini/` reference).
- **Structured transcript format**: not verified by this author at time of writing. Empirical spike required during Phase 4 onboarding.
- **Plan-mode equivalent**: Gemini CLI does not (as of this doc) expose a first-class "plan mode" tool. Its structured decisions surface via regular tool-call messages.
- **Hook equivalent**: unknown — assume none, rely on transcript tailing.

### 2.3 Codex CLI (OpenAI)

- **State directory**: `.codex/` (typically per-project).
- **Structured transcript format**: streaming JSON; precise schema not verified by this author.
- **Plan-mode equivalent**: Codex surfaces its plans via a regular `update_plan` tool invocation — similar semantics to `ExitPlanMode` but different name.
- **Hook equivalent**: unknown.

### 2.4 Other / future

- **Aider**, **Cline**, **Cursor CLI**, custom scripts, etc. — fall back to PTY-only with best-effort regex (`GenericAdapter`).

## 3. Integration strategies

Three orthogonal capture strategies. They are complementary — the recommended design combines them layered, not "pick one."

### Strategy A — File-system transcript tail

- Background task `tokio::spawn` per live session that `tail -f`-style reads the CLI's native transcript file (e.g. `~/.claude/projects/<slug>/<sid>.jsonl`).
- Parse each new line as a CLI-specific record, dispatch to the matching `AgentAdapter::parse_transcript`.
- The adapter translates native records → SeatLoom canonical events.

**Pros**: zero cooperation from the CLI required. Works for any CLI that writes a transcript. Resilient to CLI upgrades (data survives even if format shifts).
**Cons**: slight lag vs. real-time. Requires adapter-specific parsing per CLI. Can miss state that is only in memory (never persisted).

### Strategy B — Native hook / extension point

- For CLIs that expose hooks (Claude Code), SeatLoom registers a hook command that posts structured data back to the SeatLoom backend.
- Example Claude Code hook: `PostToolUse` running `seatloom hook-ingest --session $CLAUDE_SESSION_ID --tool $CLAUDE_TOOL_NAME` reading JSON from stdin.
- Hook command is a CLI subcommand we add (`src-cli/src/main.rs` already uses clap — extend it).

**Pros**: synchronous, zero parsing, precise, sanctioned by the CLI vendor. Cleanest possible integration.
**Cons**: Claude Code only (until other CLIs ship hooks). Requires modifying user's `~/.claude/settings.json` (scoped opt-in).

### Strategy C — PTY raw capture with regex + heuristics

- Fallback for CLIs with no transcript and no hooks.
- Adapter runs simple regex over the rolling PTY buffer to detect things like "Press 1 to accept" prompts.
- Produces low-confidence events — marked `prompt_kind='freeform'` with `prompt_policy='human_required'`.

**Pros**: always available, no CLI cooperation.
**Cons**: fragile, ANSI-polluted, no plan body extraction. Only useful as last-ditch signal.

## 4. Recommended layered capture

Per session, **all three layers run concurrently**; the adapter decides which layer is the authoritative source for each signal type.

```
┌────────────────────────────────────────────────────────────┐
│                    SeatLoom Session                         │
│                                                             │
│  ┌─────────────┐   ┌──────────────┐   ┌──────────────┐     │
│  │ Layer C     │   │ Layer A      │   │ Layer B      │     │
│  │ PTY bytes   │   │ Transcript   │   │ Native hooks │     │
│  │ (always)    │   │ tail (adapt.)│   │ (claude only)│     │
│  └──────┬──────┘   └───────┬──────┘   └───────┬──────┘     │
│         │                  │                  │            │
│         └────────┬─────────┴────────┬─────────┘            │
│                  ▼                  ▼                       │
│         ┌──────────────────────────────────┐                │
│         │   AgentAdapter::normalize()       │                │
│         │   (per-CLI, in seatloom-core)     │                │
│         └────────────────┬─────────────────┘                │
│                          ▼                                   │
│         ┌──────────────────────────────────┐                │
│         │  Canonical events + Prompt       │                │
│         │  authority (schema 005)          │                │
│         └──────────────────────────────────┘                │
└────────────────────────────────────────────────────────────┘
```

**Priority per signal**:

| Signal | Primary source | Fallback |
|--------|---------------|----------|
| Plan body + proposal | Layer B (hook) or Layer A (transcript) | Layer C regex "=== plan ===" |
| Tool invocation evidence | Layer A (transcript) | — |
| User approval action | Layer C (keystroke inject confirmed) + Layer A (transcript confirms accepted) | — |
| Session started/ended | Layer A (first/last JSONL record) | Layer C (process exit) |
| Terminal rendering (replay) | Layer C always | — |

**Dedup rule**: if Layer B and Layer A both fire for the same `(session_id, tool_use_id, event_kind)`, the earlier arrival wins; the later one is dropped (idempotent via `canonical_events` unique constraint on `(object_ref, event_type)` where applicable, or via `prompt_instances.id` primary key).

## 5. Data model mapping

Where each captured signal lives in PostgreSQL. This uses existing schema — no new tables needed for v0.1.

### 5.1 Plan proposal → `prompt_instances` + Artifact

When a plan mode surface is detected:

1. **Write the plan file as an Artifact** (via normal reconcile or direct insert):
   - `artifacts.template = 'T3'`
   - `artifacts.subtype = 'cli_plan'` *(new subtype — needs allow-list update in `db/document_parser.rs::validate_subtype`)*
   - `artifacts.system_kind = 'cli-plan-mode'`
   - `artifacts.source_session_id` = SeatLoom session id
   - `artifacts.storage_path` = `~/.claude/plans/<slug>.md` (or inline body for Codex)

2. **Write a `prompt_instance` row** recording the waiting approval:
   - `prompt_instances.prompt_kind = 'plan_approval'` *(new allowed value — needs schema 005 enum extension)*
   - `prompt_instances.prompt_policy` = 'needs_approval' or 'human_required' per AD-012 policy table
   - `prompt_instances.evidence_ref` = `artifact:<artifact_id>` (or direct path)
   - `prompt_instances.evidence_preview` = first 400 chars of plan body
   - `prompt_instances.available_actions` = `['approve','approve_with_edits','reject','request_clarification']`
   - `prompt_instances.assist_max_steps/tokens` = Supervisor assist budget (from AssistBudget AD-012)
   - `prompt_instances.session_id` = SeatLoom session id (FK to `sessions`)

3. **Emit `CanonicalEvent.PromptDetected`** with `payload.kind='plan_approval'`, `payload.evidence_ref=<path>`.

### 5.2 User approval → `prompt_actions` + keystroke inject

When Mr. Zhang clicks Approve in Supervisor IM:

1. Frontend → `cmd_approve_prompt(prompt_id, source_channel='desktop', note?)`.
2. Backend writes `prompt_actions` row:
   - `action_kind = 'approve'`
   - `actor_ref = 'human:zhang'` (or seat ref if automated)
   - `source_channel = 'desktop' | 'mobile' | 'supervisor'`
3. Backend looks up the adapter for this session, calls `adapter.inject_input(session, "1\n")` (Claude Code approval keybinding; adapter-specific).
4. Layer A (transcript tail) observes the subsequent `tool-result` block, marks `prompt_instances.status = 'resolved'`, sets `resolved_at` and `result_event_id`.
5. Emit `CanonicalEvent.PromptInputInjected` + `CanonicalEvent.PromptResolved`.

### 5.3 Tool invocations generally → `canonical_events`

Every `tool_use` block in the transcript (regardless of whether it's plan mode):

- `canonical_events.event_type = 'ToolInvoked'`
- `canonical_events.payload = {"tool":"<name>","input":<sanitised json>,"tool_use_id":"..."}`
- `canonical_events.actor_ref` = seat ref
- `event_object_refs`: link to session + artifact if tool produced one

This gives SeatLoom a full per-session timeline of tool invocations, which is what the v2 TimelineSection needs to show real structured activity.

## 6. Per-adapter implementation plan

### 6.1 `ClaudeAdapter` — reference implementation

File: `crates/seatloom-core/src/adapter/claude.rs`.

**Responsibilities**:

1. `launch(working_dir, initial_input)`:
   - Spawn `claude` via portable-pty in `working_dir`.
   - Return `RunningSession { pid }`.
   - Compute `native_session_id` by reading the newest file under `~/.claude/projects/<slug>/*.jsonl` whose `mtime` > spawn time.

2. `attach(pid)`:
   - Locate `~/.claude/sessions/<pid>.json`; resolve `sessionId` from it.
   - Start Layer A tail on the corresponding `projects/<slug>/<sid>.jsonl`.
   - Do **not** attempt to PTY-attach to an already-running claude; Phase 4 launches-only.

3. `inject_input(session, input)`:
   - Write `input` bytes to the PTY master fd.
   - For plan mode approval, the adapter knows the mapping `approve → '1\n'`, `reject → '2\n'`, etc.

4. `parse_transcript(raw_bytes) -> Vec<TranscriptEntry>`:
   - Accepts a chunk of JSONL lines.
   - Deserialize via `serde_json::Deserializer::from_slice(bytes).into_iter::<Value>()`.
   - For each record, match on `type`:
     - `"assistant"` with `content[].type == "tool_use"` → yield `TranscriptEntry::ToolInvoked { name, input }`
     - `"tool-result"` → yield `TranscriptEntry::ToolResult { tool_use_id, content }`
     - `"user"` → yield `TranscriptEntry::UserInput { content }`
     - unknown types → skip silently (forward-compat)

5. `detect_native_session_id(working_dir) -> Option<String>`:
   - Compute slug: replace `/` with `-` in absolute `working_dir` path.
   - Open `~/.claude/projects/<slug>/`; pick most-recently-modified `.jsonl` whose first line contains `sessionId`.

**TranscriptEntry needs extension** beyond current scaffold:

```rust
pub enum TranscriptEntry {
    UserInput { content: String },
    AssistantText { content: String },
    ToolInvoked { tool_use_id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: serde_json::Value },
    PlanProposed { plan_body: String, plan_file_path: Option<PathBuf> },
    PermissionModeChanged { mode: String },
    Other { raw: serde_json::Value },
}
```

Plan mode detection rule: `ToolInvoked { name: "ExitPlanMode", input }` → re-emit as `PlanProposed { plan_body: input["plan"].as_str().unwrap_or(""), plan_file_path: None }`. If the plan body references a file path that exists under `~/.claude/plans/`, fill in `plan_file_path`.

### 6.2 `CodexAdapter`

Same shape as `ClaudeAdapter` but:
- Transcript location and format differ (empirical spike in Phase 4).
- Plan-mode equivalent is `update_plan` tool; map similarly.
- Approval keybinding: adapter-specific.

### 6.3 `GenericAdapter`

- PTY only (Layer C).
- `parse_transcript` returns `TranscriptEntry::Other` for every chunk.
- Never emits `PlanProposed`.
- Supervisor IM for a Generic-runtime seat is chat-only; no structured prompt cards.

## 7. Hook injection for Claude Code (Phase 5)

To enable Strategy B for Claude Code, SeatLoom ships a CLI subcommand:

```bash
seatloom hook-ingest --session $CLAUDE_SESSION_ID --hook $CLAUDE_HOOK_KIND
```

reads JSON from stdin, forwards to backend over a local Unix-domain socket (`$XDG_RUNTIME_DIR/seatloom.sock` or `/tmp/seatloom-<user>.sock`).

And a one-shot injector:

```bash
seatloom install-claude-hooks [--scope user|project]
```

which edits `~/.claude/settings.json` (or `<project>/.claude/settings.json`) to register:

```json
{
  "hooks": {
    "PostToolUse": [{"matcher":"ExitPlanMode","command":"seatloom hook-ingest --kind plan-proposed"}],
    "UserPromptSubmit": [{"command":"seatloom hook-ingest --kind user-prompt"}]
  }
}
```

Only done with explicit user consent. Uninstall via `seatloom install-claude-hooks --remove`.

## 8. Phase contract

### 8.1 Phase 4 (PTY + transcript adapter)

**In scope**:
- `crates/seatloom-core/src/adapter/wrapper.rs` full implementation with portable-pty.
- `ClaudeAdapter` Layer A (transcript tail) + Layer C (PTY) capture.
- `TranscriptEntry` enum extended per §6.1.
- Tauri commands `cmd_launch_session` / `cmd_pty_write` / `cmd_pty_resize` / `cmd_kill_session`.
- `PlanProposed` events written to `canonical_events` + `prompt_instances`.
- New subtype `'cli_plan'` added to `db/document_parser.rs::validate_subtype` allow-list and referenced in `DOCUMENT_TEMPLATES.md`.
- Schema 005 `prompt_kind` CHECK constraint updated to include `'plan_approval'`.

**Explicit non-goals**:
- Gemini / Codex adapter empirical work (next packet).
- Hook injection (Phase 5).
- Plan edit-in-UI (Phase 5).

### 8.2 Phase 5 (Supervisor routing closes the loop)

**In scope**:
- `seatloom hook-ingest` CLI subcommand + Unix socket IPC.
- `seatloom install-claude-hooks` one-shot.
- Supervisor IM plan card: detect `prompt_instance.kind='plan_approval'` → render `plan_body` preview + Approve / Reject / Edit buttons.
- `cmd_approve_prompt` / `cmd_reject_prompt` / `cmd_submit_plan_edit` Tauri commands.
- Approval flow: writes `prompt_actions` → adapter keystroke inject → Layer A confirms via transcript.

**Explicit non-goals**:
- Generic plan-mode for CLIs without hooks (stays Layer A only).
- Cross-seat plan sharing (deferred).

## 9. Open questions (to resolve before Phase 4 packet issues)

1. **Transcript file location stability.** Claude Code's project-slug format is inferred from `~/.claude/projects/` directory listing. Is there a public API / env var that exposes the authoritative path? If not, the adapter must handle path migrations gracefully.
2. **Plan body persistence authority.** Should the plan be copied into PostgreSQL `documents.body_text` at capture time, or stay a reference to `~/.claude/plans/<slug>.md`? Recommendation: **copy in**, since `~/.claude/plans/` may be cleaned by the user and evidence must be reproducible. Treat disk file as a cache.
3. **Session ID mapping**. SeatLoom's `sessions.id` is a SeatLoom-generated ULID; `native_session_id` is the CLI's UUID. Both must be persisted; `native_session_id` is a nullable column already defined in schema 001.
4. **Multiple concurrent plan-mode sessions**. A supervisor may have 3 seats in plan mode simultaneously. Prompt cards in Supervisor IM must be per-seat with clear visual separation — UI spec lives in chan-06 L1 surface guidance.
5. **Approval keybinding drift**. `'1\n'` for approve is Claude Code's current default; it may change or be remappable. Adapter should read from a known config if available and fall back to the documented default.
6. **Hooks user consent**. Modifying `~/.claude/settings.json` requires explicit user action. Never do this silently in bootstrap.

## 10. Test surface (non-optional before Phase 4 acceptance)

- **Unit**: `ClaudeAdapter::parse_transcript` given a fixture JSONL file covering `user`, `assistant-tool_use`, `tool-result`, `permission-mode`, `file-history-snapshot`, and `ExitPlanMode` specifically.
- **Unit**: `detect_native_session_id` given a synthesised `~/.claude/projects/<slug>/` directory with multiple `.jsonl` files; must pick the newest with a valid `sessionId` header.
- **Integration (live)**: launch a real `claude` process in a tmp workspace, observe `SessionStarted` event within 2s.
- **Integration (live)**: trigger a plan-mode session, observe `prompt_instances` row with `kind='plan_approval'` and correct `evidence_preview` within 2s of plan file appearance.
- **Round-trip**: approve via `cmd_approve_prompt`, observe `prompt_actions` row and subsequent `tool-result` block in transcript confirming the plan was accepted.

## 11. Implementation order (NIMBUS packet checklist)

When Phase 4 NIMBUS packet is drafted, it must enumerate:

1. [ ] Extend `TranscriptEntry` enum per §6.1.
2. [ ] Implement `WrapperAdapter` with portable-pty.
3. [ ] Implement `ClaudeAdapter::launch/attach/parse_transcript/inject_input/detect_native_session_id`.
4. [ ] Background transcript-tail task (`tokio::spawn`) per live session, broadcast parsed events via `tokio::sync::broadcast`.
5. [ ] Tauri emit `session:event` per canonical event; frontend subscribes.
6. [ ] Update `db/document_parser.rs::validate_subtype` to accept `'cli_plan'` for T3.
7. [ ] Migration: schema 005 CHECK constraint on `prompt_kind` add `'plan_approval'` (document as schema 006 or in-place ALTER — decide in packet).
8. [ ] Add adapter-dispatch logic in `cmd_launch_session` (`match runtime { "ClaudeCode" => ClaudeAdapter, ... }`).
9. [ ] Persist `PlanProposed` → `prompt_instances` + `artifacts` insert.
10. [ ] Unit + integration tests per §10.

Phase 5 follow-on packet then adds:

11. [ ] `seatloom hook-ingest` subcommand + Unix socket server in `src-tauri/src/hook_server.rs`.
12. [ ] `seatloom install-claude-hooks` subcommand.
13. [ ] `cmd_approve_prompt` / `cmd_reject_prompt` / `cmd_submit_plan_edit`.
14. [ ] Supervisor IM plan card component.
15. [ ] End-to-end approval loop: click Approve → PTY inject → transcript confirms.

## 12. Risks

- **CLI format churn**: Claude Code JSONL shape may change between releases. Mitigation: adapter parses loosely (match on `type` field, ignore unknowns), version the fixture tests, treat unparseable records as `Other`.
- **Hook visibility into SeatLoom state**: The hook runs as a child of the CLI, not SeatLoom. It needs a reliable IPC path (Unix socket) and auth (session id token passed in env var).
- **Multi-user concurrency**: two users running SeatLoom on the same machine will clash on `/tmp/seatloom-<user>.sock`. Scope socket per UID.
- **Transcript file locking**: macOS POSIX reads on `.jsonl` being written to concurrently — fine (append-only), but adapter must handle partial-line reads (wait for `\n`).

## 13. Cross-reference to existing design

- AD-012 (PromptState/Kind/Policy/Action/AssistBudget): this design is the **concrete realisation** of AD-012 for the three specific CLIs we target.
- Schema 005 (`prompt_instances`, `prompt_actions`, `channel_action_receipts`): already covers the data model needs; only enum extensions required.
- `AgentAdapter` trait (`crates/seatloom-core/src/adapter/traits.rs`): already the right shape; needs only method bodies.
- AD-011 (CanonicalEvent): `PlanProposed`, `PromptDetected`, `PromptResolved`, `ToolInvoked` need to be added to the `EventType` enum.

## 14. Decision

**Adopt**: layered capture per §4, data mapping per §5, phase contract per §8.

**Register action**: add this design as a dependency for the Phase 4 NIMBUS packet — the packet MUST cite this document in its `depends_on` field.

---

*Design by Aegis · 2026-05-08 · Status: issued. To be cited as authority when the Phase 4 PTY adapter NIMBUS packet is drafted.*
