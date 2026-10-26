// LiveSeatsView — the live mirror, and the answer to "what here is real?".
//
// Everything on this screen is discovered at runtime from `cmd_list_tmux_sessions`
// and streamed from `session:output`. There is no fixture path: if tmux has no
// `-seatloom` sessions, this view says so rather than inventing rows.
//
// The design problem UM0 actually poses is that a *list* of real seats looks
// exactly like a list of fake ones. So the proof of life is not the list — it is
// movement. Each seat card carries a byte/chunk counter and a "last output"
// clock fed by the real event stream, and the dot pulses while bytes are
// arriving. A screenshot of this view is self-evidencing: fixtures cannot make
// a counter climb.
//
// R1/R3: SeatLoom observes, never owns. Detaching a tab stops our mirror and
// leaves the tmux session running untouched.
//
// 2026-10-16, defect fixed: this view is conditionally rendered by App.tsx, so
// leaving the tab unmounts it. The mirror state used to live in this component's
// own useState, which meant coming back started from an empty attachment list and
// re-attached only the first seat — five manual Connects every time, while the
// backend sessions had been alive the whole time. The durable state now lives in
// `useLiveSeatsMirrorStore`, so a remount restores what the backend still holds.
// On mount we ask the backend which session ids are still alive and drop the rest,
// so a session that exited while the view was away is not shown as connected.

import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Radio, RefreshCw, Plug, Unplug, TerminalSquare } from 'lucide-react';
import { api, isTauri, onSessionExit, onSessionOutput } from '../lib/api';
import { useDataStore } from '../stores/useDataStore';
import { boundMirrorNames } from '../lib/project-scope';
import { parseSeatSession, seatKey, sortSeats, type ParsedSeat } from '../lib/tmux-seats';
import { useLiveSessionsStore } from '../stores/useLiveSessionsStore';
import {
  EMPTY_ACTIVITY,
  useLiveSeatsMirrorStore,
  type Attachment,
} from '../stores/useLiveSeatsMirrorStore';
import { SessionTerminal } from '../app-v2/panel/SessionTerminal';

/** How often we re-discover tmux sessions. */
const POLL_MS = 3000;
/** Counter flush + relative-clock tick. Coalesces high-output panes. */
const TICK_MS = 500;
/** A seat counts as "moving" if bytes arrived within this window. */
const ACTIVE_WINDOW_MS = 2500;

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MB`;
}

function formatAgo(at: number | null, now: number): string {
  if (at == null) return '尚无输出';
  const s = Math.max(0, Math.round((now - at) / 1000));
  if (s < 1) return '刚刚';
  if (s < 60) return `${s} 秒前`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} 分钟前`;
  return `${Math.floor(m / 60)} 小时前`;
}

const LiveSeatsView: React.FC = () => {
  const activeProjectId = useDataStore((s) => s.activeProjectId);
  const selectedProject = useRef(activeProjectId);
  selectedProject.current = activeProjectId;
  const [discoveredSeats, setSeats] = useState<ParsedSeat[]>([]);
  const [discoveryProject, setDiscoveryProject] = useState<string | null>(null);
  const seats = discoveryProject === activeProjectId ? discoveredSeats : [];
  const [busy, setBusy] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<string | null>(null);
  const [discovered, setDiscovered] = useState(false);
  const [now, setNow] = useState(() => Date.now());

  // Durable across unmount: the mirror is a projection of what the backend still
  // holds, not something this component owns. See the store's header comment.
  const attachments = useLiveSeatsMirrorStore((s) => s.attachments);
  const activity = useLiveSeatsMirrorStore((s) => s.activity);
  const activeName = useLiveSeatsMirrorStore((s) => s.activeName);
  const setActiveName = useLiveSeatsMirrorStore((s) => s.setActiveName);
  const addAttachment = useLiveSeatsMirrorStore((s) => s.addAttachment);
  const removeAttachment = useLiveSeatsMirrorStore((s) => s.removeAttachment);
  const addBytes = useLiveSeatsMirrorStore((s) => s.addBytes);
  const pruneDead = useLiveSeatsMirrorStore((s) => s.pruneDead);
  const markAutoAttached = useLiveSeatsMirrorStore((s) => s.markAutoAttached);

  const setSessionForSeat = useLiveSessionsStore((s) => s.setSessionForSeat);
  const clearSession = useLiveSessionsStore((s) => s.clearSession);
  const setSeatDiscovery = useLiveSessionsStore((s) => s.setSeatDiscovery);

  // Raw event counters land here first and are flushed on the tick, so a chatty
  // pane cannot trigger a React render per chunk.
  const pendingRef = useRef<Record<string, { bytes: number; chunks: number; lastAt: number }>>({});

  // ── Discovery ──────────────────────────────────────────────────────────────
  // Each poll publishes the discovery outcome to the shared store so the
  // provenance banner above this view (and the nav-rail dot) can refuse to say
  // LIVE when there is nothing live behind it (UM0 HOLD-3).
  const refresh = useCallback(async () => {
    if (!isTauri()) {
      // No Tauri backend → no tmux IPC → provably no live seats here.
      setDiscovered(true);
      setSeatDiscovery('empty');
      return;
    }
    try {
      if (!activeProjectId) { setSeats([]); setDiscovered(true); return; }
      const [list, bindings] = await Promise.all([
        api.listTmuxSessions(), api.listSeats(activeProjectId),
      ]);
      if (selectedProject.current !== activeProjectId) return;
      const scoped = boundMirrorNames(list.map(parseSeatSession), bindings);
      setDiscoveryProject(activeProjectId);
      setSeats(sortSeats(scoped));
      setDiscovered(true);
      setSeatDiscovery(scoped.length > 0 ? 'ok' : 'empty');
      setError(null);
    } catch (e) {
      if (selectedProject.current !== activeProjectId) return;
      setSeats([]);
      setError(`会话发现失败 (cmd_list_tmux_sessions): ${String(e)}`);
      setDiscovered(true);
      setSeatDiscovery('failed');
    }
  }, [setSeatDiscovery, activeProjectId]);

  useEffect(() => {
    setSeats([]);
    let cancelled = false;
    const run = async () => {
      if (cancelled) return;
      await refresh();
    };
    run();
    const id = setInterval(run, POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [refresh]);

  // ── Live byte stream → activity counters ───────────────────────────────────
  useEffect(() => {
    let offOut: (() => void) | null = null;
    let offExit: (() => void) | null = null;

    onSessionOutput((event) => {
      // event.data is base64; decoded length is what actually crossed the FIFO.
      const approxBytes = Math.floor((event.data.length * 3) / 4);
      const slot = pendingRef.current[event.sessionId] ?? { bytes: 0, chunks: 0, lastAt: 0 };
      slot.bytes += approxBytes;
      slot.chunks += 1;
      slot.lastAt = Date.now();
      pendingRef.current[event.sessionId] = slot;
    }).then((fn) => {
      offOut = fn;
    });

    onSessionExit((event) => {
      removeAttachment(event.sessionId);
      clearSession(event.sessionId);
    }).then((fn) => {
      offExit = fn;
    });

    return () => {
      if (offOut) offOut();
      if (offExit) offExit();
    };
  }, [clearSession, removeAttachment]);

  // Single tick drives both the counter flush and the relative clocks.
  useEffect(() => {
    const id = setInterval(() => {
      const pending = pendingRef.current;
      const ids = Object.keys(pending);
      if (ids.length > 0) {
        pendingRef.current = {};
        for (const id2 of ids) {
          const p = pending[id2];
          addBytes(id2, p.bytes, p.chunks, p.lastAt);
        }
      }
      setNow(Date.now());
    }, TICK_MS);
    return () => clearInterval(id);
  }, [addBytes]);

  // ── Attach / detach ────────────────────────────────────────────────────────
  const attach = useCallback(
    async (seat: ParsedSeat) => {
      if (!isTauri()) {
        setError('Tauri 后端不可用 — 请通过 `pnpm tauri dev` 启动。');
        return;
      }
      setBusy((b) => ({ ...b, [seat.sessionName]: true }));
      setError(null);
      try {
        const dto = await api.attachTmuxSession({
          tmuxSessionName: seat.sessionName,
          seatId: seat.seatName.toLowerCase(),
        });
        addAttachment({ seat, dto });
        setActiveName(seat.sessionName);
        // Let the Supervisor IM route messages to this seat's real pane.
        setSessionForSeat(seatKey(seat.seatName), dto.id);
      } catch (e) {
        setError(`接入 ${seat.sessionName} 失败: ${String(e)}`);
      } finally {
        setBusy((b) => ({ ...b, [seat.sessionName]: false }));
      }
    },
    [addAttachment, setActiveName, setSessionForSeat],
  );

  const detach = useCallback(
    (sessionName: string) => {
      const target = useLiveSeatsMirrorStore
        .getState()
        .attachments.find((a) => a.seat.sessionName === sessionName);
      if (target) {
        clearSession(target.dto.id);
        removeAttachment(target.dto.id);
      }
    },
    [clearSession, removeAttachment],
  );

  // Attach the first discovered seat once, so the view is already moving when
  // it opens instead of asking for a click to prove itself.
  //
  // Unchanged design, one correction: the "once" flag used to be a useRef, so it
  // reset on every remount and seat 0 was attached AGAIN on top of a mirror that
  // was still live. The backend inserts unconditionally, so that second attach
  // started a second `pipe-pane` on the same pane. The flag is durable now, and a
  // remount that finds surviving attachments skips the auto-attach entirely —
  // there is nothing left to prove.
  useEffect(() => {
    if (!isTauri() || seats.length === 0) return;
    const state = useLiveSeatsMirrorStore.getState();
    if (state.autoAttached) return;
    markAutoAttached();
    if (state.attachments.length === 0) void attach(seats[0]);
  }, [seats, attachments.length, attach, markAutoAttached]);

  // Reconcile with the backend on mount: drop attachments whose session is no
  // longer alive, so a seat that exited while this view was away is not shown as
  // connected. The `session:exit` listener only runs while the view is mounted,
  // so this is the one moment we can catch up.
  //
  // A failure to list is deliberately NOT a reason to drop what we hold: losing
  // live mirrors on a transient error would be a worse defect than the one being
  // fixed here.
  useEffect(() => {
    if (!isTauri()) return;
    let cancelled = false;
    api
      .listLiveSessions()
      .then((ids) => {
        if (cancelled) return;
        pruneDead(new Set(ids));
      })
      .catch(() => {
        /* keep what we have; the next mount retries */
      });
    return () => {
      cancelled = true;
    };
  }, [pruneDead]);

  // Re-assert seat -> session routing for whatever survived, so the Supervisor IM
  // keeps piping chat into the right pane after the view comes back. The mapping
  // already lives in `useLiveSessionsStore` and survives unmount; re-asserting is
  // idempotent and repairs a partial write.
  useEffect(() => {
    if (!isTauri()) return;
    for (const a of useLiveSeatsMirrorStore.getState().attachments) {
      setSessionForSeat(seatKey(a.seat.seatName), a.dto.id);
    }
  }, [setSessionForSeat]);

  const attachedByName = useMemo(() => {
    const m = new Map<string, Attachment>();
    const names = new Set(seats.map((s) => s.sessionName));
    for (const a of attachments) {
      if (names.has(a.seat.sessionName)) m.set(a.seat.sessionName, a);
    }
    return m;
  }, [attachments, seats]);

  const active = activeName ? attachedByName.get(activeName) : undefined;

  const totalBytes = useMemo(
    () => Object.values(activity).reduce((sum, a) => sum + a.bytes, 0),
    [activity],
  );

  // ── Render ─────────────────────────────────────────────────────────────────
  return (
    <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 }}>
      {/* Keyframes kept local so no shared stylesheet is touched while a
          parallel packet is in flight. */}
      <style>{`
        @keyframes sl-live-pulse {
          0%   { transform: scale(1);   opacity: 1; }
          50%  { transform: scale(1.5); opacity: 0.45; }
          100% { transform: scale(1);   opacity: 1; }
        }
      `}</style>

      {/* Header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 12,
          padding: '12px 16px',
          borderBottom: '1px solid var(--sl-border)',
          background: 'var(--sl-surface)',
          flexShrink: 0,
        }}
      >
        <Radio size={16} style={{ color: 'var(--sl-green)' }} />
        <div style={{ minWidth: 0 }}>
          <div style={{ fontSize: 14, fontWeight: 700, color: 'var(--sl-text-primary)' }}>
            实时席位 · Live Seats
          </div>
          <div style={{ fontSize: 11.5, color: 'var(--sl-text-secondary)' }}>
            发现 {seats.length} 个 tmux 席位会话 · 已接入 {attachedByName.size} 个 · 累计镜像{' '}
            <span style={{ fontVariantNumeric: 'tabular-nums', fontWeight: 600 }}>
              {formatBytes(totalBytes)}
            </span>
          </div>
        </div>
        <button
          onClick={() => void refresh()}
          title="立即重新发现"
          style={{
            marginLeft: 'auto',
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            padding: '5px 10px',
            fontSize: 12,
            fontWeight: 600,
            color: 'var(--sl-text-secondary)',
            background: 'var(--sl-bg)',
            border: '1px solid var(--sl-border)',
            borderRadius: 'var(--sl-radius-sm)',
            cursor: 'pointer',
          }}
        >
          <RefreshCw size={13} />
          重新发现
        </button>
      </div>

      {error && (
        <div
          style={{
            padding: '8px 16px',
            background: 'var(--sl-red-subtle)',
            borderBottom: '1px solid var(--sl-red)',
            color: 'var(--sl-red)',
            fontSize: 12,
            flexShrink: 0,
          }}
        >
          {error}
        </div>
      )}

      <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
        {/* ── Seat rail ── */}
        <aside
          style={{
            width: 300,
            flexShrink: 0,
            borderRight: '1px solid var(--sl-border)',
            background: 'var(--sl-surface)',
            overflowY: 'auto',
            padding: 10,
            display: 'flex',
            flexDirection: 'column',
            gap: 8,
          }}
        >
          {!isTauri() && (
            <div style={{ fontSize: 12, lineHeight: 1.6, color: 'var(--sl-text-secondary)' }}>
              浏览器模式下没有 Tauri 后端，因此这里<strong>不显示任何席位</strong>。
              此视图只呈现真实 tmux 会话，不会用示例数据填充。请通过 <code>pnpm tauri dev</code>{' '}
              启动。
            </div>
          )}

          {isTauri() && discovered && seats.length === 0 && (
            <div style={{ fontSize: 12, lineHeight: 1.6, color: 'var(--sl-text-secondary)' }}>
              {!activeProjectId ? '请选择项目后查看实时镜像。' : '本项目没有已绑定且正在运行的席位会话；历史语料项目没有席位绑定。'}
              后端按该后缀过滤，因此此处为空即代表
              <strong>确实没有</strong>在线席位。
            </div>
          )}

          {seats.map((seat) => {
            const att = attachedByName.get(seat.sessionName);
            const act = att ? (activity[att.dto.id] ?? EMPTY_ACTIVITY) : EMPTY_ACTIVITY;
            const moving = act.lastAt != null && now - act.lastAt < ACTIVE_WINDOW_MS;
            const isActive = activeName === seat.sessionName;
            return (
              <div
                key={seat.sessionName}
                onClick={() => att && setActiveName(seat.sessionName)}
                style={{
                  border: `1px solid ${isActive ? 'var(--sl-brand)' : 'var(--sl-border-light)'}`,
                  background: isActive ? 'var(--sl-brand-subtle)' : 'var(--sl-bg)',
                  borderRadius: 'var(--sl-radius-md)',
                  padding: 10,
                  cursor: att ? 'pointer' : 'default',
                  transition: 'border-color 120ms ease, background 120ms ease',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span
                    style={{
                      width: 8,
                      height: 8,
                      borderRadius: '50%',
                      flexShrink: 0,
                      background: att
                        ? moving
                          ? 'var(--sl-green)'
                          : 'var(--sl-amber)'
                        : 'var(--sl-text-tertiary)',
                      animation: moving ? 'sl-live-pulse 1s ease-in-out infinite' : 'none',
                    }}
                  />
                  <span style={{ fontSize: 13, fontWeight: 700, color: 'var(--sl-text-primary)' }}>
                    {seat.seatName}
                  </span>
                  {seat.role && (
                    <span
                      style={{
                        fontSize: 10,
                        fontWeight: 600,
                        padding: '1px 5px',
                        borderRadius: 'var(--sl-radius-sm)',
                        background: 'var(--sl-surface-active)',
                        color: 'var(--sl-text-secondary)',
                      }}
                    >
                      {seat.role}
                    </span>
                  )}
                  {seat.attached && (
                    <span
                      title="已有 tmux 客户端连接此会话"
                      style={{ fontSize: 10, color: 'var(--sl-text-tertiary)' }}
                    >
                      tmux✓
                    </span>
                  )}
                </div>

                <div
                  style={{
                    fontFamily: 'var(--sl-font-mono, ui-monospace, monospace)',
                    fontSize: 10.5,
                    color: 'var(--sl-text-tertiary)',
                    margin: '4px 0 6px',
                    wordBreak: 'break-all',
                  }}
                >
                  {seat.sessionName}
                </div>

                {att ? (
                  <>
                    <div
                      style={{
                        display: 'flex',
                        gap: 10,
                        fontSize: 10.5,
                        color: 'var(--sl-text-secondary)',
                        fontVariantNumeric: 'tabular-nums',
                        marginBottom: 6,
                      }}
                    >
                      <span>{formatBytes(act.bytes)}</span>
                      <span>{act.chunks} 块</span>
                      <span
                        style={{
                          marginLeft: 'auto',
                          color: moving ? 'var(--sl-green)' : undefined,
                          fontWeight: moving ? 700 : 400,
                        }}
                      >
                        {formatAgo(act.lastAt, now)}
                      </span>
                    </div>
                    <div style={{ display: 'flex', gap: 6 }}>
                      <span
                        style={{
                          fontSize: 10,
                          padding: '2px 6px',
                          borderRadius: 'var(--sl-radius-sm)',
                          background: 'var(--sl-green-subtle)',
                          color: 'var(--sl-green)',
                          fontWeight: 700,
                        }}
                      >
                        已接入 {att.dto.paneCols ?? '?'}×{att.dto.paneRows ?? '?'}
                      </span>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          detach(seat.sessionName);
                        }}
                        title="停止镜像（tmux 会话不受影响）"
                        style={{
                          marginLeft: 'auto',
                          display: 'flex',
                          alignItems: 'center',
                          gap: 4,
                          padding: '2px 7px',
                          fontSize: 10.5,
                          color: 'var(--sl-text-secondary)',
                          background: 'transparent',
                          border: '1px solid var(--sl-border)',
                          borderRadius: 'var(--sl-radius-sm)',
                          cursor: 'pointer',
                        }}
                      >
                        <Unplug size={11} />
                        断开
                      </button>
                    </div>
                  </>
                ) : (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      void attach(seat);
                    }}
                    disabled={!!busy[seat.sessionName]}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      gap: 5,
                      width: '100%',
                      padding: '5px 8px',
                      fontSize: 11.5,
                      fontWeight: 600,
                      color: '#fff',
                      background: 'var(--sl-brand)',
                      border: 'none',
                      borderRadius: 'var(--sl-radius-sm)',
                      cursor: busy[seat.sessionName] ? 'wait' : 'pointer',
                      opacity: busy[seat.sessionName] ? 0.6 : 1,
                    }}
                  >
                    <Plug size={12} />
                    {busy[seat.sessionName] ? '接入中…' : '接入实时镜像'}
                  </button>
                )}
              </div>
            );
          })}
        </aside>

        {/* ── Terminal area ── */}
        <main style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column' }}>
          {active && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 10,
                padding: '6px 12px',
                background: 'var(--sl-surface)',
                borderBottom: '1px solid var(--sl-border)',
                fontSize: 11.5,
                color: 'var(--sl-text-secondary)',
                flexShrink: 0,
              }}
            >
              <TerminalSquare size={13} />
              <span style={{ fontWeight: 700, color: 'var(--sl-text-primary)' }}>
                {active.seat.seatName}
              </span>
              <code style={{ fontSize: 10.5 }}>{active.seat.sessionName}</code>
              <span style={{ marginLeft: 'auto', fontVariantNumeric: 'tabular-nums' }}>
                {formatBytes((activity[active.dto.id] ?? EMPTY_ACTIVITY).bytes)} ·{' '}
                {formatAgo((activity[active.dto.id] ?? EMPTY_ACTIVITY).lastAt, now)}
              </span>
              <span style={{ opacity: 0.7 }}>键入即写回真实 pane</span>
            </div>
          )}

          <div style={{ flex: 1, position: 'relative', minHeight: 0 }}>
            {Array.from(attachedByName.values()).map((a) => (
              <div
                key={a.dto.id}
                style={{
                  position: 'absolute',
                  inset: 0,
                  // Inactive attachments are hidden with `visibility`, NOT
                  // `display: none`. This is the fix for "the live-seats page
                  // sometimes will not scroll":
                  //
                  // A `display: none` ancestor removes the terminal from
                  // layout, so xterm cannot measure a character cell — its DOM
                  // measure strategy reads offsetWidth/offsetHeight of a
                  // measure element that no longer has a box.
                  // FitAddon.fit() then returns early WITHOUT resizing (its
                  // own proposeDimensions() bails on css.cell.width === 0),
                  // so the viewport silently stays at the pane's attach-time
                  // row/col seed — normally TALLER than this slot. When the
                  // pane's content is not longer than that seed row count the
                  // scroll area equals the viewport height and the operator
                  // cannot scroll at all.
                  //
                  // `visibility: hidden` keeps the box (these layers are
                  // already position: absolute, so they are out of flow either
                  // way), so a terminal mounted while its attachment is
                  // inactive is still measurable and is fitted correctly the
                  // moment it is armed. It also keeps the hidden terminal out
                  // of the a11y tree and unfocusable, which display: none was
                  // providing for free.
                  visibility: a.seat.sessionName === activeName ? 'visible' : 'hidden',
                  // Belt and braces: an inactive layer must never swallow a
                  // wheel or a click even if a browser still routes events.
                  pointerEvents: a.seat.sessionName === activeName ? 'auto' : 'none',
                }}
              >
                <SessionTerminal
                  sessionId={a.dto.id}
                  paneRows={a.dto.paneRows}
                  paneCols={a.dto.paneCols}
                  initialSnapshotB64={a.dto.initialSnapshotB64}
                />
              </div>
            ))}

            {attachedByName.size === 0 && (
              <div
                style={{
                  position: 'absolute',
                  inset: 0,
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 10,
                  padding: 40,
                  textAlign: 'center',
                  color: 'var(--sl-text-tertiary)',
                }}
              >
                <Radio size={40} style={{ opacity: 0.18 }} />
                <div style={{ fontSize: 15, fontWeight: 600, color: 'var(--sl-text-secondary)' }}>
                  尚未接入任何实时 pane
                </div>
                <div style={{ fontSize: 12.5, lineHeight: 1.6, maxWidth: 420 }}>
                  从左侧选择一个席位并点击「接入实时镜像」。 接入后此处显示该 tmux pane
                  的实时输出，左侧计数器会随字节到达而增长 —— 那就是它是真的证据。
                </div>
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
};

export default LiveSeatsView;
