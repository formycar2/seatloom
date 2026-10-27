// SupervisorSyncLost — what the shell shows when the supervisor panel's state
// cannot be trusted.
//
// WHY THIS EXISTS. `App.tsx` learns where the supervisor panel lives from two
// places: a STARTUP QUERY (`supervisorWindowStatus()`) that answers once, and
// two SUBSCRIPTIONS (`supervisor:detached` / `supervisor:reembedded`) that keep
// the answer current. All three used to fail silently — the query with
// `.catch(() => {})` and the subscriptions with no handler at all — which left
// `supervisorDetached` frozen at its `useState` initial `false`. That value
// reads as "the panel is in this window"; the truth after either failure is
// "I do not know".
//
// The obvious repair for the query is `.catch(() => {})`, which is what it
// already had. That is NOT what this does: swallowing converts "an error
// happened" into "nothing happened and the state is real". The state is
// load-bearing — it decides whether ⌘K opens a SECOND copy of the panel beside
// the detached one, and whether the embedded panel renders in the main window
// while the panel is actually detached — so a silently stale value produces a
// wrong visible behaviour.
//
// TWO SHAPES, BECAUSE THE TWO FAILURES ARE NOT THE SAME THING:
//
//   `never-answered` — the startup query failed, so the state has never been
//     answered at all. This is the worse of the two: there was no answer to
//     begin with. It gets a RETRY button, because a query is a single read and
//     a transient failure is exactly what re-reading fixes — the same reason
//     the database read offers one.
//
//   `stale` — the subscriptions failed, so an answer that WAS correct has
//     stopped being maintained. It gets no retry: re-subscribing against a
//     broken event bus produces noise and no state, and the honest step is a
//     restart.
//
// Collapsing them into one message would tell the operator "something is
// wrong" without telling them whether there is anything they can do about it.
// The distinction is the actionable part, so it is kept.
//
// What this deliberately does NOT do: guess. It never invents a position for
// the panel, and it does not claim to know which of the two subscription
// channels broke beyond naming the ones that reported.

import React from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

const mono: React.CSSProperties = {
  fontFamily: 'var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, monospace)',
  wordBreak: 'break-all',
};

export type SupervisorSyncLostKind = 'never-answered' | 'stale';

export const SupervisorSyncLost: React.FC<{
  kind: SupervisorSyncLostKind;
  reason: string;
  onRetry?: () => void;
}> = ({ kind, reason, onRetry }) => {
  const neverAnswered = kind === 'never-answered';

  return (
    <div
      role="status"
      data-testid="supervisor-sync-lost"
      data-kind={kind}
      style={{
        margin: '12px 16px 0',
        padding: '10px 14px',
        borderRadius: 'var(--sl-radius-md)',
        // Red for never-answered: this is the first read failing, so there is no
        // answer to fall back on. Yellow for stale: there WAS an answer, it has
        // just stopped being maintained. The colour difference is the fastest
        // read of "is there anything to fall back on".
        border: `1px solid ${neverAnswered ? 'var(--sl-red)' : 'var(--sl-yellow, #d29922)'}`,
        background: neverAnswered
          ? 'var(--sl-red-subtle)'
          : 'var(--sl-yellow-subtle, rgba(210, 153, 34, 0.12))',
        display: 'flex',
        flexDirection: 'column',
        gap: 6,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          fontWeight: 700,
          color: neverAnswered ? 'var(--sl-red)' : 'var(--sl-yellow, #d29922)',
          fontSize: 13,
        }}
      >
        <AlertTriangle size={14} />
        {neverAnswered
          ? '读不到监督面板现在在哪 —— 这个界面不知道,也不猜'
          : '监督面板的实时状态收不到了 —— 下面显示的是启动那一刻的答案'}
      </div>

      <div style={{ fontSize: 12, lineHeight: 1.7, color: 'var(--sl-text-secondary)' }}>
        {neverAnswered ? (
          <>
            <p>
              启动时要问一次桌面端「监督面板现在停在哪个窗口」,<strong>这次没问到</strong>。
              在问到之前,这个界面<strong>不知道</strong>面板在哪 —— 不拿「应该在主窗口里」顶着,
              那比不知道更坏。
            </p>
            <p style={{ marginTop: 6 }}>
              看得见的后果:把面板拖成独立窗口后,这里可能还不知道,⌘K 于是会在主窗口里再开一份;
              把面板合回主窗口后,这里也可能还以为它是分离的,⌘K 就只顾着去聚焦那个窗口,面板不回来。
            </p>
          </>
        ) : (
          <>
            <p>
              桌面端本该把「监督面板现在停在哪个窗口」的变化推送到这个界面。这个推送通道这次没接上,
              所以界面上的状态停在启动时查到的那一次,<strong>不会再变</strong>。
            </p>
            <p style={{ marginTop: 6 }}>
              看得见的后果:把面板拖成独立窗口后,这里可能还不知道,⌘K 于是会在主窗口里再开一份;
              把面板合回主窗口后,这里也可能还以为它是分离的,⌘K 就只顾着去聚焦那个窗口,面板不回来。
            </p>
          </>
        )}

        <p style={{ marginTop: 6 }}>后端原样返回的错误(实测,不是转述):</p>
        <div style={{ ...mono, marginTop: 4, fontSize: 11, color: 'var(--sl-red)' }}>{reason}</div>

        <p style={{ marginTop: 8 }}>
          {neverAnswered ? (
            <>
              下一步:点下面「再问一次」—— 偶发的连接抖动重问就好。还不行就<strong>重启应用</strong>;
              重启前,请把上面那段错误原文给 Aegis。
            </>
          ) : (
            <>
              下一步:<strong>重启应用</strong>。重启会重新查一次状态、重新接这条通道;在重启前,
              请把上面那段错误原文给 Aegis。
            </>
          )}
        </p>
      </div>

      {/* Only the never-answered shape gets a retry: re-reading is what fixes a
          single failed query, and is exactly what does NOT fix a broken event
          subscription. Offering it on the stale shape would be a button that
          cannot work. */}
      {neverAnswered && onRetry && (
        <div>
          <button
            type="button"
            onClick={onRetry}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 6,
              padding: '5px 12px',
              fontSize: 12,
              fontWeight: 600,
              background: 'var(--sl-surface)',
              border: '1px solid var(--sl-border)',
              borderRadius: 'var(--sl-radius-sm)',
              color: 'var(--sl-text-primary)',
              cursor: 'pointer',
            }}
          >
            <RefreshCw size={12} /> 再问一次
          </button>
        </div>
      )}
    </div>
  );
};

export default SupervisorSyncLost;
