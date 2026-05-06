/**
 * ProjectDashboard.tsx
 * 项目频道主视图：根据 channelId 加载频道数据，管理 UI 状态（展开/折叠），按优先级顺序渲染各区块
 * 渲染顺序（从细节到整体）：BlockersSection → ActiveWorkSection → DagWorkflow → 下一步 → TimelineSection → StageProgressSection → GoalsSection
 * Tab 导航：看板（overview）| 待办（inbox）| 工作项（workitems）| 文档（artifacts）
 */

import React, { useState, useRef, useEffect } from 'react';
import { Zap, ArrowRight, Info, ChevronRight, FileText } from 'lucide-react';
import { StageWorkflow } from '../dag-model';
import DagWorkflow from '../DagWorkflow';
import { useDataStore } from '../../stores/useDataStore';
import { TimelineEntry } from '../types';
import { MOCK_CHANNEL_DATA } from '../mock-data';
import { BlockersSection } from './BlockersSection';
import { ActiveWorkSection } from './ActiveWorkSection';
import { TimelineSection } from './TimelineSection';
import { StageProgressSection } from './StageProgressSection';
import { GoalsSection } from './GoalsSection';
import { InboxView } from '../views/InboxView';
import { WorkItemsView } from '../views/WorkItemsView';
import { ArtifactsView } from '../views/ArtifactsView';

const Chip: React.FC<{ label: string; title?: string; color?: string }> = ({ label, title, color = 'var(--sl-brand)' }) => (
  <span
    title={title || label}
    style={{
      display: 'inline-block', fontSize: 10, fontWeight: 600, padding: '2px 6px',
      borderRadius: 'var(--sl-radius-full)', background: `${color}15`, color, marginRight: 4, marginBottom: 2,
    }}
  >{label}</span>
);

export const ProjectDashboard: React.FC<{ channelId: string; projectId?: string }> = ({ channelId, projectId }) => {
  const data = MOCK_CHANNEL_DATA[channelId];
  const { projectData, removeInboxItemFromProject } = useDataStore();
  const [showEarlier, setShowEarlier] = useState(false);
  const [hoveredItem, setHoveredItem] = useState<{ type: string; data: any; x: number; y: number } | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'inbox' | 'workitems' | 'artifacts'>('overview');
  const scrollLockRef = useRef(false);
  const scrollTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined!);

  useEffect(() => {
    const el = document.getElementById('project-dashboard-scroll');
    if (!el) return;
    const lock = () => { scrollLockRef.current = true; };
    const unlock = () => { scrollTimerRef.current = setTimeout(() => { scrollLockRef.current = false; }, 150); };
    el.addEventListener('scroll', lock, { passive: true });
    el.addEventListener('wheel', lock, { passive: true });
    el.addEventListener('scrollend', unlock);
    return () => {
      el.removeEventListener('scroll', lock);
      el.removeEventListener('wheel', lock);
      el.removeEventListener('scrollend', unlock);
      if (scrollTimerRef.current) clearTimeout(scrollTimerRef.current);
    };
  }, []);

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
  const formatInboxObjectRef = (ref: string): string => {
    if (!ref) return '';
    return ref;
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
  // C-bis. Artifact projection (Slice B)
  // ═══════════════════════════════════════════════════════
  const getBasename = (path: string): string => {
    const parts = path.split('/');
    return parts[parts.length - 1] || path;
  };
  const formatArtifactSourceRefs = (a: any): string[] => {
    const refs: string[] = [];
    if (a.source_workitem_id) refs.push(a.source_workitem_id);
    if (a.source_session_id) refs.push(a.source_session_id);
    if (a.source_handoff_id) refs.push(a.source_handoff_id);
    return refs;
  };
  const projectedArtifacts = (() => {
    if (!truthData?.artifacts?.length) return [];
    return [...truthData.artifacts]
      .sort((a, b) => b.created_at.localeCompare(a.created_at))
      .slice(0, 5)
      .map(a => ({
        id: a.id,
        title: a.title,
        template: a.template,
        subtype: a.subtype,
        status: a.status,
        storageBasename: getBasename(a.storage_path),
        author: a.author,
        date: a.date,
        sourceRefs: formatArtifactSourceRefs(a),
        summary: a.summary,
        summaryPoints: a.summary_points,
        contentPreview: a.content_preview,
        fullPath: a.storage_path,
      }));
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
  // Enriched hover builders
  // ═══════════════════════════════════════════════════════
  const buildArtifactHover = (art: any) => ({
    title: art.title,
    template: art.template,
    subtype: art.subtype,
    status: art.status || '—',
    fullPath: art.fullPath,
    sourceRefs: art.sourceRefs,
    summary: art.summary,
    summaryPoints: (art.summaryPoints || []).slice(0, 3),
    contentPreview: (art.contentPreview || []).slice(0, 3),
  });
  const buildEventHover = (ev: any) => {
    // Guard: mock justNow rows (text/type) bypass truth-only enrichment
    if (!ev.eventId && !ev.evidenceRefs) return ev;
    if (!truthData) return ev;
    const text = ev.headline || ev.event_type || '活动记录';
    return {
      ...ev,
      text: typeof text === 'string' ? text : String(text),
      event_id: ev.eventId,
      event_type: ev.eventType,
      occurred_at: ev.rawTimestamp,
      actor_label: ev.actor,
      object_refs_chips: ev.objectRefs,
      evidence_refs_chips: (ev.evidenceRefs || []).map((p: string) => ({ full: p, label: p.split('/').pop() || p })),
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
    priority: item?.priority || 'Low',
    actor: item?.actor || '—',
    objectRef: item?.object_ref ? formatInboxObjectRef(item.object_ref) : '',
    timestamp: item?.timestamp || '—',
    linkedArtifactIds: item?.linked_artifact_ids || [],
  });

  if (!data && !truthData) return <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--sl-text-tertiary)', fontSize: 13 }}>暂无项目数据</div>;

  const { goals, currentGoal, justNow, earlierToday, yesterday } = mockPlanData || { goals: [], currentGoal: { name: '', stages: [], currentStageIndex: 0 }, justNow: [], earlierToday: '', yesterday: '' };
  const stages = currentGoal?.stages || [];

  const useTruth = truthData && (projectedBlockers.length > 0 || projectedActiveItems.length > 0 || projectedEvents.length > 0);

  // ── currentStage: always derived from mock plan for DAG scaffolding ──
  const currentStage = (() => {
    const idx = currentGoal?.currentStageIndex || 0;
    const stageList = currentGoal?.stages || [];
    if (stageList[idx]) {
      const s = stageList[idx];
      return {
        name: s.name,
        workItemsDone: mockPlanData?.currentStage?.workItemsDone || 0,
        workItemsTotal: mockPlanData?.currentStage?.workItemsTotal || 0,
        workflow: mockPlanData?.currentStage?.workflow || { nodes: [], edges: [] },
        blockers: mockPlanData?.currentStage?.blockers || [],
        nextStep: mockPlanData?.currentStage?.nextStep || '',
      } as any;
    }
    return mockPlanData?.currentStage as any || {
      name: '', workItemsDone: 0, workItemsTotal: 0, workflow: { nodes: [], edges: [] }, blockers: [], nextStep: ''
    } as any;
  })();

  const currentBlockers = useTruth ? projectedBlockers : (currentStage as any).blockers || [];
  const currentActiveNodes = useTruth ? projectedActiveItems : ((currentStage as any).workflow?.nodes || []).filter((n: any) => n.status === 'active' || n.status === 'blocked');

  const handleMouseMove = (e: React.MouseEvent, type: string, itemData: any) => {
    if (scrollLockRef.current) return;
    let enriched = itemData;
    if (type === 'event' && truthData) enriched = buildEventHover(itemData);
    else if (type === 'blocker') enriched = buildBlockerHover(itemData);
    else if (type === 'node') enriched = buildNodeHover(itemData);
    else if (type === 'next') enriched = buildNextHover(itemData);
    else if (type === 'artifact') enriched = buildArtifactHover(itemData);
    setHoveredItem({ type, data: enriched, x: e.clientX + 10, y: e.clientY + 10 });
  };

  // ── Tab badge counts ──
  const inboxCount = truthData ? truthData.inboxItems.length : 0;
  const activeWorkCount = truthData
    ? truthData.workItems.filter(w => !['Done', 'Verified'].includes(w.status)).length
    : 0;
  const artifactCount = truthData ? truthData.artifacts.length : 0;

  const TAB_DEFS = [
    { id: 'overview'   as const, label: '看板',  count: 0 },
    { id: 'inbox'      as const, label: '待办',  count: inboxCount },
    { id: 'workitems'  as const, label: '工作项', count: activeWorkCount },
    { id: 'artifacts'  as const, label: '文档',  count: artifactCount },
  ];

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>

      {/* ── Tab navigation (only when truth data exists) ── */}
      {truthData && (
        <div style={{ display: 'flex', borderBottom: '1px solid var(--sl-divider)', background: 'var(--sl-surface)', flexShrink: 0, padding: '0 16px' }}>
          {TAB_DEFS.map(tab => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                style={{
                  padding: '10px 14px', fontSize: 12, fontWeight: 600, border: 'none',
                  background: 'transparent', cursor: 'pointer',
                  borderBottom: `2px solid ${isActive ? 'var(--sl-brand)' : 'transparent'}`,
                  color: isActive ? 'var(--sl-brand)' : 'var(--sl-text-tertiary)',
                  transition: 'all 120ms ease', display: 'flex', alignItems: 'center', gap: 6,
                }}
              >
                {tab.label}
                {tab.count > 0 && (
                  <span style={{
                    fontSize: 10, fontWeight: 700, padding: '1px 5px',
                    borderRadius: 'var(--sl-radius-full)',
                    background: tab.id === 'inbox' ? 'var(--sl-red)' : 'var(--sl-brand-subtle)',
                    color: tab.id === 'inbox' ? 'white' : 'var(--sl-brand)',
                  }}>{tab.count}</span>
                )}
              </button>
            );
          })}
        </div>
      )}

      {/* ── P0 views: Inbox / WorkItems / Artifacts ── */}
      {activeTab === 'inbox' && truthData && (
        <div style={{ flex: 1, overflowY: 'auto', padding: '16px' }}>
          <InboxView
            items={[...truthData.inboxItems]}
            artifacts={[...truthData.artifacts]}
            onDismiss={id => projectId && removeInboxItemFromProject(projectId, id)}
          />
        </div>
      )}
      {activeTab === 'workitems' && truthData && (
        <div style={{ flex: 1, overflowY: 'auto', padding: '16px' }}>
          <WorkItemsView workItems={[...truthData.workItems]} seats={[...truthData.seats]} />
        </div>
      )}
      {activeTab === 'artifacts' && truthData && (
        <div style={{ flex: 1, overflowY: 'auto', padding: '16px' }}>
          <ArtifactsView artifacts={[...truthData.artifacts]} />
        </div>
      )}

      {/* ── Overview tab: full project dashboard ── */}
      {activeTab === 'overview' && (
      <div id="project-dashboard-scroll" style={{ flex: 1, overflowY: 'auto', padding: '24px 16px', display: 'flex', flexDirection: 'column', gap: 16 }}>

      {/* 1. ── Blockers (Unified Panel) ── */}
      <BlockersSection
        blockers={currentBlockers}
        onItemMouseEnter={(e, b) => handleMouseMove(e, 'blocker', b)}
        onItemMouseLeave={() => setHoveredItem(null)}
      />

      {/* 2. ── 正在发生 (Active Items) ── */}
      <ActiveWorkSection
        items={currentActiveNodes}
        onItemMouseEnter={(e, item) => handleMouseMove(e, 'node', item)}
        onItemMouseLeave={() => setHoveredItem(null)}
      />

      {/* 3. ── DAG Workflow with Integrated Progress ── */}
      <div style={{ padding: '16px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
        <StageProgressSection
          stageName=""
          workItemsDone={currentStage.workItemsDone}
          workItemsTotal={currentStage.workItemsTotal}
        />
        <DagWorkflow workflow={currentStage.workflow} />
      </div>

      {/* 4. ── Next step (Actionable Loop) ── */}
      <div style={{ padding: '16px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-brand-subtle)', border: '1px solid var(--sl-brand)40' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
          <Zap size={14} className="text-[var(--sl-brand)]" />
          <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--sl-brand)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>系统建议下一步 (NEXT STEP)</span>
        </div>

        {/* Interactive Action Card */}
        <div
          style={{
            background: 'var(--sl-surface)', borderRadius: 'var(--sl-radius-md)', padding: '12px 16px',
            border: '1px solid var(--sl-border-light)', boxShadow: 'var(--sl-shadow-sm)',
            cursor: 'pointer', transition: 'all 150ms ease',
            display: 'flex', alignItems: 'center', justifyContent: 'space-between'
          }}
          onMouseEnter={e => handleMouseMove(e, 'next', nextStepSource)}
          onMouseLeave={() => setHoveredItem(null)}
        >
          <div>
            <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 4 }}>
              {nextStepSource ? nextStepSource.summary : (mockPlanData?.currentStage?.nextStep || '暂无待办建议')}
            </div>
            <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
              {nextStepSource ? `${nextStepSource.actor} · ${formatInboxObjectRef(nextStepSource.object_ref)}` : '点击指派相关席位执行此建议，或转化为具体的工作项。'}
            </div>
          </div>
          <div style={{ width: 32, height: 32, borderRadius: '50%', background: 'var(--sl-bg)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--sl-brand)' }}>
            <ArrowRight size={16} />
          </div>
        </div>
      </div>

      {/* 4-bis. ── Artifacts & Documents (Truth-Backed) ── */}
      {truthData && (
        <div style={{ padding: '12px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
          <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)', marginBottom: 10, textTransform: 'uppercase', letterSpacing: '0.05em', display: 'flex', alignItems: 'center', gap: 6 }}>
            <FileText size={14} /> 文档与证据 (ARTIFACTS)
          </div>
          {projectedArtifacts.length === 0 ? (
            <div style={{ padding: '16px', textAlign: 'center', color: 'var(--sl-text-tertiary)', fontSize: 12 }}>
              本项目已有结构化 truth，但当前无 artifact 对象
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {projectedArtifacts.map(art => (
                <div key={art.id}
                  onMouseEnter={e => handleMouseMove(e, 'artifact', art)}
                  onMouseLeave={() => setHoveredItem(null)}
                  style={{
                    padding: '10px 14px', borderRadius: 'var(--sl-radius-md)',
                    border: '1px solid var(--sl-border-light)', background: 'var(--sl-bg)',
                    cursor: 'pointer', transition: 'all 150ms ease',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                    <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)', flex: 1 }}>{art.title}</span>
                    <Chip label={art.template} color="var(--sl-brand)" />
                    <Chip label={art.subtype} color="var(--sl-blue)" />
                    {art.status && <Chip label={art.status} color="var(--sl-green)" />}
                  </div>
                  <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span>{art.storageBasename}</span>
                    {art.sourceRefs.length > 0 && (
                      <span style={{ color: 'var(--sl-text-tertiary)', fontSize: 10 }}>{art.sourceRefs.join(' · ')}</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* 5. ── Timeline (Activity Log, Canonical Events) ── */}
      <TimelineSection
        projectedEvents={projectedEvents}
        justNow={justNow}
        showEarlier={showEarlier}
        earlierToday={earlierToday}
        yesterday={yesterday}
        onToggleEarlier={() => setShowEarlier(!showEarlier)}
        onItemMouseEnter={(e, item) => handleMouseMove(e, 'event', item)}
        onItemMouseLeave={() => setHoveredItem(null)}
      />

      {/* 6. ── Deep Scroll Context (Goals & Stages) ── */}
      <GoalsSection goals={goals} currentGoal={currentGoal} stages={stages} />

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
              {hoveredItem.type === 'node' ? '工作项详情' : hoveredItem.type === 'blocker' ? '阻塞深度分析' : hoveredItem.type === 'artifact' ? '文档与证据详情' : hoveredItem.type === 'event' ? '活动记录详情' : '建议操作详情'}
            </div>
          </div>

          <div>
            <div style={{ fontSize: 14, fontWeight: 700, color: 'var(--sl-text-primary)', lineHeight: 1.4 }}>
              {hoveredItem.type === 'node' ? hoveredItem.data.label : hoveredItem.type === 'artifact' ? hoveredItem.data.title : hoveredItem.type === 'event' ? (hoveredItem.data.headline || hoveredItem.data.text || hoveredItem.data.event_type || '活动记录') : hoveredItem.type === 'next' ? (hoveredItem.data.summary || hoveredItem.data.objectRef || '下一步操作') : hoveredItem.type === 'blocker' ? hoveredItem.data.text : hoveredItem.data.text || hoveredItem.data}
            </div>
            {(hoveredItem.type === 'artifact' ? hoveredItem.data.summary : hoveredItem.type === 'event' ? hoveredItem.data.object_refs_chips?.join(', ') : hoveredItem.type === 'next' ? hoveredItem.data.objectRef : hoveredItem.data.description) && (
              <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)', marginTop: 8, lineHeight: 1.6 }}>
                {hoveredItem.type === 'artifact' ? hoveredItem.data.summary : hoveredItem.type === 'event' ? hoveredItem.data.object_refs_chips?.join(', ') : hoveredItem.type === 'next' ? hoveredItem.data.objectRef : hoveredItem.data.description}
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
            {hoveredItem.type === 'artifact' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>模板 / 子类型</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-text-secondary)' }}>{hoveredItem.data.template} · {hoveredItem.data.subtype}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>状态</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-brand)' }}>{hoveredItem.data.status}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>存储路径</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-text-secondary)', fontFamily: 'monospace', fontSize: 10 }}>{hoveredItem.data.fullPath}</span>
                </div>
                {hoveredItem.data.sourceRefs.length > 0 && (
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                    <span style={{ color: 'var(--sl-text-tertiary)' }}>来源引用</span>
                    <span style={{ fontWeight: 600, color: 'var(--sl-brand)' }}>{hoveredItem.data.sourceRefs.join(', ')}</span>
                  </div>
                )}
                {hoveredItem.data.summaryPoints.length > 0 && (
                  <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)' }}>
                    <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 600 }}>要点: </span>
                    {hoveredItem.data.summaryPoints.join(' / ')}
                  </div>
                )}
                {hoveredItem.data.contentPreview.length > 0 && (
                  <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
                    <span style={{ fontWeight: 600 }}>预览:</span>
                    <ul style={{ margin: '4px 0 0 0', paddingLeft: 16 }}>
                      {hoveredItem.data.contentPreview.map((c: string, i: number) => (
                        <li key={i}>{c}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}
            {hoveredItem.type === 'next' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>优先级</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-brand)' }}>{hoveredItem.data.priority}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>执行席位</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-text-secondary)' }}>{hoveredItem.data.actor}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>关联对象</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-text-secondary)' }}>{hoveredItem.data.objectRef}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>时间</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-text-tertiary)' }}>{hoveredItem.data.timestamp}</span>
                </div>
                {hoveredItem.data.linkedArtifactIds.length > 0 && (
                  <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)' }}>
                    <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 600 }}>关联证据: </span>
                    {hoveredItem.data.linkedArtifactIds.join(', ')}
                  </div>
                )}
              </div>
            )}
            {hoveredItem.type === 'event' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>事件类型</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-brand)' }}>{hoveredItem.data.event_type || hoveredItem.data.eventType}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>执行者</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-text-secondary)' }}>{hoveredItem.data.actor_label || hoveredItem.data.actor}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>发生时间</span>
                  <span style={{ fontWeight: 600, color: 'var(--sl-text-tertiary)' }}>{hoveredItem.data.occurred_at || hoveredItem.data.rawTimestamp}</span>
                </div>
                {hoveredItem.data.object_refs_chips && hoveredItem.data.object_refs_chips.length > 0 && (
                  <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)' }}>
                    <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 600 }}>关联对象: </span>
                    {hoveredItem.data.object_refs_chips.join(', ')}
                  </div>
                )}
                {hoveredItem.data.evidence_refs_chips && hoveredItem.data.evidence_refs_chips.length > 0 && (
                  <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)' }}>
                    <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 600 }}>证据: </span>
                    {hoveredItem.data.evidence_refs_chips.map((c: any) => c.label).join(', ')}
                  </div>
                )}
              </div>
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
      )}

    </div>
  );
};
