// Maps backend DTOs (from ui/src/lib/types-dto.ts, produced by
// src-tauri/src/dto.rs) into the V1 frontend types (ui/src/types/index.ts).
//
// Backend sends snake_case lowercase strings for enums (status, runtime, role,
// priority). V1 frontend types use PascalCase union literals. We coerce here
// so V1 components render real data without rewrites.
//
// Unknown enum values fall back to a sensible default to avoid runtime
// TypeErrors — v0.1 is best-effort; later work can tighten enum parity.

import type {
  ArtifactDto,
  CanonicalEventDto,
  DelegationDto,
  DocumentDto,
  HandoffDto,
  ProjectDto,
  SeatDto,
  SessionDto,
  WorkItemDto,
} from './types-dto';

import type {
  ActorRef,
  Artifact,
  ArtifactSubtype,
  ArtifactTemplate,
  CanonicalEvent,
  DelegationRecord,
  EventType,
  Handoff,
  HandoffStatus,
  Priority,
  Project,
  Runtime,
  Seat,
  SeatRole,
  SeatStatus,
  Session,
  SessionStatus,
  WorkItem,
  WorkItemStatus,
} from '../types';

// ---------- Enum coercions ------------------------------------------------

const SEAT_ROLE_MAP: Record<string, SeatRole> = {
  product_owner: 'ProductOwner',
  architect: 'Architect',
  verifier: 'Verifier',
  designer: 'Designer',
  developer: 'Developer',
  supervisor: 'ProductOwner', // closest V1 analogue for Aegis; has no dedicated UI branch
};

const SEAT_STATUS_MAP: Record<string, SeatStatus> = {
  active: 'Active',
  paused: 'Paused',
  archived: 'Archived',
};

const RUNTIME_MAP: Record<string, Runtime> = {
  ClaudeCode: 'ClaudeCode',
  GeminiCli: 'GeminiCli',
  CursorCli: 'CursorCli',
  Codex: 'Codex',
  Custom: { Custom: 'Custom' },
};

const SESSION_STATUS_MAP: Record<string, SessionStatus> = {
  launching: 'Launching',
  running: 'Running',
  input_required: 'InputRequired',
  suspended: 'Suspended',
  completed: 'Completed',
  failed: 'Failed',
  interrupted: 'Interrupted',
};

const WORKITEM_STATUS_MAP: Record<string, WorkItemStatus> = {
  draft: 'Draft',
  ready: 'Ready',
  active: 'Active',
  blocked: 'Blocked',
  in_review: 'InReview',
  verified: 'Verified',
  done: 'Done',
  reopened: 'Reopened',
  drifted: 'Drifted',
};

const PRIORITY_MAP: Record<string, Priority> = {
  low: 'Low',
  medium: 'Medium',
  high: 'High',
  critical: 'Critical',
};

const HANDOFF_STATUS_MAP: Record<string, HandoffStatus> = {
  drafted: 'Drafted',
  sent: 'Sent',
  received: 'Received',
  accepted: 'Accepted',
  working: 'Working',
  returned: 'Returned',
  completed: 'Completed',
  expired: 'Expired',
};

function coerceRole(backend: string | null): SeatRole {
  if (!backend) return { Custom: 'Unknown' };
  return SEAT_ROLE_MAP[backend] ?? { Custom: backend };
}

function coerceSeatStatus(backend: string): SeatStatus {
  return SEAT_STATUS_MAP[backend] ?? 'Active';
}

function coerceRuntime(backend: string): Runtime {
  return RUNTIME_MAP[backend] ?? { Custom: backend };
}

function coerceSessionStatus(backend: string): SessionStatus {
  return SESSION_STATUS_MAP[backend] ?? 'Running';
}

function coerceWorkitemStatus(backend: string): WorkItemStatus {
  return WORKITEM_STATUS_MAP[backend] ?? 'Draft';
}

function coercePriority(backend: string): Priority {
  return PRIORITY_MAP[backend] ?? 'Medium';
}

function coerceHandoffStatus(backend: string): HandoffStatus {
  return HANDOFF_STATUS_MAP[backend] ?? 'Drafted';
}

function coerceActor(backend: string): ActorRef {
  if (backend.startsWith('seat-')) return { Seat: backend };
  if (backend === 'Human' || backend.startsWith('human')) return 'Human';
  if (backend === 'Automation' || backend.startsWith('automation')) return 'Automation';
  // Unknown / custom strings like 'aegis' — treat as Automation fallback.
  return 'Automation';
}

// ---------- Mappers -------------------------------------------------------

export function projectFromDto(d: ProjectDto): Project {
  return {
    id: d.id,
    name: d.name,
    path: '~/' + d.name, // backend has no path; synthesise a label
    isPinned: true,
    last_accessed: d.createdAt,
  };
}

export function seatFromDto(d: SeatDto, role?: string): Seat {
  return {
    id: d.id,
    name: d.name,
    role: coerceRole(role ?? null),
    status: coerceSeatStatus(d.status),
    created_at: d.createdAt,
    capabilities: d.capabilityTags,
    constraints: [],
  };
}

export function sessionFromDto(d: SessionDto): Session {
  return {
    id: d.id,
    seat_id: d.seatId,
    runtime: coerceRuntime(d.runtime),
    native_session_id: d.nativeSessionId ?? undefined,
    workspace_path: d.workspacePath,
    branch: d.branch ?? undefined,
    status: coerceSessionStatus(d.status),
    launch_pack_ref: d.launchPackRef ?? undefined,
    last_checkpoint_id: d.lastCheckpointId ?? undefined,
    pid: d.pid ?? undefined,
    created_at: d.createdAt,
    ended_at: d.endedAt ?? undefined,
  };
}

export function workitemFromDto(d: WorkItemDto): WorkItem {
  return {
    id: d.id,
    title: d.title,
    goal: d.goal ?? undefined,
    acceptance_criteria: d.acceptanceCriteria,
    owner_seat_id: d.ownerSeatId ?? undefined,
    status: coerceWorkitemStatus(d.status),
    priority: coercePriority(d.priority),
    depends_on: [],
    created_at: d.createdAt,
    updated_at: d.updatedAt,
  };
}

export function handoffFromDto(d: HandoffDto): Handoff {
  return {
    id: d.id,
    from_ref: coerceActor(d.fromRef),
    to_ref: coerceActor(d.toRef),
    workitem_id: d.workitemId,
    purpose: d.purpose,
    expected_outcome: d.expectedOutcome,
    artifact_ids: [],
    required_receipt: d.requiredReceipt,
    status: coerceHandoffStatus(d.status),
    created_at: d.createdAt,
    sent_at: d.sentAt ?? undefined,
  };
}

export function artifactFromDto(d: ArtifactDto): Artifact {
  return {
    id: d.id,
    title: d.title,
    template: (d.template ?? 'T3') as ArtifactTemplate,
    subtype: (d.subtype ?? 'task') as ArtifactSubtype,
    status: undefined,
    author: undefined,
    date: undefined,
    version: undefined,
    depends_on: [],
    tags: [],
    summary: d.summary ?? undefined,
    summary_points: d.summary ? [d.summary] : [],
    content_preview: [],
    source_session_id: d.sourceSessionId ?? undefined,
    source_workitem_id: d.sourceWorkitemId ?? undefined,
    storage_path: d.storagePath,
    created_at: d.createdAt,
  };
}

/**
 * Reconcile-ingested coordination markdown documents are not stored in the
 * `artifacts` table (which is reserved for runtime-produced session/workitem
 * outputs). To surface them in V1's ArtifactsView, we map each typed document
 * row into the V1 Artifact shape on the frontend. The body_text excerpt
 * provides preview rows; full body remains available via cmd_get_document.
 */
export function artifactFromDocument(d: DocumentDto): Artifact {
  const body = d.bodyText ?? '';
  const previewLines = body
    .split('\n')
    .filter((l) => l.trim().length > 0)
    .slice(0, 6);
  const summary = body
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !l.startsWith('#') && !l.startsWith('|') && !l.startsWith('---'))
    .slice(0, 1)
    .join(' ')
    .slice(0, 240);
  return {
    id: d.id,
    title: d.title,
    template: (d.template ?? 'T3') as ArtifactTemplate,
    subtype: (d.subtype ?? 'task') as ArtifactSubtype,
    status: d.status ?? undefined,
    author: d.author ?? undefined,
    date: d.docDate ?? undefined,
    version: d.version ?? undefined,
    depends_on: d.dependsOn,
    supersedes: d.supersedes ?? undefined,
    tags: d.tags,
    summary: summary || undefined,
    summary_points: summary ? [summary] : [],
    content_preview: previewLines,
    storage_path: d.filePath,
    created_at: d.createdAt,
  };
}

export function eventFromDto(d: CanonicalEventDto): CanonicalEvent {
  return {
    event_id: d.id,
    event_type: d.eventType as EventType,
    occurred_at: d.occurredAt,
    actor_ref: coerceActor(d.actorRef),
    object_refs: [],
    evidence_refs: [],
    payload: d.payload ?? undefined,
  };
}

// Delegations mostly used as records attached to workitems in V1. Provide a
// plain transform for future use.
export function delegationFromDto(d: DelegationDto): DelegationRecord {
  return {
    source_seat_id: d.fromSeatId,
    delegate_seat_id: d.toSeatId,
    scope: d.scopeDescription,
    issuer: coerceActor(d.issuerSeatId),
    expiry: d.expiresAt ?? '',
    created_at: d.issuedAt,
  };
}
