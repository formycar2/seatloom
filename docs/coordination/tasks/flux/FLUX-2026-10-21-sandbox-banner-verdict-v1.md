# Verdict — the sandbox banner: PASS, on all six criteria, mutation-proven at both layers

- **Verification dispatch**: `disp-e0osSOdxwaHYrJMelVJFV`.
- **Implementation dispatch**: **`disp-ZqvAuQ1GANGLx8Y8FLz2T`** (`verify_dispatch_id` in the payload);
  packet `AEGIS-2026-10-21-sandbox-build-banner-v1`.
- **Branch under test**: `packet/mira/knowledge-panel` @ `dbb51e7` (HEAD; the patch landed in `e879122`).
- **Verdict event**: `ev-qHnyFs2tY8EtTvJcEiDXH` — emitted as required.
- **VERDICT: PASS** — with **two recorded findings**: one count in the delivery note is wrong, and
  one real gap (the banner reads a *state*, not an *identity* — see finding 2).

## Scope

One dispatch, its own verdict. Nothing carried over from `disp-DFeH6Ut_aNfzIbuuR21V5` or
`disp-RWEf5ATPvICLkGtF__fEY`.

## Method — 15 throwaway assertions, both layers, then mutations

The claim this dispatch makes is a **two-layer** one (a backend command + a UI reaction), so my
probe tested both: the UI half by rendering the real `SandboxBanner` at every identity state, and
the backend half by source inspection plus the project's own Rust tests. Then six mutations:

| mutation | result |
|---|---|
| a throwing command → `setState({kind:'main'})` (failure reads as clean) | **1 assertion FAILS** |
| an unrecognised payload → `{kind:'main'}` | **2 assertions FAIL** |
| a symmetric "主版 · 这是正式构建" strip (the packet forbids one) | **2 assertions FAIL** |
| `未记录` → `'unknown'` | **3 assertions FAIL** |
| the banner moved inside `ProvenanceFrame` (per-tab, re-created by nav) | **1 assertion FAIL** |
| `is_sandbox` → `.is_some()` instead of `== Some("1")` | **project's own Rust test FAILS** — `"0" must not read as sandbox` |

All mutations restored byte-identically; `git status`/`git diff` empty afterwards.

## The packet's criteria, each measured

**"这是沙箱还是主版"** ✔ The sandbox renders a strip naming the command it read, the branch
(`packet/mira/knowledge-panel`), the commit and the build time. `is_sandbox` is **exactly `"1"`** —
and I verified the near-miss case at both layers: the Rust side rejects `""`, `"0"`, `"true"`,
`"yes"`, `"01"`, `" 1"`, `"1 "` (mutation-proven above), and the UI side renders nothing for a
non-sandbox identity, which I asserted directly.

**"装的是哪个分支、哪个提交 · 何时构建"** ✔ All three fields rendered from the DTO; each
independently replaceable with `未记录`.

**"缺哪项就写对应项未记录,不显示猜测值"** ✔ Per-field, not all-or-nothing: `commit: null` still
renders the real branch. Empty and whitespace strings are `未记录` too (the Rust `env_value` drops
them, and the UI's `orUnrecorded` re-checks). No `null`, no `undefined`, no invented word.

**"是主版时不渲染这条"** ✔ `container.innerHTML === ''` — literally nothing, and specifically no
symmetric strip. Adding one kills 2 assertions.

**"关闭后端命令时,前端要能说出失败,不许静默当成主版"** ✔ This is the criterion the dispatch exists
for, and it holds: a throwing command renders a red strip
(`身份读取失败:<message> —— 这份构建是沙箱还是主版,此刻无法确认`), an unrecognised payload renders
the same failure state rather than being read as the main build, and a `null` payload does too.
Mutating either branch to `{kind:'main'}` kills assertions — so the failure is load-bearing.

**"常驻 / 不能被关掉"** ✔ Mounted once in `App.tsx` **above** `ProvenanceFrame`, so it is outside the
tab-branched content and navigation cannot re-create or drop it. No button, no dismiss control, no
`aria-label` close affordance; `role="status"` present. Moving it inside the frame kills 1 assertion.

**Identity must not come from localStorage** ✔ The component's code (comments stripped — the header
*names* the localStorage problem it avoids) reads neither `localStorage` nor `sessionStorage`; it
reaches `cmd_get_runtime_identity`, whose registered-ness I verified is inside `generate_handler!`
rather than merely imported. An unregistered command would surface as an identity *failure*, so
registration is part of the claim.

## Gates, run by me in Mira's worktree

`tsc --noEmit` clean · `vitest run` **239/239** (19 files) · `vite build` green · `cargo test` **283
passed** · `cargo clippy --all-targets` **0 warnings** · `cargo fmt --all -- --check` clean.
`NODE_OPTIONS` still points at a deleted `.agentd` loader, so every node invocation ran with only
that stale preload unset.

Lint on the four changed files: `SandboxBanner.tsx` 0 errors / prettier clean;
`SandboxBanner.test.tsx` 0 errors / prettier clean; `globals.css` prettier clean; `App.tsx`
**15 → 15** against `/private/tmp/lwtM` (identical count, all pre-existing).

## Findings

**1. The delivery note's gate counts are wrong — three of them, all in the same direction.**

| claimed | measured at `dbb51e7` |
|---|---|
| `cargo test` 168 passed (12 new) | **283 passed** |
| `vitest run` 227/227, 16 files | **239/239**, 19 files |
| "9 assertions" in `SandboxBanner.test.tsx` | 9 `it()` blocks — **this one is right** |

The 5 Rust tests inside `runtime_cmds` are real and are the ones added by this patch, so the
*substance* of the gate claim is fine; the *numbers* look like they were taken from a different,
earlier run. Both wrong figures are **understated**, so the note is honest in the direction that
matters — but a verifier cannot reproduce a gate from a number that does not match the tree, and
the 283/283 figure in Mira's own same-day acceptance-card note at this same HEAD is the correct
one. Recorded so the record contains the reproducible value.

**2. The banner reports a *state*, not an *identity* — and the state is per-process, which is the
wrong scope for the question.**

The packet asked "这是沙箱还是主版" of the **build**, and the mechanism answers it of the
**process**, from the installer's environment variables. That is not a defect in what was asked for
(the packet prescribed exactly this, and the design is otherwise sound). It becomes one in the case
the packet itself names as the constraint: two windows open at once. Then:

- the sandbox window launched with `SEATLOOM_SANDBOX=1` shows the strip ✔
- the main build, correctly, shows nothing ✔
- **but a main build launched from a shell that happens to carry the variable would claim to be the
  sandbox** — a *wrongly-claimed* identity, which is the failure direction Mira's own comment says
  is worse (and which the exactly-`"1"` rule is designed to narrow).

This is inherent to an env-var mechanism and the packet chose it deliberately (avoiding localhost
storage the two builds fight over). I record it as a known boundary, not as a defect to route
around: the correct fix is to have the installer inject the values **at build time** into the
bundle (which is what `built_at` already does and what `branch`/`commit` are described as) and to
read the launch-time variable only as a *hint*. That is a design question for the operator, not
something a seat should decide silently.

## Honest limits

- **No app was launched, so no real strip was seen.** The scripts and `/Applications` are the
  operator's, and the screen is locked from a seat shell (`screencapture` fails, assistive UI
  scripting denied). The end-to-end env→command→UI path is verified by the project's Rust tests
  plus my UI assertions against real DTO shapes; the `open -a` propagation probe is Mira's, quoted
  in her note, and I did not re-run it.
- **I did not verify the `SEATLOOM_SANDBOX_BRANCH`-injection half**, because that work is Aegis's
  (`prepare-sandbox-build.sh`) and the packet explicitly scopes it out. If the installer does not
  inject those variables, three of four fields will read `未记录` — which the UI handles honestly,
  and which is visible to the operator the moment he opens a sandbox.
- **My probe's backend half is source inspection + the project's tests, not an independent
  reimplementation.** I chose not to duplicate the env-reading logic in TypeScript, because a
  fixture mirror would prove nothing about the Rust; mutation 6 above is what makes that boundary
  load-bearing rather than asserted.

## Bottom line

**PASS.** The sandbox renders a permanent, undismissable strip naming the command, branch, commit
and build time; the main build renders nothing at all; missing values are per-field `未记录` with no
guess; and — the criterion this dispatch exists for — a failed command says so in red rather than
quietly reading as a clean main build. Six mutations confirm each of those, and the project's own
tests independently catch a loosened `is_sandbox`. Gates all green at `dbb51e7`. Two things for the
record: the delivery note's two gate counts are wrong (understated: 239/239 and 283, not 227 and
168), and the identity is a per-process state rather than a build fact, so two simultaneously-open
windows is the one scenario where the answer depends on how each was launched rather than on what
it is.
