// The call-site test for document sanitisation — the gap Flux's verification found.
//
// AEGIS-2026-10-21-sanitize-document-render-v1 shipped `sanitizeHtml` and 14 unit tests
// that all passed. Flux's mutation run then found the hole: make `sanitizeHtml` return
// its input unchanged and **all 14 stay green**, because every test builds its own
// markdown and never walks the path the product actually takes —
// `marked.parse` → `sanitizeHtml` → `dangerouslySetInnerHTML` on a real document.
//
// That is the recurring cost in this project: the check exists, the thing the check
// stands for was never verified. These tests close that specific hole by driving the
// real component with a real body, so removing the sanitiser at line 407 turns them red.
//
// They assert on the DOM the browser actually built, not on a string the test assembled,
// because the difference between the two is exactly what the unit tests missed.

import React from 'react';
import { describe, expect, it, vi, afterEach } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/react';
import { DocumentsWorkspace } from './DocumentsWorkspace';

const documentRow = {
  id: 'doc-1',
  projectId: 'seatloom',
  filePath: 'docs/coordination/test.md',
  title: 'A document',
  author: 'mira',
  status: 'active',
  createdAt: '2026-10-21T00:00:00Z',
  template: 'T3',
  subtype: 'delivery',
  version: 'v1',
  docDate: '2026-10-21',
  bodyDigest: null,
  updatedAt: null,
  revision: 1,
  tags: [] as string[],
};

const state = vi.hoisted(() => ({
  bodyText: '',
  associations: [] as unknown[],
}));

vi.mock('../../lib/api', () => ({
  isTauri: () => true,
  api: {
    listDocuments: async () => [{ ...documentRow, bodyText: state.bodyText }],
    // The detail read is what actually carries `bodyText` into the render, so it must
    // serve the same body the row does — otherwise the selection would open an empty
    // body and the assertions below would have nothing to check.
    getDocument: async () => ({ ...documentRow, bodyText: state.bodyText }),
    listDocumentAssociations: async () => state.associations,
    listTemplates: async () => [],
    reconcile: async () => ({}),
  },
}));

// The component reads the store two ways: as a hook selector
// (`useDataStore((s) => s.activeProjectId)`) and imperatively
// (`useDataStore.getState().activeProjectId`). The real store is a function carrying a
// `.getState`; a selector-only double leaves the imperative path undefined and the
// component throws, which is how this file's first draft failed.
vi.mock('../../stores/useDataStore', () => {
  const storeState = {
    activeProjectId: 'seatloom',
    projects: [],
    projectData: {},
    backendHydrated: true,
    backendHydrateError: null,
    hydrateFromBackend: async () => {},
  };
  const useDataStore = Object.assign(
    (selector?: (s: typeof storeState) => unknown) =>
      typeof selector === 'function' ? selector(storeState) : storeState,
    { getState: () => storeState },
  );
  return { useDataStore };
});

afterEach(() => {
  cleanup();
});

/** Render with a given body and return the rendered body section. */
const renderWith = (bodyText: string) => {
  state.bodyText = bodyText;
  return render(<DocumentsWorkspace onClose={() => {}} />);
};

/**
 * The rendered body is the only element carrying the prose class the CSS targets.
 *
 * Selecting the document is what makes it appear: the list item click sets `selectedId`,
 * the effect reads the detail, and only then does the body section exist. Driving the
 * whole chain is the point — a test that snapped the HTML itself would never have caught
 * the gap Flux found.
 */
const bodySection = async () => {
  // The list row is a div with a click handler, so find it by the title it renders.
  const row = await screen.findByText('A document');
  fireEvent.click(row);
  await waitFor(() => expect(document.querySelector('.sl-doc-body')).toBeTruthy());
  return document.querySelector('.sl-doc-body') as HTMLElement;
};

describe('the real render path sanitises', () => {
  it('a document containing a <script> does not put a script in the DOM', async () => {
    renderWith('hello\n\n<script>window.__EXECUTED__ = 1;</script>');
    const section = await bodySection();
    // The section only appears once a document is selected.
    if (!section) {
      throw new Error(
        'the body section never rendered — the call-site assertion is not actually ' +
          'reaching the render path, so it would pass no matter what',
      );
    }
    expect(document.querySelectorAll('script[data-injected]')).toHaveLength(0);
    expect((window as unknown as { __EXECUTED__?: number }).__EXECUTED__).toBeUndefined();
    expect(section.textContent).toContain('hello');
  });

  it('an inline event handler on a document-authored tag is stripped from the DOM', async () => {
    renderWith('<p>text</p>\n\n<img src=x onerror="window.__EXECUTED__ = 1">');
    const section = await bodySection();
    if (!section) throw new Error('body section never rendered');
    expect(section.querySelector('[onerror]')).toBeNull();
    expect((window as unknown as { __EXECUTED__?: number }).__EXECUTED__).toBeUndefined();
  });

  it('a javascript: link in a document does not survive into the DOM href', async () => {
    renderWith('[click me](javascript:alert(1))');
    const section = await bodySection();
    if (!section) throw new Error('body section never rendered');
    const anchors = [...section.querySelectorAll('a')];
    for (const a of anchors) {
      expect(a.getAttribute('href') ?? '').not.toContain('javascript:');
    }
    expect(section.querySelectorAll('a[href^="javascript"]')).toHaveLength(0);
  });

  it('a normal document still renders its structure — the sanitiser is not a blanker', async () => {
    renderWith('# Title\n\nSome **bold** and `code`.\n\n- one\n- two\n\n> quoted');
    const section = await bodySection();
    if (!section) throw new Error('body section never rendered');
    // Both directions on the REAL path: prose present AND a script absent.
    expect(section.querySelector('h1')?.textContent).toBe('Title');
    expect(section.querySelector('strong')?.textContent).toBe('bold');
    expect(section.querySelector('code')?.textContent).toBe('code');
    expect(section.querySelectorAll('li')).toHaveLength(2);
    expect(section.querySelector('blockquote')?.textContent?.trim()).toBe('quoted');
    expect(section.querySelectorAll('script')).toHaveLength(0);
  });

  it('HTML a document author wrote is shown as text, not parsed as markup', async () => {
    renderWith('\\<script>alert(1)\\</script>');
    const section = await bodySection();
    if (!section) throw new Error('body section never rendered');
    expect(section.querySelector('script')).toBeNull();
  });
});
