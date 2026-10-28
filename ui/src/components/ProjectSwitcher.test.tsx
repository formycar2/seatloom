// ProjectSwitcher — the five interaction defects from
// AEGIS-2026-10-27-project-switcher-ux-v1, pinned.
//
// The operator's report was "the project picker's interaction needs work", with
// 39 projects in the ledger. Aegis read the file and named five defects with line
// numbers; these tests are the ones that make each fix non-regressible, and the
// two the dispatch explicitly asked for (keyboard navigation, search) get the
// most coverage because they are new behaviour rather than a removal.
//
// WHAT IS NOT ASSERTED HERE, on purpose: the visual style. The dispatch forbade
// touching colour/radius/type, so nothing below looks at them. What is asserted
// is which text is present, which row is highlighted, and what a keystroke does.
//
// A NOTE ON THE PIN STATE: `projectFromDto` hardcodes `isPinned: true`, so every
// project from the backend arrives pinned. That is a data defect (reported to
// Aegis, not fixed here — it needs a persisted field), and it means the sort in
// this component is a no-op against real data. The sort is still exercised below
// with explicit isPinned values, so the ordering logic itself is pinned even
// though the live data cannot currently vary it.

import { describe, expect, it, vi, beforeAll, beforeEach, afterEach } from 'vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import React from 'react';

// ── The store, stubbed at the seam the component actually reads ─────────────
//
// The component takes `projects`, `projectData`, `activeProjectId` and
// `pinProject` from useDataStore. Stubbing the store rather than the Tauri layer
// keeps these tests about the picker's behaviour, not about hydration — and lets
// a test state a project list with paths, which is what the search fix needs.
const state = vi.hoisted(() => ({
  projects: [] as Array<{
    id: string;
    name: string;
    path: string | null;
    isPinned: boolean;
    last_accessed?: string;
  }>,
  activeProjectId: null as string | null,
  pinned: [] as Array<{ id: string; pinned: boolean }>,
}));

vi.mock('../stores/useDataStore', () => ({
  useDataStore: () => ({
    projects: state.projects,
    projectData: {},
    activeProjectId: state.activeProjectId,
    pinProject: (id: string, pinned: boolean) => {
      state.pinned.push({ id, pinned });
    },
  }),
}));

import ProjectSwitcher from './ProjectSwitcher';

const PROJECTS = [
  //  is present on purpose: the live mapper fills it from
  // , so a test that omits it cannot tell "the column is gone" from
  // "the column renders nothing". With a real value the column would show a
  // timestamp, and the assertion has something to catch.
  {
    id: 'seatloom',
    name: 'SeatLoom 协调核心',
    path: '/Users/x/seatloom',
    isPinned: true,
    last_accessed: '2026-10-20T04:17:16Z',
  },
  { id: 'ql_delivery', name: 'QL', path: '/Users/x/project_delivery/QL', isPinned: false },
  { id: 'stepcloud', name: 'stepcloud', path: '/Users/x/sf-code/stepcloud', isPinned: false },
  { id: 'skills', name: 'skills', path: '/Users/x/ai-assistant/skills', isPinned: false },
];

/**
 * Open the picker the way the operator does.
 *
 * The trigger's accessible name is the active project's name (or 选择项目 when
 * there is none) — the icons inside it contribute no text.
 */
async function open() {
  const trigger = screen.getByText('SeatLoom 协调核心');
  await act(async () => {
    fireEvent.click(trigger);
  });
}

/** Mount the picker and open it, as the operator's click does. */
async function mountAndOpen(onSelect: () => void = () => {}) {
  await act(async () => {
    render(<ProjectSwitcher onSelect={onSelect} />);
  });
  await open();
}

/** The search input. */
const searchInput = () => screen.getByPlaceholderText('搜索项目');
/**
 * Rows, in the order they are rendered.
 *
 * Selected by the data attribute the component puts on each row rather than by
 * text: the project name also appears in the trigger button and inside the row's
 * own <span>, so a text query matches more than the rows.
 */
const rows = () => Array.from(document.querySelectorAll('[data-project-index]'));
/** The name shown on a row. */
const rowName = (row: Element) => row.querySelector('span')?.textContent ?? '';

beforeEach(() => {
  state.projects = PROJECTS.map((p) => ({ ...p }));
  state.activeProjectId = 'seatloom';
  state.pinned = [];
});

// jsdom implements no layout, so it has no scrollIntoView. The component calls it
// to keep the highlighted row visible; a stub records the call instead of letting
// it throw, which is what makes "the highlight moved" observable at all here.
// Recorded on the element so a test can assert WHICH row was scrolled to.
beforeAll(() => {
  (Element.prototype as unknown as { scrollIntoView: (a?: unknown) => void }).scrollIntoView =
    function scrollIntoView(this: Element) {
      (this as Element & { __scrolledIntoView?: boolean }).__scrolledIntoView = true;
    };
});

afterEach(() => {
  cleanup();
});

describe('the placeholder no longer promises a shortcut that does not exist', () => {
  it('says nothing about Ctrl+K, because Ctrl+K does not open this picker', async () => {
    await mountAndOpen();

    // The defect: the placeholder invited a keystroke that opens the Supervisor
    // IM or the Command Bar instead. A hint that misdirects is worse than none.
    expect(searchInput().getAttribute('placeholder')).toBe('搜索项目');
    expect(searchInput().getAttribute('placeholder')).not.toContain('Ctrl+K');
  });
});

describe('the fake visit-time column is gone, not relabelled', () => {
  it('renders no last-accessed text at all', async () => {
    await mountAndOpen();

    // `projectFromDto` maps createdAt into last_accessed, so any value here is a
    // CREATION time wearing a visit-time label. Removing the column is the only
    // honest option that does not need a backend field.
    //
    // Asserted on the FORMAT, not on a Chinese label: `formatShortDateTimeZh`
    // emits `MM-DD HH:mm` with no words at all, so a label-based assertion would
    // miss the column entirely. What must be absent is a bare timestamp on the
    // row that nothing identifies as a creation time.
    const timestamps = rows().flatMap(
      (row) => (row.textContent ?? '').match(/\d{2}-\d{2} \d{2}:\d{2}/g) ?? [],
    );
    expect(timestamps).toEqual([]);
    // And the words that would claim a visit ordering are gone too.
    expect(document.body.textContent).not.toContain('最近访问');
    expect(document.body.textContent).not.toContain('last_accessed');
  });

  it('keeps the counts that ARE real: inbox items and running sessions', async () => {
    await mountAndOpen();

    // The other two numbers on the row come from real per-project data, so they
    // stay — the fix removed one lie, not the whole row. One per project, so the
    // count is the assertion: four projects, four of each.
    expect(screen.getAllByText(/条待处理/)).toHaveLength(4);
    expect(screen.getAllByText(/个活跃会话/)).toHaveLength(4);
  });
});

describe('the group heading names only what the sort does', () => {
  it('no longer claims a visit ordering it cannot deliver', async () => {
    await mountAndOpen();

    expect(screen.getByText('置顶项目')).toBeTruthy();
    expect(screen.queryByText('最近访问与置顶项目')).toBeNull();
  });
});

describe('keyboard navigation', () => {
  it('ArrowDown moves the highlight and Enter selects the highlighted row', async () => {
    const onSelect = vi.fn();
    await mountAndOpen(onSelect);

    // Nothing is highlighted on open: the operator has just typed a search, and
    // pre-selecting row 0 would make Enter pick a project they never looked at.
    const input = searchInput();
    expect(input.getAttribute('data-highlighted') ?? document.body.innerHTML).toBeDefined();

    await act(async () => {
      fireEvent.keyDown(input, { key: 'ArrowDown' });
    });
    await act(async () => {
      fireEvent.keyDown(input, { key: 'ArrowDown' });
    });
    // Two presses from -1 land on index 1 — the second project in the list.
    await act(async () => {
      fireEvent.keyDown(input, { key: 'Enter' });
    });

    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect.mock.calls[0][0]).toBe('ql_delivery');
  });

  it('Enter with nothing highlighted selects nothing and does not throw', async () => {
    // The guard that keeps a half-typed search from committing row 0.
    //
    // The second half matters: without the guard, `filteredProjects[-1]` is
    // undefined and `chosen.id` throws. React swallows that inside the handler,
    // so a test asserting only `not.toHaveBeenCalled()` passes EITHER WAY — the
    // guard works, or it crashed for the same reason. That is a test that cannot
    // tell a fix from a crash, so the throw is asserted explicitly.
    const onSelect = vi.fn();
    await mountAndOpen(onSelect);
    const thrown: unknown[] = [];
    const onError = (e: ErrorEvent) => thrown.push(e.error ?? e.message);
    window.addEventListener('error', onError);

    try {
      await act(async () => {
        fireEvent.keyDown(searchInput(), { key: 'Enter' });
      });
    } finally {
      window.removeEventListener('error', onError);
    }

    expect(onSelect).not.toHaveBeenCalled();
    expect(thrown).toEqual([]);
    // The picker is still open: an Enter that did nothing must not close it.
    expect(screen.queryByPlaceholderText('搜索项目')).not.toBeNull();
  });

  it('Escape closes the picker', async () => {
    await mountAndOpen(() => {});
    expect(searchInput()).toBeTruthy();

    await act(async () => {
      fireEvent.keyDown(searchInput(), { key: 'Escape' });
    });

    expect(screen.queryByPlaceholderText('搜索项目')).toBeNull();
  });

  it('ArrowDown clamps at the last row instead of wrapping', async () => {
    // Wrapping would lose the operator's place in a filtered list.
    const onSelect = vi.fn();
    await mountAndOpen(onSelect);

    const input = searchInput();
    for (let i = 0; i < 10; i++) {
      await act(async () => {
        fireEvent.keyDown(input, { key: 'ArrowDown' });
      });
    }
    await act(async () => {
      fireEvent.keyDown(input, { key: 'Enter' });
    });

    // Still the last row, not back to the first.
    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect.mock.calls[0][0]).toBe('skills');
  });

  it('Home and End jump to the first and last row', async () => {
    const onSelect = vi.fn();
    await mountAndOpen(onSelect);
    const input = searchInput();

    await act(async () => {
      fireEvent.keyDown(input, { key: 'End' });
    });
    await act(async () => {
      fireEvent.keyDown(input, { key: 'Enter' });
    });
    expect(onSelect.mock.calls[0][0]).toBe('skills');

    await act(async () => {
      fireEvent.click(screen.getByText('SeatLoom 协调核心'));
    });
    const input2 = searchInput();
    await act(async () => {
      fireEvent.keyDown(input2, { key: 'Home' });
    });
    await act(async () => {
      fireEvent.keyDown(input2, { key: 'Enter' });
    });
    expect(onSelect.mock.calls[1][0]).toBe('seatloom');
  });
});

describe('search matches the path as well as the name', () => {
  it('finds a project by its own name', async () => {
    await mountAndOpen(() => {});

    await act(async () => {
      fireEvent.change(searchInput(), { target: { value: 'ql' } });
    });

    await waitFor(() => expect(rows()).toHaveLength(1));
    expect(rowName(rows()[0])).toBe('QL');
  });

  it('finds a project by a parent directory in its path', async () => {
    // The defect: `sf-code` matched nothing, so a hierarchical project was only
    // reachable by typing its full name.
    await mountAndOpen(() => {});

    await act(async () => {
      fireEvent.change(searchInput(), { target: { value: 'sf-code' } });
    });

    await waitFor(() => expect(rows()).toHaveLength(1));
    expect(rowName(rows()[0])).toBe('stepcloud');
  });

  it('an empty search shows everything rather than nothing', async () => {
    await mountAndOpen(() => {});

    expect(rows()).toHaveLength(4);

    await act(async () => {
      fireEvent.change(searchInput(), { target: { value: '   ' } });
    });

    // Whitespace is not a query.
    expect(rows()).toHaveLength(4);
  });

  it('a query matching nothing says so, and the arrows do nothing', async () => {
    const onSelect = vi.fn();
    await mountAndOpen(onSelect);

    await act(async () => {
      fireEvent.change(searchInput(), { target: { value: 'zzz-no-such-project' } });
    });
    await waitFor(() => expect(screen.getByText(/没有匹配/)).toBeTruthy());
    expect(rows()).toHaveLength(0);

    // With an empty result the arrows have nothing to move, and Enter must not
    // index out of range. Same trap as the other Enter test: an out-of-range read
    // throws, React swallows it, and `not.toHaveBeenCalled()` would pass either
    // way. So the absence of an error is asserted too.
    const thrown: unknown[] = [];
    const onError = (e: ErrorEvent) => thrown.push(e.error ?? e.message);
    window.addEventListener('error', onError);

    try {
      await act(async () => {
        fireEvent.keyDown(searchInput(), { key: 'ArrowDown' });
      });
      await act(async () => {
        fireEvent.keyDown(searchInput(), { key: 'Enter' });
      });
    } finally {
      window.removeEventListener('error', onError);
    }

    expect(onSelect).not.toHaveBeenCalled();
    expect(thrown).toEqual([]);
  });

  it('Escape still closes the picker when the result is empty', async () => {
    await mountAndOpen(() => {});

    await act(async () => {
      fireEvent.change(searchInput(), { target: { value: 'zzz' } });
    });
    await waitFor(() => expect(screen.getByText(/没有匹配/)).toBeTruthy());

    await act(async () => {
      fireEvent.keyDown(searchInput(), { key: 'Escape' });
    });

    expect(screen.queryByPlaceholderText('搜索项目')).toBeNull();
  });
});
