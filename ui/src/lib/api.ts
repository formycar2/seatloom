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
  HistoryContextSlice,
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
  OperatorDecisionCardDto,
  OperatorDecisionResolveResultDto,
  ProjectDto,
  ReconcileResultDto,
  ReconcileRunDto,
  RoleBindingDto,
  SeatDto,
  SessionDto,
  SessionExitEvent,
  SessionMirrorLostEvent,
  SessionOutputEvent,
  SessionTurnEvent,
  TmuxSessionInfo,
  WorkItemDto,
} from './types-dto';

// Re-exported so consumers keep importing event shapes from `api` — the same
// pattern `SessionOutputEvent` and `SessionExitEvent` already follow.
export type { SessionTurnEvent } from './types-dto';
export type { SessionMirrorLostEvent } from './types-dto';

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
  listSeats: (projectId: string): Promise<SeatDto[]> =>
    invoke('cmd_list_seats', { projectId }),
  // The GLOBAL seat registry, unfiltered. Used for exactly one purpose: naming
  // the seats that are bound in no project at all, so an empty seat list can say
  // why it is empty instead of rendering as broken. Everything else reads the
  // project-scoped `listSeats` above.
  listAllSeats: (): Promise<SeatDto[]> => invoke('cmd_list_all_seats'),
  getSeat: (seatId: string): Promise<SeatDto | null> => invoke('cmd_get_seat', { seatId }),
  listRoleBindings: (projectId?: string): Promise<RoleBindingDto[]> =>
    invoke('cmd_list_role_bindings', { projectId }),
  listDelegations: (): Promise<DelegationDto[]> => invoke('cmd_list_delegations'),
  getDelegation: (delegationId: string): Promise<DelegationDto | null> =>
    invoke('cmd_get_delegation', { delegationId }),

  // --- Workitems ---
  listWorkitems: (projectId: string): Promise<WorkItemDto[]> =>
    invoke('cmd_list_workitems', { projectId }),
  getWorkitem: (workitemId: string): Promise<WorkItemDto | null> =>
    invoke('cmd_get_workitem', { workitemId }),

  // --- Artifacts ---
  listArtifacts: (template?: string, subtype?: string): Promise<ArtifactDto[]> =>
    invoke('cmd_list_artifacts', { template, subtype }),
  getArtifact: (artifactId: string): Promise<ArtifactDto | null> =>
    invoke('cmd_get_artifact', { artifactId }),

  // --- Handoffs ---
  listHandoffs: (projectId: string): Promise<HandoffDto[]> =>
    invoke('cmd_list_handoffs', { projectId }),

  // --- Sessions (DB rows) ---
  listSessions: (projectId: string): Promise<SessionDto[]> =>
    invoke('cmd_list_sessions', { projectId }),
  listSessionsForSeat: (seatId: string): Promise<SessionDto[]> =>
    invoke('cmd_list_sessions_for_seat', { seatId }),
  listCheckpoints: (limit?: number): Promise<CheckpointDto[]> =>
    invoke('cmd_list_checkpoints', { limit }),

  // --- Documents ---
  // projectId is required: the command used to fall back to the app's default
  // project, so the Artifacts tab's document half was always seatloom's.
  listDocuments: (
    opts: { projectId: string; template?: string; subtype?: string },
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
  // projectId is required: this read had no project parameter at all and returned
  // every project's events, so the timeline was identical under every project.
  listEvents: (projectId: string, limit?: number): Promise<CanonicalEventDto[]> =>
    invoke('cmd_list_events', { projectId, limit }),
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
  // Required, not optional: the command used to fall back to the app's default
  // project when this was omitted, so an unscoped call returned another project's
  // dispatches and looked like data. The type now refuses to let a caller forget.
  listDispatches: (projectId: string, limit?: number): Promise<DispatchDto[]> =>
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

  // --- Historical session corpus (LYRA-2026-10-14 contract §8.2 slice 3) ---
  // The three reads are Nimbus's slice-2 tools over stdio; the two below them
  // dereference the turn pointers on this machine's disk. Every one returns the
  // tool's own text verbatim on rejection, so the UI shows the measured reason
  // instead of an empty list. `list_history_projects` takes no argument.
  listHistoryProjects: (): Promise<string> => invoke('cmd_list_history_projects'),
  listHistorySessions: (projectId: string): Promise<string> =>
    invoke('cmd_list_history_sessions', { projectId }),
  getHistorySessionTurns: (harness: string, id: string): Promise<string> =>
    invoke('cmd_get_history_session_turns', { harness, id }),
  readHistoryContextSlice: (
    contextPath: string,
    lineStart: number,
    lineEnd: number,
  ): Promise<HistoryContextSlice> =>
    invoke('cmd_read_history_context_slice', { contextPath, lineStart, lineEnd }),
  openLocalPath: (path: string): Promise<void> => invoke('cmd_open_local_path', { path }),

  // --- Open an http(s) URL in the operator's default browser ────────────────
  // WHY NOT window.open: xterm's WebLinksAddon, loaded without a handler, falls
  // back to window.open(), and Tauri's webview blocks that by design — so a URL
  // in the live-seats pane was recognised and clicked but never opened. The Rust
  // side shells out to the platform opener, the same mechanism openLocalPath
  // already uses. Only http(s) passes the gate there; other schemes come back as
  // an error naming the scheme, which is what callers surface.
  openExternalUrl: (url: string): Promise<void> =>
    invoke('cmd_open_external_url', { url }),

  // --- D2 acceptance loop (a delivered milestone, decided in the UI) ---
  // Nimbus's lane. The three commands below are NOT registered on `track` yet,
  // which is why `ACCEPTANCE_CMDS_REGISTERED` gates them: until it is true the
  // 待验收 section says so out loud instead of calling a command that would
  // reject every 5 seconds. Flip this one constant when his commands land.
  listAcceptances: (projectId: string): Promise<AcceptanceCardDto[]> =>
    invoke('cmd_list_acceptances', { projectId }),
  acceptDelivery: (acceptanceId: string): Promise<AcceptanceDecisionDto> =>
    invoke('cmd_accept_delivery', { acceptanceId }),
  rejectDelivery: (acceptanceId: string, feedback: string): Promise<AcceptanceDecisionDto> =>
    invoke('cmd_reject_delivery', { acceptanceId, feedback }),

  // --- Operator decisions (020): what only the operator can rule on ---
  // A DIFFERENT object from an acceptance, deliberately not merged into the
  // 待验收 list: an acceptance asks "do I take this delivered work?" and is
  // about a dispatch; an operator decision asks "which way do I rule on this?"
  // and may be about anything. A merged list would make the count wrong — the
  // exact defect 020 exists to fix.
  listOperatorDecisions: (
    projectId: string,
    status?: 'pending' | 'resolved',
  ): Promise<OperatorDecisionCardDto[]> =>
    invoke('cmd_list_operator_decisions', { projectId, status }),
  countPendingOperatorDecisions: (projectId: string): Promise<number> =>
    invoke('cmd_count_pending_operator_decisions', { projectId }),
  resolveOperatorDecision: (
    decisionId: string,
    chosen: string,
  ): Promise<OperatorDecisionResolveResultDto> =>
    invoke('cmd_resolve_operator_decision', { decisionId, chosen }),

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

/**
 * The backend could no longer prove this mirror still belongs to us.
 *
 * 2026-10-27. The operator reported: "the words I said to Aegis in the live seat
 * show up in Nimbus's live seat". Root cause was byte-level and is written up in
 * the AEGIS packet; the short version is that an agent ran `tmux pipe-pane` with
 * no argument — believing it was a query, when it is a STOP — and then pointed
 * Aegis's pane at the FIFO the app believed was Nimbus's. `#{pane_pipe}` stayed 1
 * the whole time, so nothing tmux reports distinguishes that from a healthy
 * mirror.
 *
 * What IS provable is which processes write our FIFO. The backend re-checks that
 * every 5 s and emits this the moment the answer stops being "exactly one, and it
 * is ours".
 *
 * # What a consumer MUST do
 *
 * Stop showing the session's content, and offer a reconnect. A console log is not
 * a fix: during the incident the bytes were wrong for three hours and the UI kept
 * rendering them without a word. `LiveSeatsView` does this by dropping the
 * attachment, which unmounts the terminal.
 */
export function onSessionMirrorLost(
  handler: (event: SessionMirrorLostEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionMirrorLostEvent>('session:mirror-lost', (e: Event<SessionMirrorLostEvent>) =>
    handler(e.payload),
  );
}

/**
 * The turn-aware view of the same byte stream. Subscribe to this to build a
 * conversation window; subscribe to `onSessionOutput` to drive xterm. They carry
 * the same bytes, and either can be used without the other.
 *
 * This is the surface the four known interaction pain points are meant to be
 * solved ON TOP OF (they need content a terminal cannot give):
 *   1. alternate screen drops scrollback  -> keep the accumulated text yourself
 *   2. links are blocked by window.open   -> extract URLs from the text
 *   3. "Successfully wrote" hides the path -> the text carries it, unmangled
 *   4. no search                          -> the text is searchable
 * None of those are solved here; the point is that none of them is BLOCKED here.
 */
export function onSessionTurn(
  handler: (event: SessionTurnEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionTurnEvent>('session:turn', (e: Event<SessionTurnEvent>) =>
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
