# SeatLoom 技术架构设计 v1.0

| 项目 | 内容 |
|------|------|
| 文档 | Architecture Design v1.0 |
| 状态 | Draft |
| 作者 | Aegis (for Nimbus) |
| 更新时间 | 2026-04-28 |
| 对齐版本 | v0.5 contract set (NIMBUS-2026-04-28) |
| 依赖文档 | `docs/PRODUCT_TRUTH.md`、`docs/architecture-decisions.md` |

---

Transitional note: this is an active support document during v0.5 consolidation. Product meaning comes from `docs/PRODUCT_TRUTH.md` and the active contract set; this document constrains implementation and system shape only.

## 1. 系统总览

```
┌──────────────────────────────────────────────────────────┐
│                    SeatLoom Desktop                        │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  Frontend  (React 19 / TypeScript / Vite)           │  │
│  │  ├── AppShell (Sidebar + MainPanel + DetailPane)    │  │
│  │  ├── Views (Inbox, Timeline, WorkItems, Pipeline)   │  │
│  │  ├── EmbeddedTerminal (xterm.js)                    │  │
│  │  ├── State (Zustand stores)                         │  │
│  │  └── IPC bridge (@tauri-apps/api)                   │  │
│  └──────────────────────┬──────────────────────────────┘  │
│                         │ Tauri IPC (JSON serialized)      │
│  ┌──────────────────────▼──────────────────────────────┐  │
│  │  Backend  (Rust / Tauri 2)                          │  │
│  │  ├── src-tauri/commands/  (#[tauri::command])       │  │
│  │  ├── src-tauri/pty.rs     (PTY manager)             │  │
│  │  └── src-tauri/state.rs   (AppState)                │  │
│  └──────────────────────┬──────────────────────────────┘  │
└─────────────────────────┼────────────────────────────────┘
                          │ links to
┌─────────────────────────▼────────────────────────────────┐
│  seatloom-core  (Rust library crate)                      │
│  ├── storage/     .seatloom/ 文件 IO                      │
│  ├── ledger/      append-only event store                 │
│  ├── objects/     Seat, Session, WorkItem, Artifact, ...  │
│  ├── adapter/     wrapper_capture, native_attach          │
│  ├── data_engine/  Data Engine (Pack Engine, Retrieval, Gate, Route)  │
│  ├── pipeline/    Pipeline runner                         │
│  ├── reconcile/   Git/FS ↔ Ledger reconciliation          │
│  └── git/         Git 操作封装                             │
├───────────────────────────────────────────────────────────┤
│  seatloom-cli  (Rust binary crate, reuses seatloom-core)  │
│  └── clap subcommands                                     │
└───────────────────────────────────────────────────────────┘
```

---

## 2. Cargo Workspace 结构

```
seatloom/
├── Cargo.toml                    # workspace root
├── crates/
│   └── seatloom-core/            # 核心业务逻辑（library crate）
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── storage/          # .seatloom/ 读写
│           │   ├── mod.rs
│           │   ├── project.rs    # project.yaml
│           │   ├── yaml_io.rs    # YAML 序列化/反序列化
│           │   └── jsonl_io.rs   # JSONL 追加写/读
│           ├── ledger/           # 事件账本
│           │   ├── mod.rs
│           │   ├── event.rs      # CanonicalEvent 定义
│           │   ├── writer.rs     # append-only 写入
│           │   ├── reader.rs     # 查询/过滤/回放
│           │   └── index.rs      # 内存索引（启动时从 JSONL 重建）
│           ├── objects/          # 核心对象模型
│           │   ├── mod.rs
│           │   ├── seat.rs
│           │   ├── session.rs
│           │   ├── workitem.rs
│           │   ├── artifact.rs
│           │   ├── handoff.rs
│           │   ├── pipeline.rs
│           │   ├── checkpoint.rs
│           │   ├── receipt.rs
│           │   └── id.rs         # 类型安全的 ID 系统
│           ├── adapter/          # Agent 适配器
│           │   ├── mod.rs
│           │   ├── traits.rs     # AdapterCapability trait
│           │   ├── wrapper.rs    # wrapper_capture 实现
│           │   ├── codex.rs      # Codex CLI 深适配
│           │   ├── claude.rs     # Claude Code 深适配
│           │   └── generic.rs    # 通用 CLI 浅适配
│           ├── data_engine/      # Data Engine (AD-008–AD-011)
│           │   ├── mod.rs
│           │   ├── pack_engine/       # Worker + Supervisor continuity packs
│           │   │   ├── mod.rs
│           │   │   ├── selector.rs    # Tier 0/1/2 + optional content selection
│           │   │   ├── budgeter.rs    # Token budget enforcement per tier
│           │   │   ├── assembler.rs   # LaunchPack + SupervisorPack assembly
│           │   │   └── verifier.rs    # Completeness + path validity checks
│           │   ├── retrieval.rs       # Fixed L1→L2→L3→explain order (AD-011)
│           │   ├── gate_engine.rs     # Transition + field + budget gates
│           │   ├── route_engine.rs    # Inbox projection, priority bands
│           │   ├── budget_enforcer.rs # Hard token limits, graceful stop
│           │   ├── isolation.rs       # Need-to-know access enforcement
│           │   └── audit.rs           # Ledger event binding
│           ├── pipeline/         # Pipeline 执行器
│           │   ├── mod.rs
│           │   ├── runner.rs
│           │   ├── stage.rs
│           │   └── definitions/  # 内置 Pipeline YAML
│           ├── reconcile/        # 状态一致性
│           │   ├── mod.rs
│           │   └── engine.rs
│           ├── git/              # Git 操作封装
│           │   ├── mod.rs
│           │   └── ops.rs
│           └── views/            # 视图投影
│               ├── mod.rs
│               ├── inbox.rs
│               └── timeline.rs
│
├── src-tauri/                    # Tauri 桌面应用 binary
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/             # Tauri v2 capability 权限
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/             # IPC command handlers
│   │   │   ├── mod.rs
│   │   │   ├── seat_cmds.rs
│   │   │   ├── session_cmds.rs
│   │   │   ├── workitem_cmds.rs
│   │   │   ├── handoff_cmds.rs
│   │   │   ├── pipeline_cmds.rs
│   │   │   ├── timeline_cmds.rs
│   │   │   ├── inbox_cmds.rs
│   │   │   └── reconcile_cmds.rs
│   │   ├── pty.rs                # PTY 进程管理
│   │   └── state.rs              # Tauri managed state
│   └── icons/
│
├── src-cli/                      # CLI binary
│   ├── Cargo.toml
│   └── src/
│       └── main.rs               # clap subcommands → seatloom-core
│
└── ui/                           # React 前端
    ├── package.json
    ├── pnpm-lock.yaml
    ├── vite.config.ts
    ├── tsconfig.json
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx
        ├── layouts/
        │   ├── AppShell.tsx       # Sidebar + MainPanel + DetailPane
        │   ├── Sidebar.tsx
        │   ├── MainPanel.tsx
        │   ├── DetailPane.tsx
        │   └── TerminalPanel.tsx
        ├── views/
        │   ├── InboxView.tsx
        │   ├── TimelineView.tsx
        │   ├── WorkItemsView.tsx
        │   ├── PipelineView.tsx
        │   └── SessionView.tsx
        ├── components/
        │   ├── SeatList.tsx
        │   ├── SessionList.tsx
        │   ├── WorkItemList.tsx
        │   ├── EventRow.tsx
        │   ├── InboxItem.tsx
        │   ├── DetailCard.tsx
        │   ├── HandoffForm.tsx
        │   ├── WorkItemForm.tsx
        │   ├── PipelineProgress.tsx
        │   ├── MorningDigest.tsx
        │   └── LaunchPackPreview.tsx
        ├── stores/
        │   ├── projectStore.ts    # 当前项目状态
        │   ├── seatStore.ts
        │   ├── sessionStore.ts
        │   ├── workitemStore.ts
        │   ├── inboxStore.ts
        │   └── timelineStore.ts
        ├── services/
        │   └── tauri.ts           # typed IPC wrappers
        ├── types/
        │   └── index.ts           # TS 类型（镜像 Rust structs）
        ├── hooks/
        │   ├── useKeyboard.ts     # 快捷键
        │   └── useTauriEvent.ts   # Tauri 事件监听
        └── styles/
            ├── globals.css
            └── theme.ts
```

---

## 3. Rust 核心数据类型

### 3.1 ID 系统

所有对象使用类型安全的 ID wrapper，防止混用：

```rust
// crates/seatloom-core/src/objects/id.rs

macro_rules! define_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new() -> Self {
                Self(format!("{}-{}", $prefix, nanoid::nanoid!(8)))
            }
            pub fn as_str(&self) -> &str { &self.0 }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(SeatId, "seat");
define_id!(SessionId, "ses");
define_id!(WorkItemId, "wi");
define_id!(ArtifactId, "ar");
define_id!(HandoffId, "ho");
define_id!(PipelineId, "pl");
define_id!(PipelineRunId, "plrun");
define_id!(CheckpointId, "cp");
define_id!(DelegationId, "del");
define_id!(EventId, "ev");
```

### 3.2 顶层对象

```rust
// crates/seatloom-core/src/objects/seat.rs
// Three-layer model: AD-009. Global identity separated from per-project role binding.

/// Layer 1: Durable global seat identity. Stable across projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatIdentity {
    pub id: SeatId,
    pub name: String,
    pub default_runtime: Option<Runtime>,
    pub capability_tags: Vec<String>,
    pub status: SeatStatus,
    pub created_at: DateTime<Utc>,
}

/// Layer 2: Per-project role binding. Encodes role, authority docs, constraints,
/// and active delegation for one project. Stored separately from global identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRoleBind {
    pub seat_id: SeatId,
    pub project_id: String,
    pub role: SeatRole,
    pub authority_doc_refs: Vec<String>,
    pub constraints: Vec<String>,
    pub collaboration_template_ref: Option<String>,
    pub active_delegation_id: Option<DelegationId>,
}

/// Scoped delegation overlay (AD-009). Does not rewrite original seat identity.
/// Timeline shows: "{to_seat} acting for {from_seat} on {workitem_id}".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatDelegation {
    pub id: DelegationId,
    pub issuer_seat_id: SeatId,
    pub from_seat_id: SeatId,    // original owner seat
    pub to_seat_id: SeatId,      // acting seat
    pub workitem_id: Option<WorkItemId>,
    pub scope_description: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: DelegationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationStatus { Active, Closed, Expired }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeatRole {
    ProductOwner, Architect, Verifier, Designer, Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeatStatus { Active, Paused, Archived }
```

```rust
// crates/seatloom-core/src/objects/session.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub seat_id: SeatId,
    pub runtime: Runtime,
    pub native_session_id: Option<String>,
    pub workspace_path: PathBuf,
    pub branch: Option<String>,
    pub status: SessionStatus,
    pub launch_pack_ref: Option<String>,  // path relative to .seatloom/
    pub last_checkpoint_id: Option<CheckpointId>,
    pub pid: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Runtime {
    ClaudeCode,
    Codex,
    CursorCli,
    GeminiCli,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Launching, Running,
    // PromptBlocked: wrapped runtime is blocked on interactive stdin (AD-012).
    // Distinct from InputRequired (SeatLoom-level request); this is the runtime's own blocking.
    PromptBlocked,
    InputRequired,
    Suspended, Completed, Failed, Interrupted,
}

/// Represents the current interactive prompt state when SessionStatus == PromptBlocked.
/// Evidence window is bounded to last 10-20 terminal lines; never expands into full log (AD-012).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptState {
    pub kind: PromptKind,
    pub policy: PromptPolicy,
    pub bounded_window: Vec<String>,      // last 10-20 terminal lines
    pub available_actions: Vec<PromptAction>,
    pub assist_budget: Option<AssistBudget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptKind {
    Deterministic,  // y/n, numeric choice — safe for auto-inject
    WizardMenu,     // multi-step wizard or menu
    Freeform,       // arbitrary text — requires confirmation
    Sensitive,      // password / OTP / sudo / secret — SupervisorAssist prohibited
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptPolicy {
    AutoAllowed,    // SeatLoom may inject without asking (Deterministic + config permit)
    NeedsApproval,  // requires explicit user confirmation before inject
    HumanRequired,  // human must type directly; no automation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptAction {
    Approve,
    HumanTakeover,
    SupervisorAssist,  // disabled when kind == Sensitive
    Stop,
}

/// Hard caps on Supervisor assist loops (AD-012). Exceed either limit → force HumanTakeover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistBudget {
    pub max_steps: u32,   // default 5
    pub max_tokens: u32,  // default 2048
    pub steps_used: u32,
    pub tokens_used: u32,
}
```

```rust
// crates/seatloom-core/src/objects/workitem.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: WorkItemId,
    pub title: String,
    pub goal: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub owner_seat_id: Option<SeatId>,
    pub status: WorkItemStatus,
    pub priority: Priority,
    pub depends_on: Vec<WorkItemId>,
    pub parent_id: Option<WorkItemId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkItemStatus {
    Draft, Ready, Active, Blocked,
    InReview, Verified, Done, Reopened, Drifted,
    // Review failure uses event-first path (AD-010):
    // InReview → (ReviewVerdictIssued event) → Blocked → Ready → Active on reissue.
    // No durable Rejected/Rescoped states; verdict evidence lives in event payload.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority { Low, Medium, High }
```

```rust
// crates/seatloom-core/src/objects/artifact.rs
// Dual-key typing model: AD-008.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    // Coordination artifact dual-key (AD-008). Both fields extracted from markdown header.
    // None for system-generated artifacts (diffs, test reports, packs).
    pub template: Option<ArtifactTemplate>,
    pub subtype: Option<String>,       // validated against DOCUMENT_TEMPLATES §11.1
    pub subtype_valid: Option<bool>,   // None = not yet validated; false = degraded mode
    // System-generated artifacts that carry no coordination document structure.
    pub system_kind: Option<SystemArtifactKind>,
    pub title: String,
    pub summary: Option<String>,
    pub source_session_id: Option<SessionId>,
    pub source_workitem_id: Option<WorkItemId>,
    pub storage_path: String,  // relative to .seatloom/artifacts/
    pub created_at: DateTime<Utc>,
}

/// Template family matching DOCUMENT_TEMPLATES.md T1-T7 taxonomy (AD-008).
/// Drives Detail Pane layout, Route/Gate automation, and retrieval filters.
/// Missing/invalid template → generic markdown reader + visible warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactTemplate {
    T1AuthorityDoc, T2RoleProfile, T3TaskPacket,
    T4Review, T5Acceptance, T6DailyMemory, T7GovernanceDoc,
}

/// System-generated artifact kinds with no coordination document structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemArtifactKind {
    DiffSummary, TestReport, BugReport,
    CheckpointSummary,
    WorkerContinuityPack,      // assembled LaunchPack file
    SupervisorContinuityPack,  // P1: assembled SupervisorPack file
}
```

```rust
// crates/seatloom-core/src/objects/handoff.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handoff {
    pub id: HandoffId,
    pub from_ref: ActorRef,
    pub to_ref: ActorRef,
    pub workitem_id: WorkItemId,
    pub purpose: String,
    pub expected_outcome: String,
    pub artifact_ids: Vec<ArtifactId>,
    pub required_receipt: bool,
    pub status: HandoffStatus,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandoffStatus {
    Drafted, Sent, Received, Accepted,
    // Working is P1 (US-P1-06). Live activity overlay; does not replace Ledger replay truth.
    Working,
    Returned, Completed, Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorRef {
    Human,
    Seat(SeatId),
    Automation,
}
```

```rust
// crates/seatloom-core/src/objects/checkpoint.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub session_id: SessionId,
    pub trigger: CheckpointTrigger,
    pub summary: CheckpointSummary,
    pub artifact_ids_at_checkpoint: Vec<ArtifactId>,
    pub branch: Option<String>,
    pub last_commit: Option<String>,
    pub transcript_tail_ref: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointTrigger { SessionEnded, ArtifactProduced }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSummary {
    pub what_was_done: String,
    pub current_state: String,
    pub open_questions: Vec<String>,
    pub quality: SummaryQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SummaryQuality { Full, Minimal }
```

### 3.3 Ledger Event

```rust
// crates/seatloom-core/src/ledger/event.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEvent {
    pub event_id: EventId,
    pub event_type: EventType,
    pub occurred_at: DateTime<Utc>,
    pub actor_ref: ActorRef,
    pub object_refs: Vec<ObjectRef>,
    pub evidence_refs: Vec<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    // Session lifecycle
    SessionStarted, SessionCompleted, SessionFailed, SessionInterrupted,
    // Interactive prompt lifecycle (AD-012)
    PromptDetected,      // payload: kind, policy, bounded_window_ref, risk
    PromptInputInjected, // payload: operator (user/supervisor), scope, result, assist budget stats
    // Artifact lifecycle
    ArtifactCreated,
    // Handoff lifecycle
    HandoffDrafted, HandoffSent, HandoffAccepted, HandoffReturned, HandoffCompleted,
    HandoffWorking,       // P1 live activity state (US-P1-06)
    // WorkItem lifecycle
    WorkItemCreated, WorkItemStatusChanged,
    // Review/reissue evidence chain (AD-010, INT-05): event-first, no durable Rejected/Rescoped states
    ReviewVerdictIssued,  // payload: verdict, reason, linked_evidence_artifact_id
    WorkItemRescoped,     // payload: scope_change_summary, new_ac_refs
    // Seat delegation lifecycle (AD-009)
    SeatDelegationIssued, SeatDelegationClosed,
    // Pipeline lifecycle
    PipelineStarted, PipelineStageCompleted, PipelineCompleted, PipelineFailed,
    // Checkpoint
    CheckpointCreated,
    // Reconciliation
    ReconcileCompleted, DriftDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectRef {
    Seat(SeatId),
    Session(SessionId),
    WorkItem(WorkItemId),
    Artifact(ArtifactId),
    Handoff(HandoffId),
    Delegation(DelegationId),  // AD-009
    Pipeline(PipelineId),
    PipelineRun(PipelineRunId),
    Checkpoint(CheckpointId),
}
```

---

## 4. Adapter Trait

```rust
// crates/seatloom-core/src/adapter/traits.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapabilities {
    pub can_launch: bool,
    pub can_attach: bool,
    pub can_resume: bool,
    pub can_capture_transcript: bool,
    pub can_capture_tool_calls: bool,
    pub can_inject_input: bool,
}

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn runtime(&self) -> Runtime;
    fn capabilities(&self) -> AdapterCapabilities;

    async fn launch(
        &self,
        working_dir: &Path,
        initial_input: Option<&str>,
    ) -> Result<RunningSession>;

    async fn attach(&self, pid: u32) -> Result<RunningSession>;

    async fn inject_input(
        &self,
        session: &RunningSession,
        input: &str,
    ) -> Result<()>;

    fn parse_transcript(
        &self,
        raw_output: &[u8],
    ) -> Vec<TranscriptEntry>;

    fn detect_native_session_id(
        &self,
        working_dir: &Path,
    ) -> Option<String>;
}

pub struct RunningSession {
    pub pid: u32,
    pub stdin: Option<tokio::process::ChildStdin>,
    pub stdout_rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    pub stderr_rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}
```

---

## 5. Tauri IPC 接口

前端通过 `@tauri-apps/api` 的 `invoke()` 调用 Rust backend。所有数据以 JSON 序列化。

### 5.1 命令分组

| 分组 | 命令 | 方向 |
|------|------|------|
| **Project** | `open_project(path)` → `ProjectInfo` | F→B |
| | `init_project(path)` → `ProjectInfo` | F→B |
| **Seat** | `list_seats()` → `Vec<Seat>` | F→B |
| | `add_seat(name, role)` → `Seat` | F→B |
| **Session** | `list_sessions()` → `Vec<Session>` | F→B |
| | `attach_session(seat_id, runtime, pid)` → `Session` | F→B |
| | `launch_session(seat_id, runtime, cmd, workdir)` → `Session` | F→B |
| | `launch_with_rehydrate(seat_id, runtime, from_session_id)` → `Session` | F→B |
| | `resume_session(session_id)` → `Session` | F→B |
| **WorkItem** | `list_workitems()` → `Vec<WorkItem>` | F→B |
| | `create_workitem(title, goal?, ac?, owner?, priority?)` → `WorkItem` | F→B |
| | `update_workitem(id, fields)` → `WorkItem` | F→B |
| | `transition_workitem(id, new_status)` → `Result<WorkItem, GateError>` | F→B |
| **Handoff** | `list_handoffs()` → `Vec<Handoff>` | F→B |
| | `create_handoff(from, to, wi, purpose, outcome, artifacts)` → `Handoff` | F→B |
| | `send_handoff(id)` → `Handoff` | F→B |
| | `accept_handoff(id)` → `Handoff` | F→B |
| | `return_handoff(id, reason)` → `Handoff` | F→B |
| **Artifact** | `list_artifacts(template?, subtype?, workitem_id?)` → `Vec<Artifact>` | F→B |
| | `get_artifact(id)` → `Artifact` | F→B |
| | `validate_artifact_subtype(id)` → `SubtypeValidationResult` | F→B |
| **Delegation** | `create_delegation(from_seat, to_seat, workitem_id?, scope)` → `SeatDelegation` | F→B |
| | `close_delegation(id)` → `SeatDelegation` | F→B |
| | `list_delegations(seat_id?, active_only?)` → `Vec<SeatDelegation>` | F→B |
| **Prompt** | `get_prompt_state(session_id)` → `PromptState` | F→B |
| | `approve_prompt(session_id)` → `PromptResult` | F→B |
| | `takeover_prompt(session_id)` → `()` | F→B |
| | `supervisor_assist_prompt(session_id, budget)` → `PromptResult` | F→B |
| | `stop_on_prompt(session_id)` → `Session` | F→B |
| **Timeline** | `query_timeline(filters)` → `Vec<CanonicalEvent>` | F→B |
| **Inbox** | `get_inbox()` → `Vec<InboxItem>` | F→B |
| | `dismiss_inbox_item(id)` → `()` | F→B |
| **Pipeline** | `run_pipeline(pipeline_id, workitem_id)` → stream | F→B |
| **Reconcile** | `reconcile()` → `ReconcileResult` | F→B |
| **Show** | `show_object(object_ref)` → `ObjectDetail` | F→B |
| **PTY** | `pty_write(session_id, data)` | F→B |
| | `pty_resize(session_id, cols, rows)` | F→B |

### 5.2 后端 → 前端事件（Tauri Events）

| 事件 | Payload | 用途 |
|------|---------|------|
| `pty:output` | `{ session_id, data: bytes }` | 终端输出流 |
| `session:status_changed` | `{ session_id, old, new }` | Session 状态变更 |
| `ledger:new_event` | `CanonicalEvent` | 新事件写入 Ledger（前端追加 Timeline） |
| `inbox:updated` | `Vec<InboxItem>` | Inbox 内容变更 |
| `pipeline:progress` | `{ run_id, stage, status, output }` | Pipeline stage 进度 |
| `reconcile:completed` | `ReconcileResult` | Reconcile 完成 |

前端用 `listen()` 订阅事件，Zustand store 响应式更新 UI。

---

## 6. 存储层详细设计

### 6.1 .seatloom/ 目录结构

```
.seatloom/
├── config/
│   └── project.yaml              # 项目配置
├── ledger/
│   └── events.jsonl              # append-only 事件流
├── seats/
│   ├── lyra/
│   │   ├── identity.yaml            # Layer 1: global seat identity (AD-009)
│   │   └── role-bindings/
│   │       └── seatloom.yaml        # Layer 2: per-project role binding
│   ├── nimbus/
│   │   ├── identity.yaml
│   │   └── role-bindings/
│   │       └── seatloom.yaml
│   └── flux/
│       ├── identity.yaml
│       └── role-bindings/
│           └── seatloom.yaml
├── delegations/
│   └── del_xxxxxxxx.yaml            # SeatDelegation records (AD-009)
├── sessions/
│   └── ses_xxxxxxxx/
│       ├── meta.yaml             # Session 元数据
│       ├── raw/
│       │   ├── stdout.log
│       │   ├── stderr.log
│       │   └── transcript.jsonl  # 结构化对话（如果 adapter 支持）
│       ├── checkpoints/
│       │   └── cp_xxxxxxxx.yaml
│       └── context/
│           ├── launch_pack.md        # Worker continuity pack (P0)
│           └── supervisor_pack.md    # P1: Supervisor continuity pack
├── workitems/
│   └── wi_xxxxxxxx.yaml
├── artifacts/
│   └── ar_xxxxxxxx/
│       ├── meta.yaml
│       └── payload.md            # 或 .json/.patch 等
├── handoffs/
│   └── ho_xxxxxxxx.yaml
├── pipelines/
│   ├── definitions/
│   │   ├── requirement_handoff.yaml
│   │   └── verification_loop.yaml
│   └── runs/
│       └── plrun_xxxxxxxx/
│           ├── meta.yaml
│           └── stage_outputs/
└── lock                          # Pipeline 并发锁
```

### 6.2 project.yaml

```yaml
version: "0.1"
project_name: "seatloom"
created_at: "2026-04-27T15:00:00+08:00"

pack_engine:
  worker_budget_tokens: 8192
  supervisor_budget_tokens: 32768    # P1

pipeline:
  stage_timeout_seconds: 300
  total_timeout_seconds: 900
  max_retry: 2
```

### 6.3 Seat Files (Three-Layer Model, AD-009)

**Layer 1 — `seats/nimbus/identity.yaml`** (global, project-agnostic):
```yaml
id: "seat-a1b2c3d4"
name: "nimbus"
default_runtime: "copilot"
capability_tags: ["architecture", "rust", "typescript"]
status: "active"
created_at: "2026-04-27T15:00:00+08:00"
```

**Layer 2 — `seats/nimbus/role-bindings/seatloom.yaml`** (per-project):
```yaml
seat_id: "seat-a1b2c3d4"
project_id: "seatloom"
role: "architect"
authority_doc_refs:
  - "docs/architecture-design.md"
  - "docs/architecture-decisions.md"
constraints: []
collaboration_template_ref: null
active_delegation_id: null
```

### 6.4 Session meta.yaml

```yaml
id: "ses-x1y2z3w4"
seat_id: "seat-a1b2c3d4"
runtime: "codex"
native_session_id: "codex_abc123"
workspace_path: "/Users/zhang/projects/seatloom"
branch: "feature/oauth-login"
status: "completed"
launch_pack_ref: "sessions/ses-x1y2z3w4/context/launch_pack.md"
last_checkpoint_id: "cp-m1n2o3p4"
pid: 42831
created_at: "2026-04-27T15:12:00+08:00"
ended_at: "2026-04-27T16:42:00+08:00"
```

### 6.5 events.jsonl（每行一个 JSON）

```json
{"event_id":"ev-a1","event_type":"SessionStarted","occurred_at":"2026-04-27T15:12:00Z","actor_ref":{"Seat":"seat-a1b2c3d4"},"object_refs":[{"Session":"ses-x1y2z3w4"}],"evidence_refs":[],"payload":{"runtime":"codex","pid":42831}}
{"event_id":"ev-a2","event_type":"ArtifactCreated","occurred_at":"2026-04-27T16:40:00Z","actor_ref":{"Seat":"seat-a1b2c3d4"},"object_refs":[{"Artifact":"ar-d5e6f7g8"},{"Session":"ses-x1y2z3w4"}],"evidence_refs":["sessions/ses-x1y2z3w4/raw/stdout.log"],"payload":{"kind":"DiffSummary","title":"+OAuth callback handler"}}
```

### 6.6 读写策略

| 操作 | 策略 |
|------|------|
| Ledger 写入 | 文件追加写 (O_APPEND)，一次一行 JSON，flush after write |
| Ledger 读取 | 启动时全量扫描建内存索引；运行时增量追加 |
| 对象 YAML 写入 | 原子写（写临时文件 → rename），避免写入中断导致损坏 |
| 对象 YAML 读取 | 按需读取，前端请求时从文件加载 |
| Raw 日志写入 | 持续追加写，缓冲 4KB flush |

---

## 7. 前端状态管理

### 7.1 Zustand Store 划分

```typescript
// stores/projectStore.ts
interface ProjectState {
  projectPath: string | null;
  isInitialized: boolean;
  openProject: (path: string) => Promise<void>;
  initProject: (path: string) => Promise<void>;
}

// stores/seatStore.ts
interface SeatState {
  seats: Seat[];
  selectedSeatId: string | null;
  fetchSeats: () => Promise<void>;
  addSeat: (name: string, role: string) => Promise<void>;
  selectSeat: (id: string | null) => void;
}

// stores/sessionStore.ts
interface SessionState {
  sessions: Session[];
  selectedSessionId: string | null;
  fetchSessions: () => Promise<void>;
  attachSession: (seatId: string, runtime: string, pid: number) => Promise<void>;
  launchSession: (seatId: string, runtime: string, cmd: string) => Promise<void>;
}

// stores/workitemStore.ts
interface WorkItemState {
  workitems: WorkItem[];
  selectedWorkItemId: string | null;
  fetchWorkItems: () => Promise<void>;
  createWorkItem: (data: CreateWorkItemInput) => Promise<void>;
  transitionWorkItem: (id: string, status: string) => Promise<void>;
}

// stores/inboxStore.ts
interface InboxState {
  items: InboxItem[];
  fetchInbox: () => Promise<void>;
  acceptHandoff: (handoffId: string) => Promise<void>;
  returnHandoff: (handoffId: string, reason: string) => Promise<void>;
  dismissItem: (id: string) => Promise<void>;
}

// stores/timelineStore.ts
interface TimelineState {
  events: CanonicalEvent[];
  filters: TimelineFilters;
  fetchTimeline: () => Promise<void>;
  setFilters: (filters: Partial<TimelineFilters>) => void;
  appendEvent: (event: CanonicalEvent) => void;  // from Tauri event listener
}
```

### 7.2 数据流

```
User action (click/keyboard)
  → Zustand store action
    → invoke() Tauri IPC
      → Rust #[tauri::command]
        → seatloom-core 业务逻辑
          → .seatloom/ 文件读写
          → Ledger append
        ← 返回结果
      ← JSON response
    ← store 更新
  ← React re-render

Rust 后台事件 (session output, new ledger event)
  → Tauri emit event
    → frontend listen()
      → Zustand store update
        → React re-render
```

---

## 8. PTY 管理

```rust
// src-tauri/src/pty.rs

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

pub struct PtyManager {
    sessions: HashMap<SessionId, PtySession>,
}

pub struct PtySession {
    pub pair: PtyPair,
    pub child: Box<dyn Child + Send>,
    pub reader_handle: JoinHandle<()>,
}

impl PtyManager {
    /// wrap 模式：SeatLoom 启动 agent 进程
    pub fn launch(&mut self, session_id: SessionId, cmd: &str, args: &[&str],
                  working_dir: &Path, cols: u16, rows: u16) -> Result<()>;

    /// 向 session 写入数据（前端 xterm.js → backend → pty stdin）
    pub fn write(&self, session_id: &SessionId, data: &[u8]) -> Result<()>;

    /// 调整终端大小
    pub fn resize(&self, session_id: &SessionId, cols: u16, rows: u16) -> Result<()>;

    /// session 的 stdout 输出通过 Tauri event 推送到前端
    fn spawn_reader(&self, session_id: SessionId, reader: Box<dyn Read + Send>,
                    app_handle: AppHandle);
}
```

xterm.js（前端）与 Rust PTY 的数据流：

```
用户输入 → xterm.js onData → invoke('pty_write', {session_id, data})
                                → PtyManager.write() → pty stdin

pty stdout → PtyManager reader thread → emit('pty:output', {session_id, data})
                                          → xterm.js terminal.write(data)
```

同时，PTY 输出被 tee 到 `.seatloom/sessions/<id>/raw/stdout.log` 用于 capture。

---

## 9. Rust 依赖清单

```toml
# crates/seatloom-core/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
nanoid = "0.4"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
git2 = "0.19"                   # Git 操作
glob = "0.3"
thiserror = "2"
tracing = "0.1"

# src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
seatloom-core = { path = "../crates/seatloom-core" }
portable-pty = "0.8"            # PTY 管理
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# src-cli/Cargo.toml
[dependencies]
seatloom-core = { path = "../crates/seatloom-core" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

---

## 10. 前端依赖清单

```json
{
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "zustand": "^5.0.0",
    "@xterm/xterm": "^5.5.0",
    "@xterm/addon-fit": "^0.10.0",
    "@xterm/addon-webgl": "^0.18.0",
    "lucide-react": "^0.400.0",
    "date-fns": "^4.0.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "@vitejs/plugin-react": "^4.0.0",
    "@tauri-apps/cli": "^2.0.0"
  }
}
```

---

## 11. 实现顺序建议

```
Phase 0 (Week 1-2): Adapter 验证 + 项目骨架
├── 搭建 Cargo workspace + Tauri 项目 + React 项目
├── 实现 seatloom-core/storage/ (YAML/JSONL 读写)
├── 实现 seatloom-core/adapter/wrapper.rs (wrapper_capture)
├── 验证 Codex CLI + Claude Code 的 PTY capture 可行性
└── 产出: 能用 wrap 启动 agent 并捕获 stdout 的原型

Phase 1 (Week 3-4): 核心对象 + 观察面 + 应用 shell
├── 实现 seatloom-core/objects/ (全部 6 个顶层对象)
├── 实现 seatloom-core/ledger/ (event write + read + index)
├── 实现 Tauri IPC commands (seat/session/workitem/timeline/inbox)
├── Mira 交付: AppShell 布局 + Inbox/Timeline 组件
├── Nimbus 接入: 前后端联调
├── 实现 reconcile (启动时 + 手动)
└── 产出: 可运行的桌面应用，能看 Timeline/Inbox

Phase 2 (Week 5-6): Rehydrate + 中断恢复
├── 实现 seatloom-core/data_engine/pack_engine/ (Worker Continuity Pack, Tier 0/1/2)
├── 实现 checkpoint 自动创建 (session end + artifact)
├── 实现 session launch with rehydrate
├── 实现 session resume (三级降级)
├── 实现 native_attach (Codex + Claude Code)
└── 产出: 跨工具切换能力，LaunchPack 注入

Phase 3 (Week 7-8): Handoff + Pipeline
├── 实现 handoff create/send/accept/return/complete
├── 实现 handoff form (前端)
├── 实现 pipeline runner (foreground sync)
├── 实现 requirement_handoff + verification_loop
└── 产出: 完整协作链路

Phase 4 (Week 9-10): 验收 + 打磨
├── 四个生命体征场景测试
├── 决策回溯能力验证 (timeline --verbose + show)
├── 性能优化 (Ledger 索引、大量 session 下的 UI 响应)
├── 打包 + 分发测试 (macOS dmg, Linux AppImage)
└── 产出: Go/No-Go 决策
```

---

## 12. Data Engine Architecture

### 12.1 Module Structure

The `data_engine/` subsystem (replacing the prior `context/` module) is organized as:

```
seatloom-core/src/data_engine/
├── mod.rs
├── pack_engine/           # Worker + Supervisor continuity pack assembly (AD-004)
│   ├── mod.rs
│   ├── selector.rs        # Tier 0/1/2 + optional content selection
│   ├── budgeter.rs        # Per-tier token budget enforcement
│   ├── assembler.rs       # Markdown template assembly (LaunchPack, SupervisorPack)
│   └── verifier.rs        # Completeness checks, path validation, truncation warnings
├── retrieval.rs           # Fixed L1→L2→L3→LLM-explain retrieval order (AD-011)
├── gate_engine.rs         # WorkItem transition gates, field gates, budget gates
├── route_engine.rs        # Inbox projection, seat routing, priority band assignment
├── budget_enforcer.rs     # Hard token limits and graceful truncation
├── isolation.rs           # Need-to-know access enforcement per seat
└── audit.rs               # Binds all data engine outputs to Ledger events
```

### 12.2 Continuity Pack Tiers

Worker continuity packs are assembled from deterministic content tiers (AD-004):

| Tier | Content | Always Included |
|------|---------|----------------|
| Tier 0 | `SeatIdentity` + `ProjectRoleBind` + collaboration mode | Yes |
| Tier 1 | Current WorkItem + AC, current branch + last 3 commits, last Checkpoint summary | Yes |
| Tier 2 | Last Handoff purpose/outcome, recent failed tests (≤ 3) | Yes |
| Optional | Recent Artifact summaries (≤ 5), transcript tail (last 3 turns), file refs | Budget-gated |

**Worker pack default budget**: 8 192 tokens (configurable via `pack_engine.worker_budget_tokens` in project.yaml).

**Supervisor pack budget (P1)**: 32 768 tokens. Assembles `MEMORY.md` + all active WorkItems + unresolved blockers + pending Handoffs + last Gate decision (US-P1-04).

**Fallback order** when continuity source is unavailable:
1. Native session resume (tool-native, e.g. `codex --resume`)
2. Tier 0–2 worker pack from structured Checkpoint + Ledger
3. Minimal pack: WorkItem + AC + last Handoff only

### 12.3 Retrieval Layer Contract (AD-011)

Evidence lookup must always follow this fixed order. Each layer is entered only if the previous layer returns insufficient results:

```
L1: Structured index (exact field match) — SQLite FTS5 virtual table
    Fields: template, subtype, id, status, workitem_id, object_ref
    P0: Required for INT-13 compliance
        ↓
L2: Full-text search — SQLite FTS5 (Artifact body + Ledger payload strings)
    P0: Required for INT-13 compliance
        ↓ (P1 only below)
L3: Semantic retrieval
    Embedding similarity search (backend TBD at P1)
        ↓
L4: LLM explanation
    Grounded in L1–L3 evidence only; no hallucination permitted
```

**Storage architecture (BLOCKER-001 closed — Lyra decision 2026-04-28):**
- L1 and L2 persist to **SQLite FTS5** embedded in the project's `.seatloom/` directory.
- Any in-memory index is a warm cache rebuilt from SQLite on startup; it is not the authoritative persisted store.
- PostgreSQL / pgvector is the candidate upgrade path for P1 multi-user or cloud-backed deployments; it is not a P0 requirement.

### 12.4 Prompt Engine Architecture (AD-012)

When a wrapped session enters `SessionStatus::PromptBlocked`, the Data Engine activates the Prompt Engine sub-path:

```
wrapped runtime stdin blocked
        ↓
adapter detects stdin-required signal (explicit frame or PTY heuristic)
        ↓
PromptClassifier
  input: bounded_window (last 10-20 terminal lines) + Tier 0-2 structured context
  output: PromptKind + PromptPolicy
        ↓
Ledger: emit PromptDetected { kind, policy, bounded_window_ref, risk }
        ↓
Frontend: Session Panel shows "Prompt blocked" banner + classification/policy chips
        ↓
User chooses action:
  Approve          → inject deterministic response → PromptInputInjected { operator: user }
  HumanTakeover    → PTY handed to user directly  → PromptInputInjected { operator: user }
  SupervisorAssist → AssistLoop (bounded, non-Sensitive only):
                       step budget: max 5 steps
                       token budget: max 2048 tokens
                       confirmation card shown before any injection
                       if expected_next_prompt not matched → force HumanTakeover
                     → PromptInputInjected { operator: supervisor, assist_steps_used, assist_tokens_used }
  Stop             → session stop + PromptInputInjected { result: stopped }
```

**Sensitive prompt rule**: `PromptPolicy::HumanRequired` is always set for `PromptKind::Sensitive`; `SupervisorAssist` is disabled at the architecture level, not just the UI level.

---

*This document is an engineering support document for Nimbus. All implementation should align to `docs/PRODUCT_TRUTH.md` and the active contract set, not legacy PRDs or scenario docs.*
