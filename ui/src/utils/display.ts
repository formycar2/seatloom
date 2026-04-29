import {
  ActorRef,
  HandoffStatus,
  Priority,
  Runtime,
  SeatRole,
  SeatStatus,
  SessionStatus,
  WorkItemStatus,
} from '../types';

const pad = (value: number) => value.toString().padStart(2, '0');

const seatRoleLabels: Record<string, string> = {
  ProductOwner: '产品负责人',
  Architect: '架构负责人',
  Verifier: '验证负责人',
  Designer: '体验设计',
  Developer: '工程实现',
};

const seatStatusLabels: Record<SeatStatus, string> = {
  Active: '活跃',
  Paused: '暂停',
  Archived: '归档',
};

const runtimeLabels: Record<string, string> = {
  ClaudeCode: 'Claude Code',
  Codex: 'Codex',
  CursorCli: 'Cursor CLI',
  GeminiCli: 'Gemini CLI',
};

const sessionStatusLabels: Record<SessionStatus, string> = {
  Launching: '启动中',
  Running: '运行中',
  InputRequired: '等待输入',
  Suspended: '挂起',
  Completed: '已完成',
  Failed: '失败',
  Interrupted: '中断',
};

const workItemStatusLabels: Record<WorkItemStatus, string> = {
  Draft: '草稿',
  Ready: '就绪',
  Active: '进行中',
  Blocked: '阻塞',
  InReview: '评审中',
  Verified: '已验证',
  Done: '已完成',
  Reopened: '重新打开',
  Drifted: '已漂移',
};

const handoffStatusLabels: Record<HandoffStatus, string> = {
  Drafted: '已起草',
  Sent: '已发送',
  Received: '已接收',
  Accepted: '已接单',
  Working: '处理中',
  Returned: '已退回',
  Completed: '已完成',
  Expired: '已过期',
};

const priorityLabels: Record<Priority, string> = {
  Low: '低',
  Medium: '中',
  High: '高',
  Critical: '紧急',
};

const inboxPriorityLabels: Record<string, string> = {
  Critical: '紧急',
  Normal: '常规',
  Low: '低优先级',
};

const eventTypeLabels: Record<string, string> = {
  SessionStarted: '会话启动',
  SessionCompleted: '会话完成',
  SessionFailed: '会话失败',
  SessionInterrupted: '会话中断',
  ArtifactCreated: '产物写入',
  HandoffDrafted: '交接起草',
  HandoffSent: '交接发送',
  HandoffAccepted: '交接接收',
  HandoffReturned: '交接退回',
  HandoffCompleted: '交接完成',
  WorkItemCreated: '工作项创建',
  WorkItemStatusChanged: '工作项状态变更',
  PipelineStarted: '流水线启动',
  PipelineStageCompleted: '流水线阶段完成',
  PipelineCompleted: '流水线完成',
  PipelineFailed: '流水线失败',
  CheckpointCreated: '检查点生成',
  ReconcileCompleted: '对账完成',
  DriftDetected: '检测到漂移',
  WorkItemDelegated: '工作项委派',
};

const eventCategoryLabels: Record<string, string> = {
  session: '会话',
  handoff: '交接',
  artifact: '产物',
  pipeline: '流水线',
};

const timeRangeLabels: Record<string, string> = {
  '1h': '近 1 小时',
  '6h': '近 6 小时',
  '24h': '近 24 小时',
  '7d': '近 7 天',
};

const objectTypeLabels: Record<string, string> = {
  Seat: '席位',
  Session: '会话',
  WorkItem: '工作项',
  Artifact: '产物',
  Handoff: '交接单',
  Status: '状态',
  Create: '新建',
};

export const getSeatRoleLabel = (role: SeatRole) =>
  typeof role === 'string' ? seatRoleLabels[role] ?? role : role.Custom;

export const getSeatStatusLabel = (status: SeatStatus) => seatStatusLabels[status] ?? status;

export const getRuntimeLabel = (runtime: Runtime) =>
  typeof runtime === 'string' ? runtimeLabels[runtime] ?? runtime : runtime.Custom;

export const getSessionStatusLabel = (status: SessionStatus) => sessionStatusLabels[status] ?? status;

export const getWorkItemStatusLabel = (status: WorkItemStatus) => workItemStatusLabels[status] ?? status;

export const getHandoffStatusLabel = (status: HandoffStatus) => handoffStatusLabels[status] ?? status;

export const getPriorityLabel = (priority: Priority) => priorityLabels[priority] ?? priority;

export const getInboxPriorityLabel = (priority: string) => inboxPriorityLabels[priority] ?? priority;

export const getEventTypeLabel = (eventType: string) => eventTypeLabels[eventType] ?? eventType;

export const getEventCategoryLabel = (category: string | null) => {
  if (!category) return '全部';
  return eventCategoryLabels[category] ?? category;
};

export const getTimeRangeLabel = (range: string) => timeRangeLabels[range] ?? range;

export const getObjectTypeLabel = (type: string) => objectTypeLabels[type] ?? type;

export const getActorLabel = (actor: ActorRef, seatName?: string) => {
  if (actor === 'Automation') return '自动同步';
  if (actor === 'Human') return '人工介入';
  if (typeof actor === 'object' && 'Seat' in actor) {
    return seatName ?? actor.Seat;
  }
  return '未知来源';
};

export const formatDateTimeZh = (value: string) => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
};

export const formatShortDateTimeZh = (value: string) => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
};

export const formatTimeZh = (value: string) => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`;
};
