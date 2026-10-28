// HydrationFailure — what the shell shows when the database read did not succeed.
//
// WHY THIS EXISTS. `hydrateFromBackend` has always caught its failure and stored it in
// `backendHydrateError` — and **nothing in the UI ever rendered that string**. So a
// failed hydration was completely silent: the store simply kept whatever it had. Until
// 2026-10-16 that was three hardcoded seed projects, so the operator saw three
// plausible-looking project cards and concluded the product had three projects.
//
// That is the failure this component removes. A read that did not succeed is now stated,
// with the exact error text the backend returned and the next step — never a spinner
// forever, never a silent empty list, and never fake cards standing in for real ones.
//
// What it deliberately does NOT do: retry automatically, or guess which of the eight
// reads failed. `Promise.all` rejects with the first rejection, and the other seven are
// abandoned; saying which one it was would be a guess. The error text names it.

import React from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

const mono: React.CSSProperties = {
  fontFamily: 'var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, monospace)',
  wordBreak: 'break-all',
};

export const HydrationFailure: React.FC<{
  error: string;
  onRetry: () => void;
}> = ({ error, onRetry }) => (
  <div
    role="alert"
    style={{
      margin: '12px 16px',
      padding: '12px 14px',
      borderRadius: 'var(--sl-radius-md)',
      border: '1px solid var(--sl-red)',
      background: 'var(--sl-red-subtle)',
      display: 'flex',
      flexDirection: 'column',
      gap: 8,
    }}
  >
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        fontWeight: 700,
        color: 'var(--sl-red)',
        fontSize: 13,
      }}
    >
      <AlertTriangle size={14} />
      读不到数据库 —— 上面这些不是你的项目
    </div>

    <div style={{ fontSize: 12, lineHeight: 1.7, color: 'var(--sl-text-secondary)' }}>
      <p>
        启动时要从 PostgreSQL 读项目、席位、工作项这些真数据,这一次没读成功。
        在它成功之前,这个界面**不显示任何项目** —— 不拿写死的假数据顶着,那比空着更坏。
      </p>
      <p style={{ marginTop: 6 }}>后端原样返回的错误(实测,不是转述):</p>
      <div style={{ ...mono, marginTop: 4, fontSize: 11, color: 'var(--sl-red)' }}>{error}</div>
      <p style={{ marginTop: 8 }}>下一步,按顺序试:</p>
      <ol style={{ margin: '4px 0 0 18px', padding: 0, lineHeight: 1.8 }}>
        <li>
          确认数据库容器在跑:<code style={mono}>podman ps</code> 里应该有{' '}
          <span style={mono}>seatloom-postgres</span>。不在就{' '}
          <code style={mono}>bash scripts/bootstrap.sh</code>。
        </li>
        <li>点下面「重读一次」—— 偶发的连接抖动重读就好。</li>
        <li>
          还不行就把上面那段错误原文给 Aegis:它说清了缺的是哪一样(连不上、某张表没有、
          还是某个命令没注册)。
        </li>
      </ol>
    </div>

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
        <RefreshCw size={12} /> 重读一次
      </button>
    </div>
  </div>
);

export default HydrationFailure;
