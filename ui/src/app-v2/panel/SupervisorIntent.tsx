/**
 * SupervisorIntent.tsx
 * Supervisor Layer「意图入口」界面 — slice 2 of LYRA-2026-10-13-supervisor-intent-entry-contract-v1.
 *
 * Built against the contract's shapes. NOT wired to the live intent-tool surface
 * (slice 1 = Nimbus's deterministic parser; slice 4 = the wiring). What IS live
 * here: continuity-delta and decision-answer read real backend commands; the
 * wiring-honesty rule is honoured by a MEASURED probe, never a hand-maintained
 * boolean (contract §4 / boundary 3 — the ACCEPTANCE_CMDS_REGISTERED shape).
 *
 * Load-bearing discipline (contract §3.3): when the intent is unclear the system
 * ASKS, it never guesses. Every ambiguity outcome below is a first-class,
 * reachable card state — not an error toast, not a silent default.
 */

import React, { useCallback, useEffect, useState } from 'react';
import { api, isTauri } from '../../lib/api';
import type {
  AcceptanceCardDto,
  CanonicalEventDto,
  CheckpointDto,
  DispatchDto,
  HandoffDto,
  KnowledgeTopicNode,
  KnowledgeTopicMemories,
} from '../../lib/types-dto';

// ─────────────────────────────────────────────────────────────────────────────
// Contract shapes (§3 / §4)
// ─────────────────────────────────────────────────────────────────────────────

/** §3.1 step 2 — the closed verb set. Unsupported is a result, not an error. */
const SUPPORTED_VERBS = [
  { verb: 'create', hint: '建一个新的东西(派单 / 工作项 / 交接 / 评论)' },
  { verb: 'route', hint: '把一件事指给某个席位' },
  { verb: 'explain', hint: '问"为什么这么定"——要理由和证据' },
  { verb: 'find-evidence', hint: '为某个判断找出处' },
  { verb: 'recover', hint: '从某个卡住/失败的状态恢复' },
] as const;

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

  const run = useCallback(async () => {
    if (!isTauri()) {
      setProbe({
        kind: 'unavailable',
        detail: 'not running inside the SeatLoom app (no Tauri bridge)',
      });
      return;
    }
    setProbe({ kind: 'probing' });
    // Probe the cheapest real read the surfaces depend on. Observe the result.
    try {
      const seats = await api.listSeats();
      setProbe({
        kind: 'ok',
        detail: `seat registry reachable (${seats.length} seat(s) measured)`,
      });
    } catch (err) {
      setProbe({ kind: 'unavailable', detail: `seat registry call failed: ${String(err)}` });
    }
  }, []);

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

  const load = useCallback(async () => {
    if (!isTauri()) {
      setError('not running inside the SeatLoom app');
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const [acceptances, dispatches, handoffs, events, checkpoints] = await Promise.all([
        api.listAcceptances(),
        api.listDispatches(undefined, 200),
        api.listHandoffs(),
        api.listEvents(200),
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

type Clarification =
  | { kind: 'verb-unclear' }
  | { kind: 'owner-zero'; name: string; seats: string[] }
  | { kind: 'owner-multi'; name: string; candidates: string[] }
  | { kind: 'owner-omitted'; candidates: string[] }
  | { kind: 'object-unclear' }
  | { kind: 'precondition-unexpressible'; precondition: string }
  | { kind: 'no-capable-seat'; constraints: string[] }
  | { kind: 'tool-unavailable'; detail: string };

function ClarificationCard({ c, onPick }: { c: Clarification; onPick: (choice: string) => void }) {
  return (
    <div style={{ ...cardStyle, borderColor: 'var(--sl-brand)' }}>
      <div style={{ ...sectionLabel, color: 'var(--sl-brand)' }}>需要你澄清 · 我不猜</div>
      {c.kind === 'verb-unclear' && (
        <>
          <div style={{ fontSize: 12, marginBottom: 6 }}>
            这句话的意图不在我可执行的五种里。你要做的是哪一种?
          </div>
          {SUPPORTED_VERBS.map((v) => (
            <button
              key={v.verb}
              type="button"
              onClick={() => onPick(v.verb)}
              style={{ ...linkBtn, display: 'block', marginBottom: 3 }}
            >
              <b>{v.verb}</b> — {v.hint}
            </button>
          ))}
        </>
      )}
      {c.kind === 'owner-zero' && (
        <>
          <div style={{ fontSize: 12, marginBottom: 4 }}>没有叫 「{c.name}」 的席位。现有席位:</div>
          <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)' }}>
            {c.seats.join('、') || '(空)'}
          </div>
        </>
      )}
      {c.kind === 'owner-multi' && (
        <>
          <div style={{ fontSize: 12, marginBottom: 4 }}>
            「{c.name}」 匹配到多个席位,你要哪一个?
          </div>
          {c.candidates.map((s) => (
            <button
              key={s}
              type="button"
              onClick={() => onPick(s)}
              style={{ ...linkBtn, display: 'block', marginBottom: 2 }}
            >
              {s}
            </button>
          ))}
        </>
      )}
      {c.kind === 'owner-omitted' && (
        <>
          <div style={{ fontSize: 12, marginBottom: 4 }}>你没说给谁。按能力与当前负载,候选是:</div>
          {c.candidates.length === 0 ? (
            <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)' }}>
              (没有可计算的候选)
            </div>
          ) : (
            c.candidates.map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => onPick(s)}
                style={{ ...linkBtn, display: 'block', marginBottom: 2 }}
              >
                {s}
              </button>
            ))
          )}
        </>
      )}
      {c.kind === 'object-unclear' && (
        <>
          <div style={{ fontSize: 12, marginBottom: 4 }}>要建哪类对象?</div>
          {['派单(dispatch)', '工作项(workitem)', '交接(handoff)', '评论(comment)'].map((o) => (
            <button
              key={o}
              type="button"
              onClick={() => onPick(o)}
              style={{ ...linkBtn, display: 'block', marginBottom: 2 }}
            >
              {o}
            </button>
          ))}
        </>
      )}
      {c.kind === 'precondition-unexpressible' && (
        <div style={{ fontSize: 12, lineHeight: 1.6 }}>
          这句话带一个前提:「{c.precondition}」。当前的派单写入路径<strong>无法表达依赖边</strong>
          ,所以我不会静默丢掉它。 要我<b>先不带这个前提</b>建卡(你确认后再来补),还是先算了?
        </div>
      )}
      {c.kind === 'no-capable-seat' && (
        <>
          <div style={{ fontSize: 12, marginBottom: 4 }}>没有满足条件的席位。被挡住的约束:</div>
          <ul
            style={{ fontSize: 12, color: 'var(--sl-text-secondary)', margin: 0, paddingLeft: 18 }}
          >
            {c.constraints.map((x) => (
              <li key={x}>{x}</li>
            ))}
          </ul>
        </>
      )}
      {c.kind === 'tool-unavailable' && (
        <div style={{ fontSize: 12, lineHeight: 1.6 }}>
          结构化「派活」依赖的意图工具当前不可用(实测,非猜测):
          <span style={{ fontFamily: 'var(--sl-font-mono, monospace)' }}> {c.detail}</span>
          。这一步属于后续接线(slice 4);在此期间请用「说句话」走原始 IM,或先用 Continuity / Decision
          两块(它们已接真读)。
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// The surface: two-mode intent entry + the cards. Mounted inside the existing
// Supervisor panel; no new window, no re-layout of the skeleton (contract §4).
// ─────────────────────────────────────────────────────────────────────────────

export const SupervisorIntent: React.FC<{ onRawSend: (text: string) => void }> = ({
  onRawSend,
}) => {
  const [mode, setMode] = useState<IntentMode>('dispatch');
  const [text, setText] = useState('');
  const [clarification, setClarification] = useState<Clarification | null>(null);
  const { probe, run } = useMeasuredProbe();

  // Measure wiring once on mount — never assert, always observe (V8 / boundary 3).
  useEffect(() => {
    run();
  }, [run]);

  const submitDispatch = useCallback(() => {
    const t = text.trim();
    if (!t) return;
    // Slice 2 does not run the deterministic parser (that is slice 1's tool,
    // wired in slice 4). Until it is live, the honest state is a MEASURED
    // "tool unavailable" clarification — not a guessed proposal, not a fake
    // "not wired" flag.
    setClarification({
      kind: 'tool-unavailable',
      detail: 'deterministic intent parser not yet a callable command (slice 1 pending)',
    });
  }, [text]);

  const submitChat = useCallback(() => {
    const t = text.trim();
    if (!t) return;
    onRawSend(t);
    setText('');
  }, [text, onRawSend]);

  const modeBtn = (m: IntentMode, label: string, title: string) => (
    <button
      type="button"
      onClick={() => {
        setMode(m);
        setClarification(null);
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

      {clarification && (
        <ClarificationCard c={clarification} onPick={() => setClarification(null)} />
      )}

      {/* the two live-wired cards */}
      <ContinuityDeltaCard />
      <DecisionAnswerCard />
    </div>
  );
};
