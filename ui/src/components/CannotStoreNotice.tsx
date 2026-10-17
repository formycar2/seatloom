// CannotStoreNotice — the honest thing a write-only form must say at save time.
//
// WHY. Four forms (WorkItemForm, HandoffForm, AddSeatDialog, DelegationOverlay) accept
// input and imply success, but `main.rs` registers no create command for workitem /
// handoff / seat / delegation, and the forms make zero `api.*` calls. So the data lives
// in a zustand store until reload and is then gone. The button says 创建并写入 — "create
// and write" — which is a false claim in the unsafe direction.
//
// The dispatch's rule: pick ONE per form — wire it to a real command, or tell the
// operator at save time that this build cannot store it and that closing discards the
// input. No middle state where it looks saved. There is no command to wire to, so this
// is the second option, and the missing commands are named rather than invented.
//
// WHAT IS NOT DONE HERE: the local write is not removed. That would change what the form
// does beyond honesty, and the dispatch's boundary is the interface, not the store. The
// notice makes the truth visible at the moment it matters.

import React from 'react';
import { AlertTriangle } from 'lucide-react';

/** The exact command that would have to exist for this form to store anything. */
export const MISSING_COMMANDS = {
  workitem: 'cmd_create_workitem',
  handoff: 'cmd_create_handoff',
  seat: 'cmd_create_seat',
  delegation: 'cmd_create_delegation',
} as const;

export type CannotStoreKind = keyof typeof MISSING_COMMANDS;

const KIND_LABEL: Record<CannotStoreKind, string> = {
  workitem: '工作项',
  handoff: '交接',
  seat: '席位',
  delegation: '委托',
};

export const CannotStoreNotice: React.FC<{ kind: CannotStoreKind }> = ({ kind }) => (
  <div
    className="rounded-lg border border-amber-500/30 bg-amber-500/10 p-2.5 flex items-start gap-2"
    role="note"
  >
    <AlertTriangle size={13} className="text-amber-600 shrink-0 mt-0.5" />
    <div className="text-[11px] leading-relaxed text-text-secondary">
      <p style={{ color: 'var(--sl-text-primary)', fontWeight: 600 }}>
        这个构建存不下它 —— 关掉这个窗口,你刚填的就没有了。
      </p>
      <p className="mt-1">
        后端没有登记「新建{KIND_LABEL[kind]}」这条命令(
        <span
          style={{
            fontFamily: 'var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, monospace)',
          }}
        >
          {MISSING_COMMANDS[kind]}
        </span>
        不在这个构建里),所以表单没法把它写进数据库。刚才那一闪是写进了浏览器内存,刷新就没了。
      </p>
      <p className="mt-1">
        缺的是后端一条命令,不是界面一个按钮。命令上线后这里会自动变成真的存下去,不用再改界面。
      </p>
    </div>
  </div>
);

export default CannotStoreNotice;
