// SandboxBanner — the second layer of "which build am I looking at".
//
// The operator's ruling (AEGIS-2026-10-21-sandbox-build-banner-v1): he should be able to
// tell which build he is looking at without leaving the window or alt-tabbing. The
// acceptance criteria, and therefore this file's assertions, are two-sided:
//
//   1. the sandbox shows the strip, the main build shows nothing, and a backend
//      failure says so out loud instead of silently reading as "main build"
//   2. every value on the strip is either the recorded fact or "未记录" — never a guess
//
// The second flank matters as much as the first: a banner that renders on the main build,
// or that invents a branch name, is the failure this whole component exists to prevent.

import React from 'react';
import { describe, expect, it, vi, afterEach } from 'vitest';
import { render, screen, waitFor, cleanup } from '@testing-library/react';
import { SandboxBanner } from './SandboxBanner';

// The banner's only dependency is the Tauri command. Mock it per-test so each case
// can state exactly what the backend said.
const invokeMock = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

afterEach(() => {
  cleanup();
  invokeMock.mockReset();
});

const SANDBOX_IDENTITY = {
  is_sandbox: true,
  branch: 'packet/mira/knowledge-panel',
  commit: 'abc1234',
  built_at: '2026-10-21 13:20',
};

describe('when the backend says this is the sandbox', () => {
  it('renders a strip naming the branch, the commit and the build time', async () => {
    invokeMock.mockResolvedValue(SANDBOX_IDENTITY);
    render(<SandboxBanner />);

    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const strip = screen.getByRole('status');
    expect(strip.textContent).toContain('沙箱');
    expect(strip.textContent).toContain('packet/mira/knowledge-panel');
    expect(strip.textContent).toContain('abc1234');
    expect(strip.textContent).toContain('2026-10-21 13:20');
  });

  it('states which command it read, so a wrong build can be traced', async () => {
    invokeMock.mockResolvedValue(SANDBOX_IDENTITY);
    render(<SandboxBanner />);
    await waitFor(() => expect(invokeMock).toHaveBeenCalled());
    expect(invokeMock).toHaveBeenCalledWith('cmd_get_runtime_identity');
  });
});

describe('when the backend says this is the main build', () => {
  it('renders nothing at all — no symmetric "I am the main build" strip', async () => {
    invokeMock.mockResolvedValue({
      is_sandbox: false,
      branch: null,
      commit: null,
      built_at: null,
    });
    const { container } = render(<SandboxBanner />);
    await waitFor(() => expect(invokeMock).toHaveBeenCalled());
    expect(container.innerHTML).toBe('');
    expect(screen.queryByRole('status')).toBeNull();
    expect(screen.queryByText(/沙箱/)).toBeNull();
  });
});

describe('when a value is missing', () => {
  it('says 未记录 for that one field and still names the others', async () => {
    invokeMock.mockResolvedValue({
      is_sandbox: true,
      branch: 'packet/mira/knowledge-panel',
      commit: null,
      built_at: '',
    });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('packet/mira/knowledge-panel');
    // Two absences stated, and neither is a blank or an invented value.
    expect(text.match(/未记录/g) ?? []).toHaveLength(2);
    expect(text).not.toContain('null');
    expect(text).not.toContain('undefined');
  });

  it('never substitutes a plausible-looking value for the real one', async () => {
    invokeMock.mockResolvedValue({
      is_sandbox: true,
      branch: null,
      commit: null,
      built_at: null,
    });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text.match(/未记录/g) ?? []).toHaveLength(3);
    // The words that would betray a guess.
    expect(text).not.toContain('unknown');
    expect(text).not.toContain('main');
  });
});

describe('when the command fails', () => {
  it('says it failed rather than silently rendering as the main build', async () => {
    invokeMock.mockRejectedValue(new Error('command not allowed'));
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByText(/身份读取失败/)).toBeTruthy());
    // The operator can tell this from a main build, which renders nothing.
    expect(screen.getByText(/沙箱还是主版/)).toBeTruthy();
  });

  it('treats an unrecognisable payload as a failure, not as the main build', async () => {
    // A build where the command exists but returns something else (an older backend,
    // a renamed struct) must not read as "this is the main build".
    invokeMock.mockResolvedValue({ is_sandbox: 'yes' });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByText(/身份读取失败/)).toBeTruthy());
  });

  it('does not render the sandbox strip when the command fails', async () => {
    invokeMock.mockRejectedValue(new Error('boom'));
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByText(/身份读取失败/)).toBeTruthy());
    expect(screen.queryByRole('status')).toBeNull();
  });
});

describe('the strip is an identity, not a notice', () => {
  it('offers no way to dismiss it', async () => {
    invokeMock.mockResolvedValue(SANDBOX_IDENTITY);
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const buttons = screen.queryAllByRole('button');
    expect(buttons).toHaveLength(0);
  });
});

// The banner also needs the Tauri guard to pass in a non-Tauri test environment; the
// component itself only reaches the command through the mock, but asserting the guard's
// shape here keeps this file self-describing rather than depending on jsdom globals.
