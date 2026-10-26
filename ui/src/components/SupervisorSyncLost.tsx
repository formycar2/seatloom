// SupervisorSyncLost — what the shell shows when the supervisor panel's live
// state link is broken.
//
// WHY THIS EXISTS. App.tsx subscribes to `supervisor:detached` and
// `supervisor:reembedded` so the embedded panel and ⌘K follow the panel as the
// operator moves it between the main window and its own OS window. Those two
// subscriptions had no failure handling at all, so a failing `listen()` was an
// unhandled rejection — invisible, and the state they were supposed to keep
// current silently froze at whatever the startup query had answered.
//
// The obvious repair is `.catch(() => {})`, which is what the neighbouring
// `supervisorWindowStatus()` call does. That is NOT what this does, and the
// difference matters: swallowing converts "an error happened" into "no error
// happened, and the state is real". Here the state would be a LIE with a
// confident face — `supervisorDetached === false` reads as "the panel is in this
// window", when the truth after a broken link is "I do not know any more".
//
// So the failure is stated, with the concrete consequence the operator can act
// on: ⌘K may open a second copy of the panel next to the detached one, and the
// panel may stay hidden in the main window after being re-embedded. That is not
// a hypothetical — it is exactly what a frozen `false` produces.
//
// What this deliberately does NOT do: retry. A failing event subscription is
// not a transient connection blip (that is what the database read's retry button
// is for); re-subscribing in a loop against a broken event bus would produce
// noise and no state. The step offered is the one that actually works.

import React from 'react';
import { AlertTriangle } from 'lucide-react';

export const SupervisorSyncLost: React.FC<{ reason: string }> = ({ reason }) => (
  <div
    role="status"
    data-testid="supervisor-sync-lost"
    style={{
      margin: '12px 16px 0',
      padding: '10px 14px',
      borderRadius: 'var(--sl-radius-md)',
      border: '1px solid var(--sl-yellow, #d29922)',
      background: 'var(--sl-yellow-subtle, rgba(210, 153, 34, 0.12))',
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
        color: 'var(--sl-yellow, #d29922)',
        fontSize: 13,
      }}
    >
      <AlertTriangle size={14} />
      监督面板的实时状态收不到了 —— 下面显示的是启动那一刻的答案
    </div>

    <div style={{ fontSize: 12, lineHeight: 1.7, color: 'var(--sl-text-secondary)' }}>
      <p>
        桌面端本该把「监督面板现在停在哪个窗口」的变化推送到这个界面。这个推送通道这次没接上,
        所以界面上的状态停在启动时查到的那一次,<strong>不会再变</strong>。
      </p>
      <p style={{ marginTop: 6 }}>
        看得见的后果:把面板拖成独立窗口后,这里可能还不知道,⌘K 于是会在主窗口里再开一份;
        把面板合回主窗口后,这里也可能还以为它是分离的,⌘K 就只顾着去聚焦那个窗口,面板不回来。
      </p>
      <p style={{ marginTop: 6 }}>后端原样返回的错误(实测,不是转述):</p>
      <div
        style={{
          fontFamily: 'var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, monospace)',
          wordBreak: 'break-all',
          marginTop: 4,
          fontSize: 11,
          color: 'var(--sl-red)',
        }}
      >
        {reason}
      </div>
      <p style={{ marginTop: 8 }}>
        下一步:<strong>重启应用</strong>。重启会重新查一次状态、重新接这条通道;在重启前,
        请把上面那段错误原文给 Aegis。
      </p>
    </div>
  </div>
);

export default SupervisorSyncLost;
