// SyncFreshness — when the data on screen was last actually synced.
//
// WHY THIS EXISTS. AEGIS-2026-10-20-v5-honest-surfaces-v1 item 2: the supervision
// header rendered `{new Date().toLocaleTimeString()}` under the label "最后同步时间".
// That is the PAGE RENDER TIME, not the data's freshness — so a clock that ticks every
// second was passing itself off as a sync time, and the operator could believe the
// number beside it was fresher than it was.
//
// The rule this component follows: render moment and data freshness are different
// facts, and the second one has to come from the data. When the store does not record
// when its last successful read happened, the honest answer is "未知(未记录)" plus the
// reason — not the current time, and not a guess.
//
// When the read path starts recording its own last-success timestamp, wire it in here.
// Do not go back to Date.now().

import React, { useEffect, useState } from 'react';

/** Milliseconds between re-renders, so the "页面刷新于" line is not a lie either. */
const RENDER_TICK_MS = 1000;

export const SyncFreshness: React.FC = () => {
  const [renderTime, setRenderTime] = useState(() => new Date());

  useEffect(() => {
    const id = setInterval(() => setRenderTime(new Date()), RENDER_TICK_MS);
    return () => clearInterval(id);
  }, []);

  return (
    <>
      <div className="text-sm font-bold text-[var(--sl-ink-secondary)]">未知(未记录)</div>
      <div className="text-[10px] font-medium text-[var(--sl-ink-muted)] mt-0.5">
        读取链路未记录成功同步的时刻
      </div>
      <div className="text-[10px] font-medium text-[var(--sl-ink-muted)] mt-0.5">
        页面刷新于 {renderTime.toLocaleTimeString('zh-CN')}
      </div>
    </>
  );
};

export default SyncFreshness;
