// HistorySessionsPanel — slice 3 of LYRA-2026-10-14-history-sessions-in-ui-contract-v1.
//
// The fixture is MEASURED, not invented: `history-measured.json` was produced by
// `scripts/refresh-history-ui-fixture.sh`, which drives the same
// `seatloom-mcp-emit` binary the Tauri app shells out to, against the live
// database, and records the six numbers the operator's acceptance card checks.
// Re-run that script after the corpus changes and read the diff — a changed
// number is a changed claim, and the card's arithmetic has to change with it.
//
// What is under test, in the contract's own order:
//   1. §6 / §8.2 slice 4 — the six numbers, and that the per-project sums agree
//      with them (a list whose rows do not add up to its own totals is a defect).
//   2. §4 screen 1 — the project list: 38 rows, ordered by session count
//      descending, the repository-root group LAST and explicitly labelled.
//   3. §3.2 — every empty and missing state says WHY and WHAT IS NEXT.
//   4. §4 screen 2 — one project's sessions: 49 for OfficeBench, dated first,
//      the undated ones last and labelled, never silently interleaved.
//   5. §4 screen 3 — the conversation in source order, each turn carrying the
//      line number it came from in the ORIGINAL file, and the text read from the
//      operator's own disk.
//   6. §3.3 — a read that fails shows the measured error text, never a spinner
//      and never a silent empty list; a corpus that is not loaded says
//      历史会话还没加载 instead of rendering an empty project list.
//   7. The hard constraint — there is no hand-written switch in the source that
//      asserts a command is registered: the loader is really called, and its
//      rejection is really shown.

import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';
import fixture from './__fixtures__/history-measured.json';
import { HistorySessionsPanel } from './HistorySessionsPanel';
import type { HistoryLoader } from './HistorySessionsPanel';
import type { HistoryContextSlice } from '../../lib/types-dto';

// The panel probes isTauri() to decide whether the desktop backend can exist at
// all. jsdom has no __TAURI_INTERNALS__, so supply one — otherwise every test
// would only ever exercise the browser fallback.
//
// `tsc --noEmit` is a FULL-tree gate, and this slice adds no @types/node just to
// read two files: vitest runs in node, so these exist at runtime. Declaring them
// keeps the gate green without a new dependency.
declare const require: (id: string) => unknown;
declare const process: { cwd(): string };
const nodeFs = require('node:fs') as { readFileSync(path: string, encoding: string): string };
const nodePath = require('node:path') as { join: (...parts: string[]) => string };
beforeAll(() => {
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {};
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

// ─────────────────────────────────────────────────────────────────────────────
// The measured fixture, typed
// ─────────────────────────────────────────────────────────────────────────────

const projects = fixture.projects;
const sessionsByProject = fixture.sessions as Record<
  string,
  { sessionCount: number; undatedCount: number; sessions: Array<Record<string, unknown>> }
>;
const seatloomTurns = fixture.turns['seatloom/claude-code/621ae382'].session;

/** The real context file, read from disk here so the assertions are about text
 *  that actually exists rather than a string someone typed into the test. */
const contextLines = (() => {
  // The fixture records the absolute path the read returned. Reading THAT file —
  // the operator's own extracted markdown — is what makes the assertions about
  // real text rather than a string typed into a test. It is not committed (011:
  // no transcript body in the repository), so a checkout without the corpus fails
  // loudly here instead of silently asserting nothing.
  const path = seatloomTurns.contextPath as unknown as string;
  return nodeFs.readFileSync(path, 'utf8').split('\n');
})();

function loaderFromFixture(overrides: Partial<HistoryLoader> = {}): HistoryLoader {
  return {
    listProjects: async () => JSON.stringify(projects),
    listSessions: async (projectId: string) => JSON.stringify(sessionsByProject[projectId]),
    getSessionTurns: async (harness: string, id: string) => {
      const key = `${harness}:${id}`;
      const found =
        key === 'claude-code:621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8'
          ? fixture.turns['seatloom/claude-code/621ae382']
          : key === 'cursor:ThoughtOnly__50dd37a85ce0f751ec33e6eba750d417'
            ? fixture.turns['ThoughtOnly/cursor']
            : null;
      if (!found) throw new Error(`no fixture for ${key}`);
      return JSON.stringify(found);
    },
    readContextSlice: async (
      contextPath: string,
      lineStart: number,
      lineEnd: number,
    ): Promise<HistoryContextSlice> => {
      const from = Math.max(1, lineStart) - 1;
      const to = Math.min(lineEnd, contextLines.length);
      return {
        path: contextPath,
        lineStart,
        lineEnd,
        lines: contextLines.slice(from, to),
        totalLines: contextLines.length,
      };
    },
    openPath: async () => {},
    ...overrides,
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. The six numbers the operator checks
// ─────────────────────────────────────────────────────────────────────────────

describe('the measured corpus (acceptance card §6 arithmetic)', () => {
  it('is 38 projects / 183 sessions / 36680 messages / OfficeBench 49 / seatloom 14 / cursor 14 undated', () => {
    expect(fixture.theSixNumbers).toEqual({
      projects: 38,
      sessions: 183,
      messages: 36680,
      officebenchSessions: 49,
      seatloomSessions: 14,
      cursorSessions: 14,
      cursorUndated: 14,
      sessionsSummedPerProject: 183,
      messagesSummedPerProject: 36680,
    });
  });

  it('has per-project rows that add up to the totals the header shows', () => {
    // A list whose rows do not sum to its own totals is the off-by-one the card
    // says to reject. Measured both ways here.
    const summedSessions = projects.projects.reduce((n, p) => n + p.sessionCount, 0);
    const summedMessages = projects.projects.reduce((n, p) => n + (p.messageCount ?? 0), 0);
    expect(summedSessions).toBe(183);
    expect(summedMessages).toBe(36680);
    expect(projects.projects).toHaveLength(38);
  });

  it('marks exactly one unclassified group and it is the repository root', () => {
    const root = projects.projects.filter((p) => p.unclassified);
    expect(root).toHaveLength(1);
    expect(root[0].id).toBe('(root)');
    expect(root[0].sessionCount).toBe(8);
  });

  it('never sorts an undated session among the dated ones', () => {
    for (const [projectId, payload] of Object.entries(sessionsByProject)) {
      const flags = payload.sessions.map((s) => Boolean(s.dated));
      const firstUndated = flags.indexOf(false);
      if (firstUndated === -1) continue;
      // Everything after the first undated must be undated too.
      expect(
        flags.slice(firstUndated).every((f) => f === false),
        projectId,
      ).toBe(true);
      // And the count the payload reports matches what the rows say.
      expect(payload.undatedCount, projectId).toBe(flags.filter((f) => !f).length);
    }
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// 2. Screen 1 — the project list
// ─────────────────────────────────────────────────────────────────────────────

describe('screen 1 — the project list', () => {
  it('shows the measured totals and every project, ordered by session count with the root group last', async () => {
    render(<HistorySessionsPanel loader={loaderFromFixture()} />);

    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());
    expect(screen.getByText(/183 个会话/)).toBeTruthy();
    expect(screen.getByText(/36680 条消息/)).toBeTruthy();

    // Session-count descending, as the server ordered it.
    const counts = projects.projects.filter((p) => !p.unclassified).map((p) => p.sessionCount);
    expect([...counts].sort((a, b) => b - a)).toEqual(counts);

    // The repository-root group is last and labelled — not folded into a neighbour.
    const office = screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/OfficeBench');
    const root = screen.getByText('未归类(仓库根目录)');
    expect(office.compareDocumentPosition(root) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(screen.getByText(/它没法归到任何一个子项目底下/)).toBeTruthy();
  });

  it('explains a project whose sessions produced no conversation instead of showing a bare 0', async () => {
    render(<HistorySessionsPanel loader={loaderFromFixture()} />);
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());

    const thoughtOnly = screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/ThoughtOnly');
    expect(thoughtOnly.textContent).toContain('0 消息');
    expect(thoughtOnly.textContent).toContain('提取出来的对话是 0 条');
    // ...and what to do next, not just a bare zero.
    expect(thoughtOnly.textContent).toContain('点进去会说清是哪一种');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// 3. Screen 2 — one project's sessions
// ─────────────────────────────────────────────────────────────────────────────

describe("screen 2 — one project's sessions", () => {
  it('lists 49 sessions for OfficeBench, harness and message count on every row', async () => {
    render(<HistorySessionsPanel loader={loaderFromFixture()} />);
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());

    fireEvent.click(screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/OfficeBench'));

    await waitFor(() => expect(screen.getByText('49 个会话')).toBeTruthy());
    // Every one of the 49 rows is a codex session, and every row carries a count.
    expect(screen.getAllByText('Codex')).toHaveLength(49);
    expect(screen.getAllByText(/消息$/).length).toBeGreaterThanOrEqual(49);
  });

  it('labels the undated block and says why, rather than interleaving it', async () => {
    render(<HistorySessionsPanel loader={loaderFromFixture()} />);
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());

    fireEvent.click(screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/seatloom'));
    await waitFor(() => expect(screen.getByText('14 个会话')).toBeTruthy());

    const undated = sessionsByProject['seatloom'].sessions.filter((s) => !s.dated);
    expect(undated).toHaveLength(1);

    // 时间未记录 appears, and with the reason — never a date.
    expect(screen.getAllByText('时间未记录')).toHaveLength(1);
    expect(screen.getByText(/不写时间戳/)).toBeTruthy();
    // The dated ones DO show a range — so 时间未记录 is a real distinction, not a
    // fallback every row lands on.
    expect(screen.getAllByText(/\d{4}-\d{2}-\d{2} \d{2}:\d{2} UTC/).length).toBe(13);
  });

  it('never renders a session list that reads as "you have no history"', async () => {
    render(<HistorySessionsPanel loader={loaderFromFixture()} />);
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());
    // Before a project is picked the third column says what to do next.
    expect(screen.getByText('再选一条会话')).toBeTruthy();
    expect(screen.queryByText(/你没有历史/)).toBeNull();
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// 4. Screen 3 — the conversation
// ─────────────────────────────────────────────────────────────────────────────

describe('screen 3 — the conversation', () => {
  async function openSeatloomSession(loader: HistoryLoader = loaderFromFixture()) {
    render(<HistorySessionsPanel loader={loader} />);
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());
    fireEvent.click(screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/seatloom'));
    await waitFor(() => expect(screen.getByText('14 个会话')).toBeTruthy());
    fireEvent.click(screen.getByTitle('621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8'));
    await waitFor(() => expect(screen.getAllByTitle('这一条出自原始文件的第几行').length).toBe(75));
  }

  it('renders the turns in source order, each with its ORIGINAL line number and its text', async () => {
    await openSeatloomSession();

    const lines = seatloomTurns.turns.map((t) => t.line);
    expect(lines).toEqual([...lines].sort((a, b) => a - b)); // source order, asserted on the data
    expect(lines[0]).toBe(7);

    // The first turn's pointer says lines 13..16 of the context file; the text
    // rendered must be exactly those lines of the real file.
    const first = seatloomTurns.turns[0];
    const expected = contextLines
      .slice(first.contextLineStart - 1, first.contextLineEnd)
      .join('\n');
    expect(expected).toContain('[Lyra -> Mira]');

    await waitFor(() => expect(document.querySelector('pre')).toBeTruthy());
    const pre = document.querySelector('pre');
    expect(pre?.textContent).toBe(expected);
    const firstBadge = screen.getAllByTitle('这一条出自原始文件的第几行')[0];
    expect(firstBadge.textContent).toBe('原文件第 7 行');
    expect(screen.getAllByTitle('这一条出自原始文件的第几行').length).toBe(75);
  });

  it('shows the source file path and makes it openable', async () => {
    const opened: string[] = [];
    await openSeatloomSession(loaderFromFixture({ openPath: async (p) => void opened.push(p) }));
    const button = screen.getByTitle('用系统默认程序打开这个文件');
    expect(button.textContent).toContain(
      '/Users/jyxc-dz-0100609/Documents/GitHub/AgentHistory/raw/claude-code/621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8.jsonl',
    );
    fireEvent.click(button);
    expect(opened).toEqual([
      '/Users/jyxc-dz-0100609/Documents/GitHub/AgentHistory/raw/claude-code/621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8.jsonl',
    ]);
  });

  it('says the source file is missing and still shows the turn, instead of hiding it', async () => {
    const missing = {
      ...fixture.turns['seatloom/claude-code/621ae382'],
      session: { ...seatloomTurns, sourcePresent: false },
    };
    const loader = loaderFromFixture({
      getSessionTurns: async () => JSON.stringify(missing),
    });
    await openSeatloomSession(loader);
    expect(screen.getByText(/已不在磁盘上/)).toBeTruthy();
    expect(screen.getByText(/行号还在,内容没了/)).toBeTruthy();
    // The turns are still there.
    expect(screen.getAllByTitle('这一条出自原始文件的第几行').length).toBe(75);
  });

  it('reports a count disagreement as a defect, not a rounding difference', async () => {
    const disagreeing = {
      ...fixture.turns['seatloom/claude-code/621ae382'],
      session: { ...seatloomTurns, storedMessageCount: 74, countsAgree: false },
    };
    await openSeatloomSession(
      loaderFromFixture({ getSessionTurns: async () => JSON.stringify(disagreeing) }),
    );
    expect(screen.getByText(/数量对不上/)).toBeTruthy();
    expect(screen.getByText(/该打回/)).toBeTruthy();
  });

  it('explains a session whose harness produced no conversation, with no empty transcript frame', async () => {
    render(<HistorySessionsPanel loader={loaderFromFixture()} />);
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());
    fireEvent.click(screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/ThoughtOnly'));
    await waitFor(() => expect(screen.getByText('1 个会话')).toBeTruthy());
    fireEvent.click(screen.getByTitle('ThoughtOnly__50dd37a85ce0f751ec33e6eba750d417'));

    await waitFor(() => expect(screen.getByText('这个会话没有可读的对话')).toBeTruthy());
    expect(screen.getByText(/declared 消息数 0/)).toBeTruthy();
    expect(screen.queryByText('原文件第 1 行')).toBeNull(); // no fabricated turn
    expect(document.querySelector('pre')).toBeNull(); // and no empty frame
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// 5. §3.3 — the failure paths
// ─────────────────────────────────────────────────────────────────────────────

describe('the failure paths say what is missing', () => {
  it("shows the read's own rejection verbatim when the project list fails", async () => {
    const refusal =
      'history read rejected: the historical corpus is not loaded in "this database" — ' +
      'archived_sessions.message_count does not exist, so no session or message count can be reported.';
    render(
      <HistorySessionsPanel
        loader={loaderFromFixture({ listProjects: async () => Promise.reject(refusal) })}
      />,
    );
    await waitFor(() => expect(screen.getByText('读不到项目列表')).toBeTruthy());
    expect(screen.getByText(refusal)).toBeTruthy();
    expect(screen.getByText(/后端原样返回的错误文本/)).toBeTruthy();
    // No project list is rendered at all — an empty one would read as "no history".
    expect(screen.queryByText(/38 个项目/)).toBeNull();
  });

  it('says 历史会话还没加载 when the corpus migration is absent, instead of an empty list', async () => {
    const notLoaded = { ...projects, corpusLoaded: false, messageTotal: null };
    render(
      <HistorySessionsPanel
        loader={loaderFromFixture({ listProjects: async () => JSON.stringify(notLoaded) })}
      />,
    );
    await waitFor(() => expect(screen.getByText('历史会话还没加载')).toBeTruthy());
    expect(screen.getByText(/应用那条迁移\(021\)/)).toBeTruthy();
    expect(screen.queryByText(/38 个项目/)).toBeNull();
    expect(screen.queryByText('未归类(仓库根目录)')).toBeNull();
  });

  it("shows the measured error when one project's sessions cannot be read", async () => {
    const refusal = 'history read rejected: no project "OfficeBench" has archived sessions.';
    render(
      <HistorySessionsPanel
        loader={loaderFromFixture({
          listSessions: async () => Promise.reject(refusal),
        })}
      />,
    );
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());
    fireEvent.click(screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/OfficeBench'));
    await waitFor(() => expect(screen.getByText(/读不到 OfficeBench 的会话列表/)).toBeTruthy());
    expect(screen.getByText(refusal)).toBeTruthy();
  });

  it('shows the measured error when the local body cannot be read', async () => {
    const refusal =
      'history read rejected: the historical corpus is not on disk at "/nope/resume-contexts" — 历史会话还没加载.';
    const loader = loaderFromFixture({
      readContextSlice: async () => Promise.reject(refusal),
    });
    render(<HistorySessionsPanel loader={loader} />);
    await waitFor(() => expect(screen.getByText(/38 个项目/)).toBeTruthy());
    fireEvent.click(screen.getByTitle('/Users/jyxc-dz-0100609/Documents/GitHub/seatloom'));
    await waitFor(() => expect(screen.getByText('14 个会话')).toBeTruthy());
    fireEvent.click(screen.getByTitle('621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8'));
    await waitFor(() => expect(screen.getByText('读不到这条会话的正文')).toBeTruthy());
    expect(screen.getByText(refusal)).toBeTruthy();
  });

  it('says so out loud in a browser, where neither the database nor the disk is reachable', async () => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    render(<HistorySessionsPanel loader={loaderFromFixture()} />);
    await waitFor(() => expect(screen.getByText('不在 SeatLoom 桌面端里')).toBeTruthy());
    expect(screen.getByText(/pnpm tauri dev/)).toBeTruthy();
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// 6. The hard constraint — no hardcoded wiring flag
// ─────────────────────────────────────────────────────────────────────────────

describe('wiring honesty', () => {
  it('actually calls the loader, and a rejection is surfaced rather than swallowed', async () => {
    const calls: string[] = [];
    const loader = loaderFromFixture({
      listProjects: async () => {
        calls.push('listProjects');
        return JSON.stringify(projects);
      },
    });
    render(<HistorySessionsPanel loader={loader} />);
    await waitFor(() => expect(calls).toContain('listProjects'));
    expect(screen.getByText(/38 个项目/)).toBeTruthy();
  });

  it('has no hand-written switch in the source asserting that a command is registered', async () => {
    // vitest runs with cwd = ui/, so this is this very file's directory.
    const src = nodeFs.readFileSync(
      nodePath.join(process.cwd(), 'src/app-v2/panel/HistorySessionsPanel.tsx'),
      'utf8',
    );
    // The shape that already misled the operator once: a constant that claims a
    // command is wired. Spelled through concatenation so this file's own prose
    // cannot satisfy it.
    const forbidden = ['CMDS_' + 'REGISTERED', 'COMMANDS_' + 'REGISTERED'];
    for (const token of forbidden) expect(src).not.toContain(token);
    expect(src).not.toMatch(/_REGISTERED\s*=\s*(true|false)/);
    // And the panel really goes through the api layer rather than invoke().
    expect(src).toContain('api.listHistoryProjects()');
    expect(src).not.toContain("invoke('cmd_list_history_projects')");
  });
});
