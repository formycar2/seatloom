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
  /**
   * How many uncommitted changes this build carries, or null when the installer never
   * said. `0` means it checked and the tree was clean — which is a different fact from
   * "unknown", and the two must not render the same way.
   *
   * 🔴 2026-11-04 补。`prepare-sandbox-build.sh --allow-dirty` 会明确地从带未提交
   *    改动的工作区构建,那种构建不等于上面那个提交。这个字段是唯一能让操作者
   *    在看的那块屏幕上看出来的途径 —— Lyra 量到它此前有 0 个消费者。
   */
  uncommitted: number | null;
  /**
   * The commit **this binary** was compiled from, captured at build time.
   *
   * 🔴 2026-11-07 补。`commit` above is what the INSTALLER said (a launcher env var);
   * this is what the binary itself can state. Before this field, `strings` on the
   * sandbox binary found the four env var names and six hex-looking strings, none of
   * which was a hash of any commit on the branch — so nothing could answer "what are
   * you?", and the positive case of an acceptance was unfalsifiable.
   */
  built_commit: string | null;
  /**
   * How many uncommitted changes to TRACKED files the tree had when this binary was
   * built — stated by the BINARY, not by the installer.
   *
   * 🔴 GAP fix (Flux, ev-OFHL9WBqlQnAqs8DCGAkN). The banner used to carry `uncommitted`
   * from the launcher's env var, which is the channel this whole change exists to stop
   * trusting — so a dirty build and a clean build at the same commit were
   * indistinguishable by the binary's own account. Three states, not a boolean:
   * `0` = the capture looked and it was clean; `n` = n tracked files were modified;
   * `null` = the capture could not tell. Rendering `null` as clean would let an
   * unreadable state pass as verified.
   */
  built_dirty: number | null;
  /**
   * Whether git answered at all during the capture. Separate from the fields above
   * being null, because "git is unavailable here" is a different fact from "the
   * repository had no answer", and the operator needs to know which one it is.
   */
  built_git_ok: boolean;
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

/**
 * The uncommitted-change clause — the reason this field exists.
 *
 * Three states, deliberately not collapsed into a boolean:
 *   - a count above zero  → say it loudly, and say what it means: this build is not the
 *     commit printed above it. That is the whole point of the field.
 *   - exactly zero        → the installer checked and it was clean. Stating that is
 *     what makes "0" and "did not say" distinguishable on screen.
 *   - absent              → "未记录". A build this installer did not make must not be
 *     able to read as clean by default.
 *
 * coercing an unreadable value to 0 would render as "no uncommitted changes", which is a
 * claim about the code rather than about our ability to read it.
 */
function uncommittedClause(value: number | null | undefined): string {
  if (typeof value !== 'number' || !Number.isFinite(value)) return ' · 未提交改动 未记录';
  if (value <= 0) return ' · 无未提交改动';
  return ` · 带 ${value} 个未提交改动 —— 这份构建不等于上面那个提交`;
}

/**
 * The build-embedded commit, and what to do when it disagrees with the installer's.
 *
 * 🔴 这个条款是这次补字段的全部理由。三件事必须分开:
 *   二进制自己说的提交 / 安装器说的提交 / 卡上写的提交。只有三者一致,这个点击
 *   验的才是它要验的东西。
 *
 * 一致时照常显示。不一致时**必须响**:一个说自己是一个提交、实际是另一个提交的
 * 沙箱,比没有横幅更糟 —— 它会把一个错的验收显示成对的。所以这里写成一条显眼的
 * 警告,并且不静默选一个值显示。
 *
 * `built_commit` 缺席(不是空串,是没编译进去)时说「未记录」,不猜。
 */
function identityClause(identity: RuntimeIdentityDto): string {
  const built = identity.built_commit?.trim();
  const claimed = identity.commit?.trim();
  if (identity.built_git_ok === false) {
    return ' · ⚠️ 编译时读不到 git —— 这份构建无法自证身份';
  }
  if (!built) return ' · 二进制内提交 未记录';
  if (!claimed) return ` · 二进制内提交 ${built}(安装器未说)`;
  if (claimed === built) return ` · 二进制内提交 ${built}(与安装器一致)`;
  return ` · ⚠️ 二进制内提交 ${built} ≠ 安装器说的 ${claimed} —— 这个点击验的可能不是它要验的东西`;
}

/**
 * The dirty clause, from the BINARY rather than the installer.
 *
 * 🔴 This is the other half of Flux's finding. The old clause read `uncommitted`, which
 * the launcher exported — so the banner was trusting the exact channel the identity work
 * exists to replace. A build made from a dirty tree now says so in the binary's own
 * voice, and the three states stay distinct on screen: a capture that looked and found
 * nothing, a capture that found changes, and a capture that could not tell.
 */
function builtDirtyClause(identity: RuntimeIdentityDto): string {
  if (identity.built_git_ok === false) return ' · 编译时脏状态 未记录';
  const n = identity.built_dirty;
  if (typeof n !== 'number' || !Number.isFinite(n)) return ' · 编译时脏状态 未记录';
  if (n <= 0) return ' · 编译时无未提交改动';
  return ` · 编译时带 ${n} 个未提交改动 —— 这份构建不等于上面那个提交`;
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
      {uncommittedClause(identity.uncommitted)}
      {identityClause(identity)}
      {builtDirtyClause(identity)}
    </div>
  );
};
