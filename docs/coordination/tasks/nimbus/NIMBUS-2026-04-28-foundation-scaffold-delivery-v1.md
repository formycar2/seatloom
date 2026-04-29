# NIMBUS-2026-04-28-foundation-scaffold-delivery-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-28-foundation-scaffold-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-environment-clarification-v1.md` |
| supersedes | - |
| tags | architecture, scaffold, rust, tauri, foundation |

**Packet ref:** `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-v1.md`
**Clarification ref:** `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-environment-clarification-v1.md`
**Status note:** compile verification deferred (environment blocker)

---

## 1. Micro-brief

Foundation scaffold for SeatLoom v0.5 is complete. All Cargo workspace members, core type modules, event families, adapter stubs, data engine stubs, and CLI/Tauri surface stubs are in place. The scaffold is structurally complete and architecturally aligned with AD-008 through AD-012. Compile verification is deferred due to `cargo`/`rustc` unavailability on this seat.

---

## 2. Changed Files / Scaffold Tree

```
seatloom/
├── Cargo.toml                              # workspace root (resolver=2; 3 members)
├── crates/
│   └── seatloom-core/
│       ├── Cargo.toml                      # library crate (serde/chrono/tokio/etc.)
│       └── src/
│           ├── lib.rs                      # 9 module declarations
│           ├── objects/
│           │   ├── mod.rs
│           │   ├── id.rs                   # define_id! macro + 10 typed IDs
│           │   ├── seat.rs                 # SeatIdentity/ProjectRoleBind/SeatDelegation/SeatRole/SeatStatus/DelegationStatus
│           │   ├── session.rs              # Session/Runtime/SessionStatus(PromptBlocked)/PromptState/PromptKind/PromptPolicy/PromptAction/AssistBudget
│           │   ├── workitem.rs             # WorkItem/WorkItemStatus(event-first)/Priority
│           │   ├── artifact.rs             # Artifact(dual-key)/ArtifactTemplate(T1-T7)/SystemArtifactKind
│           │   ├── handoff.rs              # Handoff/HandoffStatus(Working=P1)/ActorRef
│           │   ├── checkpoint.rs           # Checkpoint/CheckpointTrigger/CheckpointSummary/SummaryQuality
│           │   ├── pipeline.rs             # PipelineRun/PipelineRunStatus
│           │   └── receipt.rs              # HandoffReceipt
│           ├── ledger/
│           │   ├── mod.rs
│           │   ├── event.rs                # CanonicalEvent/EventType(25 variants)/ObjectRef
│           │   ├── writer.rs               # LedgerWriter stub
│           │   ├── reader.rs               # LedgerReader stub
│           │   └── index.rs                # LedgerIndex stub (cache-only, FTS5 authoritative)
│           ├── storage/
│           │   ├── mod.rs
│           │   ├── project.rs              # ProjectConfig/PackEngineConfig/PipelineConfig
│           │   ├── yaml_io.rs              # read_yaml/write_yaml stubs
│           │   └── jsonl_io.rs             # append_jsonl stub
│           ├── adapter/
│           │   ├── mod.rs
│           │   ├── traits.rs               # AgentAdapter trait/AdapterCapabilities/RunningSession/TranscriptEntry
│           │   ├── wrapper.rs              # WrapperAdapter stub
│           │   ├── codex.rs                # CodexAdapter stub
│           │   ├── claude.rs               # ClaudeAdapter stub
│           │   └── generic.rs              # GenericAdapter stub
│           ├── data_engine/
│           │   ├── mod.rs
│           │   ├── pack_engine/
│           │   │   ├── mod.rs
│           │   │   ├── selector.rs         # PackSelector stub
│           │   │   ├── budgeter.rs         # PackBudgeter stub (8192/32768 token constants noted)
│           │   │   ├── assembler.rs        # PackAssembler stub
│           │   │   └── verifier.rs         # PackVerifier stub
│           │   ├── retrieval.rs            # RetrievalEngine stub (SQLite FTS5 L1/L2 noted)
│           │   ├── gate_engine.rs          # GateEngine stub
│           │   ├── route_engine.rs         # RouteEngine stub
│           │   ├── budget_enforcer.rs      # BudgetEnforcer stub
│           │   ├── isolation.rs            # IsolationLayer stub
│           │   └── audit.rs               # AuditBinder stub
│           ├── pipeline/
│           │   ├── mod.rs
│           │   ├── runner.rs               # PipelineRunner stub
│           │   └── stage.rs               # PipelineStage stub
│           ├── reconcile/
│           │   ├── mod.rs
│           │   └── engine.rs               # ReconcileEngine stub
│           ├── git/
│           │   ├── mod.rs
│           │   └── ops.rs                  # GitOps stub
│           └── views/
│               ├── mod.rs
│               ├── inbox.rs                # InboxView stub
│               └── timeline.rs             # TimelineView stub
├── src-tauri/
│   ├── Cargo.toml                          # seatloom-tauri binary (tauri 2, tauri-build)
│   ├── build.rs                            # tauri_build::build()
│   ├── tauri.conf.json                     # app config skeleton
│   └── src/
│       ├── main.rs
│       ├── state.rs                        # AppState stub
│       └── commands/
│           ├── mod.rs
│           ├── seat_cmds.rs
│           ├── session_cmds.rs
│           ├── workitem_cmds.rs
│           ├── handoff_cmds.rs
│           ├── artifact_cmds.rs            # template/subtype filter surface (AD-008)
│           ├── delegation_cmds.rs          # Delegation command group (AD-009)
│           ├── prompt_cmds.rs              # PromptState surface (AD-012)
│           ├── timeline_cmds.rs
│           ├── inbox_cmds.rs
│           └── reconcile_cmds.rs
└── src-cli/
    ├── Cargo.toml                          # seatloom-cli binary (clap 4, tokio)
    └── src/
        └── main.rs                         # Init/Attach/Sessions/Workitems/Reconcile subcommands
```

Total new files: **62**

---

## 3. Type Coverage Matrix

| Domain        | Type / Enum                      | AD alignment  | Status    |
|---------------|----------------------------------|---------------|-----------|
| IDs           | SeatId, SessionId, WorkItemId, ArtifactId, HandoffId, PipelineId, PipelineRunId, CheckpointId, DelegationId, EventId | §3.1 | ✅ |
| Seat          | SeatIdentity, ProjectRoleBind, SeatDelegation, DelegationStatus, SeatRole, SeatStatus | AD-009 | ✅ |
| Session       | Session, Runtime, SessionStatus (incl. PromptBlocked) | AD-012 | ✅ |
| Prompt        | PromptState, PromptKind, PromptPolicy, PromptAction, AssistBudget | AD-012 | ✅ |
| WorkItem      | WorkItem, WorkItemStatus (event-first, no Rejected/Rescoped), Priority | AD-010 | ✅ |
| Artifact      | Artifact (dual-key template+subtype), ArtifactTemplate (T1–T7), SystemArtifactKind | AD-008 | ✅ |
| Handoff       | Handoff, HandoffStatus (Working=P1), ActorRef | §3.2 | ✅ |
| Checkpoint    | Checkpoint, CheckpointTrigger, CheckpointSummary, SummaryQuality | §3.2 | ✅ |
| Pipeline      | PipelineRun, PipelineRunStatus | §3.2 | ✅ |
| Receipt       | HandoffReceipt | §3.2 | ✅ |
| Event         | CanonicalEvent, EventType (25 variants incl. AD-012 prompt events), ObjectRef | AD-010, AD-012 | ✅ |
| Adapter       | AgentAdapter trait, AdapterCapabilities, RunningSession, TranscriptEntry | §4 | ✅ (interface) |
| Data Engine   | RetrievalEngine (SQLite FTS5 noted), PackSelector/Budgeter/Assembler/Verifier, GateEngine, RouteEngine, BudgetEnforcer, IsolationLayer, AuditBinder | AD-011 | ✅ (stubs) |
| Storage       | ProjectConfig, PackEngineConfig, PipelineConfig | §9 | ✅ |
| Views         | InboxView, TimelineView | §5.1 | ✅ (stubs) |

---

## 4. Compile Command + Result (Deferred)

```
$ source "$HOME/.cargo/env" && cargo check -p seatloom-core
bash: /Users/jyxc-dz-0100609/.cargo/env: No such file or directory

$ which cargo
cargo: not found
```

**Local compile verification deferred: cargo/rustc unavailable on the current seat environment.**

Per clarification artifact (`NIMBUS-2026-04-28-foundation-scaffold-environment-clarification-v1.md`), this is the expected outcome for this seat. Compile verification must be performed on a Rust-capable environment (Aegis or CI).

Primary verification target when Rust is available:
```
cargo check -p seatloom-core 2>&1
```

Secondary target (note: Tauri 2 native deps may fail in headless CI):
```
cargo check -p seatloom-cli 2>&1
```

---

## 5. Deferred Stubs

| Module                         | Deferred work                                      | Target packet        |
|--------------------------------|----------------------------------------------------|----------------------|
| `adapter/wrapper.rs`           | PTY attach, stdout tee implementation              | adapter-engine       |
| `adapter/codex.rs`             | Structured JSON output parsing, native session ID  | adapter-engine       |
| `adapter/claude.rs`            | JSONL tool-call capture, native session ID         | adapter-engine       |
| `adapter/generic.rs`           | Generic PTY capture fallback                       | adapter-engine       |
| `data_engine/retrieval.rs`     | SQLite FTS5 L1/L2 schema + query implementation    | data-engine          |
| `data_engine/pack_engine/`     | Tier 0/1/2 content selection + budgeting           | data-engine          |
| `data_engine/gate_engine.rs`   | Transition guards, field validation                | data-engine          |
| `data_engine/route_engine.rs`  | Inbox projection, priority bands                   | data-engine          |
| `ledger/writer.rs`             | JSONL append + fsync                               | ledger-engine        |
| `ledger/reader.rs`             | JSONL sequential read                              | ledger-engine        |
| `ledger/index.rs`              | In-memory cache rebuild from SQLite                | ledger-engine        |
| `storage/yaml_io.rs`           | serde_yaml read/write with file lock               | storage-engine       |
| `storage/jsonl_io.rs`          | Atomic JSONL append                                | storage-engine       |
| `reconcile/engine.rs`          | Git/FS ↔ Ledger diff + event emission              | reconcile-engine     |
| `git/ops.rs`                   | git2 commit/diff/branch operations                 | git-ops              |
| `src-tauri/src/state.rs`       | AppState engine initialization                     | runtime-engine       |
| `src-tauri/src/commands/*`     | Full Tauri command implementations                 | runtime-engine       |
| `src-cli/src/main.rs`          | CLI dispatch to seatloom-core                      | runtime-engine       |

---

## 6. Blockers / Next Owner

| ID            | Description                                                                 | Owner          |
|---------------|-----------------------------------------------------------------------------|----------------|
| ENV-001       | `cargo`/`rustc` not installed on Nimbus seat. Compile verification deferred. First compile must run on Aegis or CI. | Aegis / CI     |
| NEXT-001      | `adapter-engine` packet: full adapter implementations for Claude, Codex, wrapper, generic | Nimbus (next packet) |
| NEXT-002      | `data-engine` packet: SQLite FTS5 schema + RetrievalEngine full implementation | Nimbus (next packet) |
| NEXT-003      | `ledger-engine` packet: JSONL writer + reader + SQLite index rebuild         | Nimbus (next packet) |
| NEXT-004      | `runtime-engine` packet: AppState init + Tauri command wiring + CLI dispatch | Nimbus (next packet) |

---

*Delivery authored by Nimbus (TechArchi seat) — foundation scaffold for SeatLoom v0.5*
