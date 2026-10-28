// OperatorDecisionSection — the D2 「等你拍板」half for decisions only the
// operator can rule on (020).
//
// WHY THIS IS NOT PART OF AcceptanceSection. Until 020 a product-direction
// ruling — "do we add an exemption to the containment defence?", "which database
// does the sandbox talk to?", "do we delete that file?" — existed ONLY as a
// SupervisorMessage. The ledger had the record; nothing structured could answer
// "how many things are waiting on the human right now, and about what", and
// fleet-status.sh read acceptances alone. The failure mode the value design
// names is 静默遗漏: silent omission.
//
// An acceptance asks "do I take this delivered work?" and is about a dispatch.
// An operator decision asks "which way do I rule on this?" and may be about
// anything. They are DIFFERENT objects with different actors and different
// lifecycles, so they are listed separately — a merged list would make the
// operator's count wrong, which is the defect this whole object exists to fix.
//
// The card is built around two sentences, because those are the two that make a
// decision possible rather than a coin flip in the dark: what is being decided
// (`target`) and what follows from each option (`consequence`). The options
// themselves are the buttons — there is no free-text ruling, because a ruling
// outside the stated options is not one of the choices that was offered.

import React, { useCallback, useEffect, useState } from 'react';
import { AlertCircle, CheckCircle2, Gavel, Loader2, Scale } from 'lucide-react';
import { api } from '../lib/api';
import { useDataStore } from '../stores/useDataStore';
import type { OperatorDecisionCardDto } from '../lib/types-dto';

/** Same cadence as the acceptance section above — one rhythm for the page. */
const POLL_MS = 5000;

const EMPTY_STATE: Record<string, { busy: string | null; error: string | null }> = {};

function fmtTime(iso: string | null): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const p = (n: number) => n.toString().padStart(2, '0');
  return `${d.getMonth() + 1}/${d.getDate()} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

const OperatorDecisionSection: React.FC<{
  /** Mirrors the page's pause switch: pausing stops this poll too. */
  live: boolean;
}> = ({ live }) => {
  const [rows, setRows] = useState<OperatorDecisionCardDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [rowState, setRowState] = useState(EMPTY_STATE);

  // Scoped to the selected project, exactly like the acceptance section: an
  // unscoped read would show every project's obligations at once.
  const activeProjectId = useDataStore((s) => s.activeProjectId);

  const load = useCallback(
    async (silent = false) => {
      if (!silent) setLoading(true);
      setError(null);
      if (!activeProjectId) {
        setError('no project selected');
        if (!silent) setLoading(false);
        return;
      }
      try {
        const data = await api.listOperatorDecisions(activeProjectId, 'pending');
        setRows(data);
      } catch (e) {
        setError(String(e));
      } finally {
        if (!silent) setLoading(false);
      }
    },
    [activeProjectId],
  );

  useEffect(() => {
    if (!live) {
      setLoading(false);
      return;
    }
    void load();
    const timer = setInterval(() => void load(true), POLL_MS);
    return () => clearInterval(timer);
  }, [load, live]);

  const rule = useCallback(
    async (row: OperatorDecisionCardDto, choice: string) => {
      setRowState((s) => ({ ...s, [row.id]: { busy: choice, error: null } }));
      try {
        const result = await api.resolveOperatorDecision(row.id, choice);
        // Show the ruling in place rather than reloading: the recorded verdict
        // is the point of the action, and an immediate poll would drop the row
        // out of the list before the operator could read what was recorded.
        setRows((rs) =>
          rs.map((r) =>
            r.id === row.id
              ? {
                  ...r,
                  status: result.status,
                  chosen: result.chosen,
                  resolvedBy: result.resolvedBy,
                }
              : r,
          ),
        );
        setRowState((s) => ({ ...s, [row.id]: { busy: null, error: null } }));
      } catch (e) {
        // Verbatim. "already resolved" and "not the decider" are different
        // problems with different fixes, so neither is swallowed.
        setRowState((s) => ({ ...s, [row.id]: { busy: null, error: String(e) } }));
      }
    },
    [],
  );

  return (
    <section className="mt-6 rounded-2xl border border-border bg-panel overflow-hidden">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-border/60 bg-secondary/30">
        <Scale className="w-4 h-4 text-primary" />
        <h2 className="text-[13px] font-black text-text-primary">等你裁定</h2>
        <span className="text-[10px] font-bold px-1.5 rounded-full bg-secondary text-text-secondary">
          {rows.length}
        </span>
        <span className="ml-auto text-[10px] text-text-secondary">
          只有你能定的事 —— 和上面的「待验收」不是一回事
        </span>
      </header>

      <div className="p-4 space-y-3">
        {loading && rows.length === 0 ? (
          <div className="flex items-center gap-2 text-text-secondary text-[12px]">
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
            正在读……
          </div>
        ) : error ? (
          <div className="flex items-start gap-2 text-[12px] text-red-600">
            <AlertCircle className="w-3.5 h-3.5 mt-0.5 shrink-0" />
            <span>{error}</span>
          </div>
        ) : rows.length === 0 ? (
          <p className="text-[12px] text-text-secondary">
            没有等你裁定的事。
            <span className="block mt-1 text-text-muted">
              被卡住的派单、失败的读取、打回重做 —— 那些是席位或总控的事,不会推到这里。
            </span>
          </p>
        ) : (
          rows.map((row) => {
            const st = rowState[row.id] ?? { busy: null, error: null };
            const resolved = row.status === 'resolved';
            return (
              <article
                key={row.id}
                className="rounded-xl border border-border/60 bg-secondary/20 p-3 space-y-2"
              >
                <div className="flex items-start gap-2">
                  <Gavel className="w-3.5 h-3.5 mt-0.5 text-primary shrink-0" />
                  <div className="min-w-0 flex-1">
                    <h3 className="text-[13px] font-bold text-text-primary">{row.target}</h3>
                    <p className="text-[10px] font-mono text-text-muted mt-0.5">{row.id}</p>
                  </div>
                  {resolved && (
                    <span className="inline-flex items-center gap-1 text-[10px] font-bold px-1.5 py-0.5 rounded-full bg-emerald-50 text-emerald-700">
                      <CheckCircle2 className="w-3 h-3" />
                      已裁定
                    </span>
                  )}
                </div>

                {/* The consequence, stated before the buttons: what follows from
                    deciding is the thing that makes the decision possible. */}
                <p className="text-[12px] text-text-secondary leading-relaxed">
                  <span className="font-bold text-text-primary">定了之后:</span>
                  {row.consequence}
                </p>

                {row.evidence.length > 0 && (
                  <p className="text-[11px] text-text-muted">
                    <span className="font-bold">依据:</span>
                    {row.evidence.join(', ')}
                  </p>
                )}

                {resolved ? (
                  <p className="text-[12px] text-text-primary">
                    <span className="font-bold">裁定:</span>
                    {row.chosen ?? '(未记录)'}
                    {row.resolvedBy ? ` —— ${row.resolvedBy}` : ''}
                    {row.resolvedAt ? ` · ${fmtTime(row.resolvedAt)}` : ''}
                  </p>
                ) : (
                  <div className="flex flex-wrap gap-2 pt-1">
                    {row.choices.map((choice) => (
                      <button
                        key={choice}
                        type="button"
                        disabled={st.busy !== null}
                        onClick={() => void rule(row, choice)}
                        className="px-3 py-1.5 rounded-lg text-[12px] font-bold border border-primary text-primary hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                      >
                        {st.busy === choice ? (
                          <Loader2 className="w-3 h-3 animate-spin inline mr-1" />
                        ) : null}
                        {choice}
                      </button>
                    ))}
                  </div>
                )}

                {st.error && (
                  <div className="flex items-start gap-2 text-[11px] text-red-600">
                    <AlertCircle className="w-3 h-3 mt-0.5 shrink-0" />
                    <span>{st.error}</span>
                  </div>
                )}
              </article>
            );
          })
        )}
      </div>
    </section>
  );
};

export default OperatorDecisionSection;
