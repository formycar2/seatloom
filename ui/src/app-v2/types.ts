/**
 * types.ts
 * 类型定义：v2 前端使用的所有数据类型，包括 ChatContact、ChatMessage、PlanPhase、TimelineEntry、ProjectChannelData
 */

import { StageWorkflow } from './dag-model';

export interface ChatContact {
  id: string;
  name: string;
  type: 'supervisor' | 'seat' | 'project-channel';
  role?: string;
  seatType?: 'po' | 'worker' | 'verifier'; // governance routing role
  online: boolean;
  projectId?: string;
  unread: number;
  lastMessage?: string;
  lastTime?: string;
  avatar: string; // single char
  color: string;  // CSS color var
}

export interface ChatMessage {
  id: string;
  contactId: string;
  from: 'user' | 'contact';
  content?: string;
  time: string;
  type: 'text' | 'delivery' | 'verification' | 'gate' | 'blocker' | 'progress' | 'alert' | 'info';
  card?: {
    title: string;
    status: string;
    statusColor?: string; // CSS color
    fields?: { label: string; value: string }[];
    actions?: string[];
    detail?: string; // full detail text for popup
  };
}

export interface PlanPhase {
  name: string;
  status: 'done' | 'active' | 'upcoming' | 'revisited';
  revisitReason?: string;
}

export interface TimelineEntry {
  text: string;
  time: string;
  type: 'action' | 'decision' | 'delivery';
}

export interface ProjectChannelData {
  // Project-level goals (decided with Supervisor at project start)
  goals: { name: string; status: 'done' | 'active' | 'upcoming'; currentGoalIndex: number }[];
  // Stages within the current goal
  currentGoal: {
    name: string;
    stages: PlanPhase[];
    currentStageIndex: number;
  };
  // Current stage workflow
  currentStage: {
    name: string;
    workItemsDone: number;
    workItemsTotal: number;
    workflow: StageWorkflow;
    blockers: { text: string; owner: string; since: string }[];
    nextStep: string;
  };
  // Time-collapsed history
  justNow: TimelineEntry[];     // last few events (expanded)
  earlierToday: string;         // one-line summary (collapsed)
  yesterday: string;            // one-line summary (collapsed)
}

// AD-013: Supervisor 两层上下文模型
export type SupervisorContextMode = 'global' | 'project';

// AD-013 §7 (v2): 右侧 pane 路由（与 contextMode 正交）
export type SupervisorViewMode = 'dashboard' | 'chat';

export interface GlobalProjectSummary {
  projectId: string;
  projectName: string;
  healthStatus: 'healthy' | 'warning' | 'blocked';
  activeBlockerCount: number;
  lastActivityTime: string; // ISO 8601 timestamp
}
