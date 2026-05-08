// Typed wrapper around Tauri IPC.
//
// Every v0.1 backend command has a single entry here. Frontend code should
// import from `@/lib/api` and never call `invoke()` directly — this keeps
// command names and argument shapes in one place and lets TypeScript catch
// drift at compile time.
//
// When running in a browser without Tauri (standalone `pnpm dev`), invoke()
// will throw. We surface this via `isTauri()` so views can fall back to mocks.

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn, Event } from '@tauri-apps/api/event';
import type {
  ArtifactDto,
  CanonicalEventDto,
  CheckpointDto,
  DelegationDto,
  DocumentAssociationDto,
  DocumentDto,
  DocumentSectionDto,
  HandoffDto,
  InboxPayloadDto,
  LaunchRequest,
  LiveSessionDto,
  ProjectDto,
  ReconcileResultDto,
  ReconcileRunDto,
  RoleBindingDto,
  SeatDto,
  SessionDto,
  SessionExitEvent,
  SessionOutputEvent,
  WorkItemDto,
} from './types-dto';

/**
 * True when running inside the Tauri webview; false in plain-browser `vite`.
 * Used by stores to fall back to mock data when the backend is unavailable.
 */
export function isTauri(): boolean {
  // Tauri 2 exposes __TAURI_INTERNALS__ on window. Checking for it is more
  // reliable than feature-detecting invoke(), which throws lazily.
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export const api = {
  ping: (): Promise<string> => invoke('cmd_ping'),

  // --- Projects ---
  listProjects: (): Promise<ProjectDto[]> => invoke('cmd_list_projects'),
  getProject: (projectId: string): Promise<ProjectDto | null> =>
    invoke('cmd_get_project', { projectId }),

  // --- Seats / roles / delegations ---
  listSeats: (): Promise<SeatDto[]> => invoke('cmd_list_seats'),
  getSeat: (seatId: string): Promise<SeatDto | null> =>
    invoke('cmd_get_seat', { seatId }),
  listRoleBindings: (projectId?: string): Promise<RoleBindingDto[]> =>
    invoke('cmd_list_role_bindings', { projectId }),
  listDelegations: (): Promise<DelegationDto[]> => invoke('cmd_list_delegations'),
  getDelegation: (delegationId: string): Promise<DelegationDto | null> =>
    invoke('cmd_get_delegation', { delegationId }),

  // --- Workitems ---
  listWorkitems: (): Promise<WorkItemDto[]> => invoke('cmd_list_workitems'),
  getWorkitem: (workitemId: string): Promise<WorkItemDto | null> =>
    invoke('cmd_get_workitem', { workitemId }),

  // --- Artifacts ---
  listArtifacts: (template?: string, subtype?: string): Promise<ArtifactDto[]> =>
    invoke('cmd_list_artifacts', { template, subtype }),
  getArtifact: (artifactId: string): Promise<ArtifactDto | null> =>
    invoke('cmd_get_artifact', { artifactId }),

  // --- Handoffs ---
  listHandoffs: (): Promise<HandoffDto[]> => invoke('cmd_list_handoffs'),

  // --- Sessions (DB rows) ---
  listSessions: (): Promise<SessionDto[]> => invoke('cmd_list_sessions'),
  listSessionsForSeat: (seatId: string): Promise<SessionDto[]> =>
    invoke('cmd_list_sessions_for_seat', { seatId }),
  listCheckpoints: (limit?: number): Promise<CheckpointDto[]> =>
    invoke('cmd_list_checkpoints', { limit }),

  // --- Documents ---
  listDocuments: (
    opts: { projectId?: string; template?: string; subtype?: string } = {},
  ): Promise<DocumentDto[]> => invoke('cmd_list_documents', opts),
  getDocument: (documentId: string): Promise<DocumentDto | null> =>
    invoke('cmd_get_document', { documentId }),
  listDocumentSections: (documentId: string): Promise<DocumentSectionDto[]> =>
    invoke('cmd_list_document_sections', { documentId }),
  listDocumentAssociations: (documentId: string): Promise<DocumentAssociationDto[]> =>
    invoke('cmd_list_document_associations', { documentId }),

  // --- Reconcile ---
  reconcile: (projectId?: string): Promise<ReconcileResultDto> =>
    invoke('cmd_reconcile', { projectId }),
  listReconcileRuns: (limit?: number): Promise<ReconcileRunDto[]> =>
    invoke('cmd_list_reconcile_runs', { limit }),

  // --- Events / timeline ---
  listEvents: (limit?: number): Promise<CanonicalEventDto[]> =>
    invoke('cmd_list_events', { limit }),
  listEventsByType: (eventType: string): Promise<CanonicalEventDto[]> =>
    invoke('cmd_list_events_by_type', { eventType }),

  // --- Supervisor IM ---
  listSupervisorMessages: (
    opts: { targetSeatId?: string; limit?: number } = {},
  ): Promise<CanonicalEventDto[]> => invoke('cmd_list_supervisor_messages', opts),
  appendSupervisorMessage: (
    request: {
      targetSeatId?: string;
      content: string;
      eventType?: string;
      actorRef?: string;
    },
  ): Promise<CanonicalEventDto> =>
    invoke('cmd_append_supervisor_message', { request }),

  // --- Inbox ---
  getInbox: (): Promise<InboxPayloadDto> => invoke('cmd_get_inbox'),

  // --- Live PTY sessions (Phase 4) ---
  launchSession: (request: LaunchRequest): Promise<LiveSessionDto> =>
    invoke('cmd_launch_session', { request }),
  ptyWrite: (sessionId: string, data: string): Promise<void> =>
    invoke('cmd_pty_write', { sessionId, data }),
  ptyWriteBytes: (sessionId: string, bytes: number[]): Promise<void> =>
    invoke('cmd_pty_write_bytes', { sessionId, bytes }),
  ptyResize: (sessionId: string, rows: number, cols: number): Promise<void> =>
    invoke('cmd_pty_resize', { sessionId, rows, cols }),
  killSession: (sessionId: string): Promise<void> =>
    invoke('cmd_kill_session', { sessionId }),
  listLiveSessions: (): Promise<string[]> => invoke('cmd_list_live_sessions'),
};

// --- Event subscriptions -----------------------------------------------------

export function onSessionOutput(
  handler: (event: SessionOutputEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionOutputEvent>('session:output', (e: Event<SessionOutputEvent>) =>
    handler(e.payload),
  );
}

export function onSessionExit(
  handler: (event: SessionExitEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionExitEvent>('session:exit', (e: Event<SessionExitEvent>) =>
    handler(e.payload),
  );
}

export function onCanonicalAppended(
  handler: (event: import('./types-dto').CanonicalEventDto) => void,
): Promise<UnlistenFn> {
  return listen<import('./types-dto').CanonicalEventDto>('canonical:appended', (e) =>
    handler(e.payload),
  );
}

/**
 * Decode a base64 session:output chunk into a Uint8Array. Terminal views pass
 * this directly to xterm's writeUtf8 / write.
 */
export function decodeSessionOutput(data: string): Uint8Array {
  // atob → binary string → Uint8Array (browser-safe).
  const binary = atob(data);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}
