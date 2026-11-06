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
  // Every fixture below carries this field so that dropping it from the DTO or from the
  // render cannot pass silently. 🔴 2026-11-04: the fixtures previously mirrored a
  // four-field DTO exactly, which is why nothing failed when the installer exported a
  // fifth fact that no code read.
  uncommitted: 0,
  // 🔴 2026-11-07: the same trap again, third field in. `commit` is what the
  // INSTALLER claimed; `built_commit` is what the BINARY can state. Measured before
  // this field existed: `strings` on the sandbox binary found four env var NAMES and
  // six hex-looking strings, none a hash of any commit on the branch — so nothing could
  // answer "what are you?", and the positive case of an acceptance was unfalsifiable.
  built_commit: 'abc1234',
};

/** The installer's claim and the binary's own statement, agreeing. */
const AGREEING = { ...SANDBOX_IDENTITY };

/** The binary states a different commit than the installer claimed. */
const DISAGREEING = { ...SANDBOX_IDENTITY, commit: 'old9999' };

/** The build did not capture a commit (git unavailable, or not a repository). */
const NO_BUILT_COMMIT = { ...SANDBOX_IDENTITY, built_commit: null };

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
    // A clean build says so, which is what makes "0" and "not recorded" different things.
    expect(strip.textContent).toContain('无未提交改动');
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
      uncommitted: 0,
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
      uncommitted: 0,
      // Present, so the count below is exactly the two absences this case is about:
      // the installer's commit and the build time. Without this the fixture would
      // also be missing the build-embedded commit and the count would be 3 — which
      // tests a different thing than the name of this case says.
      built_commit: 'abc1234',
      // Absent on purpose: the dirty count is a THIRD state, not a number. It must
      // be stated as unrecorded rather than read as 0 (clean) — that is the whole
      // reason it is Option<u32> and not u32. So this case is about 2+1 absences.
      built_dirty: null,
    });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('packet/mira/knowledge-panel');
    // Three absences stated, and neither is a blank or an invented value: the
    // installer's commit, the build time, and the dirty count. The build-embedded
    // commit is present, so it is named rather than counted as absent.
    expect(text.match(/未记录/g) ?? []).toHaveLength(3);
    expect(text).not.toContain('null');
    expect(text).not.toContain('undefined');
  });

  it('never substitutes a plausible-looking value for the real one', async () => {
    invokeMock.mockResolvedValue({
      is_sandbox: true,
      branch: null,
      commit: null,
      built_at: null,
      uncommitted: 0,
    });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    // 5, not 4: branch, commit, built_at, the build-embedded commit, and the
    // dirty count. Each absent fact is STATED as absent — that is the property.
    // (The first version of this assertion said 4 and enumerated four fields while
    // a fifth was already rendering: the comment counted the fields I had written
    // down, not the ones on screen. Measured, not reasoned — hence the count above.)
    // A future field that can be absent bumps this count, and this comment is where
    // that is noticed.
    expect(text.match(/未记录/g) ?? []).toHaveLength(5);
    // And the third state is stated in its own words, not as a number and not as
    // clean. This is the assertion that an unknown is not a measured zero.
    expect(text).toContain('未记录');
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

describe('when the build carries uncommitted changes', () => {
  // 🔴 2026-11-04. These are the tests that make the field load-bearing. The DTO field,
  // the installer export and the render clause were each added at a different time and
  // none of them had a consumer-side assertion, so a `--allow-dirty` sandbox was
  // indistinguishable from a clean one in the application the operator reads while
  // deciding whether to click Accept. Delete the render clause and these fail.
  it('says out loud that this build is not the commit printed above it', async () => {
    invokeMock.mockResolvedValue({ ...SANDBOX_IDENTITY, uncommitted: 2 });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('2 个未提交改动');
    // The clause must state the consequence, not merely the count.
    expect(text).toContain('不等于上面那个提交');
  });

  it('does not let a large count pass as a small one', async () => {
    invokeMock.mockResolvedValue({ ...SANDBOX_IDENTITY, uncommitted: 124 });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('124 个未提交改动');
    expect(text).not.toContain('2 个未提交改动');
  });

  it('states "not recorded" rather than reading a missing count as clean', async () => {
    // A build this installer did not make: the variable was never exported. It must not
    // render as "no uncommitted changes", because that is a claim about the code.
    invokeMock.mockResolvedValue({ ...SANDBOX_IDENTITY, uncommitted: null });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('未提交改动 未记录');
    expect(text).not.toContain('无未提交改动');
  });

  it('states the commit the binary itself was built from', async () => {
    // The whole point of the field: the binary can now answer "what are you?", so the
    // operator can compare it against the commit the card names.
    invokeMock.mockResolvedValue(AGREEING);
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('二进制内提交 abc1234');
    expect(text).toContain('与安装器一致');
  });

  it('alerts when the binary disagrees with the installer', async () => {
    // A sandbox that claims one commit and is another is worse than no banner: it
    // renders a wrong acceptance as a right one. So this must be loud, and it must not
    // silently pick one of the two values to show.
    invokeMock.mockResolvedValue(DISAGREEING);
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('abc1234');
    expect(text).toContain('old9999');
    expect(text).toContain('⚠️');
  });

  it('says unrecorded when the build captured no commit, and does not read that as agreement', async () => {
    // Absent is not "same as the installer". A build whose capture failed must not be
    // able to read as verified-by-identity.
    invokeMock.mockResolvedValue(NO_BUILT_COMMIT);
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('二进制内提交 未记录');
    expect(text).not.toContain('与安装器一致');
  });

  it('does not read an unparseable count as zero', async () => {
    // The backend reports a parse failure as absent; the frontend must not then coerce
    // it to a clean build. "abc" is not "nothing uncommitted".
    invokeMock.mockResolvedValue({ ...SANDBOX_IDENTITY, uncommitted: 'abc' as unknown as number });
    render(<SandboxBanner />);
    await waitFor(() => expect(screen.getByRole('status')).toBeTruthy());
    const text = screen.getByRole('status').textContent ?? '';
    expect(text).toContain('未提交改动 未记录');
    expect(text).not.toContain('无未提交改动');
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
