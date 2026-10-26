// SandboxBanner — the second layer of "which build am I looking at".
//
// The first layer is the app icon (Aegis's work: an orange frame plus an orange corner
// block), which solves the Dock and Cmd-Tab case. It does NOT solve this one: with the
// window open, the insides of the sandbox and the main build look identical, and the
// icon is not visible inside the window. The operator asked for a permanent in-window
// identifier (AEGIS-2026-10-21-sandbox-build-banner-v1).
//
// WHY A BACKEND CALL AND NOT A STORED FLAG. The knowledge note
// `docs/coordination/knowledge/ui_cannot_launch_app_yet_and_two_holes_must_not_stack.md`
// records that of the four localStorage keys, three have no namespace, so the sandbox
// and the main build share state when both are open and overwrite each other. Anything
// this banner says must therefore come from outside the webview's own storage: the
// backend command `cmd_get_runtime_identity` reads four environment variables that
// Aegis's installer injects at launch.
//
// 🔴 MEASURED, not assumed (2026-10-21): I launched a throwaway .app bundle the way
// `prepare-sandbox-build.sh` does and printed what it saw. `open -a` with the variables
// exported in the launching shell **does** reach the application. So the installer's
// mechanism works and nothing there needed changing. (`launchctl setenv` + `open -a`
// does not propagate, which is why that alternative was rejected — see the backend file.)
//
// WHAT IT DOES NOT DO. The main build renders nothing at all here — no symmetric
// "I am the main build" strip, which the packet explicitly rules out. And the banner has
// no dismiss button: it is an identity, not a notice, so the operator cannot turn it off.

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** Mirrors `RuntimeIdentityDto` in `src-tauri/src/commands/runtime_cmds.rs`. */
export type RuntimeIdentityDto = {
  is_sandbox: boolean;
  branch: string | null;
  commit: string | null;
  built_at: string | null;
};

/**
 * Read the runtime identity.
 *
 * A module seam, not a second implementation: `lib/api.ts` is Nimbus's fenced file and
 * does not yet carry `getRuntimeIdentity`, so this calls the same Tauri command
 * directly. When his additive edit lands, replace this with `api.getRuntimeIdentity()`
 * — one call site, one seam.
 *
 * 规范入口是 `lib/api.ts`,但该文件在 Nimbus 的文件围栏内,本席位不许编辑。所以这里
 * 放一个同形的本地缝,等他的只增补落地后整块换成 `api.*` 即可。
 */
async function getRuntimeIdentity(): Promise<RuntimeIdentityDto> {
  return invoke<RuntimeIdentityDto>('cmd_get_runtime_identity');
}

/** Render a missing value as a stated absence, never as a guess. */
function orUnrecorded(value: string | null | undefined): string {
  return value && value.trim() !== '' ? value : '未记录';
}

type LoadState =
  | { kind: 'loading' }
  /** The command succeeded and this is the main build: nothing is rendered. */
  | { kind: 'main' }
  | { kind: 'sandbox'; identity: RuntimeIdentityDto }
  /**
   * The command failed. This is deliberately NOT treated as "this is the main build":
   * a banner missing because the backend could not answer is indistinguishable from a
   * main build that correctly shows nothing, and that is the exact confusion this
   * banner exists to remove. The packet requires the failure be said out loud.
   */
  | { kind: 'error'; message: string };

export const SandboxBanner: React.FC = () => {
  const [state, setState] = useState<LoadState>({ kind: 'loading' });

  useEffect(() => {
    let disposed = false;
    getRuntimeIdentity()
      .then((id) => {
        if (disposed) return;
        // A malformed payload must not be read as "main" — say the failure instead.
        if (!id || typeof id.is_sandbox !== 'boolean') {
          setState({ kind: 'error', message: '命令返回了无法识别的数据' });
          return;
        }
        setState(id.is_sandbox ? { kind: 'sandbox', identity: id } : { kind: 'main' });
      })
      .catch((e) => {
        if (disposed) return;
        setState({ kind: 'error', message: e instanceof Error ? e.message : String(e) });
      });
    return () => {
      disposed = true;
    };
  }, []);

  if (state.kind === 'loading' || state.kind === 'main') return null;

  if (state.kind === 'error') {
    return (
      <div
        className="sl-sandbox-banner sl-sandbox-banner--error"
        title="无法读取运行时身份:这可能表示构建损坏,也可能表示你在看主版 —— 现在两者无法区分。"
      >
        身份读取失败:{state.message} —— 这份构建是沙箱还是主版,此刻无法确认。
      </div>
    );
  }

  const { identity } = state;
  return (
    <div className="sl-sandbox-banner" role="status" aria-label="这是沙箱构建">
      沙箱 · 分支 {orUnrecorded(identity.branch)} · 提交 {orUnrecorded(identity.commit)} · 构建于{' '}
      {orUnrecorded(identity.built_at)}
    </div>
  );
};
