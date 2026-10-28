"""Author S3 records for claude-code/b0100cff (the foundational governance session).

Run from tools/wash/. L2 pointers cite raw line numbers as shown in the
dialogue render (src:NNN).
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "b0100cff-2426-4786-b165-45827c9e2713"
F = "raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl"

store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(kind, conf, title, l0, l1, a, b, topic="t000", tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=S, harness="claude-code",
                  topic_id=topic, l2_file_path=F, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


# ---------------------------------------------------------------- decisions
add("decision", "quoted",
    "SeatLoom is local-first by definition, not by phase",
    "SeatLoom must run on the machine where the agents live: seats are local tmux sessions and the corpus is on local disk, so remote deployment is architecturally dead, not deferred.",
    "pty/mod.rs shells out to tmux directly and therefore only sees tmux on the backend host; the same holds for multi-harness corpus ingestion. Corollary accepted by Mr. Zhang: the product is a local-first desktop app; the remote/8088 path was abandoned before building it because a browser gets isTauri()=false and falls back to mock data. Extends the standing principle that SeatLoom mirrors tmux rather than replacing it - a mirror must be co-located with what it mirrors.",
    853, 853, tags=["architecture", "local-first"])

add("decision", "quoted",
    "GitHub stays primary remote; GitLab added as mirror",
    "GitHub remains the primary remote (existing history + authenticated gh CLI); internal GitLab is added as a second remote because it is ~10x faster and its web UI opens, while github.com HTTPS times out from this network.",
    "Measured: ls-remote GitHub 6.54s vs GitLab 0.67s; github.com HTTPS timed out at 20s; RTT 210ms vs 11ms. Aegis explicitly retracted the earlier claim that CI must live on GitLab because GitHub runners cannot reach the intranet - DB integration tests only need any Postgres at DEFAULT_DATABASE_URL and service containers suffice; self-hosted runners are free and unlimited for private repos. Note: the later engineering standard records GitLab as primary and GitHub as mirror - the topology was flipped afterwards; both statements are true at their timestamps.",
    955, 963, tags=["git", "infrastructure"])

add("decision", "quoted",
    "Gate strategy: FULL where the baseline is green, ratchet where it is debt",
    "CI gates split into FULL (whole tree, only where the baseline already passes) and RATCHET (changed files only, with a grandfather snapshot for pre-existing debt); the ratchet only ever tightens.",
    "Measured baselines: cargo fmt --check 201 lines drift, eslint 143 errors, prettier 87 files, 125 docs missing universal headers - but tsc --noEmit already green. So fmt/clippy/test/tsc became FULL gates; eslint/prettier/docs-header became ratchets with a snapshot grandfather list (207 docs). When a grandfathered file is fixed it leaves the list. Codified in docs/coordination/ENGINEERING_STANDARDS.md.",
    1487, 1517, tags=["ci", "governance", "gates"])

add("decision", "quoted",
    "Correct DB apply order is schema 001-005, then seed, then schema 008",
    "Migration 008 (sessions.project_id NOT NULL) is a post-seed data migration: apply schema 001-005, then seed, then 008. The old verify-postgres-baseline.sh stopped at 005 and never ran 008.",
    "Discovered while pre-flighting the CI test job: seed 001 inserts sessions with project_id null, which 008 backfills then sets NOT NULL - so 008 must run after seed. The stale verify script masked a real latent break on track. DB integration tests additionally require reconcile before running and --test-threads=1 (they share one DB and race otherwise). This recipe is what apply-db.sh encodes.",
    1770, 1824, tags=["database", "ci", "migration"])

add("decision", "quoted",
    "Per-seat git worktrees; the shared checkout holds docs only",
    "Each seat works in its own git worktree (seatloom-worktrees/<seat>); the single shared checkout is reserved for docs and coordination artifacts, because six seats sharing one checkout is a scheduled collision.",
    "Proven necessary in practice: Onyx switching the shared tree to its packet branch yanked the workspace out from under five other seats (42 uncommended files). Flux later proved verification work can run inside a seat worktree without restarting the seat, which made per-seat isolation non-disruptive. Parallel packets must have disjoint file sets.",
    979, 979, tags=["git", "infrastructure", "worktree"])

add("decision", "quoted",
    "Dispatch by file, never by long send-keys strings",
    "Seat dispatch writes the instruction to a file and sends only a short pointer (scripts/seat-dispatch.sh); long tmux send-keys strings truncate and corrupt seat input.",
    "Both Flux and Lyra demonstrably received truncated dispatches. File-based dispatch was verified end-to-end (Lyra read the whole file and acted on it).",
    4337, 4388, tags=["coordination", "infrastructure"])

add("decision", "quoted",
    "event_type closed vocabulary fixed at 12 values",
    "The append-only event vocabulary is fixed at 12 types: ArtifactProduced is dropped and P0 reuses ArtifactCreated. Mr. Zhang chose this over the 13-value draft.",
    "The vocabulary is append-only so changes are expensive. Onyx had proposed 13 (pending sign-off) including both ArtifactProduced and ArtifactCreated; the decision collapses them. Same sign-off pass: event_object_refs PK widened to (event_id, ref_type, ref_id, role) with a partial unique index WHERE role=subject enforcing at-most-one subject, leaving exactly-one to the write API.",
    5189, 5217, tags=["schema", "events", "governance"])

add("decision", "quoted",
    "Every milestone must deliver a user-perceivable capability",
    "Milestones are defined by what the user can newly perceive or do, not by engineering layers; pure plumbing steps are sub-tasks inside a milestone, and every milestone acceptance states what the user can now see or do.",
    "Mr. Zhang corrected the engineering-layer milestone ladder (step1 schema, step2 write API...) as user-invisible. Replaced with the UM ladder: UM0 one-screen live team view; UM1 work trace becomes clickable; UM2 progress readable as a live work graph. ROADMAP.md became the single living entry point (stage x pain solved x done-criteria x who is blocked).",
    5516, 5574, tags=["governance", "roadmap"])

add("decision", "quoted",
    "Capture taxonomy: observe what has an external ledger, emit what does not, interpose to create one",
    "Capture reliability spectrum: file changes are observed via git (an external structured ledger exists); pure semantics (handoffs, conclusions, decisions) can only be emitted via MCP; world actions (ssh, deploy, browser) sit in between and need emit plus tool-call capture plus T3 reconciliation. Interposition - SeatLoom-owned tools such as a managed terminal with a recording shell, an ssh proxy, an egress gateway - converts a trust-emit channel into an observable one.",
    "The precise value of interposition: you manufacture the missing external ledger wherever you interpose, and capture and control become the same mechanism (an ssh proxy can also block destructive commands). Selection principle: high-risk times otherwise-uncapturable. Decisive technical precision: intercept the executing shell/PTY, not the display pane - pipe-pane only mirrors the harness TUI (lossy) and never sees the raw ssh command. For CLI actions the managed terminal with a recording shell is the single best choke point and does not depend on agent cooperation.",
    3815, 3883, tags=["architecture", "capture", "data-model"])

add("decision", "quoted",
    "project is not the working directory; path is the strongest signal but not identity",
    "Project identity is a declared or derived durable container, not a path: one project spans multiple paths, one path (home) spans many unrelated activities, and projects survive machine migration while paths do not.",
    "Four measured counterexamples from the corpus (29 codex sessions with cwd=~; 12 with cwd=~/Documents/GitHub; seatloom itself across three paths; supervision conversations with no cwd at all). sessions.project_id is nullable and backfillable with a signal order: git remote, then declaration, then cwd.",
    4005, 4010, tags=["data-model", "project"])

add("decision", "quoted",
    "topic is a data-layer concern with derived, revisable, provenance-carrying attribution",
    "Topic attribution lives in the data layer (reusing event_object_refs, zero new primitives); who applies the label - human, Aegis, or the working agent - is a producer choice, not an architecture choice. Attribution is derived and revisable, never silently overwritten.",
    "Topic is the missing middle granularity between project (too big) and event (too small); it differs from session (mechanical, spans topics) and workitem (governed, optional). Plan: P0 leaves the ref, P1 auto-segments, P2 retroactive re-segmentation. This decision is the direct ancestor of the washing pipeline S2 stage.",
    3927, 3981, tags=["data-model", "topic"])

add("decision", "quoted",
    "SeatLoom core must not be built on any harness; forking would destroy the moat",
    "The core (records plus control plane) stays above harnesses: forking an open-source harness to build SeatLoom would make SeatLoom itself just another harness and kill the defensible position. The moat is holding project truth while the agent layer churns.",
    "Evidence: within one afternoon six seats moved from Gemini/bare-shell/broken-resume to a unified Claude Code with work intact. If seat identity, history and obligations lived inside the harness, that migration would have destroyed them. Capture mechanisms, by contrast, should standardize on one open-source runtime (Codex CLI OSS plus tmux was the accepted direction) and absorb proprietary harness knowledge (skills, prompts) as portable assets.",
    4287, 4299, tags=["architecture", "strategy", "harness"])

add("decision", "quoted",
    "Harness migration to PI: four providers, provider-level apiKey, wrapper-injected default",
    "The fleet migrates Claude Code to PI (open-source). PI models.json needs provider-level apiKey (inline headers do not count as credentials); Claude models on models-proxy must use the anthropic-messages API; the persistent default model 401s for custom providers, so a zsh wrapper injects the verified explicit flags.",
    "Configured and tested: stepplan (step-5-preview unlimited, daily driver), stepplan-anthropic, modelproxy (gpt-5.5), modelproxy-anthropic (claude-opus-4-8 for hard tasks), plus subworker/dataworker (the 200-dollar-per-day codex credit, openai-responses - auth verified, upstream 503 because the credit was exhausted that month). Archive is not resume: harness session formats differ and PI --resume only reads PI sessions; history is usable as retrievable reference or as a distilled briefing, never as a drop-in resume.",
    6476, 6725, tags=["harness", "pi", "migration"])

add("decision", "quoted",
    "Aegis is the supervisor session itself, not a seat pane",
    "The Aegis tmux seat was an empty shell that never did work; the real Aegis function is the supervisor conversation. Mr. Zhang ordered the empty seat retired to stop polluting data.",
    "Retired after safety checks (not inside any tmux; the Aegis seat was a separate process tree; worker seats unaffected). COORDINATION_RULES later gained the clarification that Aegis is a supervisor session, not a seat. Five workers remain: Flux, Lyra, Mira, Nimbus, Onyx.",
    4240, 4261, tags=["governance", "coordination"])

add("decision", "quoted",
    "Seat launch must pin the model with an explicit 1M-suffix id",
    "Seats are launched as stepcode claude with model claude-opus-5[1m]; passing --model directly bypasses the stepgo 1M opt-in and silently yields 200k context, and a bare alias falls through to defaults.",
    "Found while migrating all six seats to stepcode claude plus Opus 5. Three seats briefly appeared as Opus 4.8 - traced to a stale scrollback banner, not a real regression; argv and live banner cross-verified.",
    541, 541, tags=["infrastructure", "seats"])

# ----------------------------------------------------------------- failures
add("failure", "quoted",
    "CI environment gauntlet: seven overseas-network failures before green",
    "The first GitLab pipeline exposed seven distinct environment failures, each fixed by moving the fetch source internal: Docker Hub unreachable to Harbor images; crates.io throttled to Artifactory cargo; rustup toolchain overseas (700s stall) to rsproxy; apt overseas to Artifactory debian; postgres service alias unresolvable to FF_NETWORK_PER_BUILD; Tauri missing GTK/WebKit to apt; tauri missing ui/dist to a stub before cargo check.",
    "Pipeline 900942 failed on image pull; later runs showed npm at 10-23KiB/s with ECONNRESET (classic throttled overseas route). All sources internalized and committed. Result: rust:test 343s to 138s and a single pipeline per push after the duplicate-pipeline fix.",
    2020, 2980, tags=["ci", "infrastructure", "network"])

add("failure", "quoted",
    "Ratchet base must be merge-base against a real ancestor, never origin/main",
    "On GitLab the default branch main is an auto-generated empty Initial commit with no common history, so diffing against origin/main marks the entire tree as changed and the ratchet reports phantom violations across 86 files.",
    "frontend:lint failed on the first pipeline for exactly this reason. Fix: push track/infra-foundation (the packet branches' true fork point) and resolve the ratchet base as merge-base(HEAD, track), fetching it explicitly in CI before_script. The same base-resolution logic is shared by verify-docs.sh and lint-changed.sh.",
    2577, 2627, tags=["ci", "gates"])

add("failure", "quoted",
    "pipe-pane -o makes re-attach kill the live mirror",
    "tmux pipe-pane -o is a toggle (pipe only if none exists), not idempotent install as the code comment assumed: the second attach of the same pane turns the pipe OFF, so reconnecting seats silently kills the mirror.",
    "Found by Mira while dogfooding UM0. Forbidden from touching Rust (Nimbus lane), she did not fix it - she worked around it (only attach panes with pane_pipe==0), proved the UI side correct, reported the bug with the recommended fix (drop -o; a bare pipe-pane unconditionally replaces), and stated UM0 is not truly closed until the Rust fix lands. Nimbus then delivered abe4245.",
    6063, 6063, tags=["bug", "pty", "um0"])

add("failure", "quoted",
    "AgentHistory archive had silently gone stale for a week",
    "The 2026-09-30 AgentHistory archive contained nothing after 9/30: a full week of work (runtime-triple closeout, P0 step1/step2, UM0, the whole PI migration research) was missing, and the live session's own archived copy was the 9/30 snapshot.",
    "Root cause: the build script scanned ~/.claude/projects for claude-code transcripts but the seats actually write to ~/.stepgo/agents/*/launch/*/projects (stepcode CLAUDE_CONFIG_DIR). The manifest also lacked sha256 digests. Fix: patched the source scan, re-ran the build scripts incrementally, added digests. This is the corpus the washing pipeline later consumed - its completeness is load-bearing.",
    7054, 7097, tags=["archive", "data-loss"])

add("failure", "quoted",
    "Seats do not self-drive: a week passed with zero merges",
    "After the last dispatch, no seat progressed for a week - every seat sat idle at its prompt with an unstarted next instruction and track stayed frozen at the Flux HOLD commit. Seats wait for work; they do not seek it.",
    "Diagnosed on return: all five workers showed done plus a parked, never-submitted instruction (including the Flux UM0 re-verify sitting unsent in its input line). This is precisely the gap governed dispatch exists to close, and why the dispatch loop - not seat initiative - is the scheduling mechanism.",
    6252, 6252, tags=["coordination", "g19", "failure"])

add("failure", "quoted",
    "Supervisor over-polling: repeated misreads of transient pane states",
    "Aegis repeatedly polled seat panes mid-turn, caught transient states (stale banners, spinner-vs-status races, a monitor script bug emitting multi-line zeroes), over-interpreted them as regressions or stalls, and had to retract.",
    "Self-observed pattern: check, catch a partial or instantaneous state, over-interpret, re-check. Corrected practice: respond to real milestones (merge events), not pane polling; treat momentary readings as noise. Recorded because it recurs and the fix is behavioral, not technical.",
    4487, 4538, tags=["coordination", "supervision"])

# ------------------------------------------------------------------- facts
add("fact", "quoted",
    "Harbor project zhangxiaolong is public: anonymous pull via the Docker token flow",
    "CI runners pull internal Harbor images with zero credentials because the project is public: /v2/ returns 401 with a Www-Authenticate Bearer realm, the token service issues anonymous pull-scoped tokens, and the manifest fetch then returns 200.",
    "Verified by walking the token flow manually and inspecting the token access claim (actions: pull). No DOCKER_AUTH_CONFIG needed in .gitlab-ci.yml - the question of how CI pulls without auth has this mundane answer.",
    2292, 2310, tags=["infrastructure", "harbor", "ci"])

add("fact", "quoted",
    "Internal Artifactory serves npm and cargo anonymously at full internal speed",
    "The internal Artifactory exposes npm-public (virtual, aggregates an aliyun npm remote) and cargo-remote (cargo sparse index) - both anonymously readable and measured faster than public mirrors (npm tarball 200/0.37s/640KB; serde 200/0.10s).",
    "Discovered while removing the overseas throttle: the Artifactory REST API is anonymously listable. Lockfiles are unaffected by registry choice (pnpm integrity hashes; cargo replace-with preserves crates.io checksums) so the switch is clean and reversible; public mirrors remain documented fallbacks.",
    2469, 2518, tags=["infrastructure", "artifactory", "ci"])

add("fact", "quoted",
    "models-proxy is OpenAI-compatible with 379 models, but its Claude models require the Anthropic Messages API",
    "The company models-proxy speaks OpenAI chat-completions (379 models including gpt-5.5 and step-3.7-flash) but its Claude models reject the chat interface - they need the Anthropic-native Messages API. Some models (deepseek-v4-pro) are not entitled on this account.",
    "Established by direct per-model curl probes while configuring PI. Any OpenAI-compatible harness can use the proxy for non-Claude models; Claude requires a provider entry with the anthropic-messages api and a base URL without /v1.",
    6278, 6356, tags=["pi", "providers", "models"])

add("fact", "quoted",
    "PI models.json: apiKey is a provider-level field; settings-level default model 401s for custom providers",
    "In PI models.json, credentials are the provider-level apiKey field - an inline Authorization header is sent on calls but does not satisfy pi auth check (not_ready). A settings.json defaultModel pointing at a custom provider 401s (startup probes the built-in anthropic provider); explicit --provider/--model always works, so a wrapper injects them.",
    "Both behaviors were hit and worked around during PI configuration (four providers all ready; bare pi returns PONG via the wrapper). step-5-preview reasoning latency is high - run PI checks in the background rather than under a short foreground cap.",
    6476, 6718, tags=["pi", "providers", "config"])

add("fact", "quoted",
    "Codex history is 247 files / 14GB on disk with a ready index; Claude transcripts live under stepgo launch dirs",
    "~/.codex/sessions holds 247 session files (14GB) plus session_index.jsonl; Claude Code transcripts for the seats live at ~/.stepgo/agents/claude/launch/<run-id>/projects/<session>.jsonl (about 14MB each, 9 launch dirs) - NOT under ~/.claude/projects.",
    "Neither depends on its client to survive: the files are plain data. The 200-dollar-per-day subworker credit uses the OpenAI Responses API with the key from ~/.codex/auth.json - PI can speak openai-responses, so the credit is portable to PI without the Codex client (it was exhausted/503 at the time; the config was proven correct).",
    6834, 7085, tags=["archive", "codex", "corpus"])

add("fact", "quoted",
    "The 2026-09-30 AgentHistory archive is a hand-off-oriented corpus, not a bare backup",
    "AgentHistory/ contains raw/ (copies), summaries/ (per-session distilled summaries including unfinished items), by-project/ (per-project SUMMARY.md plus sessions.md for hand-off), _manifest.json and build scripts; its README states the purpose is for a later agent or harness to take over the work.",
    "Coverage at build time: 268 sessions (codex 210, stepcode 20, claude-code 15, cursor 13, gemini 9, opencode 1), 2025-11-20 to 2026-09-30, limited to working paths under ~/Documents/GitHub. Cursor and Gemini are inherently partial. This archive is the direct input of the washing pipeline.",
    7047, 7054, tags=["archive", "corpus"])

# -------------------------------------------------------------- preferences
add("preference", "quoted",
    "Seat-to-seat messages are in English; dispatch flows Aegis to Lyra to workers",
    "Coordination between seats happens in English with self-contained instructions and artifact paths, and work is routed Aegis to Lyra to the owning seat - never Mr. Zhang as a message router, and Aegis is engaged for escalation rather than as a work queue.",
    "Stated when dispatching the first Lyra packet and codified from the coordination rules: Lyra is the packaging and dispatch layer; schema to Onyx, Rust and infra to Nimbus, verification to Flux, frontend to Mira. Ratify gate: direct seat engagement requires Mr. Zhang ratify first.",
    548, 4212, tags=["coordination", "governance"])

add("preference", "quoted",
    "Iteration-speed optimizations accumulate in a standing dev-velocity TODO bucket",
    "Any optimization that speeds up iteration is recorded in the ROADMAP dev-velocity TODO rather than done ad hoc; the bucket is seeded with six measured bottlenecks (shared compile cache, pre-push gate preflight, doc-header compliance at source, one-command worktrees, DB test parallelization, CI cargo cache).",
    "Mr. Zhang: optimizations that speed iteration can all go in the todo. The shared-compile-cache item later got a measured answer: an APFS copy-on-write clone of target into a new worktree takes 18s with zero extra disk and cuts a cold worktree build to 2 of 452 crates recompiled.",
    5866, 5902, tags=["governance", "roadmap", "velocity"])

# --------------------------------------------------------------- unresolved
add("unresolved", "inferred",
    "UM0 live-mirror closeout state at corpus end",
    "As of the last corpus evidence, UM0 (live mirror) was HOLD by Flux on three counts: prettier ratchet count instability (no .prettierignore - a gate-tool defect Aegis owned), inability to visually confirm rendering, and a LIVE label still showing after seat-discovery failure. The pipe-pane -o blocker had a delivered fix not yet merged.",
    "Inferred resolution status: the session then jumps to 2026-10-08 with the harness migration superseding this track, and later ledger evidence shows the fleet now runs on PI under governed dispatch - so the Claude-Code-era UM0 loop was likely closed or overtaken rather than completed as planned. Confidence is inferred because the corpus does not contain the closing events for this specific loop.",
    6158, 6170, tags=["um0", "unresolved"])

print(json.dumps(store.stats(), indent=1))
