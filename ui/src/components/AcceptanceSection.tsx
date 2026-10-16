// AcceptanceSection — the D2 「待验收」 half of the dispatch page.
//
// Sits on the same page as the dispatch inbox, polled on the same 5s cadence,
// and answers the question the inbox cannot: *this work is delivered and
// Flux-verified — do I take it?*
//
// The card is built around one sentence. `how_to_verify` is the acceptance
// criterion ("do A, see B"), so it gets the most prominent block on the card;
// everything else is context for that decision. 接受 merges the branch and shows
// the merge commit that closed the loop; 打回 requires one line of feedback and
// shows the rework dispatch it spawned, linked back into the inbox above.
//
// Nimbus's commands (`cmd_list_acceptances` / `cmd_accept_delivery` /
// `cmd_reject_delivery`) are not registered on `track` yet. Until
// `ACCEPTANCE_CMDS_REGISTERED` is true this section renders that fact as its
// primary state instead of polling a command that would reject every 5s and
// showing the operator a permanent error. House rule: no mock stands in for a
// real backend value.

import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  AlertCircle,
  Ban,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  ClipboardCheck,
  GitMerge,
  GitPullRequestArrow,
  Link2,
  Loader2,
  MessageSquarePlus,
  Search,
  Undo2,
  X,
} from 'lucide-react';
import { ACCEPTANCE_CMDS_REGISTERED, api } from '../lib/api';
import type { AcceptanceCardDto } from '../lib/types-dto';
import {
  acceptanceStatusStyle,
  fluxVerdictStyle,
  isCancelled,
  isDecided,
  looksLikeMergeConflict,
  normaliseAcceptanceCard,
} from '../lib/acceptance';

/** Same cadence as the dispatch inbox above — one rhythm for the whole page. */
const POLL_MS = 5000;

const EMPTY_CARD_STATE: CardState = { busy: null, error: null, decided: null };

function fmtTime(iso: string | null): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const p = (n: number) => n.toString().padStart(2, '0');
  return `${d.getMonth() + 1}/${d.getDate()} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** One decision in flight, per card. */
interface CardState {
  busy: 'accept' | 'reject' | null;
  /** Verbatim backend output — merge conflicts are shown, never swallowed. */
  error: string | null;
  /** Set once a decision lands, so the card can show its closed-loop result. */
  decided: AcceptanceCardDto | null;
  /** The operator has read the result and closed the card. */
  dismissed?: boolean;
}

const AcceptanceSection: React.FC<{
  /** Seat names come from the inbox's own DB-backed seat list. */
  seatName: (seatId: string) => string;
  /** Mirrors the inbox's pause switch: pausing stops this poll too. */
  live: boolean;
  /** Jump the inbox above to a dispatch id (used by the rework link). */
  onLocateDispatch: (dispatchId: string) => void;
}> = ({ seatName, live, onLocateDispatch }) => {
  const [cards, setCards] = useState<AcceptanceCardDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [cardState, setCardState] = useState<Record<string, CardState>>({});
  const [feedback, setFeedback] = useState<Record<string, string>>({});

  const load = useCallback(async (silent = false) => {
    if (!silent) setLoading(true);
    setError(null);
    try {
      const rows = await api.listAcceptances();
      // Normalise at the boundary: the wire spelling is a cross-lane detail.
      setCards(rows.map(normaliseAcceptanceCard).filter((c): c is AcceptanceCardDto => c !== null));
    } catch (e) {
      setError(String(e));
    } finally {
      if (!silent) setLoading(false);
    }
  }, []);

  useEffect(() => {
    // Not registered, or the operator paused the page: no fetch is armed, so the
    // section must not claim to be loading.
    if (!ACCEPTANCE_CMDS_REGISTERED || !live) {
      setLoading(false);
      return;
    }
    void load();
    const timer = setInterval(() => void load(true), POLL_MS);
    return () => clearInterval(timer);
  }, [load, live]);

  const decide = useCallback(
    async (card: AcceptanceCardDto, action: 'accept' | 'reject') => {
      const reason = (feedback[card.id] ?? '').trim();
      // 打回 is a rework order; without a reason the seat cannot act on it.
      if (action === 'reject' && !reason) return;

      setCardState((s) => ({
        ...s,
        [card.id]: { busy: action, error: null, decided: null },
      }));
      try {
        const updated =
          action === 'accept'
            ? await api.acceptDelivery(card.id)
            : await api.rejectDelivery(card.id, reason);
        const decidedCard = normaliseAcceptanceCard(updated);
        setCardState((s) => ({
          ...s,
          [card.id]: { busy: null, error: null, decided: decidedCard ?? card },
        }));
        // Deliberately NOT reloading here: the result (merge commit / rework
        // dispatch) is the point of the action, and an immediate poll would
        // drop the card out of the queue before the operator could read it.
        // The next 5s poll removes it once the backend agrees it is decided.
      } catch (e) {
        // Verbatim. A merge conflict's git output is the whole diagnosis.
        setCardState((s) => ({
          ...s,
          [card.id]: { busy: null, error: String(e), decided: null },
        }));
      }
    },
    [feedback],
  );

  const dismiss = useCallback((id: string) => {
    setCardState((s) => ({ ...s, [id]: { ...(s[id] ?? EMPTY_CARD_STATE), dismissed: true } }));
  }, []);

  // Awaiting = the backend has not settled it AND the operator has not closed it.
  // A card the user just decided stays put (with its result) until dismissed,
  // so the merge commit is readable rather than a flash.
  //
  // `isDecided` now knows about `cancelled`, so a voided card is no longer counted
  // here. It used to be: the operator's 待验收 read 7 when 5 were actually waiting
  // on him, and the extra two were cards that had been voided days earlier.
  const awaiting = useMemo(
    () => cards.filter((c) => !isDecided(c.status) && !cardState[c.id]?.dismissed),
    [cards, cardState],
  );

  // Voided cards, kept but never counted. Collapsed by default: they are not work,
  // they are the record of why something was withdrawn — and the operator has
  // asked "where did that go" before, so hiding them entirely is not an option
  // either. The reason travels in `feedback` (measured: both currently-voided
  // cards carry one), so this needs no extra read.
  const voided = useMemo(() => cards.filter((c) => isCancelled(c.status)), [cards]);

  if (!ACCEPTANCE_CMDS_REGISTERED) {
    return (
      <section className="rounded-xl border border-dashed border-border/60 bg-card/60 p-5">
        <div className="flex items-start gap-3">
          <div className="p-2 rounded-lg bg-secondary text-text-secondary shrink-0">
            <ClipboardCheck size={16} />
          </div>
          <div className="min-w-0">
            <h3 className="text-sm font-black text-text-primary">待验收</h3>
            <p className="text-xs text-text-secondary mt-1 leading-5">
              这一区按 D2 契约实现完毕,但后端命令尚未上线 ——
              <span className="font-mono text-text-primary">
                cmd_list_acceptances / cmd_accept_delivery / cmd_reject_delivery
              </span>{' '}
              还在 Nimbus 的 lane,未合入 track。命令上线后此区自动生效(切换
              <span className="font-mono text-text-primary"> ACCEPTANCE_CMDS_REGISTERED</span>
              ),不需要再改 UI。
            </p>
            <p className="text-[11px] text-text-secondary mt-2 opacity-70">
              按 house 规则,未接线部分不使用 mock 伪装后端真值。
            </p>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section className="space-y-3">
      <div className="flex items-center gap-2">
        <ClipboardCheck size={15} className="text-text-secondary" />
        <h3 className="text-sm font-black text-text-primary uppercase tracking-tight">待验收</h3>
        <span className="text-[10px] font-bold text-text-secondary uppercase tracking-widest">
          {awaiting.length} 项待决策
        </span>
        {loading ? (
          <Loader2 size={12} className="animate-spin text-text-secondary" />
        ) : (
          !live && <span className="text-[10px] font-bold text-text-secondary">已暂停</span>
        )}
      </div>

      {error ? (
        <div className="flex items-center gap-2 text-xs font-bold text-red-600">
          <AlertCircle size={14} /> 加载待验收失败:{error}
        </div>
      ) : awaiting.length === 0 ? (
        <div className="rounded-xl border border-border/40 bg-card p-6 text-center">
          <p className="text-sm font-bold text-text-secondary">没有待验收的交付</p>
          <p className="text-xs text-text-secondary mt-1 opacity-60">
            {live
              ? '席位交付并通过 Flux 技术验证后,会出现在这里等你验收。'
              : '自动刷新已暂停 —— 恢复后这里会显示待验收的交付。'}
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {awaiting.map((card) => (
            <AcceptanceCard
              key={card.id}
              card={card}
              seatLabel={seatName(card.seatId)}
              state={cardState[card.id]}
              feedbackValue={feedback[card.id] ?? ''}
              onFeedbackChange={(v) => setFeedback((f) => ({ ...f, [card.id]: v }))}
              onDecide={(action) => void decide(card, action)}
              onDismiss={() => dismiss(card.id)}
              onLocateDispatch={onLocateDispatch}
            />
          ))}
        </div>
      )}

      {voided.length > 0 && <VoidedCards cards={voided} seatName={seatName} />}
    </section>
  );
};

/**
 * 已作废的卡:看得见、说得清为什么,但不占「几项待决策」那个数。
 *
 * 默认折叠。它们不是活,是「这个东西为什么撤了」的记录 —— 张先生问过「那张卡去哪了」,
 * 所以不能干脆不显示;但每打开一次页面就摊在他眼前也不行,那不是他要决定的事。
 */
const VoidedCards: React.FC<{ cards: AcceptanceCardDto[]; seatName: (id: string) => string }> = ({
  cards,
  seatName,
}) => {
  const [open, setOpen] = useState(false);
  return (
    <div className="rounded-xl border border-dashed border-border/50 bg-card/40">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        className="w-full flex items-center gap-2 px-4 py-2.5 text-left"
      >
        {open ? (
          <ChevronDown size={13} className="text-text-secondary" />
        ) : (
          <ChevronRight size={13} className="text-text-secondary" />
        )}
        <span className="text-[11px] font-black text-text-secondary uppercase tracking-widest">
          已作废 {cards.length} 项
        </span>
        <span className="text-[10px] text-text-secondary opacity-70">
          这些不算在待决策里 —— 点开看为什么撤
        </span>
      </button>
      {open && (
        <div className="space-y-2 px-4 pb-3">
          {cards.map((card) => (
            <div
              key={card.id}
              className="rounded-lg border border-border/40 bg-card p-3 space-y-1.5"
            >
              <div className="flex items-center gap-2 flex-wrap">
                <span className="shrink-0 px-2 py-0.5 rounded-md text-[10px] font-black border bg-secondary text-text-secondary border-border/40">
                  已作废
                </span>
                <span className="text-xs font-bold text-text-primary flex-1 min-w-[160px]">
                  {card.dispatchTitle || card.dispatchId}
                </span>
                <span className="text-[10px] text-text-secondary whitespace-nowrap">
                  {seatName(card.seatId) || card.seatId} · 开卡于 {fmtTime(card.createdAt)}
                </span>
              </div>
              {card.feedback ? (
                <p className="text-[11px] text-text-secondary leading-5">「{card.feedback}」</p>
              ) : (
                <p className="text-[11px] text-text-secondary leading-5 opacity-70">
                  这张卡没记作废理由。理由在账本的 AcceptanceCancelled 事件里,监督面板那一栏能看到。
                </p>
              )}
              <p
                className="text-[10px] font-mono text-text-muted break-all"
                title={`dispatch ${card.dispatchId}`}
              >
                {card.dispatchId}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

interface AcceptanceCardProps {
  card: AcceptanceCardDto;
  seatLabel: string;
  state?: CardState;
  feedbackValue: string;
  onFeedbackChange: (value: string) => void;
  onDecide: (action: 'accept' | 'reject') => void;
  onDismiss: () => void;
  onLocateDispatch: (dispatchId: string) => void;
}

const AcceptanceCard: React.FC<AcceptanceCardProps> = ({
  card,
  seatLabel,
  state,
  feedbackValue,
  onFeedbackChange,
  onDecide,
  onDismiss,
  onLocateDispatch,
}) => {
  const st = acceptanceStatusStyle(card.status);
  const flux = fluxVerdictStyle(card.fluxVerdict.verdict);
  const busy = state?.busy ?? null;
  const trimmedFeedback = feedbackValue.trim();
  const conflict = state?.error && looksLikeMergeConflict(state?.error) ? state.error : null;

  return (
    <article className="rounded-xl border border-border/40 bg-card p-4 space-y-3">
      {/* Headline: what is being accepted, by which seat, off which branch. */}
      <div className="flex items-start gap-2 flex-wrap">
        <span
          className={`shrink-0 px-2 py-0.5 rounded-md text-[10px] font-black border whitespace-nowrap ${st.cls}`}
        >
          {st.label}
        </span>
        <h4 className="text-sm font-bold text-text-primary flex-1 min-w-[200px]">
          {card.dispatchTitle || card.dispatchId}
        </h4>
        <span className="text-[10px] font-bold text-text-secondary whitespace-nowrap">
          {fmtTime(card.createdAt)}
        </span>
      </div>

      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-text-secondary">
        <span className="font-bold text-text-primary">{seatLabel || card.seatId}</span>
        <span className="flex items-center gap-1 font-mono">
          <GitPullRequestArrow size={11} />
          {card.branch || '—'}
          <span className="text-text-muted">→</span>
          {card.base || '—'}
        </span>
        <span className="font-mono text-text-muted" title={`dispatch ${card.dispatchId}`}>
          {card.dispatchId}
        </span>
      </div>

      {/* The acceptance criterion — the one sentence the decision is made on. */}
      <div className="rounded-lg border border-primary/25 bg-accent/20 p-3">
        <p className="text-[9px] font-black text-text-secondary uppercase tracking-widest mb-1">
          验证方式 · 做 A 看到 B
        </p>
        {card.howToVerify ? (
          <p className="text-[13px] leading-6 text-text-primary whitespace-pre-wrap break-words">
            {card.howToVerify}
          </p>
        ) : (
          <p className="text-[12px] text-text-secondary">该交付没有填写验证方式。</p>
        )}
      </div>

      {/* Flux's technical verdict — a precondition, shown but not the decision. */}
      <div className="flex flex-wrap items-center gap-2 text-[11px]">
        <span
          className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-black border whitespace-nowrap ${flux.cls}`}
        >
          <CheckCircle2 size={10} />
          {flux.label}
        </span>
        {card.fluxVerdict.refId && (
          <span className="font-mono text-text-muted" title="Flux 验绿记录">
            {card.fluxVerdict.refId}
          </span>
        )}
        {card.fluxVerdict.occurredAt && (
          <span className="text-text-muted">{fmtTime(card.fluxVerdict.occurredAt)}</span>
        )}
      </div>

      {/* Decision, or the closed-loop result of one already made. */}
      {state?.decided ? (
        <DecisionResult
          card={state.decided}
          onLocateDispatch={onLocateDispatch}
          onDismiss={onDismiss}
        />
      ) : (
        <div className="space-y-2 pt-1 border-t border-border/30">
          <div className="flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => onDecide('accept')}
              disabled={busy !== null}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-primary text-white text-xs font-bold shadow-sm transition-all disabled:opacity-40 hover:bg-primary/90"
            >
              {busy === 'accept' ? (
                <Loader2 size={13} className="animate-spin" />
              ) : (
                <GitMerge size={13} />
              )}
              接受并合并
            </button>
            <button
              type="button"
              onClick={() => onDecide('reject')}
              disabled={busy !== null || !trimmedFeedback}
              title={trimmedFeedback ? '' : '打回需要一句理由,席位才能据此重做'}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-border/60 bg-card text-text-secondary text-xs font-bold hover:bg-accent transition-all disabled:opacity-40"
            >
              {busy === 'reject' ? (
                <Loader2 size={13} className="animate-spin" />
              ) : (
                <Undo2 size={13} />
              )}
              打回
            </button>
            {!trimmedFeedback && (
              <span className="text-[10px] text-text-muted">打回需填写一句理由</span>
            )}
          </div>

          <label className="block space-y-1">
            <span className="flex items-center gap-1 text-[9px] font-black text-text-secondary uppercase tracking-widest">
              <MessageSquarePlus size={10} />
              打回理由(必填一句)
            </span>
            <textarea
              value={feedbackValue}
              onChange={(e) => onFeedbackChange(e.target.value)}
              rows={2}
              placeholder="例如:派单页在 1280px 下状态列被截断,请按断点隐藏「发出」列。"
              className="w-full px-3 py-2 bg-secondary/60 border border-border/40 rounded-lg text-xs focus:outline-none focus:ring-1 focus:ring-primary focus:bg-card resize-y"
            />
          </label>
        </div>
      )}

      {/* Errors, verbatim. A merge conflict is git's own output, not a summary. */}
      {state?.error && (
        <div className="rounded-lg border border-red-500/30 bg-red-500/10 p-3 space-y-1">
          <p className="flex items-center gap-1.5 text-[11px] font-black text-red-600">
            <AlertCircle size={12} />
            {conflict ? '合并冲突 · git 原样输出' : '操作失败'}
          </p>
          <pre className="text-[11px] leading-5 text-text-primary whitespace-pre-wrap break-words font-mono max-h-48 overflow-y-auto">
            {state.error}
          </pre>
        </div>
      )}
    </article>
  );
};

/** 接受 → merge commit; 打回 → the rework dispatch, linked into the inbox. */
const DecisionResult: React.FC<{
  card: AcceptanceCardDto;
  onLocateDispatch: (dispatchId: string) => void;
  onDismiss: () => void;
}> = ({ card, onLocateDispatch, onDismiss }) => {
  const accepted = card.status === 'accepted' || Boolean(card.mergeCommit);
  const rejected = card.status === 'rejected' || Boolean(card.reworkDispatchId);
  // The same four-states-not-two shape as isDecided: without this, a cancelled
  // card falls through both branches and renders an empty result strip.
  const cancelled = isCancelled(card.status);

  return (
    <div className="flex flex-wrap items-center gap-2 pt-1 border-t border-border/30 text-[11px]">
      {cancelled ? (
        <>
          <Ban size={13} className="text-text-secondary" />
          <span className="font-bold text-text-secondary">已作废</span>
          {card.feedback && <span className="text-text-secondary">「{card.feedback}」</span>}
        </>
      ) : accepted ? (
        <>
          <CheckCircle2 size={13} className="text-emerald-600" />
          <span className="font-bold text-emerald-600">已接受并合并</span>
          {card.mergeCommit ? (
            <span className="font-mono text-text-primary break-all" title="合并提交">
              {card.mergeCommit}
            </span>
          ) : (
            <span className="text-text-secondary">未返回合并提交号。</span>
          )}
        </>
      ) : rejected ? (
        <>
          <Undo2 size={13} className="text-rose-600" />
          <span className="font-bold text-rose-600">已打回</span>
          {card.feedback && <span className="text-text-secondary">「{card.feedback}」</span>}
          {card.reworkDispatchId ? (
            <button
              type="button"
              onClick={() => onLocateDispatch(card.reworkDispatchId as string)}
              title="在上方派单收件箱中定位这条新派单"
              className="flex items-center gap-1 px-2 py-0.5 rounded-md border border-primary/30 bg-accent/20 text-primary font-bold hover:bg-accent/40 transition-colors"
            >
              <Link2 size={10} />
              新派单 {card.reworkDispatchId}
              <Search size={10} />
            </button>
          ) : (
            <span className="text-text-secondary">未返回重派单号。</span>
          )}
        </>
      ) : (
        <span className="text-text-secondary">已处理。</span>
      )}

      {/* The result is the point of the action, so it stays until read. */}
      <button
        type="button"
        onClick={onDismiss}
        title="关闭这条结果"
        aria-label="关闭这条结果"
        className="ml-auto p-1 rounded-md text-text-secondary hover:text-text-primary hover:bg-accent transition-colors"
      >
        <X size={12} />
      </button>
    </div>
  );
};

export default AcceptanceSection;
