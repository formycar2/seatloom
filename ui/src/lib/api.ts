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
  AcceptanceLineageDto,
  ArtifactDto,
  KnowledgeTopicMemories,
  KnowledgeTopicTree,
  AcceptanceCardDto,
  AcceptanceDecisionDto,
  CanonicalEventDto,
  CheckpointDto,
  DelegationDto,
  DocumentAssociationDto,
  DocumentDto,
  DocumentSectionDto,
  DispatchDto,
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
  TmuxSessionInfo,
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

/**
 * Whether the D2 acceptance-loop commands are registered on the running build.
 *
 * They are Nimbus's lane and had not landed on `track` when this UI was written
 * (dispatch `disp-me6Nm8e9D6ZRe4fKyVokY`, D2). The UI is implemented against the
 * agreed card contract so it is ready the moment they do; until then the
 * 待验收 section renders an explicit "未接线" state naming exactly what is
 * missing, rather than calling a command that rejects every poll and showing
 * the operator a permanent error. House rule: no mock stands in for a real
 * backend value.
 *
 * Flip to `true` when `cmd_list_acceptances` / `cmd_accept_delivery` /
 * `cmd_reject_delivery` are on `track` — that is the whole follow-up.
 */
export const ACCEPTANCE_CMDS_REGISTERED = true;

export const api = {
  ping: (): Promise<string> => invoke('cmd_ping'),

  // --- Projects ---
  listProjects: (): Promise<ProjectDto[]> => invoke('cmd_list_projects'),
  getProject: (projectId: string): Promise<ProjectDto | null> =>
    invoke('cmd_get_project', { projectId }),

  // --- Seats / roles / delegations ---
  listSeats: (): Promise<SeatDto[]> => invoke('cmd_list_seats'),
  getSeat: (seatId: string): Promise<SeatDto | null> => invoke('cmd_get_seat', { seatId }),
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

  // --- Acceptance card lineage (acceptance-lineage, Lane B) ---
  // Pure read: assembles the card, its dispatch (with the original task
  // statement), the event chain, the Flux verdict, the products, the dependency
  // edges and the rework. Same assembler the MCP `get_acceptance_lineage` tool
  // calls, so the two surfaces cannot drift.
  getAcceptanceLineage: (acceptanceId: string): Promise<AcceptanceLineageDto> =>
    invoke('cmd_get_acceptance_lineage', { acceptanceId }),

  // --- Knowledge read model (overnight step 4) ---
  // Pure reads: the topic tree and the memories-by-topic card list, with
  // progressive disclosure (l0 default, l1/l2 on demand).
  getKnowledgeTree: (projectId?: string): Promise<KnowledgeTopicTree> =>
    invoke('cmd_get_knowledge_tree', { projectId }),
  getTopicMemories: (
    topicId: string,
    depth?: 'l0' | 'l1' | 'l2',
    includeSuperseded?: boolean,
    limit?: number,
  ): Promise<KnowledgeTopicMemories> =>
    invoke('cmd_get_topic_memories', { topicId, depth, includeSuperseded, limit }),

  // --- Governed dispatch (G19) — seat inboxes ---
  listDispatches: (projectId?: string, limit?: number): Promise<DispatchDto[]> =>
    invoke('cmd_list_dispatches', { projectId, limit }),
  enqueueDispatch: (
    targetSeatId: string,
    title: string,
    instructions: string,
    priority?: number,
  ): Promise<string> =>
    invoke('cmd_enqueue_dispatch', { targetSeatId, title, instructions, priority }),
  // Supervisor intent entry (contract slice 1) — plan_intent over the same stdio path
  // as cmd_enqueue_dispatch (actor human:supervisor). Reads only, appends nothing;
  // returns the tool's pretty-JSON text (parse as IntentPlanResult). A rejection is
  // thrown verbatim so the UI can show the reason rather than an empty card.
  planIntent: (intent: string): Promise<string> => invoke('cmd_plan_intent', { intent }),

  // --- D2 acceptance loop (a delivered milestone, decided in the UI) ---
  // Nimbus's lane. The three commands below are NOT registered on `track` yet,
  // which is why `ACCEPTANCE_CMDS_REGISTERED` gates them: until it is true the
  // 待验收 section says so out loud instead of calling a command that would
  // reject every 5 seconds. Flip this one constant when his commands land.
  listAcceptances: (): Promise<AcceptanceCardDto[]> => invoke('cmd_list_acceptances'),
  acceptDelivery: (acceptanceId: string): Promise<AcceptanceDecisionDto> =>
    invoke('cmd_accept_delivery', { acceptanceId }),
  rejectDelivery: (acceptanceId: string, feedback: string): Promise<AcceptanceDecisionDto> =>
    invoke('cmd_reject_delivery', { acceptanceId, feedback }),

  // --- Supervisor IM ---
  listSupervisorMessages: (
    opts: { targetSeatId?: string; limit?: number } = {},
  ): Promise<CanonicalEventDto[]> => invoke('cmd_list_supervisor_messages', opts),
  appendSupervisorMessage: (request: {
    targetSeatId?: string;
    content: string;
    eventType?: string;
    actorRef?: string;
  }): Promise<CanonicalEventDto> => invoke('cmd_append_supervisor_message', { request }),
  openSupervisorWindow: (): Promise<void> => invoke('cmd_open_supervisor_window'),
  closeSupervisorWindow: (): Promise<void> => invoke('cmd_close_supervisor_window'),
  supervisorWindowStatus: (): Promise<{ detached: boolean }> =>
    invoke('cmd_supervisor_window_status'),

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
  killSession: (sessionId: string): Promise<void> => invoke('cmd_kill_session', { sessionId }),
  listLiveSessions: (): Promise<string[]> => invoke('cmd_list_live_sessions'),

  // --- tmux-mirror (v0.0.1 A3) ---
  listTmuxSessions: (): Promise<TmuxSessionInfo[]> => invoke('cmd_list_tmux_sessions'),
  attachTmuxSession: (request: {
    seatId?: string;
    tmuxSessionName: string;
    sessionId?: string;
    rows?: number;
    cols?: number;
  }): Promise<LiveSessionDto> => invoke('cmd_attach_tmux_session', { request }),
};

// --- Event subscriptions -----------------------------------------------------

export function onSessionOutput(handler: (event: SessionOutputEvent) => void): Promise<UnlistenFn> {
  return listen<SessionOutputEvent>('session:output', (e: Event<SessionOutputEvent>) =>
    handler(e.payload),
  );
}

export function onSessionExit(handler: (event: SessionExitEvent) => void): Promise<UnlistenFn> {
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

export function onSupervisorDetached(handler: () => void): Promise<UnlistenFn> {
  return listen<void>('supervisor:detached', () => handler());
}

export function onSupervisorReembedded(handler: () => void): Promise<UnlistenFn> {
  return listen<void>('supervisor:reembedded', () => handler());
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
