// Typed DTOs mirroring the Rust structs in src-tauri/src/dto.rs.
// Field names are camelCase because serde uses #[serde(rename_all = "camelCase")].
//
// These are the ground-truth shapes returned by the Tauri backend. The existing
// ui/src/types/index.ts still defines the (mock-data) shapes used by v1
// components; v2 code should prefer these types for anything freshly wired.

export interface SeatDto {
  id: string;
  name: string;
  defaultRuntime: string | null;
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
  runtime: string;
  nativeSessionId: string | null;
  workspacePath: string;
  branch: string | null;
  status: string;
  launchPackRef: string | null;
  lastCheckpointId: string | null;
  pid: number | null;
  createdAt: string;
  endedAt: string | null;
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
