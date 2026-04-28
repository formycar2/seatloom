import { create } from 'zustand';
import { Project, ProjectData, Seat, Session, WorkItem, Handoff } from '../types';

interface DataState {
  projects: Project[];
  projectData: Record<string, ProjectData>;
  activeProjectId: string | null;

  // Actions
  setActiveProject: (id: string | null) => void;
  addProject: (project: Project, data: ProjectData) => void;
  pinProject: (id: string, pinned: boolean) => void;
  addSeat: (seat: Seat) => void;
  addSession: (session: Session) => void;
  updateSession: (id: string, updates: Partial<Session>) => void;
  addWorkItem: (wi: WorkItem) => void;
  updateWorkItem: (id: string, updates: Partial<WorkItem>) => void;
  addHandoff: (handoff: Handoff) => void;
  updateHandoff: (id: string, updates: Partial<Handoff>) => void;
  removeInboxItem: (id: string) => void;
}

const LOG_DAY = '2026-04-28';
const at = (time: string) => `${LOG_DAY}T${time}:00+08:00`;
const seededAt = (time: string) => `${LOG_DAY} ${time}`;

const INITIAL_PROJECTS: Project[] = [
  { id: 'p-1', name: 'SeatLoom 协调核心', path: '~/Documents/GitHub/seatloom', isPinned: true, last_accessed: at('20:44') },
  { id: 'p-2', name: 'ThoughtOnly 叙事引擎', path: '~/Documents/GitHub/ThoughtOnly', isPinned: true, last_accessed: at('11:10') },
  { id: 'p-3', name: 'DeepSpeed 推理优化', path: '~/Documents/GitHub/deepspeed', isPinned: false, last_accessed: at('08:30') },
];

const INITIAL_DATA: Record<string, ProjectData> = {
  'p-1': {
    seats: [
      { id: 'seat-1', name: 'lyra', role: 'ProductOwner', status: 'Active', created_at: '2026-04-20T10:00:00+08:00' },
      { id: 'seat-2', name: 'nimbus', role: 'Architect', status: 'Active', created_at: '2026-04-20T10:00:00+08:00' },
      { id: 'seat-3', name: 'mira', role: 'Designer', status: 'Paused', created_at: '2026-04-20T10:00:00+08:00' },
      { id: 'seat-4', name: 'flux', role: 'Verifier', status: 'Active', created_at: '2026-04-20T10:00:00+08:00' },
      { id: 'seat-5', name: 'aegis', role: { Custom: '阶段闸门复核' }, status: 'Paused', created_at: '2026-04-24T09:00:00+08:00' },
    ],
    sessions: [
      {
        id: 'ses-401',
        seat_id: 'seat-1',
        runtime: 'Codex',
        workspace_path: '~/Documents/GitHub/seatloom',
        branch: 'coord/lyra-drive',
        status: 'Running',
        pid: 58421,
        created_at: at('09:10'),
      },
      {
        id: 'ses-402',
        seat_id: 'seat-3',
        runtime: 'GeminiCli',
        workspace_path: '~/Documents/GitHub/seatloom/ui',
        branch: 'design/v5.2',
        status: 'Interrupted',
        created_at: at('11:15'),
        ended_at: at('14:05'),
      },
      {
        id: 'ses-403',
        seat_id: 'seat-4',
        runtime: { Custom: 'OpenCode-DSV4Pro' },
        workspace_path: '~/Documents/GitHub/seatloom',
        branch: 'repair/sg-01-ui',
        status: 'Running',
        pid: 60218,
        created_at: at('15:40'),
      },
      {
        id: 'ses-404',
        seat_id: 'seat-2',
        runtime: 'ClaudeCode',
        workspace_path: '~/Documents/GitHub/seatloom',
        branch: 'implement/nimbus-blocked',
        status: 'Suspended',
        created_at: at('10:20'),
      },
      {
        id: 'ses-405',
        seat_id: 'seat-4',
        runtime: 'CursorCli',
        workspace_path: '~/Documents/GitHub/seatloom',
        branch: 'repair/sg-01-verify',
        status: 'Completed',
        pid: 61102,
        created_at: at('17:20'),
        ended_at: at('18:06'),
      },
      {
        id: 'ses-406',
        seat_id: 'seat-1',
        runtime: 'Codex',
        workspace_path: '~/Documents/GitHub/seatloom/ui',
        branch: 'demo/zh-density-pass',
        status: 'Running',
        pid: 61345,
        created_at: at('20:12'),
      },
    ],
    workItems: [
      {
        id: 'wi-401',
        title: 'SG-01 界面合同修复（代 Mira 执行）',
        goal: '在不改动侧栏/主区信息架构的前提下，完成终端面板、收件箱到详情、活动时间线筛选与交互真值修复，让原型重新回到 PRD v0.4 与交互规格的基线。',
        acceptance_criteria: [
          '终端面板必须作为底部可折叠面板存在，不得继续作为一级标签页。',
          '收件箱任意一行都必须能在真实数据上正确跳转到工作项、会话或交接详情。',
          '活动时间线需要同时支持席位、工作项、事件类型和时间范围筛选。',
          '修复说明必须写入持久化产物，不能只停留在终端输出。',
        ],
        owner_seat_id: 'seat-4',
        status: 'InReview',
        priority: 'Critical',
        depends_on: [],
        created_at: at('10:10'),
        updated_at: at('17:46'),
      },
      {
        id: 'wi-402',
        title: 'Flux 交付验收复核与 Nimbus 放行判定',
        goal: '按照 `docs/prd-v0.4.md`、`docs/interaction-spec-v1.0.md` 与 `docs/acceptance-spec-v1.0.md` 复核当前原型，给出清晰的通过 / 保持阻塞结论，并明确 Nimbus 是否可以接手工程实现。',
        acceptance_criteria: [
          '必须输出覆盖矩阵，逐条标记关键流程是否被真实实现。',
          '所有风险必须给出文件定位，避免 Nimbus 在模糊前提下开工。',
          '若存在阻塞，必须写清楚谁来修、修什么、何时回传。',
        ],
        owner_seat_id: 'seat-1',
        status: 'Active',
        priority: 'Critical',
        depends_on: ['wi-401'],
        created_at: at('15:55'),
        updated_at: at('20:18'),
      },
      {
        id: 'wi-403',
        title: '收件箱引用归一化修补',
        goal: '确保 `WI-402`、`SES-402`、`HO-404` 这类大小写或前缀不同的引用，仍能稳定映射到真实详情对象，避免复核时出现“点得到、看不到”的断链体验。',
        acceptance_criteria: [
          '工作项、会话、交接三类引用都能在当前数据集上命中。',
          '修补说明需要同步到反馈恢复产物，避免后续再次误判。',
          '验证结果必须在真实原型数据上通过，而不是只在静态示例上成立。',
        ],
        owner_seat_id: 'seat-4',
        status: 'Verified',
        priority: 'High',
        depends_on: ['wi-401'],
        created_at: at('18:12'),
        updated_at: at('18:38'),
      },
      {
        id: 'wi-404',
        title: 'Nimbus 工程接力包冻结',
        goal: '在 SG-01 基线没有明确放行前，只准备接力材料，不允许 Nimbus 继续推进落地实现，防止工程层面把未冻结的交互误当成真值。',
        acceptance_criteria: [
          'SG-01 结论明确前，该工作项只能保持待命，不可切到进行中。',
          '工程接力包必须引用通过验收的界面行为说明和对应产物路径。',
          '若阶段闸门仍保持阻塞，需要给出阻塞来源和恢复条件。',
        ],
        owner_seat_id: 'seat-2',
        status: 'Ready',
        priority: 'High',
        depends_on: ['wi-401', 'wi-402'],
        created_at: at('16:05'),
        updated_at: at('18:24'),
      },
      {
        id: 'wi-405',
        title: '日度记忆与反馈恢复写回',
        goal: '把今天的角色切换、Flux 反馈恢复、验收阻塞与后续动作写回持久记忆，确保团队在更换运行时或席位后仍能继续协同。',
        acceptance_criteria: [
          '每日记忆日志要覆盖代 Mira 角色替换与恢复结论。',
          'Flux 反馈恢复记录要落在 `docs/coordination/reviews/`。',
          '关键决策不能只存在于 tmux 或屏幕摘要。',
        ],
        owner_seat_id: 'seat-1',
        status: 'Done',
        priority: 'Medium',
        depends_on: [],
        created_at: at('09:30'),
        updated_at: at('18:20'),
      },
      {
        id: 'wi-406',
        title: '中文演示数据与真实量级刷新',
        goal: '把前端演示数据从英文占位文本切换为中文叙事，并把工作项、收件箱、时间线和详情的密度提升到更接近今日真实协作量级，而不是“小样片式”的轻量演示。',
        acceptance_criteria: [
          '高频可见内容默认以中文呈现，不再出现中英混杂的主视图。',
          'SeatLoom 核心项目至少需要 8 条收件箱记录、9 个工作项、15 条以上事件来支撑真实浏览路径。',
          '文案、数量和时间线必须与今日协调日志保持一致，不允许再出现无关题材或错误规模。',
        ],
        owner_seat_id: 'seat-1',
        status: 'Active',
        priority: 'High',
        depends_on: ['wi-405'],
        created_at: at('18:42'),
        updated_at: at('20:36'),
      },
      {
        id: 'wi-407',
        title: '字体、字号与信息密度统一校正',
        goal: '针对当前原型中的中英文字体、标题层级、正文密度与标签字号做统一修整，解决“信息真实了，但阅读仍然费劲”的体验问题。',
        acceptance_criteria: [
          '中文正文、标签和标题的字体风格需要统一，不得出现明显割裂。',
          '一级标题、卡片摘要、细节标签必须形成清晰层级，避免同屏过密。',
          '修整后需回传前后差异说明，便于 Lyra 做下一轮验收。',
        ],
        owner_seat_id: 'seat-4',
        status: 'Ready',
        priority: 'High',
        depends_on: ['wi-406'],
        created_at: at('18:34'),
        updated_at: at('18:34'),
      },
      {
        id: 'wi-408',
        title: '对账摘要提示口径校准',
        goal: '落实“漂移项进入收件箱、显式对账后仅保留短暂摘要提示”的产品口径，删除独立漂移浮层带来的噪音，保证提醒强度和用户预期一致。',
        acceptance_criteria: [
          '漂移项统一回收到收件箱 / 工作项，不再独立悬浮常驻提示。',
          '只有在显式对账流程完成后，才出现 5 秒自动消失的摘要提示。',
          '提示文案必须指向收件箱，而不是制造第二个待办入口。',
        ],
        owner_seat_id: 'seat-3',
        status: 'Verified',
        priority: 'Medium',
        depends_on: ['wi-401'],
        created_at: at('13:40'),
        updated_at: at('17:32'),
      },
      {
        id: 'wi-409',
        title: '多项目总览摘要密度校正',
        goal: '让项目总览页不仅能看到项目名，还能快速感知待处理规模、会话活跃度与最近操作时间，减少“只有壳子没有内容”的观感。',
        acceptance_criteria: [
          '每个项目卡片都要呈现待处理规模、会话活跃度与最近活动时间。',
          '文案应保持中文语境，同时保留仓库路径等技术信息。',
          '数据量级需要与项目真实活跃度大致匹配，不能全部都是一两条轻量示例。',
        ],
        owner_seat_id: 'seat-2',
        status: 'Draft',
        priority: 'Medium',
        depends_on: ['wi-406'],
        created_at: at('20:05'),
        updated_at: at('20:05'),
      },
    ],
    handoffs: [
      {
        id: 'ho-401',
        from_ref: { Seat: 'seat-1' },
        to_ref: { Seat: 'seat-4' },
        workitem_id: 'wi-401',
        purpose: '按 Lyra 审批过的修补清单执行代 Mira 合同修复，严格保持现有侧栏/主区信息架构不变。',
        expected_outcome: '交付一套通过构建验证的 P0 / P1 修补结果，并附带可追溯的交付说明。',
        artifact_ids: ['ar-401'],
        required_receipt: true,
        status: 'Completed',
        created_at: at('10:25'),
        sent_at: at('10:30'),
      },
      {
        id: 'ho-402',
        from_ref: { Seat: 'seat-4' },
        to_ref: { Seat: 'seat-1' },
        workitem_id: 'wi-402',
        purpose: 'Flux 已完成代 Mira 修复交付，请 Lyra 按 SG-01 基线做正式验收复核。',
        expected_outcome: 'Lyra 明确通过 / 保持阻塞结论，并决定 Nimbus 是否可以进入工程接力。',
        artifact_ids: ['ar-402'],
        required_receipt: true,
        status: 'Received',
        created_at: at('17:35'),
        sent_at: at('17:40'),
      },
      {
        id: 'ho-403',
        from_ref: { Seat: 'seat-1' },
        to_ref: { Seat: 'seat-4' },
        workitem_id: 'wi-403',
        purpose: '补齐收件箱对真实对象的引用归一化，并校正修补说明中的终端可见性口径。',
        expected_outcome: '真实数据下的收件箱跳转稳定可用，后续反馈不再出现对象查找断链。',
        artifact_ids: ['ar-403'],
        required_receipt: true,
        status: 'Completed',
        created_at: at('18:22'),
        sent_at: at('18:24'),
      },
      {
        id: 'ho-404',
        from_ref: { Seat: 'seat-1' },
        to_ref: { Seat: 'seat-4' },
        workitem_id: 'wi-407',
        purpose: '统一原型中的中文字体、字号和信息密度，优先消除中英字重不协调、标签过小和正文压缩导致的阅读疲劳。',
        expected_outcome: '回传一轮带前后差异说明的视觉微调结果，便于 Lyra 继续验收。',
        artifact_ids: ['ar-404'],
        required_receipt: true,
        status: 'Sent',
        created_at: at('18:34'),
        sent_at: at('18:36'),
      },
      {
        id: 'ho-405',
        from_ref: { Seat: 'seat-1' },
        to_ref: { Seat: 'seat-2' },
        workitem_id: 'wi-404',
        purpose: '在 SG-01 没有解除阻塞之前，只允许 Nimbus 维护接力材料，不得继续向实现推进。',
        expected_outcome: 'Nimbus 保持待命并准备好接手包，一旦放行即可无缝进入工程实现。',
        artifact_ids: ['ar-405'],
        required_receipt: true,
        status: 'Received',
        created_at: at('18:24'),
        sent_at: at('18:26'),
      },
      {
        id: 'ho-406',
        from_ref: { Seat: 'seat-4' },
        to_ref: { Seat: 'seat-1' },
        workitem_id: 'wi-401',
        purpose: '回收并整理对 Mira 设计建议的采纳意见，供 Lyra 决定哪些内容纳入临时合同修补包。',
        expected_outcome: 'Lyra 能直接依据建议恢复笔记做采纳/拒绝判断，不再依赖 tmux 即时上下文。',
        artifact_ids: ['ar-406'],
        required_receipt: false,
        status: 'Completed',
        created_at: at('16:48'),
        sent_at: at('17:05'),
      },
    ],
    events: [
      {
        event_id: 'ev-401',
        event_type: 'WorkItemCreated',
        occurred_at: at('10:10'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ WorkItem: 'wi-401' }],
        evidence_refs: ['docs/coordination/tasks/flux/FLUX-2026-04-28-acting-mira-contract-repair-v1.md'],
        payload: { title: 'Lyra 建立代 Mira 合同修复工作项，并冻结信息架构边界。' },
      },
      {
        event_id: 'ev-402',
        event_type: 'HandoffSent',
        occurred_at: at('10:30'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ Handoff: 'ho-401' }, { WorkItem: 'wi-401' }],
        evidence_refs: ['docs/coordination/tasks/flux/FLUX-2026-04-28-acting-mira-contract-repair-v1.md'],
        payload: { title: '修补任务包已发送给 Flux，作为 Mira 的临时替补执行席位。' },
      },
      {
        event_id: 'ev-403',
        event_type: 'SessionInterrupted',
        occurred_at: at('14:05'),
        actor_ref: { Seat: 'seat-3' },
        object_refs: [{ Session: 'ses-402' }],
        evidence_refs: [],
        payload: { summary: 'Mira 当前运行时触达限制并离线，本轮界面修整需由代 Mira 路径接管。' },
      },
      {
        event_id: 'ev-404',
        event_type: 'SessionStarted',
        occurred_at: at('15:40'),
        actor_ref: { Seat: 'seat-4' },
        object_refs: [{ Session: 'ses-403' }],
        evidence_refs: [],
        payload: { title: 'Flux 启动代 Mira 修补会话，开始处理 SG-01 合同缺口。' },
      },
      {
        event_id: 'ev-405',
        event_type: 'ArtifactCreated',
        occurred_at: at('16:48'),
        actor_ref: { Seat: 'seat-4' },
        object_refs: [{ WorkItem: 'wi-401' }],
        evidence_refs: ['docs/coordination/reviews/2026-04-28-flux-feedback-recovery.md'],
        payload: { title: 'Flux 整理出设计建议恢复说明，供 Lyra 做采纳判断。' },
      },
      {
        event_id: 'ev-406',
        event_type: 'HandoffSent',
        occurred_at: at('17:05'),
        actor_ref: { Seat: 'seat-4' },
        object_refs: [{ Handoff: 'ho-406' }, { WorkItem: 'wi-401' }],
        evidence_refs: ['docs/coordination/reviews/2026-04-28-flux-feedback-recovery.md'],
        payload: { title: 'Flux 把建议恢复结果正式交回 Lyra，等待下一轮裁定。' },
      },
      {
        event_id: 'ev-407',
        event_type: 'ReconcileCompleted',
        occurred_at: at('17:32'),
        actor_ref: 'Automation',
        object_refs: [{ WorkItem: 'wi-408' }],
        evidence_refs: [],
        payload: { summary: '显式对账完成：核心 UI 修补已对齐，漂移提醒改为短暂摘要，后续仍待 Lyra 做正式验收。' },
      },
      {
        event_id: 'ev-408',
        event_type: 'ArtifactCreated',
        occurred_at: at('17:38'),
        actor_ref: { Seat: 'seat-4' },
        object_refs: [{ WorkItem: 'wi-401' }],
        evidence_refs: ['docs/coordination/tasks/flux/FLUX-2026-04-28-acting-mira-contract-repair-delivery-v1.md'],
        payload: { title: 'Flux 写入代 Mira 修补交付产物。' },
      },
      {
        event_id: 'ev-409',
        event_type: 'HandoffSent',
        occurred_at: at('17:40'),
        actor_ref: { Seat: 'seat-4' },
        object_refs: [{ Handoff: 'ho-402' }, { WorkItem: 'wi-402' }],
        evidence_refs: ['docs/coordination/tasks/flux/FLUX-2026-04-28-acting-mira-contract-repair-delivery-v1.md'],
        payload: { title: 'Flux 请求 Lyra 进入 SG-01 正式验收复核。' },
      },
      {
        event_id: 'ev-410',
        event_type: 'WorkItemStatusChanged',
        occurred_at: at('17:46'),
        actor_ref: { Seat: 'seat-4' },
        object_refs: [{ WorkItem: 'wi-401' }],
        evidence_refs: [],
        payload: { summary: '代 Mira 修补包在构建通过后转入评审中，等待 Lyra 结论。' },
      },
      {
        event_id: 'ev-411',
        event_type: 'ArtifactCreated',
        occurred_at: at('18:05'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ WorkItem: 'wi-405' }],
        evidence_refs: ['docs/coordination/reviews/2026-04-28-flux-feedback-recovery.md'],
        payload: { title: 'Lyra 写入 Flux 反馈恢复记录，并补齐持久写回。' },
      },
      {
        event_id: 'ev-412',
        event_type: 'WorkItemStatusChanged',
        occurred_at: at('18:18'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ WorkItem: 'wi-403' }],
        evidence_refs: ['ui/src/App.tsx', 'ui/src/stores/useDataStore.ts'],
        payload: { summary: '收件箱引用归一化在真实数据上复核通过，后续关注点转向视觉一致性与信息密度。' },
      },
      {
        event_id: 'ev-413',
        event_type: 'HandoffSent',
        occurred_at: at('18:36'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ Handoff: 'ho-404' }, { WorkItem: 'wi-407' }],
        evidence_refs: ['docs/coordination/reviews/2026-04-28-flux-feedback-recovery.md'],
        payload: { title: 'Lyra 将字体、字号与阅读密度修整要求发给 Flux。' },
      },
      {
        event_id: 'ev-414',
        event_type: 'WorkItemCreated',
        occurred_at: at('18:42'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ WorkItem: 'wi-406' }],
        evidence_refs: ['docs/coordination/tasks/lyra/LYRA-2026-04-28-frontend-demo-data-v2.md'],
        payload: { title: 'Lyra 新建中文演示数据与真实量级刷新任务，开始替换占位叙事。' },
      },
      {
        event_id: 'ev-415',
        event_type: 'HandoffAccepted',
        occurred_at: at('18:55'),
        actor_ref: { Seat: 'seat-4' },
        object_refs: [{ Handoff: 'ho-404' }, { WorkItem: 'wi-407' }],
        evidence_refs: [],
        payload: { summary: 'Flux 已接收视觉统一修整包，后续将回传前后差异说明。' },
      },
      {
        event_id: 'ev-416',
        event_type: 'SessionStarted',
        occurred_at: at('20:12'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ Session: 'ses-406' }, { WorkItem: 'wi-406' }],
        evidence_refs: [],
        payload: { title: 'Lyra 启动中文数据与信息密度校准会话，准备直接修正前端内容。' },
      },
      {
        event_id: 'ev-417',
        event_type: 'ArtifactCreated',
        occurred_at: at('20:28'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ WorkItem: 'wi-406' }],
        evidence_refs: ['docs/coordination/tasks/lyra/LYRA-2026-04-28-frontend-demo-data-v2.md'],
        payload: { title: 'Lyra 写入中文内容与真实量级刷新方案产物。' },
      },
      {
        event_id: 'ev-418',
        event_type: 'WorkItemStatusChanged',
        occurred_at: at('20:36'),
        actor_ref: { Seat: 'seat-1' },
        object_refs: [{ WorkItem: 'wi-406' }],
        evidence_refs: ['ui/src/stores/useDataStore.ts', 'ui/src/views/TimelineView.tsx'],
        payload: { summary: '主视图已切换到中文高密度叙事版本，正在继续清理细节页与弹窗残留英文。' },
      },
      {
        event_id: 'ev-419',
        event_type: 'ReconcileCompleted',
        occurred_at: at('20:44'),
        actor_ref: 'Automation',
        object_refs: [{ WorkItem: 'wi-406' }, { WorkItem: 'wi-407' }],
        evidence_refs: [],
        payload: { summary: '内容对账完成：核心数据规模已贴近今日协作量级，剩余关注点收敛到字体协调与信息密度细修。' },
      },
    ],
    inboxItems: [
      {
        id: 'inbox-401',
        priority: 'Critical',
        type: '待处理交接',
        actor: 'flux -> lyra',
        object_ref: 'WI-402',
        summary: '代 Mira 修复包已交付，Lyra 需要给出 SG-01 验收结论，并明确是否允许 Nimbus 接手后续工程实现。',
        timestamp: seededAt('17:40'),
      },
      {
        id: 'inbox-402',
        priority: 'Critical',
        type: '需确认范围',
        actor: 'lyra',
        object_ref: 'WI-404',
        summary: 'Nimbus 接力包仍被 SG-01 基线拦截，在通过 / 保持阻塞结论明确之前，任何实现推进都应继续冻结。',
        timestamp: seededAt('18:24'),
      },
      {
        id: 'inbox-403',
        priority: 'Normal',
        type: '背景记录',
        actor: 'mira/gemini',
        object_ref: 'SES-402',
        summary: '原设计会话因运行时限制中断，后续界面修整需要保留这段恢复背景，避免误判为设计主动中止。',
        timestamp: seededAt('14:05'),
      },
      {
        id: 'inbox-404',
        priority: 'Normal',
        type: '待处理交接',
        actor: 'lyra -> flux',
        object_ref: 'WI-407',
        summary: '请统一中文字体、字号与信息密度，优先解决标题、标签和正文在同屏阅读时节奏割裂的问题。',
        timestamp: seededAt('18:36'),
      },
      {
        id: 'inbox-405',
        priority: 'Normal',
        type: '需要验收',
        actor: 'lyra',
        object_ref: 'WI-406',
        summary: '中文演示数据已切换为高密度叙事草案，仍需逐页检查详情面板、总览卡片与弹窗是否有英文残留。',
        timestamp: seededAt('20:36'),
      },
      {
        id: 'inbox-406',
        priority: 'Low',
        type: '已记录决策',
        actor: 'automation',
        object_ref: 'WI-408',
        summary: '漂移项统一进入收件箱 / 工作项；显式对账后只保留 5 秒“对账摘要”提示，不再维持独立漂移浮层。',
        timestamp: seededAt('17:32'),
      },
      {
        id: 'inbox-407',
        priority: 'Critical',
        type: '待补产物',
        actor: 'flux',
        object_ref: 'HO-404',
        summary: '字体与信息密度修整完成后，需要补一份前后差异说明产物，便于 Lyra 做下一轮验收归档。',
        timestamp: seededAt('18:55'),
      },
      {
        id: 'inbox-408',
        priority: 'Low',
        type: '需要同步',
        actor: 'aegis',
        object_ref: 'WI-402',
        summary: 'Aegis 只保留阶段闸门复核职责；Lyra 的验收结论必须落到可追溯产物中，不能只写在终端沟通里。',
        timestamp: seededAt('20:20'),
      },
    ],
  },
  'p-2': {
    seats: [
      { id: 'seat-lo', name: 'lyra', role: 'ProductOwner', status: 'Active', created_at: '2026-04-10T10:00:00+08:00' },
      { id: 'seat-nm', name: 'nimbus', role: 'Architect', status: 'Active', created_at: '2026-04-10T10:00:00+08:00' },
    ],
    sessions: [
      {
        id: 'ses-201',
        seat_id: 'seat-nm',
        runtime: 'Codex',
        workspace_path: '~/Documents/GitHub/ThoughtOnly',
        branch: 'schema/story-arc-v2',
        status: 'Interrupted',
        created_at: '2026-04-27T09:00:00+08:00',
        ended_at: '2026-04-27T09:15:00+08:00',
      },
    ],
    workItems: [
      {
        id: 'wi-201',
        title: '剧情弧数据模型收敛',
        goal: '为叙事引擎定义可审计的 JSON Schema，让剧情分支、角色状态和证据链可以被统一记录与回放。',
        acceptance_criteria: ['支持多分支推进', '支持证据追溯', '支持版本升级迁移'],
        owner_seat_id: 'seat-nm',
        status: 'Blocked',
        priority: 'High',
        created_at: '2026-04-25T10:00:00+08:00',
        updated_at: '2026-04-27T08:00:00+08:00',
        depends_on: [],
      },
      {
        id: 'wi-202',
        title: '角色状态快照压缩实验',
        goal: '验证在不损失叙事一致性的情况下，把角色状态快照压缩到适合长流程回放的体量。',
        acceptance_criteria: ['单次快照小于 64KB', '角色心智状态字段可回放', '压缩后可通过一致性校验'],
        owner_seat_id: 'seat-lo',
        status: 'Ready',
        priority: 'Medium',
        created_at: '2026-04-26T12:00:00+08:00',
        updated_at: '2026-04-27T11:10:00+08:00',
        depends_on: ['wi-201'],
      },
    ],
    handoffs: [],
    events: [
      {
        event_id: 'ev-201',
        event_type: 'SessionInterrupted',
        occurred_at: '2026-04-27T09:15:00+08:00',
        actor_ref: { Seat: 'seat-nm' },
        object_refs: [{ Session: 'ses-201' }, { WorkItem: 'wi-201' }],
        evidence_refs: [],
        payload: { summary: '数据模型推演过程中发生网络超时，当前 schema 收敛仍然阻塞。' },
      },
    ],
    inboxItems: [
      {
        id: 'inbox-201',
        priority: 'Critical',
        type: '会话恢复',
        actor: 'system',
        object_ref: 'SES-201',
        summary: '叙事模型推演会话异常中断，仍有 2 组未提交的剧情弧变更需要恢复。',
        timestamp: '2026-04-27 09:15',
      },
      {
        id: 'inbox-202',
        priority: 'Low',
        type: '依赖提醒',
        actor: 'lyra',
        object_ref: 'WI-202',
        summary: '角色状态快照压缩实验依赖数据模型先收敛，目前只能保持 Ready。',
        timestamp: '2026-04-27 11:10',
      },
    ],
  },
  'p-3': {
    seats: [
      { id: 'seat-fx', name: 'flux', role: 'Verifier', status: 'Active', created_at: '2026-04-22T10:00:00+08:00' },
    ],
    sessions: [
      {
        id: 'ses-301',
        seat_id: 'seat-fx',
        runtime: 'CursorCli',
        workspace_path: '~/Documents/GitHub/deepspeed',
        branch: 'bench/transformer-v4',
        status: 'Running',
        pid: 9912,
        created_at: '2026-04-27T00:00:00+08:00',
      },
    ],
    workItems: [
      {
        id: 'wi-301',
        title: 'Transformer-v4 H100 集群基准跑数',
        goal: '在 H100 集群上测量多 batch 配置下的时延、吞吐和显存占用，产出可比的优化前后基线。',
        status: 'Active',
        priority: 'Medium',
        acceptance_criteria: ['关键指标完整记录', '误差率低于 1%', '输出可复现实验说明'],
        owner_seat_id: 'seat-fx',
        created_at: '2026-04-27T00:00:00+08:00',
        updated_at: at('08:30'),
        depends_on: [],
      },
      {
        id: 'wi-302',
        title: '通信热点剖析',
        goal: '识别 Transformer-v4 在跨卡通信上的热点瓶颈，为后续 fused kernel 优化提供依据。',
        status: 'Ready',
        priority: 'High',
        acceptance_criteria: ['热点列表可排序', '给出复现实验命令', '关联至少一条优化建议'],
        owner_seat_id: 'seat-fx',
        created_at: '2026-04-27T07:20:00+08:00',
        updated_at: at('08:20'),
        depends_on: ['wi-301'],
      },
    ],
    handoffs: [],
    events: [
      {
        event_id: 'ev-301',
        event_type: 'SessionStarted',
        occurred_at: '2026-04-27T00:00:00+08:00',
        actor_ref: { Seat: 'seat-fx' },
        object_refs: [{ Session: 'ses-301' }, { WorkItem: 'wi-301' }],
        evidence_refs: [],
        payload: { title: 'Flux 启动 Transformer-v4 基准跑数会话。' },
      },
      {
        event_id: 'ev-302',
        event_type: 'ArtifactCreated',
        occurred_at: at('08:30'),
        actor_ref: { Seat: 'seat-fx' },
        object_refs: [{ WorkItem: 'wi-301' }],
        evidence_refs: [],
        payload: { summary: '首轮基准日志已写出，当前重点转向通信热点定位。' },
      },
    ],
    inboxItems: [
      {
        id: 'inbox-301',
        priority: 'Normal',
        type: '实验观察',
        actor: 'flux',
        object_ref: 'WI-302',
        summary: 'H100 跑数显示跨卡通信波动高于预期，建议尽快补做通信热点剖析。',
        timestamp: seededAt('08:30'),
      },
    ],
  },
};

export const useDataStore = create<DataState>((set) => ({
  projects: INITIAL_PROJECTS,
  projectData: INITIAL_DATA,
  activeProjectId: 'p-1',

  setActiveProject: (id) => set((state) => ({
    activeProjectId: id,
    projects: state.projects.map((p) => (p.id === id ? { ...p, last_accessed: new Date().toISOString() } : p)),
  })),

  addProject: (project, data) => set((state) => ({
    projects: [project, ...state.projects],
    projectData: { ...state.projectData, [project.id]: data },
  })),

  pinProject: (id, pinned) => set((state) => ({
    projects: state.projects.map((p) => (p.id === id ? { ...p, isPinned: pinned } : p)),
  })),

  addSeat: (seat) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: { ...data, seats: [...data.seats, seat] },
      },
    };
  }),

  addSession: (session) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: { ...data, sessions: [...data.sessions, session] },
      },
    };
  }),

  updateSession: (id, updates) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: {
          ...data,
          sessions: data.sessions.map((s) => (s.id === id ? { ...s, ...updates } : s)),
        },
      },
    };
  }),

  addWorkItem: (wi) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: { ...data, workItems: [...data.workItems, wi] },
      },
    };
  }),

  updateWorkItem: (id, updates) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: {
          ...data,
          workItems: data.workItems.map((wi) => (wi.id === id ? { ...wi, ...updates } : wi)),
        },
      },
    };
  }),

  addHandoff: (handoff) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: { ...data, handoffs: [...data.handoffs, handoff] },
      },
    };
  }),

  updateHandoff: (id, updates) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: {
          ...data,
          handoffs: data.handoffs.map((ho) => (ho.id === id ? { ...ho, ...updates } : ho)),
        },
      },
    };
  }),

  removeInboxItem: (id) => set((state) => {
    if (!state.activeProjectId) return state;
    const data = state.projectData[state.activeProjectId];
    return {
      projectData: {
        ...state.projectData,
        [state.activeProjectId]: {
          ...data,
          inboxItems: data.inboxItems.filter((item) => item.id !== id),
        },
      },
    };
  }),
}));
