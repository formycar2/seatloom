// The status-bar repo link must actually open — same defect as the terminal
// links, different surface.
//
// THE BUG: this used to be `window.open('https://github.com/seatloom')` bound to
// onKeyDown only. Two things were wrong with it at once: Tauri's webview blocks
// window.open by design (so even the keyboard path did nothing), and there was no
// onClick at all (so a mouse click was never wired to anything).
//
// Both are fixed the same way as the terminal links: route through
// `api.openExternalUrl`, which reaches a Rust command that shells out to the
// platform opener.

import '../test/installLocalStorage';
import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { fireEvent, render, screen, cleanup } from '@testing-library/react';
import React from 'react';

vi.mock('../lib/api', () => ({
  api: {
    openExternalUrl: vi.fn(async () => {}),
  },
}));

import StatusBar from './StatusBar';
import { api } from '../lib/api';

const REPO_URL = 'https://github.com/seatloom';

beforeEach(() => {
  vi.mocked(api.openExternalUrl).mockClear();
});

afterEach(() => {
  cleanup();
});

const renderBar = () =>
  render(
    <StatusBar
      onInitProject={() => {}}
      isTerminalOpen={false}
      onTerminalToggle={() => {}}
      onShowHelp={() => {}}
    />,
  );

/** The repo link, found by its accessible role rather than by text. */
const repoLink = () => screen.getByRole('link');

describe('the status-bar repo link', () => {
  it('a mouse click opens the repo URL', async () => {
    // This is the path that was missing entirely — the old element had onKeyDown
    // and no onClick, so clicking it did nothing at all.
    renderBar();
    fireEvent.click(repoLink());
    expect(vi.mocked(api.openExternalUrl).mock.calls.map((c) => c[0])).toContain(REPO_URL);
  });

  it('the Enter key still opens it (the path that existed before)', async () => {
    renderBar();
    fireEvent.keyDown(repoLink(), { key: 'Enter' });
    expect(vi.mocked(api.openExternalUrl).mock.calls.map((c) => c[0])).toContain(REPO_URL);
  });

  it('the Space key opens it too, so the keyboard path is not a single-key trap', async () => {
    // A role="link" is expected to respond to Space as well as Enter; the old
    // handler answered only Enter, and Space on a focused link did nothing.
    renderBar();
    fireEvent.keyDown(repoLink(), { key: ' ' });
    expect(vi.mocked(api.openExternalUrl).mock.calls.map((c) => c[0])).toContain(REPO_URL);
  });

  it('a rejected open is reported rather than swallowed', async () => {
    // Same reason as the terminal links: a click that silently does nothing is
    // the bug being fixed, and a click that logs nothing is its hardest version.
    vi.mocked(api.openExternalUrl).mockRejectedValueOnce(new Error('refused to open'));
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

    renderBar();
    fireEvent.click(repoLink());
    await Promise.resolve();

    expect(consoleError).toHaveBeenCalled();
    const logged = consoleError.mock.calls.flat().join(' ');
    expect(logged).toContain('openExternalUrl failed');
    expect(logged).toContain(REPO_URL);
    consoleError.mockRestore();
  });

  it('the link exposes the URL as its title, so hovering says where it goes', async () => {
    // The visible text is a local path ("~/Documents/GitHub/seatloom"), which is
    // not where the click leads. Without the title the mismatch is invisible.
    renderBar();
    expect(repoLink().getAttribute('title')).toBe(REPO_URL);
  });
});
