/**
 * KnowledgeSection.tsx
 * 知识面板区块（D5 Phase 2 / 夜间头条 step 5）—— 界面上第一块「从历史会话提炼出的知识」。
 *
 * 张先生醒来要看到的那一件事：项目 → 主题树；点主题 → 记忆卡；每张卡默认一句话摘要，
 * 展开给详述，再展开给**指回原文的行号**。
 *
 * 三层披露与 D5 的 L0/L1/L2 完全同一套，不造第四种：
 *   L0（默认）  title + kind 徽章 + confidence + l0 一句话摘要
 *   L1（展开）  l1 详述
 *   L2（再展开）l2 行级证据：refKind + documentId/filePath + bodyDigest + lineStart/lineEnd
 *
 * 数据契约：Lyra 夜间拆解 §3 的 DTO 逐字镜像在此（Nimbus 的 read model 并行落地，
 * 本文件据此先行编码；集成在他命令上线那一刻发生，届时只需把本文件的 load 缝接到
 * lib/api.ts 的包装上）。
 *
 * 文件围栏（双边派单同文，硬规则）：本文件、ProjectDashboard.tsx 的一行挂载、
 * KnowledgeSection.test.tsx、tokens.css 的只增补是 MIRA 可碰的全部。
 * lib/types-dto.ts 与 lib/api.ts 在 NIMBUS 围栏内 —— 本席位永不编辑；需要变更经
 * emit_event 上报 Lyra 等派单。因此下面的 wire 类型与 invoke 缝是本文件的**本地缝**，
 * 标注清楚，等 Nimbus 的只增补落地后整块替换，不产生第二个权威源。
 */

import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  BookOpen,
  Brain,
  ChevronDown,
  ChevronRight,
  FileText,
  FolderTree,
  Hash,
  Loader2,
  ScrollText,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────────────────
// DTO 契约（Lyra 夜间拆解 §3，逐字镜像）
// ─────────────────────────────────────────────────────────────────────────────

export interface KnowledgeTopicNode {
  topicId: string;
  title: string;
  status: string; // open|closed|merged (010 vocab)
  parentTopicId: string | null;
  childTopicIds: string[];
  /** active 且未被 supersede 的记忆数 */
  memoryCount: number;
}

export interface KnowledgeL2 {
  refKind: string; // document|file
  documentId?: string | null;
  filePath?: string | null;
  bodyDigest?: string | null;
  lineStart?: number | null;
  lineEnd?: number | null;
}

export interface KnowledgeMemoryCard {
  memoryId: string;
  topicId: string;
  title: string;
  kind: string; // decision|failure|fact|preference|artifact|unresolved
  confidence: string; // quoted|inferred
  tier: string; // cold|warm|hot
  status: string; // active|forgotten|resolved
  /** ~100 token，唯一默认层 */
  l0Summary: string;
  l1Overview?: string | null;
  l2?: KnowledgeL2 | null;
  /** includeSuperseded 时标注取代者 */
  supersededBy?: string | null;
  sourceKind: string;
  sourceRef: string;
}

export interface KnowledgeTopicTree {
  projectId: string;
  topics: KnowledgeTopicNode[];
}

export interface KnowledgeTopicMemories {
  topicId: string;
  /** ancestors-first */
  topicChain: { topicId: string; title: string }[];
  depth: string;
  memories: KnowledgeMemoryCard[];
}

// ─────────────────────────────────────────────────────────────────────────────
// 本地 invoke 缝 + 已注册标志（house rule：命令未注册就显式渲染该事实，不用 mock 顶替）
// ─────────────────────────────────────────────────────────────────────────────

/**
 * 接线状态是**实测**的,不是手写的。
 *
 * 2026-10-16:这里原本是 `const KNOWLEDGE_CMDS_REGISTERED = true`,注释说它「已实证」,
 * 但实证的内容是「Nimbus 的分支合进了 track、main.rs 里注册了这两条命令」—— 那是代码层
 * 的事实,不是这个构建能不能真的调通。派单里点名的就是这一条:没有任何东西测过这个 true。
 *
 * 换成和 SupervisorIntent 的 capability 探针同一形状的实测:挂载时真调一次
 * `cmd_get_knowledge_tree`,按结果显示「后端可达(实测)」/「后端不可达(实测)」。
 * 要么真测,要么不显示 —— 没有未测的接线断言。
 *
 * Nimbus 正在做的「接线诚实度(后端)」探针还没进树(量过:src-tauri 与 api.ts 里都没有)。
 * 所以这里用树内已有的先例,没有等他,也没有自己加第四个手写常量。他的探针落地后,
 * 这个 hook 换成消费他那一个即可。
 */
type WiringProbe = { kind: 'probing' } | { kind: 'ok' } | { kind: 'unavailable'; error: string };

function useKnowledgeWiring(projectId: string): WiringProbe {
  const [probe, setProbe] = useState<WiringProbe>({ kind: 'probing' });
  useEffect(() => {
    let alive = true;
    setProbe({ kind: 'probing' });
    knowledgeApi
      .getTree(projectId)
      .then(() => {
        if (alive) setProbe({ kind: 'ok' });
      })
      .catch((e) => {
        if (alive) setProbe({ kind: 'unavailable', error: String(e) });
      });
    return () => {
      alive = false;
    };
  }, [projectId]);
  return probe;
}

/**
 * 本地缝：直接 invoke 两条知识命令。
 *
 * 规范入口是 `lib/api.ts`，但该文件在 Nimbus 的文件围栏内，本席位不许编辑。所以这里
 * 放一个同形的本地缝，等他的只增补（`getKnowledgeTree` / `getTopicMemories`）落地后
 * 整块换成 `api.*` 即可 —— 这是**一个**缝，不是第二个实现：底层仍是同一条 Tauri 命令、
 * 同一个 repo 函数、同一个 MCP 读工具。
 */
const knowledgeApi = {
  getTree: (projectId: string): Promise<KnowledgeTopicTree> =>
    invoke('cmd_get_knowledge_tree', { projectId }),
  getTopicMemories: (
    topicId: string,
    depth: 'l0' | 'l1' | 'l2' = 'l0',
    includeSuperseded = false,
  ): Promise<KnowledgeTopicMemories> =>
    invoke('cmd_get_topic_memories', { topicId, depth, includeSuperseded }),
};

/** 测试注入点：默认走本地缝，测试可替换。 */
export interface KnowledgeLoader {
  getTree: (projectId: string) => Promise<KnowledgeTopicTree>;
  getTopicMemories: (
    topicId: string,
    depth: 'l0' | 'l1' | 'l2',
    includeSuperseded: boolean,
  ) => Promise<KnowledgeTopicMemories>;
}

// ─────────────────────────────────────────────────────────────────────────────
// 展示助手
// ─────────────────────────────────────────────────────────────────────────────

const KIND_LABELS: Record<string, string> = {
  decision: '裁决',
  failure: '失败',
  fact: '事实',
  preference: '偏好',
  artifact: '产物',
  unresolved: '未决',
};

function kindLabel(kind: string): string {
  return KIND_LABELS[kind] ?? kind;
}

const KIND_COLORS: Record<string, string> = {
  decision: 'var(--sl-purple)',
  failure: 'var(--sl-red)',
  fact: 'var(--sl-blue)',
  preference: 'var(--sl-amber)',
  artifact: 'var(--sl-teal)',
  unresolved: 'var(--sl-text-tertiary)',
};

function kindColor(kind: string): string {
  return KIND_COLORS[kind] ?? 'var(--sl-text-tertiary)';
}

const CONFIDENCE_LABELS: Record<string, string> = {
  quoted: '原话',
  inferred: '推断',
};

function confidenceLabel(confidence: string): string {
  return CONFIDENCE_LABELS[confidence] ?? confidence;
}

const TIER_LABELS: Record<string, string> = {
  cold: '冷',
  warm: '温',
  hot: '热',
};

function tierLabel(tier: string): string {
  return TIER_LABELS[tier] ?? tier;
}

const STATUS_LABELS: Record<string, string> = {
  open: '开放',
  closed: '已闭',
  merged: '已并',
  active: '有效',
  forgotten: '已忘',
  resolved: '已解',
};

function statusLabel(status: string): string {
  return STATUS_LABELS[status] ?? status;
}

/** L2 行号区间的人类可读形式;缺一端就说缺哪一端,不猜。 */
function l2LineRange(l2: KnowledgeL2 | null | undefined): string {
  if (!l2) return '';
  const { lineStart, lineEnd } = l2;
  if (lineStart == null && lineEnd == null) return '未记行号';
  if (lineStart != null && lineEnd != null) return `第 ${lineStart}–${lineEnd} 行`;
  return lineStart != null ? `自第 ${lineStart} 行起` : `至第 ${lineEnd} 行止`;
}

// ─────────────────────────────────────────────────────────────────────────────
// 组件
// ─────────────────────────────────────────────────────────────────────────────

interface KnowledgeSectionProps {
  projectId: string;
  /** 测试注入；默认本地 invoke 缝。 */
  loader?: KnowledgeLoader;
  /** 测试注入用;生产走实测探针(useKnowledgeWiring)。 */
  probe?: WiringProbe;
}

export const KnowledgeSection: React.FC<KnowledgeSectionProps> = ({ projectId, loader, probe }) => {
  const measured = useKnowledgeWiring(projectId);
  const wiring = probe ?? measured;
  const load = useMemo<KnowledgeLoader>(() => loader ?? knowledgeApi, [loader]);
  const [topics, setTopics] = useState<KnowledgeTopicNode[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [memories, setMemories] = useState<KnowledgeTopicMemories | null>(null);
  const [memoriesError, setMemoriesError] = useState<string | null>(null);
  const [loadingMemories, setLoadingMemories] = useState(false);
  /** 每张卡的展开档：0=L0，1=L1，2=L2。 */
  const [levels, setLevels] = useState<Record<string, number>>({});

  // 一次性加载主题树；按 depth 逐层按需取记忆。事件驱动，零轮询。
  useEffect(() => {
    if (wiring.kind !== 'ok') return;
    let alive = true;
    setTopics(null);
    setError(null);
    setSelected(null);
    setMemories(null);
    load
      .getTree(projectId)
      .then((tree) => {
        if (alive) setTopics(tree.topics ?? []);
      })
      .catch((e) => {
        if (alive) setError(String(e));
      });
    return () => {
      alive = false;
    };
  }, [projectId, load, wiring.kind]);

  const openTopic = useCallback(
    (topicId: string) => {
      if (wiring.kind !== 'ok') return;
      setSelected(topicId);
      setMemories(null);
      setMemoriesError(null);
      setLoadingMemories(true);
      // 三层按需：先取 L0 摘要列表，l1/l2 在该卡展开时再取。
      load
        .getTopicMemories(topicId, 'l0', false)
        .then((m) => setMemories(m))
        .catch((e) => setMemoriesError(String(e)))
        .finally(() => setLoadingMemories(false));
    },
    [load, wiring.kind],
  );

  const expandCard = useCallback(
    (memory: KnowledgeMemoryCard) => {
      const cur = levels[memory.memoryId] ?? 0;
      if (cur >= 2) {
        setLevels((s) => ({ ...s, [memory.memoryId]: 0 }));
        return;
      }
      // L0 -> L1：l1 就在卡上（卡片默认带 l1Overview）；缺了就在 L1 区显式说明，
      // 不用一个假详述顶替。L1 -> L2：l2 行级证据按需再取一次。
      if (cur === 0) {
        setLevels((s) => ({ ...s, [memory.memoryId]: 1 }));
        return;
      }
      setLoadingMemories(true);
      load
        .getTopicMemories(memory.topicId, 'l2', false)
        .then((m) => {
          const fresh = m.memories.find((x) => x.memoryId === memory.memoryId) ?? null;
          setMemories((prev) =>
            prev && fresh
              ? {
                  ...prev,
                  memories: prev.memories.map((x) => (x.memoryId === fresh.memoryId ? fresh : x)),
                }
              : prev,
          );
          setLevels((s) => ({ ...s, [memory.memoryId]: 2 }));
        })
        .catch((e) => setMemoriesError(String(e)))
        .finally(() => setLoadingMemories(false));
    },
    [levels, load],
  );

  const shell: React.CSSProperties = {
    padding: '12px 14px',
    borderRadius: 'var(--sl-radius-md)',
    background: 'var(--sl-surface)',
    border: '1px solid var(--sl-border-light)',
  };
  const heading: React.CSSProperties = {
    fontSize: 11,
    fontWeight: 700,
    color: 'var(--sl-text-tertiary)',
    marginBottom: 10,
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
    display: 'flex',
    alignItems: 'center',
    gap: 6,
  };

  // ── 接线态：实测出来的,分三种,没有第四种「假定它通」 ──────────────────────
  if (wiring.kind === 'probing') {
    return (
      <div style={shell}>
        <div style={heading}>
          <Brain size={14} /> 知识 (KNOWLEDGE)
        </div>
        <p
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            fontSize: 12,
            color: 'var(--sl-text-secondary)',
          }}
        >
          <Loader2 size={13} className="animate-spin" /> 正在实测读取命令是否可用……
        </p>
      </div>
    );
  }
  if (wiring.kind === 'unavailable') {
    return (
      <div style={shell}>
        <div style={heading}>
          <Brain size={14} /> 知识 (KNOWLEDGE)
        </div>
        <p style={{ fontSize: 12, lineHeight: 1.6, color: 'var(--sl-text-primary)' }}>
          读取命令调不通(实测,不是猜的):
          <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}> {wiring.error} </span>
        </p>
        <p
          style={{ fontSize: 11, lineHeight: 1.6, color: 'var(--sl-text-secondary)', marginTop: 6 }}
        >
          cmd_get_knowledge_tree / cmd_get_topic_memories 有两条路会到这里:不在这个构建里,
          或者数据库没连上。哪一种,上面那句原样返回的错误文本会说明。按 house 规则,未接线的 部分不用
          mock 伪装后端真值;命令能调通时此区自动生效,不需要再改 UI。
        </p>
      </div>
    );
  }

  return (
    <div style={shell}>
      <div style={heading}>
        <Brain size={14} /> 知识 (KNOWLEDGE)
        {topics && (
          <span
            style={{
              marginLeft: 'auto',
              fontSize: 10,
              fontWeight: 600,
              color: 'var(--sl-text-tertiary)',
            }}
          >
            {topics.length} 个主题
          </span>
        )}
      </div>

      {error ? (
        <div style={{ fontSize: 12, color: 'var(--sl-red)', lineHeight: 1.6 }}>
          读取主题树失败（后端原样返回）：
          <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>{error}</span>
        </div>
      ) : topics === null ? (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            fontSize: 12,
            color: 'var(--sl-text-secondary)',
          }}
        >
          <Loader2 size={13} className="animate-spin" /> 正在读取主题树……
        </div>
      ) : topics.length === 0 ? (
        <MeaningfulEmpty projectId={projectId} hasTopics={false} />
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
          <TopicTree topics={topics} parentId={null} selected={selected} onSelect={openTopic} />
          {selected && (
            <div style={{ borderTop: '1px solid var(--sl-divider)', paddingTop: 10 }}>
              {loadingMemories && !memories ? (
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    fontSize: 12,
                    color: 'var(--sl-text-secondary)',
                  }}
                >
                  <Loader2 size={13} className="animate-spin" /> 正在读取该主题的记忆……
                </div>
              ) : memoriesError ? (
                <div style={{ fontSize: 12, color: 'var(--sl-red)', lineHeight: 1.6 }}>
                  读取记忆失败（后端原样返回）：
                  <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>
                    {memoriesError}
                  </span>
                </div>
              ) : memories && memories.memories.length === 0 ? (
                <MeaningfulEmpty projectId={projectId} hasTopics />
              ) : memories ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {memories.topicChain.length > 0 && (
                    <div
                      style={{
                        fontSize: 10,
                        color: 'var(--sl-text-tertiary)',
                        display: 'flex',
                        alignItems: 'center',
                        gap: 4,
                        flexWrap: 'wrap',
                      }}
                    >
                      <FolderTree size={11} />
                      {memories.topicChain.map((t, i) => (
                        <React.Fragment key={t.topicId}>
                          {i > 0 && <span>/</span>}
                          <span>{t.title}</span>
                        </React.Fragment>
                      ))}
                    </div>
                  )}
                  {memories.memories.map((m) => (
                    <MemoryCardView
                      key={m.memoryId}
                      memory={m}
                      level={levels[m.memoryId] ?? 0}
                      onToggle={() => expandCard(m)}
                    />
                  ))}
                </div>
              ) : null}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

/**
 * 有意义的空态：说清为什么空、接下来发生什么，而不是一个空盒子。
 */
const MeaningfulEmpty: React.FC<{ projectId: string; hasTopics: boolean }> = ({
  projectId,
  hasTopics,
}) => (
  <div style={{ fontSize: 12, lineHeight: 1.7, color: 'var(--sl-text-secondary)' }}>
    {hasTopics ? (
      <>
        <p style={{ color: 'var(--sl-text-primary)', fontWeight: 600 }}>这个主题下还没有记忆。</p>
        <p style={{ marginTop: 4 }}>
          主题已经开了，但还没有从历史会话里提炼出落在这个主题下的记忆。
          提炼流水线跑完（并经过剪枝/脱敏）后，记忆卡会出现在这里 —— 每张卡默认一句话摘要，
          展开给详述，再展开给指回原文的行号。
        </p>
      </>
    ) : (
      <>
        <p style={{ color: 'var(--sl-text-primary)', fontWeight: 600 }}>
          项目 {projectId} 还没有任何主题。
        </p>
        <p style={{ marginTop: 4 }}>
          主题是记忆的归属框架（project + topic 是唯一的作用域）。开一个主题、
          再让提炼流水线把历史会话里的决策/失败/事实/未决问题归到主题下，
          这里就会长出主题树和记忆卡。现在为空是正常的：说明还没有人开过主题，
          也没有记忆被提炼落库。
        </p>
      </>
    )}
  </div>
);

const TopicTree: React.FC<{
  topics: KnowledgeTopicNode[];
  /** 父亲的 topicId；null = 根层。 */
  parentId: string | null;
  selected: string | null;
  onSelect: (topicId: string) => void;
}> = ({ topics, parentId, selected, onSelect }) => {
  const byId = useMemo(() => new Map(topics.map((t) => [t.topicId, t])), [topics]);

  // 定序：childTopicIds 声明过就按声明序（服务端次序优先）；否则按 parentTopicId 归属。
  // 定根：parentTopicId 为 null 或指向一个不在树里的节点。
  const nodes = useMemo(() => {
    if (parentId === null) {
      return topics.filter((t) => !t.parentTopicId || !byId.has(t.parentTopicId));
    }
    const parent = byId.get(parentId);
    if (!parent) return [];
    if (parent.childTopicIds.length > 0) {
      return parent.childTopicIds
        .map((c) => byId.get(c))
        .filter((n): n is KnowledgeTopicNode => Boolean(n));
    }
    return topics.filter((t) => t.parentTopicId === parentId);
  }, [byId, parentId, topics]);

  if (nodes.length === 0) return null;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
      {nodes.map((t) => {
        const isSel = selected === t.topicId;
        const hasKids =
          t.childTopicIds.length > 0 || topics.some((x) => x.parentTopicId === t.topicId);
        return (
          <div key={t.topicId}>
            <button
              type="button"
              onClick={() => onSelect(t.topicId)}
              style={{
                width: '100%',
                textAlign: 'left',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '7px 10px',
                borderRadius: 'var(--sl-radius-md)',
                border: `1px solid ${isSel ? 'var(--sl-brand)' : 'var(--sl-border-light)'}`,
                background: isSel ? 'var(--sl-brand-subtle)' : 'var(--sl-bg)',
                transition: 'all 150ms ease',
              }}
            >
              {hasKids ? <ChevronRight size={12} /> : <Hash size={12} />}
              <span
                style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)', flex: 1 }}
              >
                {t.title}
              </span>
              <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)' }}>
                {statusLabel(t.status)}
              </span>
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 600,
                  padding: '1px 7px',
                  borderRadius: 'var(--sl-radius-full)',
                  background:
                    t.memoryCount > 0 ? 'var(--sl-brand-subtle)' : 'var(--sl-surface-active)',
                  color: t.memoryCount > 0 ? 'var(--sl-brand-text)' : 'var(--sl-text-tertiary)',
                }}
                title="active 且未被 supersede 的记忆数"
              >
                {t.memoryCount}
              </span>
            </button>
            {hasKids && (
              <div style={{ paddingLeft: 14 }}>
                <TopicTree
                  topics={topics}
                  parentId={t.topicId}
                  selected={selected}
                  onSelect={onSelect}
                />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
};

/**
 * 一张记忆卡，三层披露。L0 默认；展开一次到 L1；再展开一次到 L2（行级证据）。
 */
const MemoryCardView: React.FC<{
  memory: KnowledgeMemoryCard;
  level: number;
  onToggle: () => void;
}> = ({ memory, level, onToggle }) => {
  const color = kindColor(memory.kind);
  return (
    <div
      style={{
        padding: '10px 12px',
        borderRadius: 'var(--sl-radius-md)',
        border: `1px solid ${color}40`,
        background: 'var(--sl-bg)',
      }}
    >
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={level > 0}
        style={{
          width: '100%',
          textAlign: 'left',
          cursor: 'pointer',
          background: 'none',
          border: 'none',
          padding: 0,
          display: 'flex',
          alignItems: 'flex-start',
          gap: 8,
        }}
      >
        {level > 0 ? (
          <ChevronDown size={13} style={{ marginTop: 2, flexShrink: 0 }} />
        ) : (
          <ChevronRight size={13} style={{ marginTop: 2, flexShrink: 0 }} />
        )}
        <span style={{ flex: 1, minWidth: 0 }}>
          <span style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
            <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)' }}>
              {memory.title}
            </span>
            <span
              style={{
                fontSize: 10,
                fontWeight: 600,
                padding: '1px 7px',
                borderRadius: 'var(--sl-radius-full)',
                background: `${color}15`,
                color,
              }}
            >
              {kindLabel(memory.kind)}
            </span>
            <span
              style={{
                fontSize: 10,
                fontWeight: 600,
                padding: '1px 7px',
                borderRadius: 'var(--sl-radius-full)',
                background: 'var(--sl-surface-active)',
                color: 'var(--sl-text-secondary)',
              }}
              title={memory.confidence === 'quoted' ? '原话：来源自己的用词' : '推断：从证据推导'}
            >
              {confidenceLabel(memory.confidence)}
            </span>
            <span
              style={{ fontSize: 10, color: 'var(--sl-text-tertiary)' }}
              title={`tier ${memory.tier}`}
            >
              {tierLabel(memory.tier)}
            </span>
            {memory.status !== 'active' && (
              <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)' }}>
                {statusLabel(memory.status)}
              </span>
            )}
          </span>
          {/* L0：唯一默认层，一句话摘要 */}
          <span
            style={{
              display: 'block',
              fontSize: 11,
              color: 'var(--sl-text-secondary)',
              marginTop: 4,
              lineHeight: 1.6,
            }}
          >
            {memory.l0Summary || '（该记忆没有一句话摘要）'}
          </span>
        </span>
      </button>

      {/* L1：详述 */}
      {level >= 1 && (
        <div
          style={{
            marginTop: 8,
            paddingLeft: 21,
            fontSize: 12,
            lineHeight: 1.7,
            color: 'var(--sl-text-primary)',
            whiteSpace: 'pre-wrap',
          }}
        >
          {memory.l1Overview ? (
            memory.l1Overview
          ) : (
            <span style={{ color: 'var(--sl-text-tertiary)' }}>
              该记忆没有详述层（l1_overview 未记录）。默认摘要即全部内容。
            </span>
          )}
        </div>
      )}

      {/* L2：行级证据，指回原文 */}
      {level >= 2 && (
        <div
          style={{
            marginTop: 8,
            marginLeft: 21,
            padding: '8px 10px',
            borderRadius: 'var(--sl-radius-sm)',
            background: 'var(--sl-surface)',
            border: '1px solid var(--sl-border-light)',
            fontSize: 11,
            color: 'var(--sl-text-secondary)',
            lineHeight: 1.7,
          }}
        >
          {memory.l2 ? (
            <>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  fontWeight: 600,
                  color: 'var(--sl-text-primary)',
                }}
              >
                {memory.l2.refKind === 'document' ? (
                  <FileText size={12} />
                ) : (
                  <ScrollText size={12} />
                )}
                证据（{memory.l2.refKind}）
              </div>
              {memory.l2.refKind === 'document' ? (
                memory.l2.documentId ? (
                  <div style={{ fontFamily: 'var(--sl-font-mono, monospace)', marginTop: 2 }}>
                    documentId: {memory.l2.documentId}
                  </div>
                ) : (
                  <div style={{ marginTop: 2, color: 'var(--sl-red)' }}>
                    该证据声明为 document，但没有记 documentId。
                  </div>
                )
              ) : memory.l2.filePath ? (
                <div
                  style={{
                    fontFamily: 'var(--sl-font-mono, monospace)',
                    marginTop: 2,
                    wordBreak: 'break-all',
                  }}
                >
                  {memory.l2.filePath}
                </div>
              ) : (
                <div style={{ marginTop: 2, color: 'var(--sl-red)' }}>
                  该证据声明为 file，但没有记 filePath。
                </div>
              )}
              <div style={{ marginTop: 2 }}>
                <BookOpen size={11} style={{ verticalAlign: -1, marginRight: 4 }} />
                {l2LineRange(memory.l2)}
              </div>
              {memory.l2.bodyDigest && (
                <div
                  style={{
                    fontFamily: 'var(--sl-font-mono, monospace)',
                    marginTop: 2,
                    wordBreak: 'break-all',
                  }}
                >
                  sha256: {memory.l2.bodyDigest}
                </div>
              )}
            </>
          ) : (
            <span style={{ color: 'var(--sl-text-tertiary)' }}>
              该记忆没有行级证据（l2 未记录）—— 无法指回原文，只能信摘要与详述。
            </span>
          )}
          <div
            style={{
              marginTop: 6,
              paddingTop: 6,
              borderTop: '1px solid var(--sl-divider)',
              fontSize: 10,
            }}
          >
            来源:{memory.sourceKind} ·{' '}
            <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>{memory.sourceRef}</span>
            {memory.supersededBy && (
              <>
                {' '}
                · 已被{' '}
                <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>
                  {memory.supersededBy}
                </span>{' '}
                取代
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default KnowledgeSection;
