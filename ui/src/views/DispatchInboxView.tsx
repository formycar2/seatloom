import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { RefreshCw, Send, AlertCircle } from 'lucide-react';
import { api } from '../lib/api';
import { DispatchDto } from '../lib/types-dto';
import { useDataStore } from '../stores/useDataStore';

// Scenario 3 — the governed-dispatch (G19) seat-inbox view. Reads the live
// `dispatches` projection via cmd_list_dispatches and groups by seat. Mirrors
// the V1 AppShell visual conventions (same tokens as TimelineView).

const STATUS_STYLE: Record<string, { label: string; cls: string }> = {
  in_flight: { label: '进行中', cls: 'bg-blue-500/15 text-blue-600 border-blue-500/30' },
  ready: { label: '就绪', cls: 'bg-emerald-500/15 text-emerald-600 border-emerald-500/30' },
  blocked: { label: '阻塞', cls: 'bg-amber-500/15 text-amber-600 border-amber-500/30' },
  done: { label: '完成', cls: 'bg-secondary text-text-secondary border-border/40' },
};

const DispatchInboxView: React.FC = () => {
  const { activeProjectId, projectData } = useDataStore();
  const [dispatches, setDispatches] = useState<DispatchDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const seatName = useCallback(
    (seatId: string): string => {
      const data = activeProjectId ? projectData[activeProjectId] : null;
      const seat = data?.seats.find((s) => s.id === seatId);
      return seat?.name ? `${seat.name}` : seatId;
    },
    [activeProjectId, projectData],
  );

  const load = useCallback(
    async (silent = false) => {
      if (!silent) setLoading(true);
      setError(null);
      try {
        const rows = await api.listDispatches(activeProjectId ?? undefined);
        setDispatches(rows);
      } catch (e) {
        setError(String(e));
      } finally {
        if (!silent) setLoading(false);
      }
    },
    [activeProjectId],
  );

  useEffect(() => {
    void load();
    // Live refresh: poll every 5s so dispatches flow ready -> in_flight -> done
    // in real time (e.g. while watching a seat pull its next task).
    const timer = setInterval(() => void load(true), 5000);
    return () => clearInterval(timer);
  }, [load]);

  // Group by target seat, each group's rows already ordered by the backend.
  const bySeat = useMemo(() => {
    const groups = new Map<string, DispatchDto[]>();
    for (const d of dispatches) {
      if (!groups.has(d.targetSeatId)) groups.set(d.targetSeatId, []);
      groups.get(d.targetSeatId)!.push(d);
    }
    return Array.from(groups.entries());
  }, [dispatches]);

  return (
    <div className="flex-1 flex flex-col h-full bg-background overflow-hidden">
      <div className="p-8 border-b border-border/40 flex items-center justify-between">
        <div>
          <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">
            派单收件箱
          </h2>
          <p className="text-xs text-text-secondary mt-1">
            每个席位的治理派单队列(G19)· 进行中 → 就绪 → 阻塞 → 完成
          </p>
        </div>
        <button
          onClick={() => void load()}
          className="flex items-center gap-2 px-3 py-2 rounded-xl border border-border/60 bg-card text-text-secondary hover:bg-accent transition-all shadow-sm text-xs font-bold"
        >
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          刷新
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-8 py-6">
        {error ? (
          <div className="flex items-center gap-2 text-sm font-bold text-red-600">
            <AlertCircle size={16} /> 加载派单失败:{error}
          </div>
        ) : dispatches.length === 0 && !loading ? (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <div className="p-4 bg-secondary rounded-full border border-border/40 text-text-secondary mb-4 opacity-20">
              <Send className="w-8 h-8" />
            </div>
            <p className="text-sm font-bold text-text-secondary">暂无派单</p>
            <p className="text-xs text-text-secondary uppercase tracking-widest mt-1 opacity-60">
              Aegis 下达的指令会出现在这里
            </p>
          </div>
        ) : (
          <div className="space-y-8 max-w-4xl">
            {bySeat.map(([seatId, rows]) => (
              <div key={seatId}>
                <div className="flex items-center gap-2 mb-3">
                  <h3 className="text-sm font-black text-text-primary">{seatName(seatId)}</h3>
                  <span className="text-[10px] font-bold text-text-secondary uppercase tracking-widest">
                    {rows.length} 项
                  </span>
                </div>
                <div className="space-y-2">
                  {rows.map((d) => {
                    const st = STATUS_STYLE[d.status] ?? {
                      label: d.status,
                      cls: 'bg-secondary text-text-secondary border-border/40',
                    };
                    return (
                      <div
                        key={d.id}
                        className="sl-card p-4 flex items-start gap-3 border border-border/40"
                      >
                        <span
                          className={`shrink-0 px-2 py-0.5 rounded-md text-[10px] font-black uppercase tracking-wider border ${st.cls}`}
                        >
                          {st.label}
                        </span>
                        <div className="min-w-0 flex-1">
                          <p className="text-sm font-bold text-text-primary">{d.title}</p>
                          <div className="flex flex-wrap items-center gap-x-3 gap-y-1 mt-1 text-[11px] text-text-secondary">
                            <span>优先级 {d.priority}</span>
                            <span>来自 {d.issuedBy}</span>
                            <span className="font-mono opacity-70">{d.id}</span>
                            {d.deps.length > 0 && (
                              <span className="text-amber-600">
                                依赖 {d.deps.length} 项(
                                {d.deps.map((x) => `${x.depKind}:${x.depValue}`).join(', ')})
                              </span>
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default DispatchInboxView;
