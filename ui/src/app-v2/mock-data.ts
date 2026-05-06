/**
 * mock-data.ts
 * Mock 数据：用于 v2 前端开发阶段的静态数据
 * 包括：MOCK_CONTACTS（联系人列表）、MOCK_MESSAGES（每个联系人的消息）、MOCK_CHANNEL_DATA（项目频道工作流数据）
 */

import { ChatContact, ChatMessage, ProjectChannelData } from './types';

export const MOCK_CONTACTS: ChatContact[] = [
  // ── Global ──
  { id: 'supervisor', name: 'Aegis · Supervisor', type: 'supervisor', online: true, unread: 1, lastMessage: '3 个项目有待处理事项', lastTime: '刚刚', avatar: 'A', color: 'var(--sl-brand)' },

  // ── SeatLoom 主项目 ──
  { id: 'ch-p1', name: 'SeatLoom 主项目', type: 'project-channel', online: true, projectId: 'p-1', unread: 2, lastMessage: '[Gate] Product Baseline Freeze → GO', lastTime: '14:30', avatar: '#', color: 'var(--sl-teal)' },
  { id: 'p1-lyra', name: 'Lyra', type: 'seat', role: '产品负责人', seatType: 'po', online: true, projectId: 'p-1', unread: 2, lastMessage: '已签发 Nimbus 下一个工程包', lastTime: '14:28', avatar: 'L', color: 'var(--sl-purple)' },
  { id: 'p1-nimbus', name: 'Nimbus', type: 'seat', role: '架构师', seatType: 'worker', online: true, projectId: 'p-1', unread: 1, lastMessage: '存储层基础已提交，等待编译验证', lastTime: '14:23', avatar: 'N', color: 'var(--sl-blue)' },
  { id: 'p1-mira', name: 'Mira', type: 'seat', role: 'UX 设计', seatType: 'worker', online: false, projectId: 'p-1', unread: 0, lastMessage: 'S7B 命令栏已交付', lastTime: '昨天', avatar: 'M', color: 'var(--sl-amber)' },
  { id: 'p1-flux', name: 'Flux', type: 'seat', role: '质量验证', seatType: 'verifier', online: true, projectId: 'p-1', unread: 3, lastMessage: 'UI 视觉审计: 5 个 P1 缺陷', lastTime: '14:15', avatar: 'F', color: 'var(--sl-green)' },

  // ── DataForge 数据平台 ──
  { id: 'ch-p2', name: 'DataForge 数据平台', type: 'project-channel', online: true, projectId: 'p-2', unread: 5, lastMessage: '[告警] Pipeline-03 超时失败', lastTime: '13:45', avatar: '#', color: 'var(--sl-red)' },
  { id: 'p2-iris', name: 'Iris', type: 'seat', role: '数据架构师', seatType: 'po', online: true, projectId: 'p-2', unread: 1, lastMessage: 'Schema v3 迁移脚本已就绪', lastTime: '13:40', avatar: 'I', color: 'var(--sl-blue)' },
  { id: 'p2-bolt', name: 'Bolt', type: 'seat', role: '工程实现', seatType: 'worker', online: true, projectId: 'p-2', unread: 2, lastMessage: 'Pipeline-03 重试中...', lastTime: '13:42', avatar: 'B', color: 'var(--sl-amber)' },
  { id: 'p2-sage', name: 'Sage', type: 'seat', role: '数据质量', seatType: 'verifier', online: true, projectId: 'p-2', unread: 1, lastMessage: '字段覆盖率 92%，目标 98%', lastTime: '13:30', avatar: 'S', color: 'var(--sl-green)' },
  { id: 'p2-nova', name: 'Nova', type: 'seat', role: '可视化', seatType: 'worker', online: false, projectId: 'p-2', unread: 0, lastMessage: 'Dashboard 原型 v2 待确认', lastTime: '昨天', avatar: 'N', color: 'var(--sl-purple)' },

  // ── CloudNest 基础设施 ──
  { id: 'ch-p3', name: 'CloudNest 基础设施', type: 'project-channel', online: true, projectId: 'p-3', unread: 0, lastMessage: 'K8s 集群扩容完成', lastTime: '12:00', avatar: '#', color: 'var(--sl-green)' },
  { id: 'p3-atlas', name: 'Atlas', type: 'seat', role: 'SRE', seatType: 'po', online: true, projectId: 'p-3', unread: 0, lastMessage: '监控面板已更新', lastTime: '11:55', avatar: 'A', color: 'var(--sl-teal)' },
  { id: 'p3-terra', name: 'Terra', type: 'seat', role: 'IaC 工程', seatType: 'worker', online: true, projectId: 'p-3', unread: 1, lastMessage: 'Terraform plan 需要审批', lastTime: '12:10', avatar: 'T', color: 'var(--sl-amber)' },
  { id: 'p3-shield', name: 'Shield', type: 'seat', role: '安全审计', seatType: 'verifier', online: false, projectId: 'p-3', unread: 0, lastMessage: 'CVE 扫描: 0 Critical, 2 Medium', lastTime: '昨天', avatar: 'S', color: 'var(--sl-red)' },
];

export const MOCK_MESSAGES: Record<string, ChatMessage[]> = {
  supervisor: [
    { id: 'm1', contactId: 'supervisor', from: 'contact', time: '09:00', type: 'info', card: {
      title: '全局健康摘要', status: '3 项目 · 16 席位 · 2 告警',
      fields: [
        { label: 'SeatLoom', value: '1 阻塞 · Freeze GO' },
        { label: 'DataForge', value: 'Pipeline 失败 · 重试中' },
        { label: 'CloudNest', value: '正常 · 1 待审批' },
      ],
      actions: ['查看全部项目'],
      detail: '全局视角：SeatLoom 主项目的 ENV-001 编译验证剩余图标修复，DataForge 的 Pipeline-03 因 Schema v3 迁移 JOIN 超时正在重试，CloudNest 有一个 Terraform Plan 等待审批。',
    }},
    { id: 'm3', contactId: 'supervisor', from: 'user', time: '09:02', type: 'text', content: 'DataForge 的 Pipeline 什么情况？' },
    { id: 'm4', contactId: 'supervisor', from: 'contact', time: '09:02', type: 'text', content: 'Pipeline-03 在 13:45 超时失败，Bolt 正在重试。Iris 已准备好索引修复脚本。' },
    { id: 'm5', contactId: 'supervisor', from: 'contact', time: '13:50', type: 'gate', card: {
      title: 'Product Baseline Freeze', status: 'GO', statusColor: 'var(--sl-green)',
      fields: [
        { label: 'UI 基线', value: 'SG-01 通过' },
        { label: '存储基线', value: '验收通过' },
        { label: 'ENV-001', value: '独立跟踪 (非阻塞)' },
      ],
      actions: ['查看审查文档'],
      detail: 'Product Baseline Freeze 可关闭为 GO。活跃产品真值入口和合约集明确且一致。SG-01 UI 基线已关闭，Nimbus 存储基础和席位注册已验收。',
    }},
  ],
  'p1-lyra': [
    { id: 'l1', contactId: 'p1-lyra', from: 'contact', time: '14:25', type: 'gate', card: {
      title: 'Product Baseline Freeze', status: 'GO', statusColor: 'var(--sl-green)',
      fields: [
        { label: '审查方', value: 'Aegis' },
        { label: 'UI 基线', value: 'SG-01 通过' },
        { label: '存储基线', value: '验收通过' },
        { label: '遗留项', value: 'ENV-001 独立跟踪' },
      ],
      actions: ['查看审查文档'],
      detail: 'Aegis 的阶段审查确认活跃产品合约集、已验收的 UI 基线和存储切片足够关闭 Freeze。ENV-001 编译验证作为独立轨道继续。',
    }},
    { id: 'l2', contactId: 'p1-lyra', from: 'contact', time: '14:28', type: 'delivery', card: {
      title: '已签发: 席位注册 + 委派存储', status: '→ Nimbus', statusColor: 'var(--sl-blue)',
      fields: [
        { label: '范围', value: '持久化席位身份、项目角色绑定、委派' },
        { label: '优先级', value: 'P0' },
      ],
      actions: ['查看任务包'],
    }},
    { id: 'l3', contactId: 'p1-lyra', from: 'user', time: '14:30', type: 'text', content: '收到，Nimbus 应该很快能完成。' },
  ],
  'p1-nimbus': [
    { id: 'n1', contactId: 'p1-nimbus', from: 'contact', time: '14:20', type: 'delivery', card: {
      title: '交付: 席位注册 + 委派存储', status: '已验收', statusColor: 'var(--sl-green)',
      fields: [
        { label: '测试', value: '9/9 通过' },
        { label: '文件数', value: '8 files changed' },
        { label: '编译', value: 'ENV-001 待验证' },
      ],
      actions: ['查看代码', '查看测试'],
      detail: '确定性 YAML/JSONL 项目 IO 和 Ledger 原语已验收。席位身份、项目角色绑定和委派持久化已落地，无范围扩展。',
    }},
    { id: 'n3', contactId: 'p1-nimbus', from: 'user', time: '14:25', type: 'text', content: 'ENV-001 编译已验过，core 全部通过，tauri 缺图标' },
    { id: 'n4', contactId: 'p1-nimbus', from: 'contact', time: '14:26', type: 'text', content: '收到，我来补图标文件。' },
  ],
  'p1-flux': [
    { id: 'f1', contactId: 'p1-flux', from: 'contact', time: '14:15', type: 'verification', card: {
      title: 'UI 视觉审计报告', status: '5 P1 · 11 P2', statusColor: 'var(--sl-amber)',
      fields: [
        { label: 'hover 状态', value: '不一致 — P1' },
        { label: '主题 token', value: '未统一应用 — P1' },
        { label: '对比度', value: '部分不足 — P1' },
        { label: '间距', value: '卡片间距偏差 — P1' },
        { label: '字体回退', value: '中文回退链 — P1' },
      ],
      actions: ['查看完整报告', '签发修复包'],
      detail: '覆盖三个主题预设的完整视觉审计。5 个 P1 缺陷需要立即修复，11 个 P2 项可延后。',
    }},
    { id: 'f2', contactId: 'p1-flux', from: 'contact', time: '14:30', type: 'verification', card: {
      title: 'v0.5 UI 基线证据包', status: '10/10 通过', statusColor: 'var(--sl-green)',
      fields: [
        { label: 'Shell 真值', value: '✅' },
        { label: '收件箱循环', value: '✅' },
        { label: '命令栏', value: '✅' },
        { label: '委派覆盖', value: '✅' },
      ],
      actions: ['查看证据包'],
    }},
  ],
  'p2-iris': [
    { id: 'i1', contactId: 'p2-iris', from: 'contact', time: '13:40', type: 'delivery', card: {
      title: 'Schema v3 迁移脚本', status: '就绪', statusColor: 'var(--sl-green)',
      fields: [
        { label: '回填', value: '100% 完成' },
        { label: '索引', value: '已创建' },
        { label: '影响表', value: '3 张' },
      ],
      actions: ['查看 DDL', '对比 v2→v3'],
    }},
  ],
  'p2-bolt': [
    { id: 'b1', contactId: 'p2-bolt', from: 'contact', time: '13:42', type: 'progress', card: {
      title: 'Pipeline-03 重试', status: '进行中 60%', statusColor: 'var(--sl-amber)',
      fields: [
        { label: 'extract', value: '✅ 完成' },
        { label: 'transform', value: '🔄 60%' },
        { label: 'load', value: '⏳ 等待' },
      ],
      actions: ['查看日志', '取消重试'],
    }},
    { id: 'b2', contactId: 'p2-bolt', from: 'user', time: '13:45', type: 'text', content: '如果再失败就先降级，明天再处理' },
    { id: 'b3', contactId: 'p2-bolt', from: 'contact', time: '13:45', type: 'text', content: '收到，已设置超时自动降级策略。' },
  ],
  'p2-sage': [
    { id: 's1', contactId: 'p2-sage', from: 'contact', time: '13:30', type: 'verification', card: {
      title: '数据质量报告 04/29', status: '92% (目标 98%)', statusColor: 'var(--sl-amber)',
      fields: [
        { label: '字段覆盖', value: '92%' },
        { label: '空值异常', value: '3 列' },
        { label: '格式偏差', value: '1 列' },
      ],
      actions: ['查看明细', '签发修复'],
    }},
  ],
  'p3-terra': [
    { id: 't1', contactId: 'p3-terra', from: 'contact', time: '12:10', type: 'gate', card: {
      title: 'Terraform Plan 待审批', status: '等待审批', statusColor: 'var(--sl-amber)',
      fields: [
        { label: '环境', value: 'staging' },
        { label: '变更', value: 'CNAME → A record' },
        { label: '影响', value: 'api.staging.cloudnest.io' },
      ],
      actions: ['批准', '拒绝', '查看 Plan'],
      detail: 'Staging 环境 DNS 配置变更，将 api.staging.cloudnest.io 从 CNAME 切换为 A record，指向新的负载均衡器 IP。',
    }},
  ],
};

// ── Project Channel Data (workflow + metrics, not messages) ──
export const MOCK_CHANNEL_DATA: Record<string, ProjectChannelData> = {
  'ch-p1': {
    goals: [
      { name: '可行性论证', status: 'done', currentGoalIndex: 0 },
      { name: 'MVP 交付', status: 'active', currentGoalIndex: 1 },
      { name: '第一次迭代', status: 'upcoming', currentGoalIndex: 2 },
    ],
    currentGoal: {
      name: 'MVP 交付',
      currentStageIndex: 3,
      stages: [
        { name: '产品合约', status: 'done' },
        { name: '基线建立', status: 'done' },
        { name: '设计', status: 'revisited', revisitReason: 'UI 重构：Mira 产出不达标，Aegis 主导重新设计' },
        { name: '核心实现', status: 'active' },
        { name: '集成验证', status: 'upcoming' },
        { name: 'MVP 发布', status: 'upcoming' },
      ],
    },
    currentStage: {
      name: '核心实现',
      workItemsDone: 3,
      workItemsTotal: 8,
      workflow: { nodes: [
        { id: 'w1', type: 'task', label: '签发工程包', owner: 'Lyra', ownerAvatar: 'L', ownerColor: 'var(--sl-purple)', status: 'done', dependsOn: [], description: '已签发 3 个工程包：包括存储基础、席位注册和 UI 框架重构。' },
        { id: 'w2', type: 'task', label: 'ENV-001 图标修复', owner: 'Nimbus', ownerAvatar: 'N', ownerColor: 'var(--sl-blue)', status: 'active', dependsOn: ['w1'], description: '补充缺失的 src-tauri/icons 资源，修复构建脚本中的静态路径引用。', workItemRef: 'WI-411', waitingSince: '2h' },
        { id: 'w3', type: 'task', label: 'UI 重构 (设计先行)', owner: 'Mira', ownerAvatar: 'M', ownerColor: 'var(--sl-amber)', status: 'active', dependsOn: ['w1'], description: '实现 v2 版本全新的 App Shell，包括 NavRail 和 Supervisor IM 模式。', workItemRef: 'WI-392' },
        { id: 'w4', type: 'task', label: '编译验证', owner: 'Flux', ownerAvatar: 'F', ownerColor: 'var(--sl-green)', status: 'waiting', dependsOn: ['w2'], description: '等待 Nimbus 修复后重新验证 cargo check' },
        { id: 'w5', type: 'task', label: 'UI 视觉复核', owner: 'Flux', ownerAvatar: 'F', ownerColor: 'var(--sl-green)', status: 'waiting', dependsOn: ['w3'], description: '等待 Mira 重构后验证视觉质量' },
        { id: 'w6', type: 'gate', label: 'Gate: 实现完整性', owner: 'Lyra', ownerAvatar: 'L', ownerColor: 'var(--sl-purple)', status: 'waiting', dependsOn: ['w4', 'w5'], description: '确认所有实现包和验证通过', icon: '⬧' },
      ] },
      blockers: [
        { text: 'src-tauri/icons/icon.png 缺失，workspace cargo check 失败', owner: 'Nimbus', since: '2h' },
        { text: 'v0.5 UI 基线验证在 Tablet 模式下存在布局错位', owner: 'Mira', since: '45min' },
      ],
      nextStep: 'Nimbus 补充图标 → Flux 重跑 cargo check → 关闭 ENV-001',
    },
    justNow: [
      { text: 'Product Baseline Freeze → GO', time: '14:30', type: 'decision' },
      { text: 'Nimbus 交付: 席位注册 + 委派存储 (9/9 测试通过)', time: '14:23', type: 'delivery' },
      { text: 'Flux UI 视觉审计: 5 P1 / 11 P2', time: '14:15', type: 'delivery' },
    ],
    earlierToday: 'SG-01 UI 基线通过 · Mira S7B/S7C 验收 · 命令栏键盘修复',
    yesterday: '架构基线冻结 · Nimbus scaffold 验收 · 移动端 companion 修复',
  },
  'ch-p2': {
    goals: [
      { name: '平台评估', status: 'done', currentGoalIndex: 0 },
      { name: 'v2 升级', status: 'active', currentGoalIndex: 1 },
      { name: '全量切换', status: 'upcoming', currentGoalIndex: 2 },
    ],
    currentGoal: {
      name: 'v2 升级',
      currentStageIndex: 1,
      stages: [
        { name: 'Schema 迁移', status: 'done' },
        { name: 'Pipeline 稳定', status: 'active' },
        { name: '质量达标', status: 'upcoming' },
        { name: '上线切换', status: 'upcoming' },
      ],
    },
    currentStage: {
      name: 'Pipeline 稳定',
      workItemsDone: 1,
      workItemsTotal: 3,
      workflow: { nodes: [
        { id: 'df1', type: 'task', label: '索引优化', owner: 'Iris', ownerAvatar: 'I', ownerColor: 'var(--sl-blue)', status: 'done', dependsOn: [], description: 'Schema v3 索引已就绪' },
        { id: 'df2', type: 'task', label: 'Pipeline 重试', owner: 'Bolt', ownerAvatar: 'B', ownerColor: 'var(--sl-amber)', status: 'active', dependsOn: ['df1'], description: 'transform 阶段 60%', progress: { done: 60, total: 100, unit: '%' }, waitingSince: '45min' },
        { id: 'df3', type: 'task', label: '质量验证', owner: 'Sage', ownerAvatar: 'S', ownerColor: 'var(--sl-green)', status: 'waiting', dependsOn: ['df2'], description: '等待 Pipeline 成功后验证' },
        { id: 'df4', type: 'task', label: '性能基准测试', owner: 'Iris', ownerAvatar: 'I', ownerColor: 'var(--sl-blue)', status: 'waiting', dependsOn: ['df2'], description: '等待 Pipeline 稳定后测试' },
        { id: 'df5', type: 'gate', label: '稳定性确认', status: 'waiting', dependsOn: ['df3', 'df4'], description: '确认 Pipeline 稳定性', icon: '⬧' },
      ] },
      blockers: [
        { text: 'Pipeline-03 transform 阶段因 JOIN 超时失败', owner: 'Bolt', since: '45min' },
      ],
      nextStep: 'Bolt 重试完成 → Sage 验证数据质量 → 确认稳定性',
    },
    justNow: [
      { text: 'Pipeline-03 重试中 (60%)', time: '13:42', type: 'action' },
      { text: 'Iris 交付 Schema v3 迁移脚本', time: '13:40', type: 'delivery' },
      { text: 'Pipeline-03 超时失败告警', time: '13:30', type: 'action' },
    ],
    earlierToday: 'Schema v3 迁移批准 · 回填完成 · 索引创建',
    yesterday: 'Schema v2→v3 差异分析 · 影响评估完成',
  },
  'ch-p3': {
    goals: [
      { name: '基础设施评估', status: 'done', currentGoalIndex: 0 },
      { name: '现代化改造', status: 'active', currentGoalIndex: 1 },
    ],
    currentGoal: {
      name: '现代化改造',
      currentStageIndex: 1,
      stages: [
        { name: '容量扩展', status: 'done' },
        { name: '网络优化', status: 'active' },
        { name: '安全加固', status: 'upcoming' },
        { name: '可观测性', status: 'upcoming' },
      ],
    },
    currentStage: {
      name: '网络优化',
      workItemsDone: 1,
      workItemsTotal: 3,
      workflow: { nodes: [
        { id: 'cn1', type: 'task', label: 'DNS 迁移', owner: 'Terra', ownerAvatar: 'T', ownerColor: 'var(--sl-amber)', status: 'active', dependsOn: [], description: 'Terraform Plan 待审批', badge: '待审批', badgeColor: 'var(--sl-amber)' },
        { id: 'cn2', type: 'task', label: '监控验证', owner: 'Atlas', ownerAvatar: 'A', ownerColor: 'var(--sl-teal)', status: 'waiting', dependsOn: ['cn1'], description: '等待 DNS 变更后验证' },
        { id: 'cn3', type: 'gate', label: '网络验收', status: 'waiting', dependsOn: ['cn2'], description: '确认网络优化完成', icon: '⬧' },
      ] },
      blockers: [],
      nextStep: '审批 Terraform Plan → DNS 切换 → Atlas 监控验证',
    },
    justNow: [
      { text: 'Terra 提交 Terraform Plan 待审批', time: '12:10', type: 'action' },
      { text: 'K8s 集群扩容完成 (6→8 nodes)', time: '12:00', type: 'decision' },
    ],
    earlierToday: 'Atlas 更新监控面板 · 扩容方案确认',
    yesterday: 'CVE 扫描完成: 0 Critical / 2 Medium',
  },
};
