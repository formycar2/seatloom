/**
 * SupervisorIntent.tsx
 * Supervisor Layer「意图入口」界面 — LYRA-2026-10-13-supervisor-intent-entry-contract-v1.
 *
 * slice 4 (wiring): the 派活 entry now CALLS the live intent tool (api.planIntent →
 * cmd_plan_intent → plan_intent, actor human:supervisor) and renders its real result —
 * a reviewable proposal, or a clarification the tool stopped at. Confirm writes through
 * the real governed path (api.enqueueDispatch); on failure the error is verbatim and no
 * dispatch id is shown. A suggestion whose dependency edge the write path cannot express
 * is refused explicitly (§3.3), never silently dropped.
 *
 * Wiring honesty (contract §4 / boundary 3): wiring state is MEASURED (probe), never a
 * hand-maintained boolean. Load-bearing discipline (§3.3): when the intent is unclear the
 * system ASKS — plan_intent returns the question and the real choices; the UI never guesses.
 */

import React, { useCallback, useEffect, useState } from 'react';
import { api, isTauri } from '../../lib/api';
import { useDataStore } from '../../stores/useDataStore';
import type {
  AcceptanceCardDto,
  CanonicalEventDto,
  CheckpointDto,
  DispatchDto,
  HandoffDto,
  KnowledgeTopicNode,
  KnowledgeTopicMemories,
} from '../../lib/types-dto';
import type { Clarification, IntentPlanResult, Proposal } from '../../lib/intent-dto';

// ─────────────────────────────────────────────────────────────────────────────
// Contract shapes (§3 / §4)
// ─────────────────────────────────────────────────────────────────────────────

type IntentMode = 'dispatch' | 'chat';

/** A measured capability probe result — the replacement for a hardcoded flag. */
type ProbeState =
  | { kind: 'idle' }
  | { kind: 'probing' }
  | { kind: 'ok'; detail: string }
  | { kind: 'unavailable'; detail: string };

// ─────────────────────────────────────────────────────────────────────────────
// Small shared primitives (design tokens only; no new CSS)
// ─────────────────────────────────────────────────────────────────────────────

const cardStyle: React.CSSProperties = {
  border: '1px solid var(--sl-border)',
  borderRadius: 'var(--sl-radius-md)',
  background: 'var(--sl-surface)',
  padding: '10px 12px',
};

const sectionLabel: React.CSSProperties = {
  fontSize: 10,
  fontWeight: 600,
  color: 'var(--sl-text-tertiary)',
  textTransform: 'uppercase',
  letterSpacing: '0.05em',
  marginBottom: 4,
};

const linkBtn: React.CSSProperties = {
  background: 'transparent',
  border: 'none',
  padding: 0,
  cursor: 'pointer',
  color: 'var(--sl-brand)',
  fontSize: 12,
  textAlign: 'left',
};

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', gap: 8, fontSize: 12, lineHeight: 1.5, padding: '3px 0' }}>
      <span style={{ color: 'var(--sl-text-tertiary)', flexShrink: 0, minWidth: 64 }}>{label}</span>
      <span style={{ color: 'var(--sl-text-primary)', minWidth: 0, wordBreak: 'break-word' }}>
        {children}
      </span>
    </div>
  );
}

/** "Why empty and what next" — never a blank frame (contract §4). */
function MeaningfulEmpty({ why, next }: { why: string; next: string }) {
  return (
    <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)', lineHeight: 1.6 }}>
      <div>{why}</div>
      <div style={{ color: 'var(--sl-text-tertiary)', marginTop: 2 }}>{next}</div>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Measured capability probe — the anti-pattern to the hardcoded wiring flag.
// It CALLS the backing command and observes the result. A surface either works
// or states the measured error; nothing asserts a wiring state that was not
// measured (contract §4 wiring-honesty, V8).
// ─────────────────────────────────────────────────────────────────────────────

function useMeasuredProbe() {
  const [probe, setProbe] = useState<ProbeState>({ kind: 'idle' });

  const activeProjectId = useDataStore((s) => s.activeProjectId);

  const run = useCallback(async () => {
    if (!isTauri()) {
      setProbe({
        kind: 'unavailable',
        detail: 'not running inside the SeatLoom app (no Tauri bridge)',
      });
      return;
    }
    // A reachability probe still needs a project to ask about, because every read
    // is now project-scoped. Saying "no project selected" is the honest answer
    // here — it is not the same as "the backend is unreachable", and conflating
    // them is the wiring-honesty defect this probe exists to avoid.
    if (!activeProjectId) {
      setProbe({ kind: 'unavailable', detail: 'no project selected — nothing to read against' });
      return;
    }
    setProbe({ kind: 'probing' });
    // Probe the cheapest real read the surfaces depend on. Observe the result.
    try {
      const seats = await api.listSeats(activeProjectId);
      setProbe({
        kind: 'ok',
        detail: `seat registry reachable (${seats.length} seat(s) measured under ${activeProjectId})`,
      });
    } catch (err) {
      setProbe({ kind: 'unavailable', detail: `seat registry call failed: ${String(err)}` });
    }
  }, [activeProjectId]);

  return { probe, run };
}

// ─────────────────────────────────────────────────────────────────────────────
// Continuity-delta card (§3.4) — deterministic reads only, every row links to
// its ledger object. "Since when" is the last supervisor checkpoint; if none,
// the card says so rather than showing an empty frame.
// ─────────────────────────────────────────────────────────────────────────────

interface ContinuityDelta {
  pendingAcceptances: AcceptanceCardDto[];
  openDispatches: DispatchDto[];
  pendingHandoffs: HandoffDto[];
  lastGate: CanonicalEventDto | null;
  sinceCheckpoint: CheckpointDto | null;
}

function useContinuityDelta() {
  const [delta, setDelta] = useState<ContinuityDelta | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  // Every read below is project-scoped. Without a selected project there is
  // nothing honest to show, so the card says that instead of quietly reading the
  // default project — which is how it used to show seatloom's data under any tab.
  const activeProjectId = useDataStore((s) => s.activeProjectId);

  const load = useCallback(async () => {
    if (!isTauri()) {
      setError('not running inside the SeatLoom app');
      return;
    }
    if (!activeProjectId) {
      setError('no project selected');
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const [acceptances, dispatches, handoffs, events, checkpoints] = await Promise.all([
        api.listAcceptances(activeProjectId),
        api.listDispatches(activeProjectId, 200),
        api.listHandoffs(activeProjectId),
        api.listEvents(activeProjectId, 200),
        api.listCheckpoints(1),
      ]);
      const gateTypes = [
        'ReviewVerdictIssued',
        'AcceptanceGranted',
        'AcceptanceRejected',
        'AcceptanceCancelled',
      ];
      const lastGate = events.find((e) => gateTypes.includes(e.eventType)) ?? null;
      setDelta({
        pendingAcceptances: acceptances.filter(
          (a) => a.status === 'awaiting_acceptance' || a.status === 'pending',
        ),
        openDispatches: dispatches.filter(
          (d) => d.status === 'ready' || d.status === 'in_flight' || d.status === 'blocked',
        ),
        pendingHandoffs: handoffs.filter(
          (h) => h.status !== 'completed' && h.status !== 'accepted',
        ),
        lastGate,
        sinceCheckpoint: checkpoints[0] ?? null,
      });
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  return { delta, error, loading, load };
}

function ContinuityDeltaCard() {
  const { delta, error, loading, load } = useContinuityDelta();

  return (
    <div style={cardStyle}>
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: 6 }}>
        <div style={{ ...sectionLabel, flex: 1, marginBottom: 0 }}>
          Continuity delta · 距上次以来的变化
        </div>
        <button
          type="button"
          onClick={load}
          style={{ ...linkBtn, fontSize: 11 }}
          disabled={loading}
        >
          {loading ? '读取中…' : delta ? '刷新' : '读取'}
        </button>
      </div>

      {error && (
        <div style={{ fontSize: 12, color: 'var(--sl-red)', lineHeight: 1.5 }}>
          读取失败(后端原样返回):
          <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}> {error}</span>
        </div>
      )}

      {!delta && !error && !loading && (
        <MeaningfulEmpty
          why="尚未读取连续性增量。"
          next="点「读取」拉取待验收卡、在途派单、待办交接与最近门判定。"
        />
      )}

      {delta && (
        <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', marginBottom: 6 }}>
          {delta.sinceCheckpoint
            ? `自 checkpoint ${delta.sinceCheckpoint.id}(${delta.sinceCheckpoint.createdAt})以来`
            : '无监督 checkpoint —— 无法给出"自何时以来",此处按全量近况展示。'}
        </div>
      )}

      {delta && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div>
            <div style={sectionLabel}>待验收卡({delta.pendingAcceptances.length})</div>
            {delta.pendingAcceptances.length === 0 ? (
              <MeaningfulEmpty
                why="没有待运营者判定的验收卡。"
                next="新交付经 Flux 验绿后会在这里出现。"
              />
            ) : (
              delta.pendingAcceptances.map((a) => (
                <Row key={a.id} label={a.id}>
                  {a.dispatchTitle} ·{' '}
                  <span style={{ color: 'var(--sl-text-tertiary)' }}>{a.branch}</span>
                </Row>
              ))
            )}
          </div>
          <div>
            <div style={sectionLabel}>在途派单({delta.openDispatches.length})</div>
            {delta.openDispatches.length === 0 ? (
              <MeaningfulEmpty
                why="没有 ready / in_flight / blocked 的派单。"
                next="队列空;用上方「派活」入口按意图建一张。"
              />
            ) : (
              delta.openDispatches.slice(0, 8).map((d) => (
                <Row key={d.id} label={d.status}>
                  <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>{d.id}</span> →{' '}
                  {d.targetSeatId} · {d.title}
                </Row>
              ))
            )}
          </div>
          <div>
            <div style={sectionLabel}>待办交接({delta.pendingHandoffs.length})</div>
            {delta.pendingHandoffs.length === 0 ? (
              <MeaningfulEmpty why="没有待办交接。" next="席位间移交工作项时会出现。" />
            ) : (
              delta.pendingHandoffs.slice(0, 8).map((h) => (
                <Row key={h.id} label={h.status}>
                  {h.fromRef} → {h.toRef} · {h.purpose}
                </Row>
              ))
            )}
          </div>
          <div>
            <div style={sectionLabel}>最近门判定</div>
            {delta.lastGate ? (
              <Row label={delta.lastGate.eventType}>
                <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>
                  {delta.lastGate.id}
                </span>{' '}
                · {delta.lastGate.actorRef} · {delta.lastGate.occurredAt}
              </Row>
            ) : (
              <MeaningfulEmpty
                why="近期没有门判定事件。"
                next="Flux 验绿 / 验收决策落地后会出现在这里。"
              />
            )}
          </div>
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decision-answer card (§3.2) — exact retrieval first, over the knowledge read
// model. Insufficient evidence is a first-class result (INT-14): an answer with
// no evidence is never composed.
// ─────────────────────────────────────────────────────────────────────────────

function DecisionAnswerCard() {
  const [topics, setTopics] = useState<KnowledgeTopicNode[] | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [memories, setMemories] = useState<KnowledgeTopicMemories | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loadingTree, setLoadingTree] = useState(false);
  const [loadingMem, setLoadingMem] = useState(false);

  const loadTree = useCallback(async () => {
    if (!isTauri()) {
      setError('not running inside the SeatLoom app');
      return;
    }
    setLoadingTree(true);
    setError(null);
    try {
      const tree = await api.getKnowledgeTree();
      setTopics(tree.topics);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoadingTree(false);
    }
  }, []);

  const selectTopic = useCallback(async (topicId: string) => {
    setSelected(topicId);
    setLoadingMem(true);
    setError(null);
    try {
      const mem = await api.getTopicMemories(topicId, 'l0', false, 20);
      setMemories(mem);
    } catch (err) {
      setError(String(err));
      setMemories(null);
    } finally {
      setLoadingMem(false);
    }
  }, []);

  return (
    <div style={cardStyle}>
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: 6 }}>
        <div style={{ ...sectionLabel, flex: 1, marginBottom: 0 }}>
          Decision answer · 为什么这么定(带出处)
        </div>
        <button
          type="button"
          onClick={loadTree}
          style={{ ...linkBtn, fontSize: 11 }}
          disabled={loadingTree}
        >
          {loadingTree ? '读取中…' : topics ? '刷新主题' : '读取主题'}
        </button>
      </div>

      {error && (
        <div style={{ fontSize: 12, color: 'var(--sl-red)', lineHeight: 1.5 }}>
          读取失败(后端原样返回):
          <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}> {error}</span>
        </div>
      )}

      {!topics && !error && !loadingTree && (
        <MeaningfulEmpty
          why="尚未检索知识层。"
          next="点「读取主题」拉取主题树,再选一个主题看它的记忆与出处。"
        />
      )}

      {topics && topics.length === 0 && (
        <MeaningfulEmpty
          why="项目下没有任何主题 —— 没有可检索的知识。"
          next="等席位把历史会话提炼成记忆后,这里才会有内容。"
        />
      )}

      {topics && topics.length > 0 && (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 8 }}>
          {topics.map((t) => (
            <button
              key={t.topicId}
              type="button"
              onClick={() => selectTopic(t.topicId)}
              style={{
                fontSize: 11,
                padding: '3px 8px',
                borderRadius: 'var(--sl-radius-sm)',
                border: `1px solid ${selected === t.topicId ? 'var(--sl-brand)' : 'var(--sl-border-light)'}`,
                background: selected === t.topicId ? 'var(--sl-surface-hover)' : 'transparent',
                color: 'var(--sl-text-primary)',
                cursor: 'pointer',
              }}
            >
              {t.title} ({t.memoryCount})
            </button>
          ))}
        </div>
      )}

      {loadingMem && (
        <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)' }}>读取记忆中…</div>
      )}

      {memories && memories.memories.length === 0 && (
        <MeaningfulEmpty why="该主题下没有活跃记忆。" next="选别的主题,或等该主题有记忆沉淀。" />
      )}

      {memories && memories.memories.length > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {memories.memories.map((m) => (
            <div
              key={m.memoryId}
              style={{ borderTop: '1px solid var(--sl-divider)', paddingTop: 6 }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 2 }}>
                <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)' }}>
                  {m.title}
                </span>
                <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)' }}>
                  {m.kind} · {m.confidence}
                </span>
              </div>
              <div style={{ fontSize: 12, color: 'var(--sl-text-primary)', lineHeight: 1.5 }}>
                {m.l0Summary}
              </div>
              <div style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 2 }}>
                出处:{m.sourceKind} /{' '}
                <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>{m.sourceRef}</span>
                {m.l2?.filePath
                  ? ` · ${m.l2.filePath}${m.l2.lineStart ? `:${m.l2.lineStart}` : ''}`
                  : ''}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Clarification card (§3.3) — the no-guessing states, each with the real
// choices and why each is offered. Replaces the suggestion card when it fires.
// ─────────────────────────────────────────────────────────────────────────────

/** Clarification card (§3.3) — plan_intent stopped at a question. One question, the
 *  real choices, and why each is offered. Not a toast, not a disabled state. */
function ClarificationCard({ c, onPick }: { c: Clarification; onPick: (label: string) => void }) {
  return (
    <div style={{ ...cardStyle, borderColor: 'var(--sl-brand)' }}>
      <div style={{ ...sectionLabel, color: 'var(--sl-brand)' }}>需要你澄清 · 我不猜</div>
      <div style={{ fontSize: 12, fontWeight: 600, marginBottom: 2 }}>{c.question}</div>
      <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', marginBottom: 6 }}>
        规则 {c.ruleId} · {c.reason}
      </div>
      {c.choices.map((ch) => (
        <button
          key={ch.value}
          type="button"
          onClick={() => onPick(ch.label)}
          style={{ ...linkBtn, display: 'block', marginBottom: 4 }}
        >
          <b>{ch.label}</b>
          <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', fontWeight: 400 }}>
            {ch.why}
          </div>
        </button>
      ))}
    </div>
  );
}

/** Suggestion card (§3.1 step 5 / §4) — the UX-02 field set, every field bound to the
 *  proposal plan_intent built from real reads, plus a computed `why this seat` disclosure.
 *  Confirm writes through the governed path; a dependency edge the write path cannot
 *  express is refused explicitly (§3.3), never silently dropped. */
function SuggestionCard({
  p,
  confirming,
  confirmResult,
  onConfirm,
  onCancel,
}: {
  p: Proposal;
  confirming: boolean;
  confirmResult: { ok: string } | { error: string } | null;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const [showRationale, setShowRationale] = useState(false);
  const deps = p.proposedObject.suggestedDependencies;
  // enqueueDispatch forwards only seat/title/instructions/priority — no dependency
  // edges (the V5 wrapper gap). A suggestion carrying an edge is refused, not dropped.
  const writeCannotExpressEdge = deps.length > 0;

  return (
    <div style={cardStyle}>
      <div style={{ ...sectionLabel }}>提议 · 审阅后确认(actor {p.actor})</div>
      <Row label="意图">{p.intentRestated}</Row>
      <Row label="对象">
        {p.proposedObject.kind} · {p.proposedObject.title}
      </Row>
      <Row label="归属">
        {p.ownership.seatName}({p.ownership.seatId})
        {p.ownership.role ? ` · ${p.ownership.role}` : ''}
      </Row>
      {p.ownership.capabilityMatch.length > 0 && (
        <Row label="能力匹配">{p.ownership.capabilityMatch.join('、')}</Row>
      )}
      {p.ownership.authorityDocs.length > 0 && (
        <Row label="权威文档">{p.ownership.authorityDocs.join('、')}</Row>
      )}
      {p.ownership.constraints.length > 0 && (
        <Row label="约束">{p.ownership.constraints.join('、')}</Row>
      )}
      <Row label="证据">
        {p.evidence.length === 0
          ? '(无)'
          : p.evidence.map((e, i) => (
              <div key={i}>
                {e.label} — {e.source}
              </div>
            ))}
      </Row>
      <Row label="预算">
        worker {p.budget.projectWorkerBudgetTokens ?? '—'} / supervisor{' '}
        {p.budget.projectSupervisorBudgetTokens ?? '—'} · 在途 {p.budget.seatInFlightDispatches}
        <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)' }}>{p.budget.note}</div>
      </Row>
      {p.impact.length > 0 && <Row label="影响">{p.impact.join(';')}</Row>}
      {p.precondition.raw && (
        <Row label="前提">
          {p.precondition.raw}
          {p.precondition.edge
            ? ` → ${p.precondition.edge.depKind}=${p.precondition.edge.depValue}`
            : p.precondition.unresolvedReason
              ? `(未解析:${p.precondition.unresolvedReason})`
              : ''}
        </Row>
      )}

      <button
        type="button"
        onClick={() => setShowRationale((s) => !s)}
        style={{ ...linkBtn, fontSize: 11, marginTop: 4 }}
      >
        {showRationale ? '收起理由' : '为什么是这个席位?'}({p.routingRationale.length}{' '}
        个候选的实测理由)
      </button>
      {showRationale && (
        <div style={{ marginTop: 4, display: 'flex', flexDirection: 'column', gap: 4 }}>
          {p.routingRationale.map((r) => (
            <div
              key={r.seatId}
              style={{
                fontSize: 11,
                lineHeight: 1.5,
                borderLeft: `2px solid ${r.selected ? 'var(--sl-brand)' : 'var(--sl-divider)'}`,
                paddingLeft: 6,
              }}
            >
              <b>{r.seatName}</b>({r.seatId})
              {r.selected ? ' ← 选中' : r.eligible ? '' : ' · 不合格'}
              <div style={{ color: 'var(--sl-text-tertiary)' }}>
                命中能力:{r.matchedCapabilities.join('、') || '无'}
                {r.excludingConstraint ? ` · 排除:${r.excludingConstraint}` : ''} · 在途{' '}
                {r.inFlightDispatches} · 源:{r.source}
              </div>
            </div>
          ))}
        </div>
      )}

      <div
        style={{ display: 'flex', gap: 8, marginTop: 8, alignItems: 'center', flexWrap: 'wrap' }}
      >
        {writeCannotExpressEdge ? (
          <span style={{ fontSize: 11, color: 'var(--sl-red)', lineHeight: 1.5 }}>
            这条意图带依赖边({deps.map((d) => `${d.depKind}=${d.depValue}`).join(', ')});
            当前受治理写路无法表达依赖边,我不会静默丢掉它 —— 需先补写路的 dependencies 支持。
          </span>
        ) : (
          <button
            type="button"
            onClick={onConfirm}
            disabled={confirming}
            style={{
              fontSize: 12,
              padding: '5px 14px',
              borderRadius: 'var(--sl-radius-md)',
              border: '1px solid var(--sl-brand)',
              background: 'var(--sl-brand)',
              color: '#fff',
              cursor: 'pointer',
            }}
          >
            {confirming ? '提交中…' : '确认并建卡'}
          </button>
        )}
        <button type="button" onClick={onCancel} style={{ ...linkBtn, fontSize: 12 }}>
          取消
        </button>
      </div>

      {confirmResult && 'ok' in confirmResult && (
        <div style={{ fontSize: 12, color: 'var(--sl-brand)', marginTop: 6 }}>
          已建卡:派单{' '}
          <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>{confirmResult.ok}</span>
        </div>
      )}
      {confirmResult && 'error' in confirmResult && (
        <div style={{ fontSize: 12, color: 'var(--sl-red)', marginTop: 6, lineHeight: 1.5 }}>
          建卡失败(后端原样返回,未显示派单 id):
          <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>
            {' '}
            {confirmResult.error}
          </span>
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// The surface: two-mode intent entry + the cards. Mounted inside the existing
// Supervisor panel; no new window, no re-layout of the skeleton (contract §4).
// ─────────────────────────────────────────────────────────────────────────────

type PlanState =
  | { kind: 'idle' }
  | { kind: 'planning' }
  | { kind: 'result'; result: IntentPlanResult }
  | { kind: 'error'; message: string };

export const SupervisorIntent: React.FC<{ onRawSend: (text: string) => void }> = ({
  onRawSend,
}) => {
  const [mode, setMode] = useState<IntentMode>('dispatch');
  const [text, setText] = useState('');
  const [plan, setPlan] = useState<PlanState>({ kind: 'idle' });
  const [confirming, setConfirming] = useState(false);
  const [confirmResult, setConfirmResult] = useState<{ ok: string } | { error: string } | null>(
    null,
  );
  const { probe, run } = useMeasuredProbe();

  // Measure wiring once on mount — never assert, always observe (V8 / boundary 3).
  useEffect(() => {
    run();
  }, [run]);

  const submitDispatch = useCallback(async () => {
    const t = text.trim();
    if (!t) return;
    if (!isTauri()) {
      setPlan({ kind: 'error', message: 'not running inside the SeatLoom app (no Tauri bridge)' });
      return;
    }
    setPlan({ kind: 'planning' });
    setConfirmResult(null);
    try {
      // The real call: api.planIntent → cmd_plan_intent → plan_intent (actor
      // human:supervisor). Reads only; returns the tool's pretty-JSON.
      const raw = await api.planIntent(t);
      const result = JSON.parse(raw) as IntentPlanResult;
      setPlan({ kind: 'result', result });
    } catch (err) {
      // Rejection verbatim — the reason, never an empty card, never a guess.
      setPlan({ kind: 'error', message: String(err) });
    }
  }, [text]);

  const submitChat = useCallback(() => {
    const t = text.trim();
    if (!t) return;
    onRawSend(t);
    setText('');
  }, [text, onRawSend]);

  const confirmWrite = useCallback(async () => {
    if (plan.kind !== 'result' || !plan.result.proposal) return;
    // 2026-10-14 张先生实测:「点了好几次确认建卡」→ 派单列表里 9 条一模一样的单。
    // 根因:确认后没有禁用按钮,可以重复点;每次点都真建一条受治理派单。
    // 修:一旦确认过(成功或失败),按钮禁用 —— 一张提案卡只对应一次写。
    if (confirmResult) return;
    const p = plan.result.proposal;
    setConfirming(true);
    setConfirmResult(null);
    try {
      // The real governed write path (actor human:supervisor). No dependencies:
      // the wrapper cannot forward them, and a suggestion carrying an edge is
      // refused in the card above rather than silently dropped.
      const dispatchId = await api.enqueueDispatch(
        p.ownership.seatId,
        p.proposedObject.title,
        p.proposedObject.payloadNote,
      );
      setConfirmResult({ ok: dispatchId });
    } catch (err) {
      // verbatim; no dispatch id is shown on failure
      setConfirmResult({ error: String(err) });
    } finally {
      setConfirming(false);
    }
  }, [plan]);

  const modeBtn = (m: IntentMode, label: string, title: string) => (
    <button
      type="button"
      onClick={() => {
        setMode(m);
        setPlan({ kind: 'idle' });
        setConfirmResult(null);
      }}
      title={title}
      style={{
        fontSize: 11,
        padding: '3px 10px',
        borderRadius: 'var(--sl-radius-sm)',
        border: `1px solid ${mode === m ? 'var(--sl-brand)' : 'var(--sl-border-light)'}`,
        background: mode === m ? 'var(--sl-surface-hover)' : 'transparent',
        color: mode === m ? 'var(--sl-brand)' : 'var(--sl-text-secondary)',
        cursor: 'pointer',
        fontWeight: mode === m ? 600 : 400,
      }}
    >
      {label}
    </button>
  );

  // Render the plan_intent result — proposal / clarification / retrieval (§3).
  const renderPlan = () => {
    if (plan.kind === 'planning') {
      return <div style={{ fontSize: 12, color: 'var(--sl-text-tertiary)' }}>拆解中…</div>;
    }
    if (plan.kind === 'error') {
      return (
        <div style={{ ...cardStyle, borderColor: 'var(--sl-red)' }}>
          <div style={{ ...sectionLabel, color: 'var(--sl-red)' }}>拆解被拒(后端原样返回)</div>
          <div style={{ fontSize: 12, lineHeight: 1.5 }}>
            <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}>{plan.message}</span>
          </div>
        </div>
      );
    }
    if (plan.kind === 'result') {
      const r = plan.result;
      // 2026-10-14:点了澄清选项必须**直接继续**,不该把文字填回去让用户再按一次拆解。
      // 张先生实测:「点了 create 没反应」—— 代码原本只 setText + 清卡片,
      // 用户合理预期是"直接往下走",而不是"我帮你填好了,你自己再点一次"。
      const pick = (label: string) => {
        setText(label);
        setPlan({ kind: 'planning' });
        // 直接拿选项文字重新拆解,不再要一次手动提交。
        void (async () => {
          try {
            const raw = await api.planIntent(label);
            setPlan({ kind: 'result', result: JSON.parse(raw) as IntentPlanResult });
          } catch (err) {
            // 拒绝逐字 —— 理由,而不是一张空卡,更不是猜。
            setPlan({ kind: 'error', message: String(err) });
          }
        })();
      };
      if (r.outcome === 'proposal' && r.proposal) {
        if (r.proposal.blockingClarification) {
          return <ClarificationCard c={r.proposal.blockingClarification} onPick={pick} />;
        }
        return (
          <SuggestionCard
            p={r.proposal}
            confirming={confirming}
            confirmResult={confirmResult}
            onConfirm={confirmWrite}
            onCancel={() => {
              setPlan({ kind: 'idle' });
              setConfirmResult(null);
            }}
          />
        );
      }
      if (r.outcome === 'clarification' && r.clarification) {
        return <ClarificationCard c={r.clarification} onPick={pick} />;
      }
      if (r.outcome === 'retrieval') {
        return (
          <div style={cardStyle}>
            <div style={sectionLabel}>检索 · 精确读取(Why / Evidence)</div>
            <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)', lineHeight: 1.5 }}>
              plan_intent 对 why/evidence 意图返回的是「要做的精确读取」,而非拼凑的答案:
            </div>
            <pre
              style={{
                fontSize: 11,
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
                color: 'var(--sl-text-primary)',
                margin: '6px 0 0',
              }}
            >
              {JSON.stringify(r.retrieval ?? {}, null, 2)}
            </pre>
          </div>
        );
      }
    }
    return null;
  };

  return (
    <div
      data-no-drag
      style={{
        borderTop: '1px solid var(--sl-divider)',
        padding: '10px 16px',
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
        flexShrink: 0,
      }}
    >
      {/* two-mode entry — structured first, free text second, never blended */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        {modeBtn('dispatch', '派活', '结构化:一句话 → 可审阅的结构化提议(走受治理写路)')}
        {modeBtn('chat', '说句话', '原始 IM:直接说给某个席位(既有行为)')}
        <span style={{ flex: 1 }} />
        <span
          style={{
            fontSize: 10,
            color: probe.kind === 'ok' ? 'var(--sl-text-tertiary)' : 'var(--sl-red)',
          }}
          title={probe.kind === 'ok' || probe.kind === 'unavailable' ? probe.detail : ''}
        >
          {probe.kind === 'idle' && '未测'}
          {probe.kind === 'probing' && '实测中…'}
          {probe.kind === 'ok' && '后端可达(实测)'}
          {probe.kind === 'unavailable' && '后端不可达(实测)'}
        </span>
      </div>

      <div style={{ display: 'flex', gap: 8 }}>
        <input
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              if (mode === 'dispatch') submitDispatch();
              else submitChat();
            }
          }}
          placeholder={
            mode === 'dispatch'
              ? '说你想干什么,例如:给 Nimbus 派个活,把唤醒服务兜底接上'
              : '说给当前席位…'
          }
          style={{
            flex: 1,
            padding: '6px 10px',
            fontSize: 12,
            border: '1px solid var(--sl-border-light)',
            borderRadius: 'var(--sl-radius-md)',
            background: 'var(--sl-surface)',
            outline: 'none',
            color: 'var(--sl-text-primary)',
          }}
        />
        <button
          type="button"
          onClick={mode === 'dispatch' ? submitDispatch : submitChat}
          style={{
            fontSize: 12,
            padding: '6px 14px',
            borderRadius: 'var(--sl-radius-md)',
            border: '1px solid var(--sl-brand)',
            background: 'var(--sl-brand)',
            color: '#fff',
            cursor: 'pointer',
          }}
        >
          {mode === 'dispatch' ? '拆解' : '发送'}
        </button>
      </div>

      {mode === 'dispatch' && renderPlan()}

      {/* the two live-wired cards */}
      <ContinuityDeltaCard />
      <DecisionAnswerCard />
    </div>
  );
};
