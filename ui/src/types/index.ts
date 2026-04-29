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

export type ArtifactTemplate = 'T1' | 'T2' | 'T3' | 'T4' | 'T5' | 'T6' | 'T7';

export type ArtifactSubtype =
  | 'prd'
  | 'ux_spec'
  | 'interaction_spec'
  | 'acceptance_spec'
  | 'architecture_design'
  | 'architecture_decisions'
  | 'seat_role'
  | 'task'
  | 'fix'
  | 'integration'
  | 'verification'
  | 'gap_review'
  | 'benchmark'
  | 'process_mapping'
  | 'design_proposal'
  | 'acceptance_review'
  | 'gate_decision'
  | 'daily_log'
  | 'coordination_rules'
  | 'workflow_principles'
  | 'collaboration_protocol'
  | 'document_templates';

export type HandoffStatus = 'Drafted' | 'Sent' | 'Received' | 'Accepted' | 'Working' | 'Returned' | 'Completed' | 'Expired';
export type ActorRef = 'Human' | { Seat: SeatId } | 'Automation';

export type ReviewTier = 'L1' | 'L2' | 'L3';
export type ReviewAckMode = 'DirectPatch' | 'CompactAck' | 'FullGate';

export interface ReviewChangeRecord {
  tier: ReviewTier;
  reason: string;
  changed_clauses: string[];
  impact_level: 'Low' | 'Medium' | 'High';
  executor: ActorRef;
  reviewer: ActorRef;
  ack_mode: ReviewAckMode;
  evidence_refs: string[];
}

export interface DelegationRecord {
  source_seat_id: SeatId;
  delegate_seat_id: SeatId;
  scope: string;
  issuer: ActorRef;
  expiry: string;
  authority_limit?: string;
  created_at: string;
}

export type PromptClassification = 'deterministic' | 'wizard/menu' | 'freeform' | 'sensitive';
export type PromptPolicy = 'auto_allowed' | 'needs_approval' | 'human_required';
export type PromptAction = 'Approve' | 'HumanTakeover' | 'SupervisorAssist' | 'Stop';

export interface PromptState {
  classification: PromptClassification;
  policy: PromptPolicy;
  preview: string; // Bounded terminal window
  step_count?: number;
  token_budget?: number;
  expected_next?: string;
}

export interface ContinuityPreview {
  tier_0_identity: {
    seat_id: SeatId;
    runtime: Runtime;
    workItem_id?: WorkItemId;
    branch?: string;
  };
  tier_1_state: {
    ac_progress: string[];
    latest_commit?: string;
    current_blocker?: string;
  };
  tier_2_decisions: {
    summary: string;
    evidence_refs: string[];
  };
  seat_skills?: string[];
  playbook_matches?: string[];
  budget_estimate: number;
  fallback_path: string;
}

export type CheckpointTrigger = 'SessionEnded' | 'ArtifactProduced';
export type SummaryQuality = 'Full' | 'Minimal';

export type EventType =
  | 'SessionStarted'
  | 'SessionCompleted'
  | 'SessionFailed'
  | 'SessionInterrupted'
  | 'ArtifactCreated'
  | 'HandoffDrafted'
  | 'HandoffSent'
  | 'HandoffAccepted'
  | 'HandoffReturned'
  | 'HandoffCompleted'
  | 'WorkItemCreated'
  | 'WorkItemStatusChanged'
  | 'PipelineStarted'
  | 'PipelineStageCompleted'
  | 'PipelineCompleted'
  | 'PipelineFailed'
  | 'CheckpointCreated'
  | 'ReconcileCompleted'
  | 'DriftDetected'
  | 'WorkItemDelegated';

export type SelectedObjectType = 'Seat' | 'Session' | 'WorkItem' | 'Artifact' | 'Handoff';

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
  capabilities?: string[];
  accepted_input_types?: string[];
  output_types?: string[];
  input_budget?: number;
  output_budget?: number;
  constraints?: string[];
  attached_skills?: string[];
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
  prompt_state?: PromptState;
  continuity_pack?: ContinuityPreview;
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
  change_tier_record?: ReviewChangeRecord;
  active_delegation?: DelegationRecord;
}

export interface Artifact {
  id: ArtifactId;
  title: string;
  template: ArtifactTemplate;
  subtype: ArtifactSubtype;
  status?: string;
  author?: string;
  date?: string;
  version?: string;
  depends_on?: string[];
  supersedes?: string;
  tags?: string[];
  summary?: string;
  summary_points?: string[];
  content_preview?: string[];
  source_session_id?: SessionId;
  source_workitem_id?: WorkItemId;
  source_handoff_id?: HandoffId;
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
  linked_artifact_ids?: ArtifactId[];
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
  artifacts: Artifact[];
  handoffs: Handoff[];
  events: CanonicalEvent[];
  inboxItems: InboxItem[];
}

export interface ProjectUIState {
  activeTab: string;
  selectedObjectId: string | null;
  selectedObjectType?: SelectedObjectType | null;
  filters: Record<string, any>;
}
