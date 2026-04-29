import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Zap, AlertTriangle, Clock, ArrowRight, Activity, Target, Info, ChevronRight } from 'lucide-react';
import './styles/tokens.css';
import { WorkNode, StageWorkflow } from './dag-model';
import DagWorkflow from './DagWorkflow';
import { useDataStore } from '../stores/useDataStore';

// ═══════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════

interface ChatContact {
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

interface ChatMessage {
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

// Project channel data (workflow + plan progress, not flat metrics)
interface PlanPhase {
  name: string;
  status: 'done' | 'active' | 'upcoming' | 'revisited';
  revisitReason?: string;
}

// (WorkNode imported from dag-model.ts)

interface TimelineEntry {
  text: string;
  time: string;
  type: 'action' | 'decision' | 'delivery';
}

interface ProjectChannelData {
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

// ═══════════════════════════════════════════════════════
// Mock Data
// ═══════════════════════════════════════════════════════

const MOCK_CONTACTS: ChatContact[] = [
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

const MOCK_MESSAGES: Record<string, ChatMessage[]> = {
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
const MOCK_CHANNEL_DATA: Record<string, ProjectChannelData> = {
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

// ═══════════════════════════════════════════════════════
// Small Components
// ═══════════════════════════════════════════════════════

const SeatLoomLogo: React.FC<{ size?: number }> = ({ size = 22 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
    <rect width="24" height="24" rx="6" fill="var(--sl-brand)" />
    <path d="M7 8.5C7 8.5 9.5 6 12 6C14.5 6 17 8.5 17 8.5" stroke="white" strokeWidth="1.8" strokeLinecap="round" />
    <path d="M7 12C7 12 9.5 14.5 12 14.5C14.5 14.5 17 12 17 12" stroke="white" strokeWidth="1.8" strokeLinecap="round" />
    <path d="M7 15.5C7 15.5 9.5 18 12 18C14.5 18 17 15.5 17 15.5" stroke="white" strokeWidth="1.8" strokeLinecap="round" />
  </svg>
);

const StatusDot: React.FC<{ color: string; label: string; value?: number | string }> = ({ color, label, value }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
    <div style={{ width: 8, height: 8, borderRadius: '50%', backgroundColor: color, boxShadow: `0 0 0 2px ${color}20` }} />
    <span style={{ fontSize: 12, color: 'var(--sl-text-secondary)', fontWeight: 500 }}>{label}</span>
    {value !== undefined && <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{value}</span>}
  </div>
);

// ═══════════════════════════════════════════════════════
// Avatar
// ═══════════════════════════════════════════════════════

const Avatar: React.FC<{ char: string; color: string; size?: number; online?: boolean }> = ({ char, color, size = 36, online }) => (
  <div style={{ position: 'relative', width: size, height: size, flexShrink: 0 }}>
    <div style={{
      width: size, height: size, borderRadius: size * 0.28,
      background: `${color}18`, color: color,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      fontSize: size * 0.38, fontWeight: 700,
      border: `1.5px solid ${color}40`,
    }}>{char}</div>
    {online !== undefined && (
      <div style={{
        position: 'absolute', bottom: -1, right: -1,
        width: 10, height: 10, borderRadius: '50%',
        background: online ? 'var(--sl-green)' : 'var(--sl-border)',
        border: '2px solid var(--sl-surface)',
      }} />
    )}
  </div>
);

// ═══════════════════════════════════════════════════════
// Message Bubble (structured card with hover/click detail)
// ═══════════════════════════════════════════════════════

const TYPE_BADGES: Record<string, { label: string; color: string; bg: string }> = {
  delivery: { label: '交付', color: 'var(--sl-blue)', bg: 'var(--sl-blue-subtle)' },
  verification: { label: '验证', color: 'var(--sl-teal)', bg: 'var(--sl-teal-subtle)' },
  gate: { label: '审批', color: 'var(--sl-green)', bg: 'var(--sl-green-subtle)' },
  blocker: { label: '阻塞', color: 'var(--sl-red)', bg: 'var(--sl-red-subtle)' },
  progress: { label: '进度', color: 'var(--sl-amber)', bg: 'var(--sl-amber-subtle)' },
  alert: { label: '告警', color: 'var(--sl-red)', bg: 'var(--sl-red-subtle)' },
  info: { label: '摘要', color: 'var(--sl-brand)', bg: 'var(--sl-brand-subtle)' },
};

const MessageBubble: React.FC<{ msg: ChatMessage }> = ({ msg }) => {
  const [showDetail, setShowDetail] = useState(false);
  const [hovered, setHovered] = useState(false);

  // Plain text from user
  if (msg.type === 'text') {
    return (
      <div style={{
        display: 'flex', flexDirection: 'column',
        alignItems: msg.from === 'user' ? 'flex-end' : 'flex-start',
        alignSelf: msg.from === 'user' ? 'flex-end' : 'flex-start',
        maxWidth: '78%',
      }}>
        <div style={{
          padding: '8px 12px',
          borderRadius: msg.from === 'user' ? '12px 12px 2px 12px' : '12px 12px 12px 2px',
          background: msg.from === 'user' ? 'var(--sl-brand)' : 'var(--sl-surface-hover)',
          color: msg.from === 'user' ? 'white' : 'var(--sl-text-primary)',
          fontSize: 13, lineHeight: 1.5,
          boxShadow: 'var(--sl-shadow-sm)',
        }}>
          {msg.content}
        </div>
        <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 2, padding: '0 4px' }}>{msg.time}</span>
      </div>
    );
  }

  // Structured card
  const badge = TYPE_BADGES[msg.type] || TYPE_BADGES.info;
  const card = msg.card!;

  return (
    <div style={{
      alignSelf: msg.from === 'user' ? 'flex-end' : 'flex-start',
      maxWidth: '85%', position: 'relative',
    }}>
      <div
        onClick={() => card.detail && setShowDetail(true)}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        style={{
          padding: '10px 14px',
          borderRadius: 'var(--sl-radius-lg)',
          border: `1px solid ${hovered ? badge.color + '60' : 'var(--sl-border)'}`,
          background: 'var(--sl-surface)',
          boxShadow: hovered ? 'var(--sl-shadow-md)' : 'var(--sl-shadow-sm)',
          cursor: card.detail ? 'pointer' : 'default',
          transition: 'all 150ms ease',
          minWidth: 220,
        }}
      >
        {/* Type badge + title */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
          <span style={{
            fontSize: 10, fontWeight: 600, padding: '2px 6px',
            borderRadius: 'var(--sl-radius-full)',
            background: badge.bg, color: badge.color,
          }}>{badge.label}</span>
          <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{card.title}</span>
        </div>

        {/* Status line */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: card.fields?.length ? 8 : 0 }}>
          {card.statusColor && <div style={{ width: 6, height: 6, borderRadius: '50%', background: card.statusColor }} />}
          <span style={{ fontSize: 12, color: 'var(--sl-text-secondary)' }}>{card.status}</span>
        </div>

        {/* Fields (key-value pairs) */}
        {card.fields && card.fields.length > 0 && (
          <div style={{
            display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '3px 12px',
            fontSize: 12, marginBottom: card.actions?.length ? 8 : 0,
            padding: '6px 0',
            borderTop: '1px solid var(--sl-divider)',
          }}>
            {card.fields.map(f => (
              <React.Fragment key={f.label}>
                <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 500 }}>{f.label}</span>
                <span style={{ color: 'var(--sl-text-primary)' }}>{f.value}</span>
              </React.Fragment>
            ))}
          </div>
        )}

        {/* Action buttons */}
        {card.actions && card.actions.length > 0 && (
          <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', paddingTop: 4 }}>
            {card.actions.map(a => (
              <button key={a} onClick={e => e.stopPropagation()} style={{
                padding: '3px 10px', fontSize: 11, fontWeight: 500,
                color: 'var(--sl-brand)', background: 'var(--sl-brand-subtle)',
                border: '1px solid transparent', borderRadius: 'var(--sl-radius-full)',
                cursor: 'pointer', transition: 'all 120ms ease',
              }}
                onMouseEnter={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; }}
                onMouseLeave={e => { e.currentTarget.style.borderColor = 'transparent'; }}
              >{a}</button>
            ))}
          </div>
        )}

        {/* "Click for detail" hint */}
        {card.detail && hovered && (
          <div style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 4, textAlign: 'right' }}>
            点击查看详情
          </div>
        )}
      </div>
      <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 2, padding: '0 4px', display: 'block' }}>{msg.time}</span>

      {/* Detail popup */}
      {showDetail && card.detail && (
        <>
          <div onClick={() => setShowDetail(false)} style={{ position: 'fixed', inset: 0, zIndex: 10000, background: 'rgba(0,0,0,0.2)' }} />
          <div style={{
            position: 'fixed', top: '50%', left: '50%', transform: 'translate(-50%,-50%)',
            zIndex: 10001, width: 420, background: 'var(--sl-surface)',
            border: '1px solid var(--sl-border)', borderRadius: 'var(--sl-radius-xl)',
            boxShadow: 'var(--sl-shadow-overlay)', padding: '20px',
          }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ fontSize: 10, fontWeight: 600, padding: '2px 6px', borderRadius: 'var(--sl-radius-full)', background: badge.bg, color: badge.color }}>{badge.label}</span>
                <span style={{ fontSize: 15, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{card.title}</span>
              </div>
              <button onClick={() => setShowDetail(false)} style={{ border: 'none', background: 'transparent', fontSize: 16, color: 'var(--sl-text-tertiary)', cursor: 'pointer' }}>✕</button>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 12 }}>
              {card.statusColor && <div style={{ width: 8, height: 8, borderRadius: '50%', background: card.statusColor }} />}
              <span style={{ fontSize: 13, fontWeight: 500, color: 'var(--sl-text-secondary)' }}>{card.status}</span>
            </div>
            {card.fields && (
              <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '6px 16px', fontSize: 13, marginBottom: 16, padding: '12px', background: 'var(--sl-bg)', borderRadius: 'var(--sl-radius-md)' }}>
                {card.fields.map(f => (
                  <React.Fragment key={f.label}>
                    <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 500 }}>{f.label}</span>
                    <span style={{ color: 'var(--sl-text-primary)' }}>{f.value}</span>
                  </React.Fragment>
                ))}
              </div>
            )}
            <p style={{ fontSize: 13, lineHeight: 1.7, color: 'var(--sl-text-secondary)' }}>{card.detail}</p>
          </div>
        </>
      )}
    </div>
  );
};

// ═══════════════════════════════════════════════════════
// Project Dashboard (shown in project channels)
// ═══════════════════════════════════════════════════════

const ProjectDashboard: React.FC<{ channelId: string; projectId?: string }> = ({ channelId, projectId }) => {
  const data = MOCK_CHANNEL_DATA[channelId];
  const { projectData } = useDataStore();
  const [showEarlier, setShowEarlier] = useState(false);
  const [hoveredItem, setHoveredItem] = useState<{ type: string; data: any; x: number; y: number } | null>(null);

  // ── Hybrid source model ──
  const mockPlanData = data;
  const truthData = projectId ? projectData[projectId] : undefined;

  // ── Seat lookup helper ──
  const seatMap = new Map<string, string>();
  const truthSeats = truthData?.seats || [];
  truthSeats.forEach(s => seatMap.set(s.id, s.name));
  const mockSeatOverrides: Record<string, string> = {
    'seat-1': 'Lyra', 'seat-2': 'Nimbus', 'seat-3': 'Mira', 'seat-4': 'Flux', 'seat-5': 'Aegis',
  };
  const getSeatName = (seatId?: string): string => {
    if (!seatId) return 'Unknown';
    return seatMap.get(seatId) || mockSeatOverrides[seatId] || seatId;
  };

  // ── Formatting helpers ──
  const formatClock = (iso: string): string => {
    const m = iso.match(/T(\d{2}:\d{2})/);
    return m ? m[1] : iso;
  };
  const formatSince = (iso: string): string => {
    const now = new Date('2026-04-28T23:59:00+08:00');
    const t = new Date(iso.replace(' ', 'T'));
    const diffMin = Math.round((now.getTime() - t.getTime()) / 60000);
    if (diffMin < 60) return `${diffMin}分钟前`;
    const diffH = Math.round(diffMin / 60);
    if (diffH < 24) return `${diffH}小时前`;
    return `${Math.round(diffH / 24)}天前`;
  };
  const formatObjectRef = (ref: any): string => {
    if (!ref) return '';
    const entries = Object.entries(ref);
    const [kind, id] = entries[0] as [string, string];
    const label = kind === 'WorkItem' ? 'WI' : kind === 'Session' ? 'SES' : kind === 'Handoff' ? 'HO' : kind === 'Artifact' ? 'AR' : kind === 'Seat' ? 'SEAT' : kind.toUpperCase();
    return `${label}-${id.replace(/^(wi|ses|ho|ar|seat)-/, '')}`;
  };
  const formatActorRef = (actorRef: any): string => {
    if (actorRef === 'Automation') return 'Automation';
    if (typeof actorRef === 'object' && actorRef.Seat) return getSeatName(actorRef.Seat);
    return String(actorRef);
  };
  const getEventHeadline = (ev: any): string => {
    if (ev.payload?.title) return ev.payload.title;
    if (ev.payload?.summary) return ev.payload.summary;
    return ev.event_type;
  };

  // ── Event color ──
  const eventColor = (type: string): string => {
    if (['WorkItemStatusChanged', 'ReconcileCompleted', 'CheckpointCreated'].includes(type)) return 'var(--sl-green)';
    if (['ArtifactCreated', 'HandoffSent', 'HandoffAccepted', 'HandoffReturned', 'HandoffCompleted'].includes(type)) return 'var(--sl-blue)';
    return 'var(--sl-amber)';
  };

  // ── Type guards ──
  const isBlockedWI = (s: any): s is any => s.status === 'Blocked';
  const isInputRequired = (s: any): s is any => s.status === 'InputRequired';
  const isActiveHandoff = (s: any): s is any => ['Returned', 'Sent', 'Received'].includes(s.status);

  // ═══════════════════════════════════════════════════════
  // A. Blockers projection
  // ═══════════════════════════════════════════════════════
  const projectedBlockers: any[] = (() => {
    if (!truthData) return [];
    const blockers: any[] = [];
    // 1. sessions with InputRequired
    const sessionBlockers = truthData.sessions
      .filter(isInputRequired)
      .map(s => ({
        sourceKind: 'Session' as const,
        sourceId: s.id,
        text: s.prompt_state ? `Prompt blocked: ${getSeatName(s.seat_id)} requires ${s.prompt_state.classification}` : `Session requires input: ${getSeatName(s.seat_id)}`,
        owner: getSeatName(s.seat_id),
        since: formatSince(s.created_at),
        detail: s.prompt_state?.preview,
      }));
    blockers.push(...sessionBlockers);
    // 2. workItems with Blocked
    const wiBlockers = truthData.workItems
      .filter(isBlockedWI)
      .map(w => ({
        sourceKind: 'WorkItem' as const,
        sourceId: w.id,
        text: w.title,
        owner: getSeatName(w.owner_seat_id),
        since: formatSince(w.updated_at),
      }));
    blockers.push(...wiBlockers);
    // 3. handoffs with Returned/Sent/Received
    const hoBlockers = truthData.handoffs
      .filter(isActiveHandoff)
      .map(h => {
        const toSeat = typeof h.to_ref === 'object' && 'Seat' in h.to_ref ? getSeatName((h.to_ref as any).Seat) : getSeatName((h.to_ref as any).Seat);
        const owner = toSeat !== 'Unknown' ? toSeat : getSeatName((h.from_ref as any).Seat);
        return {
          sourceKind: 'Handoff' as const,
          sourceId: h.id,
          text: `Handoff awaiting action: ${h.purpose}`,
          owner,
          since: formatSince(h.created_at),
        };
      });
    blockers.push(...hoBlockers);
    // Sort: sessions first, workitems second, handoffs third; then newest first
    const kindOrder: Record<string, number> = { Session: 0, WorkItem: 1, Handoff: 2 };
    blockers.sort((a, b) => {
      const kd = kindOrder[a.sourceKind] - kindOrder[b.sourceKind];
      if (kd !== 0) return kd;
      return b.sourceId.localeCompare(a.sourceId);
    });
    return blockers.slice(0, 5);
  })();

  // ═══════════════════════════════════════════════════════
  // B. Active-items projection
  // ═══════════════════════════════════════════════════════
  const projectedActiveItems: any[] = (() => {
    if (!truthData) return [];
    const items: any[] = [];
    // Active/InReview/Reopened workItems
    const activeWIs = truthData.workItems
      .filter(w => ['Active', 'InReview', 'Reopened'].includes(w.status))
      .map(w => ({
        sourceKind: 'WorkItem' as const,
        sourceId: w.id,
        title: `${getSeatName(w.owner_seat_id)}: ${w.title}`,
        owner: getSeatName(w.owner_seat_id),
        ownerAvatar: mockSeatOverrides[w.owner_seat_id || '']?.charAt(0).toUpperCase(),
        ownerColor: w.owner_seat_id === 'seat-1' ? 'var(--sl-purple)' : w.owner_seat_id === 'seat-2' ? 'var(--sl-blue)' : w.owner_seat_id === 'seat-4' ? 'var(--sl-green)' : 'var(--sl-amber)',
        statusLabel: w.status === 'Active' ? '推进中' : w.status === 'InReview' ? '审阅中' : '已重开',
        refLabel: w.id,
        description: w.goal,
        reviewTier: w.change_tier_record?.tier as 'L1' | 'L2' | 'L3' | undefined,
      }));
    items.push(...activeWIs);
    // Running/Launching sessions
    const liveSessions = truthData.sessions
      .filter(s => s.status === 'Running' || s.status === 'Launching')
      .map(s => ({
        sourceKind: 'Session' as const,
        sourceId: s.id,
        title: `${getSeatName(s.seat_id)}: ${s.runtime} session`,
        owner: getSeatName(s.seat_id),
        ownerAvatar: mockSeatOverrides[s.seat_id]?.charAt(0).toUpperCase(),
        ownerColor: s.seat_id === 'seat-1' ? 'var(--sl-purple)' : s.seat_id === 'seat-2' ? 'var(--sl-blue)' : s.seat_id === 'seat-4' ? 'var(--sl-green)' : 'var(--sl-amber)',
        statusLabel: s.status === 'Running' ? '运行中' : '启动中',
        refLabel: s.id,
        promptBadge: s.prompt_state?.classification,
      }));
    items.push(...liveSessions);
    // Sort: active workitems first, then live sessions, then newest
    items.sort((a, b) => {
      if (a.sourceKind !== b.sourceKind) return a.sourceKind === 'WorkItem' ? -1 : 1;
      return b.sourceId.localeCompare(a.sourceId);
    });
    return items.slice(0, 6);
  })();

  // ═══════════════════════════════════════════════════════
  // C. Next-step projection (top inbox item)
  // ═══════════════════════════════════════════════════════
  const nextStepSource = (() => {
    if (truthData?.inboxItems?.length) {
      const sorted = [...truthData.inboxItems].sort((a, b) => {
        const p = { Critical: 0, Normal: 1, Low: 2 };
        const pd = p[a.priority] - p[b.priority];
        if (pd !== 0) return pd;
        return b.timestamp.localeCompare(a.timestamp);
      });
      return sorted[0];
    }
    return null;
  })();

  // ═══════════════════════════════════════════════════════
  // D. Activity log projection
  // ═══════════════════════════════════════════════════════
  const projectedEvents = (() => {
    if (!truthData?.events?.length) return [];
    return [...truthData.events]
      .sort((a, b) => b.occurred_at.localeCompare(a.occurred_at))
      .slice(0, 6)
      .map(ev => ({
        eventId: ev.event_id,
        eventType: ev.event_type,
        time: formatClock(ev.occurred_at),
        headline: getEventHeadline(ev),
        actor: formatActorRef(ev.actor_ref),
        objectRefs: ev.object_refs.map(formatObjectRef),
        evidenceRefs: ev.evidence_refs,
        rawTimestamp: ev.occurred_at,
      }));
  })();

  // ═══════════════════════════════════════════════════════
  // E. Chip helper
  // ═══════════════════════════════════════════════════════
  const Chip: React.FC<{ label: string; title?: string; color?: string }> = ({ label, title, color = 'var(--sl-brand)' }) => (
    <span
      title={title || label}
      style={{
        display: 'inline-block', fontSize: 10, fontWeight: 600, padding: '2px 6px',
        borderRadius: 'var(--sl-radius-full)', background: `${color}15`, color, marginRight: 4, marginBottom: 2,
      }}
    >{label}</span>
  );

  // ═══════════════════════════════════════════════════════
  // Enriched hover builders
  // ═══════════════════════════════════════════════════════
  const buildEventHover = (ev: any) => {
    if (!truthData) return ev;
    return {
      ...ev,
      event_id: ev.eventId,
      event_type: ev.eventType,
      occurred_at: ev.rawTimestamp,
      actor_label: ev.actor,
      object_refs_chips: ev.objectRefs,
      evidence_refs_chips: ev.evidenceRefs.map((p: string) => ({ full: p, label: p.split('/').pop() || p })),
    };
  };
  const buildBlockerHover = (b: any) => ({
    sourceFamily: b.sourceKind,
    sourceId: b.sourceId,
    owner: b.owner,
    since: b.since,
    detail: b.detail,
  });
  const buildNodeHover = (node: any) => {
    if (!truthData) return node;
    const match = truthData.workItems.find(w => w.id === node.workItemRef || w.id === `wi-${node.workItemRef?.replace('WI-', '') || ''}`);
    if (!match) return node;
    const artifactCount = truthData.artifacts.filter(a => a.source_workitem_id === match.id).length;
    return {
      ...node,
      workitem_id: match.id,
      status: match.status,
      priority: match.priority,
      ownerSeat: getSeatName(match.owner_seat_id),
      reviewTier: match.change_tier_record?.tier,
      delegationScope: match.active_delegation?.scope,
      artifactCount,
    };
  };
  const buildNextHover = (item: any) => ({
    priority: item.priority,
    actor: item.actor,
    objectRef: formatObjectRef(item.object_ref),
    timestamp: item.timestamp,
    linkedArtifactIds: item.linked_artifact_ids,
  });

  if (!data && !truthData) return <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--sl-text-tertiary)', fontSize: 13 }}>暂无项目数据</div>;

  const { goals, currentGoal, justNow, earlierToday, yesterday } = mockPlanData || { goals: [], currentGoal: { name: '', stages: [], currentStageIndex: 0 }, justNow: [], earlierToday: '', yesterday: '' };
  const stages = currentGoal?.stages || [];
  const ci = currentGoal?.currentStageIndex || 0;

  const useTruth = truthData && (projectedBlockers.length > 0 || projectedActiveItems.length > 0 || projectedEvents.length > 0);

  // ── currentStage: always derived from mock plan for DAG scaffolding ──
  const currentStage = (() => {
    const idx = currentGoal?.currentStageIndex || 0;
    const stageList = currentGoal?.stages || [];
    if (stageList[idx]) {
      const s = stageList[idx];
      // PlanPhase → enriched stage shape expected by downstream panels
      return {
        name: s.name,
        workItemsDone: mockPlanData?.currentStage?.workItemsDone || 0,
        workItemsTotal: mockPlanData?.currentStage?.workItemsTotal || 0,
        workflow: mockPlanData?.currentStage?.workflow || { nodes: [], edges: [] },
        blockers: mockPlanData?.currentStage?.blockers || [],
        nextStep: mockPlanData?.currentStage?.nextStep || '',
      } as any;
    }
    // Fallback: use mock data's currentStage directly
    return mockPlanData?.currentStage as any || {
      name: '', workItemsDone: 0, workItemsTotal: 0, workflow: { nodes: [], edges: [] }, blockers: [], nextStep: ''
    } as any;
  })();

  const currentBlockers = useTruth ? projectedBlockers : (currentStage as any).blockers || [];
  const currentActiveNodes = useTruth ? projectedActiveItems : ((currentStage as any).workflow?.nodes || []).filter((n: any) => n.status === 'active' || n.status === 'blocked');

  const handleMouseMove = (e: React.MouseEvent, type: string, itemData: any) => {
    let enriched = itemData;
    if (type === 'event' && truthData) enriched = buildEventHover(itemData);
    else if (type === 'blocker') enriched = buildBlockerHover(itemData);
    else if (type === 'node') enriched = buildNodeHover(itemData);
    else if (type === 'next') enriched = buildNextHover(itemData);
    setHoveredItem({ type, data: enriched, x: e.clientX + 10, y: e.clientY + 10 });
  };

  return (
    <div style={{ flex: 1, overflowY: 'auto', padding: '24px 16px', display: 'flex', flexDirection: 'column', gap: 16 }}>

      {/* 1. ── Blockers (Unified Panel) ── */}
      {currentBlockers.length > 0 && (
        <div style={{ 
          padding: '12px 14px', borderRadius: 'var(--sl-radius-md)', 
          background: 'var(--sl-surface)', border: '1px solid var(--sl-red)' 
        }}>
          <div style={{ 
            fontSize: 11, fontWeight: 700, color: 'var(--sl-red)', 
            marginBottom: 10, textTransform: 'uppercase', letterSpacing: '0.05em',
            display: 'flex', alignItems: 'center', gap: 6 
          }}>
            <AlertTriangle size={14} /> 目前阻塞 (BLOCKERS)
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {currentBlockers.map((b: any, i: number) => (
              <div key={i} 
                onMouseEnter={e => {
                  const btn = e.currentTarget.querySelector('button');
                  if (btn) btn.style.opacity = '1';
                  handleMouseMove(e, 'blocker', b);
                }}
                onMouseMove={e => handleMouseMove(e, 'blocker', b)}
                onMouseLeave={e => {
                  const btn = e.currentTarget.querySelector('button');
                  if (btn) btn.style.opacity = '0';
                  setHoveredItem(null);
                }}
                style={{
                  display: 'flex', alignItems: 'center', gap: 12,
                  padding: '10px 14px', borderRadius: 'var(--sl-radius-md)',
                  background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
                  transition: 'all 150ms ease',
                }}
              >
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 13, color: 'var(--sl-text-primary)', fontWeight: 600 }}>{b.text}</div>
                  <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)', marginTop: 4, display: 'flex', alignItems: 'center', gap: 4 }}>
                    <span style={{ fontWeight: 600 }}>{b.owner}</span>
                    <span style={{ color: 'var(--sl-text-tertiary)' }}>· 阻塞已持续 {b.since}</span>
                  </div>
                </div>
                <button 
                  style={{
                    opacity: 0, transition: 'opacity 200ms ease',
                    fontSize: 11, fontWeight: 600, padding: '6px 12px', cursor: 'pointer',
                    borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-red)', color: 'white', border: 'none',
                  }}
                >立即处理</button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 2. ── 正在发生 (Active Items) ── */}
      {currentActiveNodes.length > 0 && (
        <div style={{ padding: '12px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
          <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)', marginBottom: 10, textTransform: 'uppercase', letterSpacing: '0.05em', display: 'flex', alignItems: 'center', gap: 6 }}>
            <Activity size={14} /> 正在发生 (ACTIVE)
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {currentActiveNodes.map((item: any) => {
              const color = item.ownerColor || (item.statusLabel === '受阻塞' ? 'var(--sl-red)' : 'var(--sl-brand)');
              return (
                <div key={item.sourceId} 
                  onMouseEnter={e => handleMouseMove(e, 'node', item)}
                  onMouseMove={e => handleMouseMove(e, 'node', item)}
                  onMouseLeave={() => setHoveredItem(null)}
                  style={{
                    padding: '10px 14px', borderRadius: 'var(--sl-radius-md)',
                    border: `1px solid ${color}40`, background: 'var(--sl-bg)',
                    cursor: 'pointer'
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                    {item.ownerAvatar && (
                      <div style={{
                        width: 20, height: 20, borderRadius: '50%',
                        background: `${color}18`, color: color,
                        display: 'flex', alignItems: 'center', justifyContent: 'center',
                        fontSize: 10, fontWeight: 700, border: `1px solid ${color}40`,
                      }}>{item.ownerAvatar}</div>
                    )}
                    <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)', flex: 1 }}>
                      {item.title}
                    </span>
                    <span style={{
                      fontSize: 10, fontWeight: 600, padding: '2px 8px', borderRadius: 'var(--sl-radius-full)',
                      background: `${color}15`, color: color,
                    }}>
                      {item.statusLabel}
                    </span>
                  </div>
                  <div style={{ fontSize: 11, paddingLeft: 28, display: 'flex', alignItems: 'center', gap: 12 }}>
                    {item.sourceKind === 'WorkItem' && (
                      <>
                        <div style={{ color: 'var(--sl-text-tertiary)' }}>
                          评审阶层: {item.reviewTier || 'N/A'}
                        </div>
                        <div style={{ color: 'var(--sl-brand)', fontWeight: 600 }}>{item.refLabel}</div>
                      </>
                    )}
                    {item.sourceKind === 'Session' && (
                      <>
                        <div style={{ color: 'var(--sl-text-tertiary)' }}>
                          会话ID: {item.refLabel}
                        </div>
                        {item.promptBadge && (
                          <Chip label={item.promptBadge} color="var(--sl-amber)" />
                        )}
                      </>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* 3. ── DAG Workflow with Integrated Progress ── */}
      <div style={{ padding: '16px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
          <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            工作流全景 (DAG)
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)' }}>
              进度 <span style={{ fontWeight: 600, color: 'var(--sl-text-primary)' }}>{currentStage.workItemsDone}/{currentStage.workItemsTotal}</span> 项
            </div>
            <div style={{ width: 120, height: 6, borderRadius: 3, background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)', overflow: 'hidden' }}>
              <div style={{ width: `${(currentStage.workItemsDone / currentStage.workItemsTotal) * 100}%`, height: '100%', background: 'var(--sl-green)', transition: 'width 1s ease' }} />
            </div>
          </div>
        </div>
        <DagWorkflow workflow={currentStage.workflow} />
      </div>

      {/* 4. ── Next step (Actionable Loop) ── */}
      <div style={{ padding: '16px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-brand-subtle)', border: '1px solid var(--sl-brand)40' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
          <Zap size={14} className="text-[var(--sl-brand)]" />
          <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--sl-brand)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>系统建议下一步 (NEXT STEP)</span>
        </div>
        
        {/* Interactive Action Card */}
        <div style={{ 
          background: 'var(--sl-surface)', borderRadius: 'var(--sl-radius-md)', padding: '12px 16px',
          border: '1px solid var(--sl-border-light)', boxShadow: 'var(--sl-shadow-sm)',
          cursor: 'pointer', transition: 'all 150ms ease',
          display: 'flex', alignItems: 'center', justifyContent: 'space-between'
        }}
        onMouseEnter={e => handleMouseMove(e, 'next', nextStepSource)}
        onMouseMove={e => handleMouseMove(e, 'next', nextStepSource)}
        onMouseLeave={() => setHoveredItem(null)}
        >
          <div>
            <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 4 }}>
              {nextStepSource ? nextStepSource.summary : (mockPlanData?.currentStage?.nextStep || '暂无待办建议')}
            </div>
            <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
              {nextStepSource ? `${nextStepSource.actor} · ${formatObjectRef(nextStepSource.object_ref)}` : '点击指派相关席位执行此建议，或转化为具体的工作项。'}
            </div>
          </div>
          <div style={{ width: 32, height: 32, borderRadius: '50%', background: 'var(--sl-bg)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--sl-brand)' }}>
            <ArrowRight size={16} />
          </div>
        </div>
      </div>

      {/* 5. ── Timeline (Activity Log, Canonical Events) ── */}
      <div style={{ padding: '12px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
        <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)', marginBottom: 12, textTransform: 'uppercase', letterSpacing: '0.05em', display: 'flex', alignItems: 'center', gap: 6 }}>
          <Clock size={14} /> 活动日志 (ACTIVITY)
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {projectedEvents.map((ev) => (
            <div key={ev.eventId} 
              onMouseEnter={evEnt => {
                evEnt.currentTarget.style.background = 'var(--sl-surface-hover)';
                handleMouseMove(evEnt, 'event', ev);
              }}
              onMouseMove={evEnt => handleMouseMove(evEnt, 'event', ev)}
              onMouseLeave={evEnt => {
                evEnt.currentTarget.style.background = 'var(--sl-bg)';
                setHoveredItem(null);
              }}
              style={{ 
                display: 'flex', alignItems: 'center', gap: 12, padding: '10px 12px', 
                borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-bg)', border: '1px solid var(--sl-border-light)',
                cursor: 'pointer', transition: 'background 150ms ease'
              }}
            >
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4, width: 44, flexShrink: 0 }}>
                <span style={{ color: 'var(--sl-text-tertiary)', fontFamily: 'monospace', fontSize: 11 }}>{ev.time}</span>
                <div style={{ width: 6, height: 6, borderRadius: '50%', background: eventColor(ev.eventType) }} />
              </div>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 13, color: 'var(--sl-text-primary)', fontWeight: 500 }}>{ev.headline}</div>
                <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)', marginTop: 2, display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span>{ev.actor}</span>
                  {ev.objectRefs.length > 0 && <span style={{ color: 'var(--sl-text-tertiary)' }}>{ev.objectRefs.length} refs</span>}
                  {ev.evidenceRefs.length > 0 && <span style={{ color: 'var(--sl-text-tertiary)' }}>{ev.evidenceRefs.length} evidence</span>}
                </div>
              </div>
            </div>
          ))}
        </div>
        <div style={{ marginTop: 12, paddingTop: 12, borderTop: '1px dashed var(--sl-border)' }}>
          <button
            onClick={() => setShowEarlier(!showEarlier)}
            style={{
              display: 'flex', alignItems: 'center', gap: 8, width: '100%',
              padding: '8px 12px', border: 'none', background: 'var(--sl-bg)', borderRadius: 'var(--sl-radius-md)',
              cursor: 'pointer', fontSize: 12, color: 'var(--sl-text-secondary)', fontWeight: 500
            }}
          >
            <span style={{ transform: showEarlier ? 'rotate(90deg)' : 'none', transition: 'transform 150ms ease', display: 'inline-block', fontSize: 14 }}>▸</span>
            <div style={{ flex: 1, textAlign: 'left' }}>
              <span style={{ color: 'var(--sl-text-tertiary)', marginRight: 8 }}>今天早些:</span>
              {earlierToday}
            </div>
          </button>
          {showEarlier && (
            <div className="animate-in fade-in slide-in-from-top-2 duration-200" style={{ padding: '12px 12px 4px 34px', fontSize: 12, color: 'var(--sl-text-secondary)', display: 'flex', flexDirection: 'column', gap: 8 }}>

              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ color: 'var(--sl-text-tertiary)' }}>昨天:</span>
                {yesterday}
              </div>
              <div style={{ color: 'var(--sl-brand)', cursor: 'pointer', fontWeight: 500, marginTop: 4 }}>查看完整账本历史记录 →</div>
            </div>
          )}
        </div>
      </div>

      {/* 6. ── Deep Scroll Context (Goals & Stages) ── */}
      <div style={{ 
        marginTop: 60, paddingTop: 24, borderTop: '2px dashed var(--sl-border-light)',
        opacity: 0.6, transition: 'opacity 300ms ease',
        display: 'flex', flexDirection: 'column', gap: 16
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, color: 'var(--sl-text-tertiary)', fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.05em' }}>
          <Target size={14} /> 宏观背景：项目目标与阶段全景
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
          <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', fontWeight: 600, flexShrink: 0, textTransform: 'uppercase' }}>总目标</span>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, flex: 1, overflowX: 'auto', paddingBottom: 2 }}>
            {goals.map((g, i) => (
              <React.Fragment key={g.name}>
                <span style={{
                  fontSize: 12, fontWeight: g.status === 'active' ? 600 : 500,
                  color: g.status === 'active' ? 'var(--sl-brand)' : g.status === 'done' ? 'var(--sl-text-secondary)' : 'var(--sl-text-tertiary)',
                  padding: g.status === 'active' ? '4px 10px' : '4px 8px',
                  background: g.status === 'active' ? 'var(--sl-brand-subtle)' : 'transparent',
                  borderRadius: 'var(--sl-radius-full)', whiteSpace: 'nowrap',
                }}>
                  {g.status === 'done' && '✓ '}{g.name}
                </span>
                {i < goals.length - 1 && <span style={{ color: 'var(--sl-border)', fontSize: 12 }}>→</span>}
              </React.Fragment>
            ))}
          </div>
        </div>
        <div style={{ padding: '14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
          <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 12 }}>{currentGoal.name} - 详细路线图</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 0, flexWrap: 'wrap', rowGap: 8 }}>
            {stages.map((stage, i, arr) => {
              const dotColor = stage.status === 'done' ? 'var(--sl-green)' : stage.status === 'active' ? 'var(--sl-brand)' : stage.status === 'revisited' ? 'var(--sl-amber)' : 'var(--sl-border)';
              return (
                <React.Fragment key={stage.name}>
                  <div style={{
                    display: 'flex', alignItems: 'center', gap: 6, padding: '4px 10px', borderRadius: 'var(--sl-radius-full)',
                    background: stage.status === 'active' ? 'var(--sl-brand-subtle)' : stage.status === 'revisited' ? 'var(--sl-amber-subtle)' : 'transparent',
                    border: stage.status === 'active' ? '1px solid var(--sl-brand)' : stage.status === 'revisited' ? '1px solid var(--sl-amber)' : '1px solid transparent',
                  }}>
                    <div style={{ width: 8, height: 8, borderRadius: '50%', background: dotColor }} />
                    <span style={{ fontSize: 12, fontWeight: stage.status === 'active' || stage.status === 'revisited' ? 600 : 500, color: stage.status === 'active' ? 'var(--sl-brand)' : stage.status === 'revisited' ? 'var(--sl-amber)' : stage.status === 'done' ? 'var(--sl-text-secondary)' : 'var(--sl-text-tertiary)', whiteSpace: 'nowrap' }}>
                      {stage.name}{stage.status === 'revisited' && ' ⟲'}
                    </span>
                  </div>
                  {i < arr.length - 1 && <span style={{ color: 'var(--sl-border)', fontSize: 12, margin: '0 4px' }}>→</span>}
                </React.Fragment>
              );
            })}
          </div>
          {stages.filter(s => s.status === 'revisited').length > 0 && (
            <div style={{ marginTop: 12, display: 'flex', flexDirection: 'column', gap: 6 }}>
              {stages.filter(s => s.status === 'revisited').map(s => (
                <div key={s.name} style={{ padding: '8px 12px', fontSize: 11, color: 'var(--sl-amber)', background: 'var(--sl-amber-subtle)', border: '1px solid var(--sl-amber)40', borderRadius: 'var(--sl-radius-md)', display: 'flex', alignItems: 'center', gap: 6 }}>
                  <AlertTriangle size={14} />
                  <span style={{ fontWeight: 600 }}>{s.name} 阶段被重新打开</span>
                  {s.revisitReason && <span style={{ color: 'var(--sl-text-secondary)' }}>: {s.revisitReason}</span>}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* ─── Global Hover Popup ─── */}
      {hoveredItem && (
        <div style={{
          position: 'fixed', left: hoveredItem.x, top: hoveredItem.y,
          width: 320, background: 'var(--sl-surface)', borderRadius: 'var(--sl-radius-lg)',
          border: '1px solid var(--sl-border)', boxShadow: 'var(--sl-shadow-lg)',
          padding: '16px', zIndex: 1000, display: 'flex', flexDirection: 'column', gap: 12,
          pointerEvents: 'none'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <div style={{ width: 24, height: 24, borderRadius: '50%', background: 'var(--sl-brand-subtle)', color: 'var(--sl-brand)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <Info size={14} />
            </div>
            <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--sl-text-tertiary)', textTransform: 'uppercase' }}>
              {hoveredItem.type === 'node' ? '工作项详情' : hoveredItem.type === 'blocker' ? '阻塞深度分析' : hoveredItem.type === 'event' ? '活动记录详情' : '建议操作详情'}
            </div>
          </div>

          <div>
            <div style={{ fontSize: 14, fontWeight: 700, color: 'var(--sl-text-primary)', lineHeight: 1.4 }}>
              {hoveredItem.type === 'node' ? hoveredItem.data.label : hoveredItem.type === 'blocker' ? hoveredItem.data.text : hoveredItem.data.text || hoveredItem.data}
            </div>
            {hoveredItem.data.description && (
              <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)', marginTop: 8, lineHeight: 1.6 }}>
                {hoveredItem.data.description}
              </div>
            )}
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 8, paddingTop: 8, borderTop: '1px solid var(--sl-bg)' }}>
             {hoveredItem.type === 'node' && (
               <>
                 <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                   <span style={{ color: 'var(--sl-text-tertiary)' }}>当前负责人</span>
                   <span style={{ fontWeight: 600, color: 'var(--sl-text-secondary)' }}>{hoveredItem.data.owner}</span>
                 </div>
                 <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                   <span style={{ color: 'var(--sl-text-tertiary)' }}>关联 ID</span>
                   <span style={{ fontWeight: 600, color: 'var(--sl-brand)' }}>{hoveredItem.data.workItemRef || 'N/A'}</span>
                 </div>
               </>
             )}
             {hoveredItem.type === 'blocker' && (
               <div style={{ fontSize: 11, color: 'var(--sl-red)', background: 'var(--sl-red-subtle)', padding: '8px', borderRadius: 4 }}>
                 <strong>影响范围：</strong> 该阻塞正在阻碍多个下游节点的推进。
               </div>
             )}
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 4, color: 'var(--sl-brand)', fontSize: 11, fontWeight: 600, marginTop: 4 }}>
            <span>点击进入详细透视图</span>
            <ChevronRight size={12} />
          </div>
        </div>
      )}

    </div>
  );
};

// ═══════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════
// Chat Input with Routing Governance
// ═══════════════════════════════════════════════════════

const ChatInput: React.FC<{
  contact: ChatContact;
  allContacts: ChatContact[];
  input: string;
  onInputChange: (v: string) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
  onEscape: () => void;
  onRouteViaPO: (poId: string) => void;
}> = ({ contact, allContacts, input, onInputChange, inputRef, onEscape, onRouteViaPO }) => {
  const [showRoutingGuard, setShowRoutingGuard] = useState(false);

  const isDirectWorker = contact.type === 'seat' && (contact.seatType === 'worker' || contact.seatType === 'verifier');
  const po = isDirectWorker
    ? allContacts.find(c => c.type === 'seat' && c.seatType === 'po' && c.projectId === contact.projectId)
    : null;

  const handleSend = () => {
    if (!input.trim()) return;
    if (isDirectWorker && !showRoutingGuard) {
      setShowRoutingGuard(true);
      return;
    }
    // Direct send (after guard confirmation)
    setShowRoutingGuard(false);
    onInputChange('');
  };

  const handleRouteViaPO = () => {
    if (po) {
      setShowRoutingGuard(false);
      onRouteViaPO(po.id);
    }
  };

  return (
    <div style={{ borderTop: '1px solid var(--sl-divider)', flexShrink: 0 }}>
      {/* Routing guard banner */}
      {showRoutingGuard && po && (
        <div style={{
          padding: '10px 16px',
          background: 'var(--sl-amber-subtle)',
          borderBottom: '1px solid var(--sl-amber)',
          display: 'flex', flexDirection: 'column', gap: 8,
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span style={{ fontSize: 13 }}>⚡</span>
            <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)' }}>
              直接消息将绕过 {po.name} (PO)
            </span>
          </div>
          <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)', lineHeight: 1.5 }}>
            通过 PO 转发的消息 token 效率更高——PO 会打包为精确的任务包。直接发送将自动通知 {po.name}。
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <button
              onClick={handleRouteViaPO}
              style={{
                padding: '5px 14px', fontSize: 12, fontWeight: 600,
                background: 'var(--sl-brand)', color: 'white',
                border: 'none', borderRadius: 'var(--sl-radius-md)',
                cursor: 'pointer', transition: 'all 120ms ease',
              }}
              onMouseEnter={e => { e.currentTarget.style.background = 'var(--sl-brand-hover)'; }}
              onMouseLeave={e => { e.currentTarget.style.background = 'var(--sl-brand)'; }}
            >
              通过 {po.name} 转发 (推荐)
            </button>
            <button
              onClick={() => { setShowRoutingGuard(false); onInputChange(''); }}
              style={{
                padding: '5px 14px', fontSize: 12, fontWeight: 500,
                background: 'transparent', color: 'var(--sl-text-secondary)',
                border: '1px solid var(--sl-border)', borderRadius: 'var(--sl-radius-md)',
                cursor: 'pointer', transition: 'all 120ms ease',
              }}
            >
              仍然直接发送
            </button>
            <button
              onClick={() => setShowRoutingGuard(false)}
              style={{
                padding: '5px 10px', fontSize: 12,
                background: 'transparent', color: 'var(--sl-text-tertiary)',
                border: 'none', cursor: 'pointer',
              }}
            >
              取消
            </button>
          </div>
        </div>
      )}

      {/* Input area */}
      <div style={{ padding: '10px 16px 12px' }}>
        {/* Direct worker hint (subtle, always visible) */}
        {isDirectWorker && po && !showRoutingGuard && (
          <div style={{
            display: 'flex', alignItems: 'center', gap: 6,
            marginBottom: 6, fontSize: 11, color: 'var(--sl-text-tertiary)',
          }}>
            <span>⚡</span>
            <span>直接消息 · 将通知 {po.name}</span>
            <span style={{ color: 'var(--sl-border)' }}>|</span>
            <button
              onClick={() => po && onRouteViaPO(po.id)}
              style={{
                background: 'transparent', border: 'none', cursor: 'pointer',
                fontSize: 11, color: 'var(--sl-brand)', fontWeight: 500,
                padding: 0,
              }}
            >
              通过 {po.name} 转发 →
            </button>
          </div>
        )}

        <div style={{
          display: 'flex', alignItems: 'center', gap: 8,
          background: 'var(--sl-bg)',
          border: `1.5px solid ${isDirectWorker ? 'var(--sl-amber)' : 'var(--sl-border)'}`,
          borderRadius: 'var(--sl-radius-lg)', padding: '8px 12px',
          transition: 'border-color 120ms ease, box-shadow 120ms ease',
        }}
          onFocus={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; e.currentTarget.style.boxShadow = '0 0 0 3px var(--sl-brand-subtle)'; }}
          onBlur={e => { e.currentTarget.style.borderColor = isDirectWorker ? 'var(--sl-amber)' : 'var(--sl-border)'; e.currentTarget.style.boxShadow = 'none'; }}
        >
          <input
            ref={inputRef} type="text" value={input}
            onChange={e => onInputChange(e.target.value)}
            placeholder={`发消息给 ${contact.name}...`}
            style={{ flex: 1, border: 'none', outline: 'none', fontSize: 13, color: 'var(--sl-text-primary)', background: 'transparent' }}
            onKeyDown={e => {
              if (e.key === 'Escape') onEscape();
              if (e.key === 'Enter' && input.trim()) handleSend();
            }}
          />
          <button
            onClick={handleSend}
            style={{
              padding: '4px 12px', fontSize: 12, fontWeight: 600,
              color: input.trim() ? 'white' : 'var(--sl-text-tertiary)',
              background: input.trim() ? 'var(--sl-brand)' : 'var(--sl-surface-hover)',
              border: 'none', borderRadius: 'var(--sl-radius-md)',
              cursor: input.trim() ? 'pointer' : 'default', transition: 'all 120ms ease',
            }}
          >发送</button>
        </div>

        <div style={{ display: 'flex', gap: 8, marginTop: 6 }}>
          {['📎 附件', '📋 任务包', '🔍 证据'].map(a => (
            <button key={a} style={{
              fontSize: 11, color: 'var(--sl-text-tertiary)', background: 'transparent',
              border: 'none', cursor: 'pointer', padding: '2px 0', transition: 'color 120ms ease',
            }}
              onMouseEnter={e => { e.currentTarget.style.color = 'var(--sl-brand)'; }}
              onMouseLeave={e => { e.currentTarget.style.color = 'var(--sl-text-tertiary)'; }}
            >{a}</button>
          ))}
        </div>
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════════════════
// IM-style Supervisor Panel
// ═══════════════════════════════════════════════════════

const STORAGE_KEY_POS = 'sl-supervisor-pos';
const STORAGE_KEY_SIZE = 'sl-supervisor-size';

function loadSaved() {
  try {
    return {
      pos: JSON.parse(localStorage.getItem(STORAGE_KEY_POS) || 'null'),
      size: JSON.parse(localStorage.getItem(STORAGE_KEY_SIZE) || 'null'),
    };
  } catch { return { pos: null, size: null }; }
}

const SupervisorPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [activeContactId, setActiveContactId] = useState(() => {
    const saved = localStorage.getItem('sl-supervisor-active-contact');
    // Validate saved contact still exists
    if (saved && MOCK_CONTACTS.some(c => c.id === saved)) return saved;
    return 'supervisor';
  });
  const [input, setInput] = useState(() => localStorage.getItem(`sl-supervisor-draft-${localStorage.getItem('sl-supervisor-active-contact') || 'supervisor'}`) || '');
  const [search, setSearch] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  // Position & size persistence
  const savedRef = useRef(loadSaved());
  const [pos, setPos] = useState({ x: savedRef.current.pos?.x ?? -1, y: savedRef.current.pos?.y ?? -1 });
  const [size, setSize] = useState({ w: savedRef.current.size?.w ?? 680, h: savedRef.current.size?.h ?? 520 });
  const [dragging, setDragging] = useState(false);
  const [resizing, setResizing] = useState(false);
  const dragOff = useRef({ x: 0, y: 0 });
  const resizeStart = useRef({ x: 0, y: 0, w: 0, h: 0 });
  const panelRef = useRef<HTMLDivElement>(null);

  // Center on first open
  useEffect(() => {
    if (pos.x === -1) {
      setPos({ x: Math.max(0, (window.innerWidth - size.w) / 2), y: Math.max(40, (window.innerHeight - size.h) / 2) });
    }
    inputRef.current?.focus();
  }, []);

  // Persist active contact
  useEffect(() => {
    localStorage.setItem('sl-supervisor-active-contact', activeContactId);
  }, [activeContactId]);

  // Persist draft per contact
  useEffect(() => {
    const key = `sl-supervisor-draft-${activeContactId}`;
    if (input) localStorage.setItem(key, input);
    else localStorage.removeItem(key);
  }, [input, activeContactId]);

  // When switching contact, load that contact's draft
  const switchContact = (id: string) => {
    // Save current draft
    const curKey = `sl-supervisor-draft-${activeContactId}`;
    if (input) localStorage.setItem(curKey, input);
    else localStorage.removeItem(curKey);
    // Switch
    setActiveContactId(id);
    setInput(localStorage.getItem(`sl-supervisor-draft-${id}`) || '');
    setSearch('');
  };

  // Drag
  const onDragStart = useCallback((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('input, button, [data-no-drag]')) return;
    setDragging(true);
    const r = panelRef.current!.getBoundingClientRect();
    dragOff.current = { x: e.clientX - r.left, y: e.clientY - r.top };
    e.preventDefault();
  }, []);

  useEffect(() => {
    if (!dragging) return;
    const move = (e: MouseEvent) => setPos({ x: Math.max(0, Math.min(window.innerWidth - 100, e.clientX - dragOff.current.x)), y: Math.max(0, Math.min(window.innerHeight - 60, e.clientY - dragOff.current.y)) });
    const up = () => { setDragging(false); setPos(p => { localStorage.setItem(STORAGE_KEY_POS, JSON.stringify(p)); return p; }); };
    window.addEventListener('mousemove', move); window.addEventListener('mouseup', up);
    return () => { window.removeEventListener('mousemove', move); window.removeEventListener('mouseup', up); };
  }, [dragging]);

  // Resize
  const onResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault(); e.stopPropagation(); setResizing(true);
    const r = panelRef.current!.getBoundingClientRect();
    resizeStart.current = { x: e.clientX, y: e.clientY, w: r.width, h: r.height };
  }, []);

  useEffect(() => {
    if (!resizing) return;
    const move = (e: MouseEvent) => setSize({ w: Math.max(480, Math.min(1000, resizeStart.current.w + e.clientX - resizeStart.current.x)), h: Math.max(360, Math.min(800, resizeStart.current.h + e.clientY - resizeStart.current.y)) });
    const up = () => { setResizing(false); setSize(s => { localStorage.setItem(STORAGE_KEY_SIZE, JSON.stringify(s)); return s; }); };
    window.addEventListener('mousemove', move); window.addEventListener('mouseup', up);
    return () => { window.removeEventListener('mousemove', move); window.removeEventListener('mouseup', up); };
  }, [resizing]);

  // Escape
  useEffect(() => {
    const fn = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', fn); return () => window.removeEventListener('keydown', fn);
  }, [onClose]);

  // Data
  const activeContact = MOCK_CONTACTS.find(c => c.id === activeContactId) || MOCK_CONTACTS[0];
  const messages = MOCK_MESSAGES[activeContactId] || [];

  // Group contacts by project
  const projects = new Map<string, ChatContact[]>();
  const channels: ChatContact[] = [];
  const supervisorContact = MOCK_CONTACTS.find(c => c.type === 'supervisor')!;

  MOCK_CONTACTS.forEach(c => {
    if (c.type === 'supervisor') return;
    if (c.type === 'project-channel') { channels.push(c); return; }
    const key = c.projectId || '_';
    if (!projects.has(key)) projects.set(key, []);
    projects.get(key)!.push(c);
  });

  const filteredContacts = search.trim()
    ? MOCK_CONTACTS.filter(c => c.name.toLowerCase().includes(search.toLowerCase()) || (c.role || '').includes(search))
    : null;

  const listWidth = 220;

  return (
    <>
      {/* Backdrop */}
      <div onClick={onClose} style={{ position: 'fixed', inset: 0, zIndex: 9998, background: 'rgba(0,0,0,0.08)', backdropFilter: 'blur(1px)' }} />

      {/* Panel */}
      <div ref={panelRef} style={{
        position: 'fixed',
        left: pos.x === -1 ? '50%' : pos.x, top: pos.y === -1 ? '50%' : pos.y,
        transform: pos.x === -1 ? 'translate(-50%,-50%)' : 'none',
        zIndex: 9999, width: size.w, height: size.h,
        background: 'var(--sl-surface)', border: '1px solid var(--sl-border)',
        borderRadius: 'var(--sl-radius-xl)', boxShadow: 'var(--sl-shadow-overlay)',
        display: 'flex', flexDirection: 'column',
        cursor: dragging ? 'grabbing' : 'default',
        userSelect: (dragging || resizing) ? 'none' : 'auto',
        overflow: 'hidden',
      }}>

        {/* ── Invisible drag handle (top edge) ── */}
        <div
          style={{ height: 6, cursor: 'grab', flexShrink: 0 }}
          onMouseDown={e => {
            setDragging(true);
            const r = panelRef.current!.getBoundingClientRect();
            dragOff.current = { x: e.clientX - r.left, y: e.clientY - r.top };
            e.preventDefault();
          }}
        />

        {/* ── Body: sidebar + chat ── */}
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>

          {/* ── Left: Contact list ── */}
          <div data-no-drag style={{
            width: listWidth, flexShrink: 0,
            borderRight: '1px solid var(--sl-divider)',
            display: 'flex', flexDirection: 'column',
            background: 'var(--sl-bg)',
            cursor: 'default',
          }}>
            {/* Search */}
            <div style={{ padding: '8px 10px' }}>
              <input
                type="text" value={search} onChange={e => setSearch(e.target.value)}
                placeholder="搜索席位或项目..."
                style={{
                  width: '100%', padding: '6px 10px', fontSize: 12,
                  border: '1px solid var(--sl-border-light)', borderRadius: 'var(--sl-radius-md)',
                  background: 'var(--sl-surface)', outline: 'none', color: 'var(--sl-text-primary)',
                }}
                onFocus={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; }}
                onBlur={e => { e.currentTarget.style.borderColor = 'var(--sl-border-light)'; }}
              />
            </div>

            {/* Contact list */}
            <div style={{ flex: 1, overflowY: 'auto' }}>
              {filteredContacts ? (
                // Search results
                filteredContacts.map(c => (
                  <ContactRow key={c.id} contact={c} active={c.id === activeContactId} onClick={() => { switchContact(c.id); }} />
                ))
              ) : (
                <>
                  {/* Supervisor */}
                  <ContactRow contact={supervisorContact} active={activeContactId === 'supervisor'} onClick={() => switchContact('supervisor')} />

                  {/* Project channels */}
                  {channels.length > 0 && (
                    <div style={{ padding: '10px 12px 4px', fontSize: 10, fontWeight: 600, color: 'var(--sl-text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>频道</div>
                  )}
                  {channels.map(c => (
                    <ContactRow key={c.id} contact={c} active={c.id === activeContactId} onClick={() => switchContact(c.id)} />
                  ))}

                  {/* Seats grouped by project */}
                  {Array.from(projects.entries()).map(([projId, seats]) => {
                    const projChannel = channels.find(c => c.projectId === projId);
                    const projName = projChannel?.name || projId;
                    return (
                      <React.Fragment key={projId}>
                        <div style={{ padding: '10px 12px 4px', fontSize: 10, fontWeight: 600, color: 'var(--sl-text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                          {projName}
                        </div>
                        {seats.map(c => (
                          <ContactRow key={c.id} contact={c} active={c.id === activeContactId} onClick={() => switchContact(c.id)} />
                        ))}
                      </React.Fragment>
                    );
                  })}
                </>
              )}
            </div>
          </div>

          {/* ── Right: Context-aware content ── */}
          <div data-no-drag style={{ flex: 1, display: 'flex', flexDirection: 'column', cursor: 'default', minWidth: 0 }}>

            {/* Header */}
            <div style={{
              display: 'flex', alignItems: 'center', gap: 10,
              padding: '10px 16px', borderBottom: '1px solid var(--sl-divider)', flexShrink: 0,
            }}>
              <Avatar char={activeContact.avatar} color={activeContact.color} size={32} online={activeContact.type === 'seat' ? activeContact.online : undefined} />
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{activeContact.name}</div>
                <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
                  {activeContact.type === 'project-channel' ? '项目工作台' : activeContact.role || '全局监督'}
                </div>
              </div>
              <button onClick={onClose} style={{
                width: 24, height: 24, borderRadius: 'var(--sl-radius-sm)', border: 'none',
                background: 'transparent', color: 'var(--sl-text-tertiary)', cursor: 'pointer',
                display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 14,
              }}
                onMouseEnter={e => { e.currentTarget.style.background = 'var(--sl-surface-hover)'; }}
                onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
              >✕</button>
            </div>

            {activeContact.type === 'project-channel' ? (
              /* ════ Project Channel: Workflow Dashboard ════ */
              <ProjectDashboard channelId={activeContact.id} projectId={activeContact.projectId} />
            ) : (
              /* ════ Seat / Supervisor: IM Chat ════ */
              <>
                {/* Messages */}
                <div style={{ flex: 1, overflowY: 'auto', padding: '16px', display: 'flex', flexDirection: 'column', gap: 10 }}>
                  {messages.length === 0 && (
                    <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                      <p style={{ fontSize: 13, color: 'var(--sl-text-tertiary)' }}>开始和 {activeContact.name} 对话</p>
                    </div>
                  )}
                  {messages.map(msg => <MessageBubble key={msg.id} msg={msg} />)}
                </div>

                {/* Routing governance + Input */}
                <ChatInput
                  contact={activeContact}
                  allContacts={MOCK_CONTACTS}
                  input={input}
                  onInputChange={setInput}
                  inputRef={inputRef}
                  onEscape={onClose}
                  onRouteViaPO={(poId) => switchContact(poId)}
                />
              </>
            )}
          </div>
        </div>

        {/* Resize handle */}
        <div onMouseDown={onResizeStart} style={{
          position: 'absolute', bottom: 2, right: 2, width: 14, height: 14,
          cursor: 'nwse-resize', display: 'flex', alignItems: 'center', justifyContent: 'center', opacity: 0.3,
        }}>
          <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
            <path d="M7 1L1 7M7 4L4 7" stroke="var(--sl-text-tertiary)" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </div>
      </div>

      <style>{`
        @keyframes sv-fadeIn { from { opacity: 0; } to { opacity: 1; } }
      `}</style>
    </>
  );
};

// ── Contact Row ──
const ContactRow: React.FC<{ contact: ChatContact; active: boolean; onClick: () => void }> = ({ contact, active, onClick }) => (
  <button onClick={onClick} style={{
    width: '100%', display: 'flex', alignItems: 'center', gap: 8,
    padding: '8px 12px', border: 'none', textAlign: 'left',
    background: active ? 'var(--sl-surface)' : 'transparent',
    cursor: 'pointer', transition: 'background 100ms ease',
    borderLeft: active ? '2px solid var(--sl-brand)' : '2px solid transparent',
  }}
    onMouseEnter={e => { if (!active) e.currentTarget.style.background = 'var(--sl-surface-hover)'; }}
    onMouseLeave={e => { if (!active) e.currentTarget.style.background = 'transparent'; }}
  >
    <Avatar char={contact.avatar} color={contact.color} size={32} online={contact.type === 'seat' ? contact.online : undefined} />
    <div style={{ flex: 1, minWidth: 0 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{ fontSize: 12, fontWeight: active ? 600 : 500, color: 'var(--sl-text-primary)', truncate: true } as any}>{contact.name}</span>
        {contact.lastTime && <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', flexShrink: 0 }}>{contact.lastTime}</span>}
      </div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: 1 }}>
        <span style={{
          fontSize: 11, color: 'var(--sl-text-tertiary)',
          overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 120,
        }}>{contact.lastMessage || contact.role || ''}</span>
        {contact.unread > 0 && (
          <span style={{
            minWidth: 16, height: 16, borderRadius: 8,
            background: 'var(--sl-brand)', color: 'white',
            fontSize: 10, fontWeight: 600,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            padding: '0 4px', flexShrink: 0,
          }}>{contact.unread}</span>
        )}
      </div>
    </div>
  </button>
);

// ═══════════════════════════════════════════════════════
// Main App
// ═══════════════════════════════════════════════════════

const AppV2: React.FC = () => {
  const [showSupervisor, setShowSupervisor] = useState(() => localStorage.getItem('sl-supervisor-open') === 'true');

  // Persist open state
  useEffect(() => {
    localStorage.setItem('sl-supervisor-open', showSupervisor ? 'true' : 'false');
  }, [showSupervisor]);

  useEffect(() => {
    const fn = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') { e.preventDefault(); setShowSupervisor(p => !p); }
    };
    window.addEventListener('keydown', fn); return () => window.removeEventListener('keydown', fn);
  }, []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: 'var(--sl-bg)', overflow: 'hidden' }}>

      {/* ═══ Top Bar ═══ */}
      <header style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        height: 52, padding: '0 20px',
        background: 'var(--sl-surface)', borderBottom: '1px solid var(--sl-border)', flexShrink: 0,
      }}>
        {/* Left */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer' }}>
            <SeatLoomLogo />
            <span style={{ fontSize: 15, fontWeight: 700, color: 'var(--sl-text-primary)', letterSpacing: '-0.02em' }}>SeatLoom</span>
          </div>
          <div style={{ width: 1, height: 20, background: 'var(--sl-border)' }} />
          <button style={{
            display: 'flex', alignItems: 'center', gap: 6, padding: '4px 10px',
            background: 'var(--sl-surface-hover)', border: '1px solid var(--sl-border-light)',
            borderRadius: 'var(--sl-radius-md)', cursor: 'pointer', fontSize: 13, fontWeight: 500, color: 'var(--sl-text-primary)',
          }}>
            <span style={{ fontSize: 14 }}>📁</span>
            <span>SeatLoom 主项目</span>
            <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>▾</span>
          </button>
        </div>

        {/* Center: Supervisor trigger */}
        <button onClick={() => setShowSupervisor(true)} style={{
          display: 'flex', alignItems: 'center', gap: 10, padding: '6px 16px',
          background: 'var(--sl-bg)', border: '1px solid var(--sl-border)',
          borderRadius: 'var(--sl-radius-lg)', cursor: 'pointer', fontSize: 13,
          color: 'var(--sl-text-tertiary)', minWidth: 300, transition: 'all 120ms ease',
        }}
          onMouseEnter={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; e.currentTarget.style.color = 'var(--sl-text-secondary)'; }}
          onMouseLeave={e => { e.currentTarget.style.borderColor = 'var(--sl-border)'; e.currentTarget.style.color = 'var(--sl-text-tertiary)'; }}
        >
          <span style={{ fontSize: 14, opacity: 0.5 }}>💬</span>
          <span style={{ flex: 1, textAlign: 'left' }}>打开 Supervisor...</span>
          <kbd style={{
            padding: '1px 6px', fontSize: 11, fontWeight: 500,
            background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)',
            borderRadius: 'var(--sl-radius-sm)', fontFamily: 'inherit',
          }}>⌘K</kbd>
        </button>

        {/* Right */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 20 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <StatusDot color="var(--sl-red)" label="阻塞" value={1} />
            <StatusDot color="var(--sl-amber)" label="待决" value={3} />
            <StatusDot color="var(--sl-green)" label="进行" value={5} />
          </div>
          <div style={{ width: 1, height: 20, background: 'var(--sl-border)' }} />
          <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
            {MOCK_CONTACTS.filter(c => c.type === 'seat').map(c => (
              <Avatar key={c.id} char={c.avatar} color={c.color} size={28} online={c.online} />
            ))}
          </div>
        </div>
      </header>

      {/* ═══ Workspace ═══ */}
      <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 40 }}>
        <div style={{ textAlign: 'center', maxWidth: 480 }}>
          <div style={{ fontSize: 48, marginBottom: 20, opacity: 0.12 }}>💬</div>
          <h2 style={{ fontSize: 20, fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 8 }}>监督工作台</h2>
          <p style={{ fontSize: 14, color: 'var(--sl-text-secondary)', lineHeight: 1.7, marginBottom: 28 }}>
            通过 Supervisor 与席位团队协作。
            <br />按 <kbd style={{ padding: '1px 6px', fontSize: 12, background: 'var(--sl-surface-hover)', border: '1px solid var(--sl-border-light)', borderRadius: 'var(--sl-radius-sm)' }}>⌘K</kbd> 打开对话。
          </p>
          <button onClick={() => setShowSupervisor(true)} style={{
            padding: '10px 24px', fontSize: 14, fontWeight: 600, color: 'white',
            background: 'var(--sl-brand)', border: 'none', borderRadius: 'var(--sl-radius-md)',
            cursor: 'pointer', boxShadow: '0 1px 3px rgba(47,111,235,0.3)', transition: 'all 120ms ease',
          }}
            onMouseEnter={e => { e.currentTarget.style.background = 'var(--sl-brand-hover)'; }}
            onMouseLeave={e => { e.currentTarget.style.background = 'var(--sl-brand)'; }}
          >打开 Supervisor</button>
        </div>
      </div>

      {/* ═══ Floating Supervisor ═══ */}
      {showSupervisor && <SupervisorPanel onClose={() => setShowSupervisor(false)} />}
    </div>
  );
};

export default AppV2;
