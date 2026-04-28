// ID Types
export type SeatId = string;
export type SessionId = string;
export type WorkItemId = string;
export type ArtifactId = string;
export type HandoffId = string;
export type PipelineId = string;
export type PipelineRunId = string;
export type CheckpointId = string;
export type EventId = string;
export type ProjectId = string;

// Enums
export type SeatRole = 'ProductOwner' | 'Architect' | 'Verifier' | 'Designer' | 'Developer' | { Custom: string };
export type SeatStatus = 'Active' | 'Paused' | 'Archived';

export type Runtime = 'ClaudeCode' | 'Codex' | 'CursorCli' | 'GeminiCli' | { Custom: string };
export type SessionStatus = 'Launching' | 'Running' | 'InputRequired' | 'Suspended' | 'Completed' | 'Failed' | 'Interrupted';

export type WorkItemStatus = 'Draft' | 'Ready' | 'Active' | 'Blocked' | 'InReview' | 'Verified' | 'Done' | 'Reopened' | 'Drifted';
export type Priority = 'Low' | 'Medium' | 'High' | 'Critical';

export type ArtifactKind = 
  | 'Brief' | 'AcceptanceCriteria' | 'DesignNote' 
  | 'DiffSummary' | 'TestReport' | 'BugReport' 
  | 'ReviewNote' | 'DecisionRecord' | 'ContextPack' 
  | 'CheckpointSummary';

export type HandoffStatus = 'Drafted' | 'Sent' | 'Received' | 'Accepted' | 'Returned' | 'Completed' | 'Expired';
export type ActorRef = 'Human' | { Seat: SeatId } | 'Automation';

export type CheckpointTrigger = 'SessionEnded' | 'ArtifactProduced';
export type SummaryQuality = 'Full' | 'Minimal';

export type EventType = 
  | 'SessionStarted' | 'SessionCompleted' | 'SessionFailed' | 'SessionInterrupted'
  | 'ArtifactCreated'
  | 'HandoffDrafted' | 'HandoffSent' | 'HandoffAccepted' | 'HandoffReturned' | 'HandoffCompleted'
  | 'WorkItemCreated' | 'WorkItemStatusChanged'
  | 'PipelineStarted' | 'PipelineStageCompleted' | 'PipelineCompleted' | 'PipelineFailed'
  | 'CheckpointCreated'
  | 'ReconcileCompleted' | 'DriftDetected';

export type ObjectRef = 
  | { Seat: SeatId }
  | { Session: SessionId }
  | { WorkItem: WorkItemId }
  | { Artifact: ArtifactId }
  | { Handoff: HandoffId }
  | { Pipeline: PipelineId }
  | { PipelineRun: PipelineRunId }
  | { Checkpoint: CheckpointId };

// Objects
export interface Seat {
  id: SeatId;
  name: string;
  role: SeatRole;
  status: SeatStatus;
  created_at: string;
}

export interface Session {
  id: SessionId;
  seat_id: SeatId;
  runtime: Runtime;
  native_session_id?: string;
  workspace_path: string;
  branch?: string;
  status: SessionStatus;
  launch_pack_ref?: string;
  last_checkpoint_id?: CheckpointId;
  pid?: number;
  created_at: string;
  ended_at?: string;
}

export interface WorkItem {
  id: WorkItemId;
  title: string;
  goal?: string;
  acceptance_criteria: string[];
  owner_seat_id?: SeatId;
  status: WorkItemStatus;
  priority: Priority;
  depends_on: WorkItemId[];
  parent_id?: WorkItemId;
  created_at: string;
  updated_at: string;
}

export interface Artifact {
  id: ArtifactId;
  kind: ArtifactKind;
  title: string;
  summary?: string;
  source_session_id?: SessionId;
  source_workitem_id?: WorkItemId;
  storage_path: string;
  created_at: string;
}

export interface Handoff {
  id: HandoffId;
  from_ref: ActorRef;
  to_ref: ActorRef;
  workitem_id: WorkItemId;
  purpose: string;
  expected_outcome: string;
  artifact_ids: ArtifactId[];
  required_receipt: boolean;
  status: HandoffStatus;
  created_at: string;
  sent_at?: string;
}

export interface CanonicalEvent {
  event_id: EventId;
  event_type: EventType;
  occurred_at: string;
  actor_ref: ActorRef;
  object_refs: ObjectRef[];
  evidence_refs: string[];
  payload?: any;
}

export interface InboxItem {
  id: string;
  priority: 'Critical' | 'Normal' | 'Low';
  type: string;
  actor: string;
  object_ref: string;
  summary: string;
  timestamp: string;
}

export interface Project {
  id: ProjectId;
  name: string;
  path: string;
  isPinned: boolean;
  last_accessed: string;
}

export interface ProjectData {
  seats: Seat[];
  sessions: Session[];
  workItems: WorkItem[];
  handoffs: Handoff[];
  events: CanonicalEvent[];
  inboxItems: InboxItem[];
}

export interface ProjectUIState {
  activeTab: string;
  selectedObjectId: string | null;
  filters: Record<string, any>;
}
