/**
 * HistorySessionsPanel — the historical session corpus, by project.
 *
 * Product contract: LYRA-2026-10-14-history-sessions-in-ui-contract-v1
 * §8.2 slice 3 (Mira) + §4 (screen definition) + §3.2 (empty and missing states).
 *
 * Three screens, inside the existing `sessions` tab and the existing skeleton:
 *   1. PROJECT LIST   — one row per home path that has sessions
 *   2. SESSION LIST   — one row per session for the chosen project
 *   3. SESSION DETAIL — the conversation in source order, each turn carrying the
 *                       line number it came from in the ORIGINAL file
 *
 * # The three rules this file is written to
 *
 * 1. **Wiring state is measured or it is not shown.** There is no
 *    hand-written "these commands are wired" switch anywhere. The panel calls the
 *    real commands and renders whatever they answer — including their rejections,
 *    verbatim. A hardcoded flag asserting a command is registered is the shape
 *    that already misled the operator once.
 * 2. **Nothing is invented.** A session with no timestamps reads 时间未记录, never
 *    a date. The repository-root group is its own labelled group, never folded
 *    into a neighbour. A missing source file is named as missing, and its turn is
 *    still shown. `messageCount` is `null` when the corpus migration is absent,
 *    and that renders as "not loaded", never as 0.
 * 3. **The default view is already readable.** Every turn's text is rendered by
 *    default; progressive disclosure exists only to keep a very long turn from
 *    flooding the page, and a collapsed turn says it is collapsed and how long it
 *    is. There is no "click to reveal everything".
 *
 * # Where the bodies come from
 *
 * The read tools return POINTERS ONLY (011's rule: no transcript body in the
 * database, and the originals may contain secrets). `cmd_read_history_context_slice`
 * dereferences them against the operator's own extracted markdown, so the text the
 * operator reads is his own file, read locally — it is never stored and never
 * crosses MCP.
 */

import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  AlertTriangle,
  Brain,
  ChevronRight,
  Clock,
  FileText,
  FolderOpen,
  Loader2,
  MessageSquare,
  RefreshCw,
} from 'lucide-react';
import { api, isTauri } from '../../lib/api';
// The session<->work read model's Tauri command. Reached through `invoke`
// directly rather than through `api.ts`, because `api.ts` and `types-dto.ts` are
// Nimbus's fence (dispatch disp-session-key-ui: "never edit them") and this seat's
// contract adds no shape to them. The command returns the read model's own JSON
// text verbatim, so there is no second, summarised shape that could drift.
import { invoke } from '@tauri-apps/api/core';
import type {
  HistoryContextSlice,
  HistoryProject,
  HistoryProjectsPayload,
  HistorySession,
  HistorySessionDetail,
  HistorySessionsPayload,
  HistoryTurnsPayload,
} from '../../lib/types-dto';

// ─────────────────────────────────────────────────────────────────────────────
// Injectable seams (tests replace these; production uses the real api)
// ─────────────────────────────────────────────────────────────────────────────

export interface HistoryLoader {
  listProjects: () => Promise<string>;
  listSessions: (projectId: string) => Promise<string>;
  getSessionTurns: (harness: string, id: string) => Promise<string>;
  readContextSlice: (
    contextPath: string,
    lineStart: number,
    lineEnd: number,
  ) => Promise<HistoryContextSlice>;
  openPath: (path: string) => Promise<void>;
  /**
   * The forward read of the session<->work bridge: one session's distilled
   * records plus the three separated numbers. Returns the read model's own JSON
   * text verbatim — a refusal (the bridge view is absent, or the session is not
   * in the archive) comes back as the tool's typed message, so this screen can
   * name what is missing instead of rendering an empty list.
   */
  sessionWork: (harness: string, id: string) => Promise<string>;
}

const liveLoader: HistoryLoader = {
  listProjects: () => api.listHistoryProjects(),
  listSessions: (projectId) => api.listHistorySessions(projectId),
  getSessionTurns: (harness, id) => api.getHistorySessionTurns(harness, id),
  readContextSlice: (contextPath, lineStart, lineEnd) =>
    api.readHistoryContextSlice(contextPath, lineStart, lineEnd),
  openPath: (path) => api.openLocalPath(path),
  sessionWork: (harness, id) => invoke('cmd_session_work', { harness, id }),
};

// ─────────────────────────────────────────────────────────────────────────────
// Display helpers — every one of them refuses to invent
// ─────────────────────────────────────────────────────────────────────────────

const HARNESS_LABELS: Record<string, string> = {
  codex: 'Codex',
  'claude-code': 'Claude Code',
  stepcode: 'Stepcode',
  cursor: 'Cursor',
  gemini: 'Gemini',
  opencode: 'OpenCode',
};

function harnessLabel(harness: string): string {
  return HARNESS_LABELS[harness] ?? harness;
}

const TURN_LABELS: Record<string, string> = {
  system: '系统',
  user: '你',
  assistant: '助手',
};

function turnLabelText(label: string, labelRaw: string | null): string {
  return TURN_LABELS[label] ?? labelRaw ?? label;
}

/**
 * `2026-09-28 13:31:44.413+00` -> `2026-09-28 13:31 UTC`.
 *
 * The timestamp is shown exactly as the source recorded it (UTC), truncated to
 * the minute. No timezone conversion: the operator checks these against his own
 * files, and a converted time is a number he cannot reproduce from the source.
 */
function formatTs(ts: string): string {
  const m = /^(\d{4}-\d{2}-\d{2})[ T](\d{2}:\d{2})/.exec(ts.trim());
  if (!m) return ts.trim();
  return `${m[1]} ${m[2]} UTC`;
}

/** The session's date range, or the explicit "not recorded". */
function dateRange(session: HistorySession): { text: string; undated: boolean } {
  if (!session.dated || !session.firstTs) {
    return { text: '时间未记录', undated: true };
  }
  const first = formatTs(session.firstTs);
  if (!session.lastTs) return { text: first, undated: false };
  const last = formatTs(session.lastTs);
  // Same day -> show the range on one line without repeating the date.
  if (first.slice(0, 10) === last.slice(0, 10)) {
    return { text: `${first} → ${last.slice(11)}`, undated: false };
  }
  return { text: `${first} → ${last}`, undated: false };
}

function countText(n: number | null): string {
  return n == null ? '未加载' : String(n);
}

/** Split a context file's lines into the slice each turn points at. */
function sliceForTurn(
  lines: string[],
  start: number,
  end: number,
): { text: string; truncated: boolean } {
  const from = Math.max(1, start) - 1;
  const to = Math.min(end, lines.length);
  if (from >= lines.length) return { text: '', truncated: true };
  const text = lines.slice(from, to).join('\n');
  return { text, truncated: to < end };
}

// ─────────────────────────────────────────────────────────────────────────────
// Small presentational pieces
// ─────────────────────────────────────────────────────────────────────────────

const shell: React.CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  minHeight: 0,
  overflow: 'hidden',
};

const columnHead: React.CSSProperties = {
  fontSize: 11,
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.05em',
  color: 'var(--sl-text-tertiary)',
  display: 'flex',
  alignItems: 'center',
  gap: 6,
  flexShrink: 0,
};

const rowButton: React.CSSProperties = {
  width: '100%',
  textAlign: 'left',
  cursor: 'pointer',
  display: 'flex',
  flexDirection: 'column',
  gap: 3,
  padding: '8px 10px',
  borderRadius: 'var(--sl-radius-md)',
  border: '1px solid var(--sl-border-light)',
  background: 'var(--sl-bg)',
  transition: 'all 120ms ease',
};

const note: React.CSSProperties = {
  fontSize: 11,
  lineHeight: 1.6,
  color: 'var(--sl-text-secondary)',
};

const mono: React.CSSProperties = {
  fontFamily: 'var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, monospace)',
  wordBreak: 'break-all',
};

/** A block that says why there is nothing here and what happens next. */
const Explanation: React.FC<{ title: string; children: React.ReactNode }> = ({
  title,
  children,
}) => (
  <div
    style={{
      padding: '10px 12px',
      borderRadius: 'var(--sl-radius-md)',
      border: '1px solid var(--sl-border-light)',
      background: 'var(--sl-surface)',
      fontSize: 12,
      lineHeight: 1.7,
      color: 'var(--sl-text-secondary)',
    }}
  >
    <div style={{ fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 4 }}>{title}</div>
    {children}
  </div>
);

/** The measured failure text, verbatim — never paraphrased, never hidden. */
const MeasuredError: React.FC<{ what: string; error: string }> = ({ what, error }) => (
  <div
    style={{
      padding: '10px 12px',
      borderRadius: 'var(--sl-radius-md)',
      border: '1px solid var(--sl-red)',
      background: 'var(--sl-red-subtle)',
      fontSize: 12,
      lineHeight: 1.6,
      color: 'var(--sl-red)',
    }}
  >
    <div
      style={{ fontWeight: 700, marginBottom: 4, display: 'flex', alignItems: 'center', gap: 6 }}
    >
      <AlertTriangle size={13} /> {what}
    </div>
    <div style={mono}>{error}</div>
    <div style={{ marginTop: 6, color: 'var(--sl-text-secondary)' }}>
      上面是后端原样返回的错误文本(实测,非转述)。修好它缺的那一环,这个面板就会自己好。
    </div>
  </div>
);

const Loading: React.FC<{ what: string }> = ({ what }) => (
  <div
    style={{
      display: 'flex',
      alignItems: 'center',
      gap: 8,
      fontSize: 12,
      color: 'var(--sl-text-secondary)',
    }}
  >
    <Loader2 size={13} className="animate-spin" /> {what}
  </div>
);

// ─────────────────────────────────────────────────────────────────────────────
// Screen 1 — the project list
// ─────────────────────────────────────────────────────────────────────────────

export const HistoryProjectList: React.FC<{
  projects: HistoryProject[];
  activeProjectId: string | null;
  onSelect: (projectId: string) => void;
}> = ({ projects, activeProjectId, onSelect }) => {
  // The server already orders by session count descending. The repository-root
  // group is moved to the very end and labelled, per contract §4 — it is a
  // path-prefix ancestor of 37 other homes, so leaving it mid-list reads as if it
  // were one project among equals.
  const ordered = useMemo(() => {
    const root = projects.filter((p) => p.unclassified);
    const rest = projects.filter((p) => !p.unclassified);
    return [...rest, ...root];
  }, [projects]);

  const totalSessions = projects.reduce((n, p) => n + p.sessionCount, 0);
  const totalMessages = projects.reduce<number | null>(
    (acc, p) => (p.messageCount == null ? acc : (acc ?? 0) + p.messageCount),
    null,
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 6, minHeight: 0 }}>
      <div style={{ ...columnHead, justifyContent: 'space-between' }}>
        <span>
          <FolderOpen size={13} /> 历史会话 · 按项目
        </span>
        <span style={{ fontWeight: 600, textTransform: 'none', letterSpacing: 0 }}>
          {ordered.length} 个项目 · {totalSessions} 个会话 · {countText(totalMessages)} 条消息
        </span>
      </div>

      <div
        style={{ display: 'flex', flexDirection: 'column', gap: 4, overflow: 'auto', minHeight: 0 }}
      >
        {ordered.map((p) => {
          const active = p.id === activeProjectId;
          const noMessages = p.messageCount === 0;
          return (
            <button
              key={p.id}
              type="button"
              onClick={() => onSelect(p.id)}
              title={p.homePath ?? p.id}
              style={{
                ...rowButton,
                border: `1px solid ${active ? 'var(--sl-brand)' : 'var(--sl-border-light)'}`,
                background: active ? 'var(--sl-brand-subtle)' : 'var(--sl-bg)',
              }}
            >
              <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                <ChevronRight size={12} style={{ flexShrink: 0, opacity: active ? 1 : 0.4 }} />
                <span
                  style={{
                    fontSize: 12,
                    fontWeight: 600,
                    color: active ? 'var(--sl-brand-text)' : 'var(--sl-text-primary)',
                    flex: 1,
                    minWidth: 0,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  }}
                >
                  {p.unclassified ? '未归类(仓库根目录)' : p.name}
                </span>
                <span
                  style={{
                    fontSize: 10,
                    fontWeight: 600,
                    padding: '1px 7px',
                    borderRadius: 'var(--sl-radius-full)',
                    background: noMessages ? 'var(--sl-surface-active)' : 'var(--sl-brand-subtle)',
                    color: noMessages ? 'var(--sl-text-tertiary)' : 'var(--sl-brand-text)',
                    flexShrink: 0,
                  }}
                >
                  {p.sessionCount} 会话
                </span>
                <span
                  style={{
                    fontSize: 10,
                    color: noMessages ? 'var(--sl-amber)' : 'var(--sl-text-tertiary)',
                    flexShrink: 0,
                  }}
                >
                  {countText(p.messageCount)} 消息
                </span>
              </span>
              {p.unclassified && (
                <span style={{ ...note, paddingLeft: 18 }}>
                  这些会话的 home path 就是仓库根目录本身, stripped
                  之后是空串。它没法归到任何一个子项目底下, 所以单独成组 ——
                  合进邻居或者直接藏掉都是一种猜测,这里两种都不做。
                </span>
              )}
              {noMessages && (
                <span style={{ ...note, paddingLeft: 18 }}>
                  有 {p.sessionCount} 个会话,但提取出来的对话是 0 条:这个 home path
                  下启动的工具没有产出可提取的 对话内容(例如 cursor 的 workspace
                  记录)。会话仍在列表里,点进去会说清是哪一种。
                </span>
              )}
              {p.messageCount == null && (
                <span style={{ ...note, paddingLeft: 18, color: 'var(--sl-red)' }}>
                  消息数未加载:语料迁移(021)还没应用到当前数据库,所以这里给不出数字 —— 不是 0。
                </span>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// Screen 2 — one project's sessions
// ─────────────────────────────────────────────────────────────────────────────

export const HistorySessionList: React.FC<{
  project: HistoryProject;
  sessions: HistorySession[];
  activeSessionKey: string | null;
  onSelect: (session: HistorySession) => void;
}> = ({ project, sessions, activeSessionKey, onSelect }) => {
  // The server already returns dated first (newest first) and undated last. The
  // UI does not re-sort: re-sorting would be a second ordering rule that can
  // disagree with the one the numbers were measured under.
  const firstUndated = sessions.findIndex((s) => !s.dated);
  const undatedCount = sessions.filter((s) => !s.dated).length;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 6, minHeight: 0 }}>
      <div style={columnHead}>
        <MessageSquare size={13} />
        <span style={{ textTransform: 'none', letterSpacing: 0, fontWeight: 700 }}>
          {project.unclassified ? '未归类(仓库根目录)' : project.name}
        </span>
        <span
          style={{ marginLeft: 'auto', fontWeight: 600, textTransform: 'none', letterSpacing: 0 }}
        >
          {sessions.length} 个会话
        </span>
      </div>
      {project.homePath && (
        <div style={{ ...note, ...mono, fontSize: 10, flexShrink: 0 }}>{project.homePath}</div>
      )}

      <div
        style={{ display: 'flex', flexDirection: 'column', gap: 4, overflow: 'auto', minHeight: 0 }}
      >
        {sessions.map((s, i) => {
          const key = `${s.harness}:${s.id}`;
          const active = key === activeSessionKey;
          const range = dateRange(s);
          const noMessages = s.messageCount === 0;
          return (
            <React.Fragment key={key}>
              {i === firstUndated && undatedCount > 0 && (
                <div
                  style={{
                    ...note,
                    padding: '6px 10px',
                    marginTop: 2,
                    borderTop: '1px dashed var(--sl-border)',
                    color: 'var(--sl-text-primary)',
                    fontWeight: 600,
                  }}
                >
                  以下 {undatedCount} 条没有时间记录(这个工具不记录时间戳),所以排在最后、 单独标注
                  —— 不按时间混排,也不当成最早的一条。
                </div>
              )}
              <button
                type="button"
                onClick={() => onSelect(s)}
                style={{
                  ...rowButton,
                  border: `1px solid ${active ? 'var(--sl-brand)' : 'var(--sl-border-light)'}`,
                  background: active ? 'var(--sl-brand-subtle)' : 'var(--sl-bg)',
                }}
              >
                <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <span
                    style={{
                      fontSize: 10,
                      fontWeight: 600,
                      padding: '1px 7px',
                      borderRadius: 'var(--sl-radius-full)',
                      background: 'var(--sl-surface-active)',
                      color: 'var(--sl-text-secondary)',
                      flexShrink: 0,
                    }}
                  >
                    {harnessLabel(s.harness)}
                  </span>
                  <span
                    style={{
                      fontSize: 11,
                      color: range.undated ? 'var(--sl-amber)' : 'var(--sl-text-secondary)',
                      flex: 1,
                      minWidth: 0,
                      display: 'flex',
                      alignItems: 'center',
                      gap: 4,
                    }}
                  >
                    {range.undated && <Clock size={10} />}
                    <span style={{ ...mono, fontSize: 10 }}>{range.text}</span>
                  </span>
                  <span
                    style={{
                      fontSize: 10,
                      color: noMessages ? 'var(--sl-amber)' : 'var(--sl-text-tertiary)',
                      flexShrink: 0,
                    }}
                  >
                    {countText(s.messageCount)} 消息
                  </span>
                </span>
                <span style={{ ...note, ...mono, fontSize: 10, paddingLeft: 2 }} title={s.id}>
                  {s.id}
                </span>
                {range.undated && (
                  <span style={{ ...note, paddingLeft: 2 }}>
                    时间未记录:{harnessLabel(s.harness)} 不写时间戳,所以这里没有日期 ——
                    不编一个出来,也不按最早排。
                  </span>
                )}
                {noMessages && !range.undated && (
                  <span style={{ ...note, paddingLeft: 2 }}>
                    这条会话没有提取到对话内容(0 条消息)。点进去会说清原因。
                  </span>
                )}
              </button>
            </React.Fragment>
          );
        })}
      </div>
    </div>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// Screen 3 — one session's conversation
// ─────────────────────────────────────────────────────────────────────────────

const HistorySessionDetailView: React.FC<{
  detail: HistorySessionDetail;
  loader: HistoryLoader;
}> = ({ detail, loader }) => {
  const [lines, setLines] = useState<string[] | null>(null);
  const [readError, setReadError] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState<Record<number, boolean>>({});
  const [openError, setOpenError] = useState<string | null>(null);
  const contextPath = detail.contextPath;

  useEffect(() => {
    let alive = true;
    setLines(null);
    setReadError(null);
    setCollapsed({});
    if (!contextPath) return;
    loader
      .readContextSlice(contextPath, 1, Number.MAX_SAFE_INTEGER)
      .then((slice) => {
        if (alive) setLines(slice.lines);
      })
      .catch((e) => {
        if (alive) setReadError(String(e));
      });
    return () => {
      alive = false;
    };
  }, [contextPath, loader]);

  const open = useCallback(
    (path: string) => {
      setOpenError(null);
      loader.openPath(path).catch((e) => setOpenError(String(e)));
    },
    [loader],
  );

  const missingSource = !detail.sourcePresent;

  return (
    <div
      style={{ display: 'flex', flexDirection: 'column', gap: 10, minHeight: 0, overflow: 'auto' }}
    >
      {/* Header: what this session is, and where its body lives */}
      <div style={{ flexShrink: 0, display: 'flex', flexDirection: 'column', gap: 6 }}>
        <div style={{ ...columnHead, justifyContent: 'space-between' }}>
          <span>
            <FileText size={13} /> {harnessLabel(detail.harness)} · 会话原文
          </span>
          <span style={{ fontWeight: 600, textTransform: 'none', letterSpacing: 0 }}>
            {detail.turns.length} 条
          </span>
        </div>

        <div style={{ ...note, ...mono, fontSize: 10 }}>
          原始文件:{' '}
          {missingSource ? (
            <span style={{ color: 'var(--sl-red)', fontWeight: 700 }}>
              已不在磁盘上({detail.sourcePath})
            </span>
          ) : (
            <button
              type="button"
              onClick={() => open(detail.sourcePath)}
              title="用系统默认程序打开这个文件"
              style={{
                background: 'none',
                border: 'none',
                padding: 0,
                color: 'var(--sl-brand)',
                cursor: 'pointer',
                font: 'inherit',
                textAlign: 'left',
              }}
            >
              {detail.sourcePath} ↗
            </button>
          )}
        </div>
        {missingSource && (
          <div style={{ ...note, color: 'var(--sl-red)' }}>
            这个会话的原始文件已经不在了。下面的每一条仍然带着它在原文件里的行号 ——
            行号还在,内容没了,这两件事分开说,不一起藏掉。
          </div>
        )}

        {detail.countsAgree === false && (
          <div style={{ ...note, color: 'var(--sl-red)' }}>
            数量对不上:提取文件自己声明 {detail.declaredMessageCount ?? '未声明'} 条,数据库里记的是{' '}
            {detail.storedMessageCount ?? '未记录'} 条。这是一个缺陷,不是约数误差 —— 按契约该打回。
          </div>
        )}
        {detail.countsAgree === null && (
          <div style={note}>
            消息数只在一侧有记录(声明值 {detail.declaredMessageCount ?? '未声明'} / 库内值{' '}
            {detail.storedMessageCount ?? '未记录'}
            ),所以没法比对。有一半缺失不是「对不上」,这里不假装比过。
          </div>
        )}

        {openError && (
          <div style={{ ...note, color: 'var(--sl-red)' }}>
            打开文件失败(实测错误文本):
            <span style={mono}>{openError}</span>
          </div>
        )}
      </div>

      {/* Empty / missing states — every one says why and what is next */}
      {detail.turns.length === 0 ? (
        <Explanation title="这个会话没有可读的对话">
          <p>{detail.emptyReason ?? '没有给出原因。'}</p>
          <p style={{ marginTop: 6 }}>
            上面是读取链路原样返回的原因。这里不渲染一个空的对话框 ——
            一个空框会被读成「你没说过话」, 而这个项目已经为这句话付过一次代价了。
          </p>
        </Explanation>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {/* A body that cannot be read is reported above the structure, not instead
              of it: the line numbers come from the read and survive the failure. */}
          {readError && <MeasuredError what="读不到这条会话的正文" error={readError} />}
          {lines === null && !readError && (
            <div style={{ ...note, display: 'flex', alignItems: 'center', gap: 8 }}>
              <Loader2 size={13} className="animate-spin" /> 正在从本机读取正文……
              （上面的行号已经可以核对,正文随后出现）
            </div>
          )}
          {detail.turns.map((t, i) => {
            const isCollapsed = collapsed[i] === true;
            const slice = lines ? sliceForTurn(lines, t.contextLineStart, t.contextLineEnd) : null;
            return (
              <div
                key={`${t.line}-${i}`}
                style={{
                  padding: '8px 10px',
                  borderRadius: 'var(--sl-radius-md)',
                  border: '1px solid var(--sl-border-light)',
                  background: 'var(--sl-bg)',
                }}
              >
                <button
                  type="button"
                  onClick={() => setCollapsed((s) => ({ ...s, [i]: !s[i] }))}
                  aria-expanded={!isCollapsed}
                  style={{
                    width: '100%',
                    textAlign: 'left',
                    cursor: 'pointer',
                    background: 'none',
                    border: 'none',
                    padding: 0,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                  }}
                >
                  <ChevronRight
                    size={12}
                    style={{
                      flexShrink: 0,
                      transform: isCollapsed ? 'none' : 'rotate(90deg)',
                      transition: 'transform 120ms ease',
                    }}
                  />
                  <span
                    style={{
                      fontSize: 10,
                      fontWeight: 700,
                      padding: '1px 7px',
                      borderRadius: 'var(--sl-radius-full)',
                      background: 'var(--sl-surface-active)',
                      color: 'var(--sl-text-secondary)',
                      flexShrink: 0,
                    }}
                  >
                    {turnLabelText(t.label, t.labelRaw)}
                  </span>
                  <span
                    style={{ ...mono, fontSize: 10, color: 'var(--sl-brand)', flexShrink: 0 }}
                    title="这一条出自原始文件的第几行"
                  >
                    原文件第 {t.line} 行
                  </span>
                  <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', flexShrink: 0 }}>
                    {t.chars} 字
                  </span>
                  <span
                    style={{ marginLeft: 'auto', fontSize: 10, color: 'var(--sl-text-tertiary)' }}
                  >
                    {isCollapsed ? '已折叠(点开)' : '折叠'}
                  </span>
                </button>
                {!isCollapsed && (
                  <div style={{ marginTop: 6, paddingLeft: 20 }}>
                    {slice ? (
                      <>
                        <pre
                          style={{
                            margin: 0,
                            fontFamily:
                              'var(--sl-font-mono, ui-monospace, SFMono-Regular, Menlo, monospace)',
                            fontSize: 11,
                            lineHeight: 1.65,
                            whiteSpace: 'pre-wrap',
                            wordBreak: 'break-word',
                            color: 'var(--sl-text-primary)',
                            maxHeight: 420,
                            overflow: 'auto',
                          }}
                        >
                          {slice.text || '(这一段在本地文件里是空的)'}
                        </pre>
                        {slice.truncated && (
                          <div style={{ ...note, marginTop: 4, color: 'var(--sl-amber)' }}>
                            本地文件比指针说的短,到这里就结束了。
                          </div>
                        )}
                      </>
                    ) : (
                      <div style={note}>这一条的正文还在读。</div>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* What this session distilled. Sits AFTER the conversation and OUTSIDE
          its empty-state branch on purpose: a session that distilled nothing is
          a real answer about this session, and it must be reachable whether or
          not the conversation itself could be read. */}
      <SessionWorkSection harness={detail.harness} id={detail.id} loader={loader} />
    </div>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// What this session distilled — the session<->work bridge, read side
//
// Three states, and they must not look alike. The whole point of the bridge's
// read model is that it separates "nothing here" from "something here that does
// not line up", and a UI that renders both as an empty list destroys the only
// information the read model actually carries.
//
//   (a) total === 0  -> neutral. The read model hands back two DIFFERENT reasons
//                       for the same empty count and refuses to merge them; this
//                       section shows both and picks neither.
//   (b) unresolved > 0 -> red. There ARE records, but N of them do not resolve.
//                       Each one names what is missing.
//   (c) all resolved  -> the list, plus a health line.
//
// The read is `invoke('cmd_session_work')` and it returns the read model's own
// JSON text. It is parsed here, never reshaped: a second, summarised shape would
// be a place for the two models to drift apart, and this screen exists precisely
// to show where they do NOT line up.
// ─────────────────────────────────────────────────────────────────────────────

/** The shape as the read model actually emits it. Field names copied, not invented. */
type SessionWorkRecord = {
  memoryId: string;
  title: string;
  kind: string;
  confidence: string;
  status: string;
  sourcePath: string;
  lineStart: number;
  lineEnd: number;
  joinState: string;
  resolvedBy: string[];
  missing: string[];
  topicId: string | null;
};

type SessionWorkPayload = {
  ref: { harness: string; id: string };
  session: {
    harness: string;
    id: string;
    rawPath: string;
    project: string | null;
    messageCount: number | null;
    corpusSession: boolean;
  } | null;
  records: SessionWorkRecord[];
  numbers: { total: number; resolved: number; unresolved: number };
  driftUnresolvedRefs: number;
  notes: string[];
  contract: string;
};

/** Parse the read model's own text. A parse failure is surfaced, never swallowed. */
function parseSessionWork(text: string): SessionWorkPayload {
  const parsed = JSON.parse(text) as SessionWorkPayload;
  // The three numbers are the whole contract. If the read model stops sending
  // them, this screen must say so rather than compute look-alikes from the array.
  if (!parsed.numbers || typeof parsed.numbers.total !== 'number') {
    throw new Error(
      `session_work 没有返回 numbers.total —— 读模型形状变了,不猜。原文: ${text.slice(0, 200)}`,
    );
  }
  return parsed;
}

const kindLabel: Record<string, string> = {
  decision: '决定',
  failure: '失败',
  unresolved: '未决',
  preference: '偏好',
  fact: '事实',
};

const SessionWorkSection: React.FC<{
  harness: string;
  id: string;
  loader: HistoryLoader;
}> = ({ harness, id, loader }) => {
  const [state, setState] = useState<
    | { kind: 'loading' }
    | { kind: 'error'; error: string }
    | { kind: 'ready'; payload: SessionWorkPayload }
  >({ kind: 'loading' });

  useEffect(() => {
    let alive = true;
    setState({ kind: 'loading' });
    loader
      .sessionWork(harness, id)
      .then((text) => {
        if (!alive) return;
        try {
          setState({ kind: 'ready', payload: parseSessionWork(text) });
        } catch (e) {
          // A malformed payload is a defect in the bridge, not an empty answer.
          setState({ kind: 'error', error: String(e) });
        }
      })
      .catch((e) => {
        if (alive) setState({ kind: 'error', error: String(e) });
      });
    return () => {
      alive = false;
    };
  }, [harness, id, loader]);

  if (state.kind === 'loading') {
    return (
      <div style={{ ...note, display: 'flex', alignItems: 'center', gap: 8 }}>
        <Loader2 size={13} className="animate-spin" /> 正在读这个会话提炼出的记录……
      </div>
    );
  }

  if (state.kind === 'error') {
    // The read failed. Named, verbatim — this is a refusal from the read model,
    // and every one of its refusals says what is missing.
    return <MeasuredError what="读不到这个会话的提炼记录" error={state.error} />;
  }

  const { numbers, records, notes, session } = state.payload;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8, flexShrink: 0 }}>
      <div style={columnHead}>
        <Brain size={13} /> 这个会话提炼出了什么
      </div>

      {/* Health line — always present, in every state. The three numbers are
          what the operator checks first; hiding them in state (a) would mean
          the one state where "nothing distilled" is a real answer shows no
          evidence at all. */}
      <div style={{ ...note, ...mono, fontSize: 11 }}>
        {numbers.total} 条记录 · {numbers.resolved} 条对得上原文 · {numbers.unresolved} 条对不上
        {session ? ` · 原文 ${session.messageCount ?? '未记录'} 条消息` : ' · 语料里没有这条会话'}
        {state.payload.driftUnresolvedRefs > 0
          ? ` · 另有 ${state.payload.driftUnresolvedRefs} 处 drift 引用未消解`
          : ''}
      </div>

      {/* ── (a) no records at all ─────────────────────────────────────────── */}
      {numbers.total === 0 ? (
        <Explanation title="这个会话没有提炼出任何记录">
          {/* The read model hands back two different reasons for the same empty
              count and explicitly refuses to merge them. Showing both, and not
              picking one, is the whole content of this state. */}
          <p style={{ margin: 0 }}>{notes[0] ?? '读模型没有给出原因。'}</p>
          {notes[1] && <p style={{ marginTop: 6, marginBottom: 0 }}>{notes[1]}</p>}
        </Explanation>
      ) : /* ── (b) records exist but some do not resolve ─────────────────── */
      numbers.unresolved > 0 ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          {/* Deliberately red, and deliberately NOT the same component as (a):
              "there is nothing here" and "there is something here that does not
              line up" must never look alike. */}
          <div
            style={{
              padding: '10px 12px',
              borderRadius: 'var(--sl-radius-md)',
              border: '1px solid var(--sl-red)',
              background: 'var(--sl-red-subtle)',
              fontSize: 12,
              lineHeight: 1.6,
              color: 'var(--sl-red)',
            }}
          >
            <div style={{ fontWeight: 700, marginBottom: 4 }}>
              {numbers.unresolved} / {numbers.total} 条记录对不上原文
            </div>
            <div>记录是有的,但这个数对不上。对不上的每一条下面说了它缺什么。</div>
          </div>

          {records
            .filter((r) => r.missing.length > 0)
            .map((r) => (
              <div
                key={r.memoryId}
                style={{
                  padding: '8px 10px',
                  borderRadius: 'var(--sl-radius-md)',
                  border: '1px solid var(--sl-red)',
                  background: 'var(--sl-red-subtle)',
                  fontSize: 12,
                }}
              >
                <div style={{ fontWeight: 600, color: 'var(--sl-red)' }}>
                  {r.title} · {kindLabel[r.kind] ?? r.kind}
                </div>
                <div style={{ ...note, ...mono, fontSize: 10, color: 'var(--sl-red)' }}>
                  {r.memoryId} · 第 {r.lineStart}–{r.lineEnd} 行 · join={r.joinState}
                </div>
                <ul style={{ margin: '4px 0 0', paddingLeft: 18 }}>
                  {r.missing.map((m) => (
                    <li key={m} style={{ ...note, color: 'var(--sl-red)' }}>
                      {m}
                    </li>
                  ))}
                </ul>
              </div>
            ))}
        </div>
      ) : /* ── (c) everything resolves ────────────────────────────────────── */
      records.length > 0 ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <div style={{ ...note }}>{notes[0] ?? ''}</div>
          {records.map((r) => (
            <div
              key={r.memoryId}
              style={{
                padding: '8px 10px',
                borderRadius: 'var(--sl-radius-md)',
                border: '1px solid var(--sl-border-light)',
                background: 'var(--sl-bg)',
                fontSize: 12,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'baseline', gap: 6, flexWrap: 'wrap' }}>
                <span style={{ fontWeight: 600 }}>{r.title}</span>
                <span style={{ ...note, fontSize: 10, ...mono }}>
                  {kindLabel[r.kind] ?? r.kind} · {r.confidence}
                </span>
              </div>
              <div style={{ ...note, ...mono, fontSize: 10 }}>
                第 {r.lineStart}–{r.lineEnd} 行 · {r.joinState} · {r.resolvedBy.join(' + ') || '无'}
              </div>
              {r.sourcePath && (
                <button
                  type="button"
                  onClick={() => loader.openPath(r.sourcePath)}
                  title="打开这条记录的来源文件"
                  style={{
                    background: 'none',
                    border: 'none',
                    padding: 0,
                    color: 'var(--sl-brand)',
                    cursor: 'pointer',
                    font: 'inherit',
                    fontSize: 11,
                    textAlign: 'left',
                    ...mono,
                  }}
                >
                  {r.sourcePath} ↗
                </button>
              )}
            </div>
          ))}
        </div>
      ) : (
        /* total > 0 but the array is empty. That is a disagreement between the
           read model's two own numbers, not an empty session. */
        <Explanation title={`记录数是 ${numbers.total},但一条也没返回`}>
          <p style={{ margin: 0 }}>
            读模型自己说一共 {numbers.total} 条、{numbers.resolved} 条对得上,但 records
            数组是空的。这两个说法对不上,这里把它们都摆出来,不挑一个顺眼的。
          </p>
        </Explanation>
      )}

      {notes.length > 1 && numbers.total > 0 && numbers.unresolved === 0 && (
        <div style={{ ...note, fontSize: 10 }}>
          {notes.slice(1).map((n) => (
            <div key={n}>{n}</div>
          ))}
        </div>
      )}
    </div>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// The panel
// ─────────────────────────────────────────────────────────────────────────────

export const HistorySessionsPanel: React.FC<{ loader?: HistoryLoader }> = ({ loader }) => {
  const load = useMemo<HistoryLoader>(() => loader ?? liveLoader, [loader]);

  const [projects, setProjects] = useState<HistoryProjectsPayload | null>(null);
  const [projectsError, setProjectsError] = useState<string | null>(null);
  // The project list renders a measured error, a not-loaded explanation or the
  // list itself; there is no separate "is the corpus loaded" spinner to keep in
  // sync with those three, so the flag is not kept.
  const [, setLoadingProjects] = useState(true);

  const [projectId, setProjectId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<HistorySessionsPayload | null>(null);
  const [sessionsError, setSessionsError] = useState<string | null>(null);
  const [loadingSessions, setLoadingSessions] = useState(false);

  const [sessionKey, setSessionKey] = useState<string | null>(null);
  const [turns, setTurns] = useState<HistoryTurnsPayload | null>(null);
  const [turnsError, setTurnsError] = useState<string | null>(null);
  const [loadingTurns, setLoadingTurns] = useState(false);

  const reloadProjects = useCallback(() => {
    setLoadingProjects(true);
    setProjectsError(null);
    load
      .listProjects()
      .then((text) => setProjects(JSON.parse(text) as HistoryProjectsPayload))
      .catch((e) => setProjectsError(String(e)))
      .finally(() => setLoadingProjects(false));
  }, [load]);

  useEffect(reloadProjects, [reloadProjects]);

  // Picking a project loads its sessions and clears any open session.
  useEffect(() => {
    if (!projectId) return;
    let alive = true;
    setLoadingSessions(true);
    setSessionsError(null);
    setSessions(null);
    setSessionKey(null);
    setTurns(null);
    load
      .listSessions(projectId)
      .then((text) => {
        if (alive) setSessions(JSON.parse(text) as HistorySessionsPayload);
      })
      .catch((e) => {
        if (alive) setSessionsError(String(e));
      })
      .finally(() => {
        if (alive) setLoadingSessions(false);
      });
    return () => {
      alive = false;
    };
  }, [projectId, load]);

  // Picking a session loads its turns.
  useEffect(() => {
    if (!sessionKey) return;
    const [harness, ...rest] = sessionKey.split(':');
    const id = rest.join(':');
    let alive = true;
    setLoadingTurns(true);
    setTurnsError(null);
    setTurns(null);
    load
      .getSessionTurns(harness, id)
      .then((text) => {
        if (alive) setTurns(JSON.parse(text) as HistoryTurnsPayload);
      })
      .catch((e) => {
        if (alive) setTurnsError(String(e));
      })
      .finally(() => {
        if (alive) setLoadingTurns(false);
      });
    return () => {
      alive = false;
    };
  }, [sessionKey, load]);

  const activeProject = useMemo(
    () => projects?.projects.find((p) => p.id === projectId) ?? null,
    [projects, projectId],
  );
  const activeSession = useMemo(
    () => sessions?.sessions.find((s) => `${s.harness}:${s.id}` === sessionKey) ?? null,
    [sessions, sessionKey],
  );

  const notInTauri = !isTauri();

  return (
    <div style={{ ...shell, gap: 10, padding: '12px 14px', height: '100%' }}>
      {notInTauri && (
        <Explanation title="不在 SeatLoom 桌面端里">
          历史会话要读本机的数据库和本机的文件,浏览器里两样都没有。用{' '}
          <span style={mono}>pnpm tauri dev</span> 或装好的 SeatLoom 应用打开,这里才会有内容 ——
          现在给不出任何数字,也不编一个空列表冒充「你没有历史」。
        </Explanation>
      )}

      {!notInTauri && projectsError && (
        <MeasuredError what="读不到项目列表" error={projectsError} />
      )}
      {!notInTauri && !projectsError && projects && !projects.corpusLoaded && (
        <Explanation title="历史会话还没加载">
          <p>
            后端说语料迁移还没应用到当前数据库,所以一个项目、一个会话、一条消息都报不出来。
            下一步:应用那条迁移(021),然后回到这个页面。
          </p>
          <p style={{ marginTop: 6 }}>
            这里不渲染一个空的项目列表 ——
            一个空列表会被读成「你没有历史」,而这个项目已经为这句话付过一次代价。
          </p>
        </Explanation>
      )}

      {!notInTauri && !projectsError && !projects && <Loading what="正在读取项目列表……" />}

      {projects && projects.corpusLoaded && (
        <div style={{ display: 'flex', gap: 12, flex: 1, minHeight: 0 }}>
          {/* Screen 1 */}
          <div
            style={{
              width: 300,
              flexShrink: 0,
              display: 'flex',
              flexDirection: 'column',
              minHeight: 0,
              borderRight: '1px solid var(--sl-divider)',
              paddingRight: 12,
            }}
          >
            <HistoryProjectList
              projects={projects.projects}
              activeProjectId={projectId}
              onSelect={setProjectId}
            />
          </div>

          {/* Screen 2 */}
          <div
            style={{
              width: 320,
              flexShrink: 0,
              display: 'flex',
              flexDirection: 'column',
              minHeight: 0,
              borderRight: '1px solid var(--sl-divider)',
              paddingRight: 12,
            }}
          >
            {!activeProject ? (
              <Explanation title="先选一个项目">
                左边每个项目就是你以前启动过 agent
                的一个目录。点一个,这里列出那个目录下真实开过的会话。
              </Explanation>
            ) : sessionsError ? (
              <MeasuredError
                what={`读不到 ${activeProject.name} 的会话列表`}
                error={sessionsError}
              />
            ) : loadingSessions || !sessions ? (
              <Loading what="正在读取这个项目的会话……" />
            ) : sessions.sessions.length === 0 ? (
              <Explanation title="这个项目一个会话都没有">
                读取链路只返回「至少有一个会话」的项目,所以正常情况下这里不会空。如果真的空了,
                说明这个项目的会话记录被删过或者从没加载过 —— 不是「你没干过活」。
              </Explanation>
            ) : (
              <HistorySessionList
                project={activeProject}
                sessions={sessions.sessions}
                activeSessionKey={sessionKey}
                onSelect={(s) => setSessionKey(`${s.harness}:${s.id}`)}
              />
            )}
          </div>

          {/* Screen 3 */}
          <div
            style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', minHeight: 0 }}
          >
            {!activeSession ? (
              <Explanation title="再选一条会话">
                中间每一条是那个目录下真实开过的一次会话。点一条,这里按原来的顺序读出当时的对话,
                每一条都标着它出自原始文件的第几行。
              </Explanation>
            ) : turnsError ? (
              <MeasuredError what="读不到这条会话的对话" error={turnsError} />
            ) : loadingTurns || !turns ? (
              <Loading what="正在读取这条会话……" />
            ) : (
              <HistorySessionDetailView detail={turns.session} loader={load} />
            )}
          </div>
        </div>
      )}

      {projects && (
        <div style={{ flexShrink: 0, display: 'flex', justifyContent: 'flex-end' }}>
          <button
            type="button"
            onClick={reloadProjects}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '4px 10px',
              fontSize: 11,
              fontWeight: 600,
              background: 'var(--sl-surface)',
              border: '1px solid var(--sl-border-light)',
              borderRadius: 'var(--sl-radius-sm)',
              color: 'var(--sl-text-secondary)',
              cursor: 'pointer',
            }}
          >
            <RefreshCw size={11} /> 重新读取
          </button>
        </div>
      )}
    </div>
  );
};

export default HistorySessionsPanel;
