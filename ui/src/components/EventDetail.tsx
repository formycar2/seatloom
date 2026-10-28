// EventDetail — what a clicked timeline row shows.
//
// WHY THIS EXISTS. `TimelineView`'s row click has always called
// `onSelectObject('Event', event)`, but `SelectedObjectType` had no 'Event', so the
// selection never resolved and `MasterDetail` rendered no detail pane at all — a click
// that did nothing. Adding 'Event' to the type makes the selection resolvable; this
// component is what it resolves to.
//
// WHAT IT IS NOT. It is not the lineage pane. The lineage read model
// (`disp-lineage-ui`) is blocked: `lineage` is served by a binary built from
// `packet/nimbus/gate-vs-announcement` and is not in this tree or on `track`, and the
// DTO that would carry its answer is Nimbus's fence. So this detail says exactly that,
// in the operator's language, rather than either doing nothing or pretending to be the
// pane. Coordination went out as a SupervisorMessage; when the pane lands, this
// component is the entry point it replaces.
//
// WHAT IT DOES show: everything the event row itself carries — what happened, when,
// who did it, and the raw payload, so a click is never a dead end.

import React from 'react';
import { AlertTriangle, Clock, User } from 'lucide-react';
import type { ActorRef } from '../types';

/**
 * The shape TimelineView passes: a canonical event, already in the UI's own model
 * (`ui/src/types`). `event_id` is the id; the rest is what the row renders. Modelled
 * rather than `any` so a shape change cannot silently blank the pane.
 */
interface EventLike {
  event_id: string;
  event_type: string;
  occurred_at: string;
  /** `ActorRef` is a union — 'Human' | { Seat } | 'Automation' — so render it, do not
   *  assume it is a string. */
  actor_ref: ActorRef;
  payload?: unknown;
  summary?: string;
  title?: string;
}

const mono: React.CSSProperties = {
  fontFamily: 'var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, monospace)',
  wordBreak: 'break-all',
};

function fmtWhen(iso: string | undefined | null): string {
  if (!iso) return '没记时间';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const p = (n: number) => n.toString().padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

export const EventDetail: React.FC<{ event: EventLike }> = ({ event }) => {
  const type = event.event_type || '(没记类型)';
  const when = event.occurred_at;
  const who =
    typeof event.actor_ref === 'string'
      ? event.actor_ref
      : `席位 ${Object.values(event.actor_ref)[0]}`;
  const headline = event.summary ?? event.title ?? null;

  return (
    <div className="space-y-3">
      <div className="flex items-start gap-2 flex-wrap">
        <span className="shrink-0 px-2 py-0.5 rounded-md text-[10px] font-black border bg-secondary text-text-secondary border-border/40 whitespace-nowrap">
          账本事件
        </span>
        <h3 className="text-sm font-bold text-text-primary flex-1 min-w-[200px]">
          {headline ?? type}
        </h3>
      </div>

      <dl className="space-y-1.5 text-[11px]">
        <div className="flex gap-2">
          <dt className="text-text-tertiary w-14 shrink-0">类型</dt>
          <dd className="font-bold text-text-primary" style={mono}>
            {type}
          </dd>
        </div>
        <div className="flex gap-2 items-center">
          <dt className="text-text-tertiary w-14 shrink-0">时间</dt>
          <dd className="text-text-secondary flex items-center gap-1">
            <Clock size={11} /> {fmtWhen(when)}
          </dd>
        </div>
        <div className="flex gap-2 items-center">
          <dt className="text-text-tertiary w-14 shrink-0">是谁</dt>
          <dd className="text-text-secondary flex items-center gap-1">
            <User size={11} /> <span style={mono}>{who ?? '没记'}</span>
          </dd>
        </div>
        <div className="flex gap-2">
          <dt className="text-text-tertiary w-14 shrink-0">事件号</dt>
          <dd className="text-text-secondary" style={mono}>
            {event.event_id}
          </dd>
        </div>
      </dl>

      {event.payload != null && (
        <div>
          <div className="text-[9px] font-black text-text-tertiary uppercase tracking-widest mb-1">
            这条事件自带的内容
          </div>
          <pre
            className="rounded-lg border border-border/40 bg-card p-2.5 text-[10.5px] leading-relaxed overflow-auto max-h-72"
            style={{ ...mono, whiteSpace: 'pre-wrap' }}
          >
            {typeof event.payload === 'string'
              ? event.payload
              : JSON.stringify(event.payload, null, 2)}
          </pre>
        </div>
      )}

      {/* The honest part: the full story of this event is a pane that does not exist yet. */}
      <div className="rounded-lg border border-amber-500/30 bg-amber-500/10 p-2.5 flex items-start gap-2">
        <AlertTriangle size={13} className="text-amber-600 shrink-0 mt-0.5" />
        <div className="text-[11px] leading-relaxed text-text-secondary">
          <p style={{ color: 'var(--sl-text-primary)', fontWeight: 600 }}>
            这条事件的完整来龙去脉现在读不到。
          </p>
          <p className="mt-1">
            「这是什么 / 谁做的 / 什么时候 / 产出了什么 / 卡在谁 / 证据在哪」那一屏还没建: 它要用的
            lineage 读取能力在另一条分支上,没有合进 track,而这个构建里没有它。
            上面这些是这条事件自己带着的,是真的;那一屏缺着,这里不说它有。
          </p>
        </div>
      </div>
    </div>
  );
};

export default EventDetail;
