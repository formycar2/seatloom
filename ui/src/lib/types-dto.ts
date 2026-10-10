// Typed DTOs mirroring the Rust structs in src-tauri/src/dto.rs.
// Field names are camelCase because serde uses #[serde(rename_all = "camelCase")].
//
// These are the ground-truth shapes returned by the Tauri backend. The existing
// ui/src/types/index.ts still defines the (mock-data) shapes used by v1
// components; v2 code should prefer these types for anything freshly wired.

export interface ProjectDto {
  id: string;
  name: string;
  createdAt: string;
  workerBudgetTokens: number;
  supervisorBudgetTokens: number;
}

export interface SeatDto {
  id: string;
  name: string;
  // Runtime triple (schema 009). `null` means not recorded, not "default".
  defaultHarness: string | null;
  defaultAgent: string | null;
  defaultModel: string | null;
  capabilityTags: string[];
  status: string;
  createdAt: string;
}

export interface RoleBindingDto {
  seatId: string;
  projectId: string;
  role: string;
  authorityDocRefs: string[];
  constraints: string[];
  collaborationTemplateRef: string | null;
  activeDelegationId: string | null;
}

export interface DelegationDto {
  id: string;
  issuerSeatId: string;
  fromSeatId: string;
  toSeatId: string;
  workitemId: string | null;
  scopeDescription: string;
  issuedAt: string;
  expiresAt: string | null;
  status: string;
}

export interface SessionDto {
  id: string;
  seatId: string;
  // Runtime triple (schema 009). `harness`/`model` null means not recorded.
  harness: string | null;
  agent: string;
  model: string | null;
  nativeSessionId: string | null;
  workspacePath: string;
  branch: string | null;
  status: string;
  launchPackRef: string | null;
  lastCheckpointId: string | null;
  pid: number | null;
  createdAt: string;
  endedAt: string | null;
  projectId: string;
}

export interface WorkItemDto {
  id: string;
  title: string;
  goal: string | null;
  acceptanceCriteria: string[];
  ownerSeatId: string | null;
  status: string;
  priority: string;
  createdAt: string;
  updatedAt: string;
  projectId: string;
}

export interface HandoffDto {
  id: string;
  fromRef: string;
  toRef: string;
  workitemId: string;
  purpose: string;
  expectedOutcome: string;
  requiredReceipt: boolean;
  status: string;
  createdAt: string;
  sentAt: string | null;
  projectId: string;
}

export interface ArtifactDto {
  id: string;
  template: string | null;
  subtype: string | null;
  subtypeValid: boolean | null;
  systemKind: string | null;
  title: string;
  summary: string | null;
  sourceSessionId: string | null;
  sourceWorkitemId: string | null;
  storagePath: string;
  createdAt: string;
}

export interface CanonicalEventDto {
  id: string;
  eventType: string;
  occurredAt: string;
  actorRef: string;
  payload: unknown | null;
  createdAt: string;
  projectId: string;
}

export interface DispatchDependencyDto {
  dispatchId: string;
  depKind: string;
  depValue: string;
}

export interface DispatchDto {
  id: string;
  targetSeatId: string;
  issuedBy: string;
  projectId: string;
  title: string;
  payload: unknown | null;
  status: string; // 'ready' | 'blocked' | 'in_flight' | 'done'
  priority: number;
  issuedAt: string;
  startedAt: string | null;
  completedAt: string | null;
  deps: DispatchDependencyDto[];
}

// ── D2 acceptance loop ──────────────────────────────────────────────────────
// The half of the D2 milestone that closes a delivery inside the product: a
// dispatched, delivered, Flux-verified branch becomes an acceptance card the
// user decides on — 接受 merges it, 打回 re-dispatches the seat.
//
// Nimbus owns the Rust side (`cmd_list_acceptances` / `cmd_accept_delivery` /
// `cmd_reject_delivery` and the backing schema); this is the TypeScript mirror
// of the agreed card shape. Field names follow the house convention — camelCase,
// because the backend serialises with `rename_all = "camelCase"` (see the header
// of this file). `ui/src/lib/acceptance.ts` normalises the other spelling too,
// because the lane that commissioned this UI spelled the same fields snake_case
// and the contract cannot be read off Nimbus's code until his commands land.

/** Flux's technical verdict, carried on the card it was made about. */
export interface FluxVerdictDto {
  refId: string;
  verdict: string; // 'pass' | 'fail' | 'error' | …
  occurredAt: string;
}

export interface AcceptanceCardDto {
  id: string;
  dispatchId: string;
  dispatchTitle: string;
  seatId: string;
  branch: string;
  base: string;
  /**
   * The single "do A, see B" sentence the acceptance is decided on. Rendered
   * prominently: it is the acceptance criterion, not a description.
   */
  howToVerify: string;
  fluxVerdict: FluxVerdictDto;
  /** 'awaiting_acceptance' | 'accepted' | 'rejected' | … */
  status: string;
  createdAt: string;
  /** Set by cmd_accept_delivery — the merge that closed the loop. */
  mergeCommit?: string | null;
  /** Set by cmd_reject_delivery — the operator's one-line reason. */
  feedback?: string | null;
  /** Set by cmd_reject_delivery — the rework dispatch the rejection created. */
  reworkDispatchId?: string | null;
}

/**
 * What cmd_accept_delivery / cmd_reject_delivery hand back: the card as it
 * stands after the decision, so the UI can show the closed-loop result
 * (`mergeCommit`) or the rework it spawned (`reworkDispatchId`) without a
 * second round-trip.
 *
 * A merge that cannot be applied (conflict) is a REJECTED promise carrying the
 * verbatim git output — never a card with the conflict swallowed.
 */
export type AcceptanceDecisionDto = AcceptanceCardDto;

// ---------------------------------------------------------------------------
// Acceptance card lineage (acceptance-lineage milestone, Lane B)
// ---------------------------------------------------------------------------
// Mirrors src-tauri/src/dto.rs's AcceptanceLineageDto ONE field at a time: the
// seven blocks of the value-domain contract §3, in the operator's reading
// order. The Rust DTO is the authority; this file is the wire shape the UI
// consumes (Mira's card UI builds on it).

/** The structured acceptance plan (016, shape v1). */
export interface AcceptancePlanV1Dto {
  planVersion: number;
  scenario: string;
  steps: { action: string; input?: string }[];
  expected: { output: string; expect: string }[];
  preconditions?: string[];
}

export interface AcceptanceLineageCardDto {
  id: string;
  dispatchId: string;
  projectId: string;
  branch: string;
  base: string;
  howToVerify: string;
  /** null = the legacy row has no plan yet (Aegis backfill pending). */
  acceptancePlan: AcceptancePlanV1Dto | null;
  fluxVerdictRef: string;
  /** 'pending' | 'accepted' | 'rejected' | 'cancelled'. */
  status: string;
  feedback: string | null;
  reworkDispatchId: string | null;
  mergeCommit: string | null;
  decidedBy: string | null;
  decidedAt: string | null;
  createdAt: string;
}

export interface AcceptanceLineageDispatchDto {
  id: string;
  title: string;
  issuedBy: string;
  targetSeatId: string;
  priority: number;
  status: string;
  issuedAt: string;
  startedAt: string | null;
  completedAt: string | null;
  /** The ORIGINAL task statement (packet path / instructions), verbatim. */
  payload: unknown | null;
}

export interface AcceptanceLineageEventDto {
  id: string;
  eventType: string;
  occurredAt: string;
  actorRef: string;
  payload: unknown | null;
}

export interface AcceptanceLineageVerdictDto {
  refId: string;
  verdict: string | null;
  evidence: string | null;
  evidenceDetail: string | null;
  targetBranch: string | null;
  lane: string | null;
  occurredAt: string | null;
}

export interface AcceptanceLineageProductDto {
  refType: string; // 'artifact' | 'document'
  refId: string;
  /** null when the referent could not be joined (the id is still carried). */
  title: string | null;
  path: string | null;
  /** false = the referent is not in the database. */
  resolved: boolean;
}

export interface AcceptanceLineageDependencyDto {
  depKind: string; // 'event' | 'artifact' | 'dispatch'
  depValue: string;
  /** The referent's live existence (012: readiness is an existence query). */
  exists: boolean;
}

export interface AcceptanceLineageDto {
  acceptance: AcceptanceLineageCardDto;
  dispatch: AcceptanceLineageDispatchDto;
  events: AcceptanceLineageEventDto[];
  fluxVerdict: AcceptanceLineageVerdictDto | null;
  products: AcceptanceLineageProductDto[];
  dependencies: AcceptanceLineageDependencyDto[];
  rework: AcceptanceLineageDispatchDto | null;
}

export interface DocumentDto {
  id: string;
  projectId: string;
  artifactId: string | null;
  template: string | null;
  subtype: string | null;
  subtypeValid: boolean | null;
  docId: string | null;
  title: string;
  status: string | null;
  author: string | null;
  docDate: string | null;
  version: string | null;
  dependsOn: string[];
  supersedes: string | null;
  tags: string[];
  filePath: string;
  bodyText: string | null;
  bodyDigest: string | null;
  bodyLength: number | null;
  parseStatus: string;
  revision: number;
  createdAt: string;
  updatedAt: string;
}

export interface DocumentSectionDto {
  id: string;
  documentId: string;
  ordinal: number;
  headingText: string;
  headingLevel: number;
  anchorSlug: string;
  bodyExcerpt: string | null;
  createdAt: string;
}

export interface DocumentAssociationDto {
  id: string;
  documentId: string;
  assocType: string;
  assocId: string;
  isPrimary: boolean;
  createdAt: string;
}

export interface CheckpointDto {
  id: string;
  sessionId: string;
  trigger: string;
  summaryWhatWasDone: string;
  summaryCurrentState: string;
  summaryOpenQuestions: string[];
  summaryQuality: string;
  artifactIdsAtCheckpoint: string[];
  branch: string | null;
  lastCommit: string | null;
  continuityBudgetTokens: number | null;
  createdAt: string;
}

export interface ReconcileResultDto {
  runId: string;
  trigger: string;
  scanned: number;
  inserted: number;
  updated: number;
  unchanged: number;
  failed: number;
  conflicted: number;
}

export interface ReconcileRunDto {
  id: string;
  trigger: string;
  status: string;
  scanned: number;
  inserted: number;
  updated: number;
  unchanged: number;
  failed: number;
  conflicted: number;
  startedAt: string;
  completedAt: string | null;
}

export interface InboxPayloadDto {
  blockedWorkitems: WorkItemDto[];
  inReviewWorkitems: WorkItemDto[];
  pendingHandoffs: HandoffDto[];
}

// --- Live PTY session types (Phase 4) -----------------------------------------

/**
 * Discovered tmux session info (from cmd_list_tmux_sessions).
 * Mirrors Rust `crates/seatloom-core/src/pty/mod.rs:114 TmuxSessionInfo`.
 * No serde rename_all — fields are snake_case on the wire.
 */
export interface TmuxSessionInfo {
  session_name: string;
  created_at: number;
  attached: boolean;
}

export interface LaunchRequest {
  seatId?: string;
  runtime: string;
  command: string;
  args: string[];
  workingDir?: string;
  rows?: number;
  cols?: number;
}

export interface LiveSessionDto {
  id: string;
  seatId: string | null;
  runtime: string;
  command: string;
  args: string[];
  workingDir: string;
  transcriptPath: string;
  tmuxSessionName?: string;
  fifoPath?: string;
  paneRows?: number;
  paneCols?: number;
  /// Base64-encoded snapshot of the tmux pane content at attach time. Written
  /// to xterm before subscribing to session:output events so idle panes don't
  /// render as a black screen.
  initialSnapshotB64?: string;
}

export interface SessionOutputEvent {
  sessionId: string;
  data: string; // base64-encoded bytes
}

export interface SessionExitEvent {
  sessionId: string;
  exitCode: number | null;
  signal: string | null;
}
