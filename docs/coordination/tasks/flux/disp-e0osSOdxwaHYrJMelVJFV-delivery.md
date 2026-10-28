# Delivery: the sandbox banner — PASS verdict, mutation-proven at both layers

| Field | Value |
|---|---|
| dispatch_id | disp-e0osSOdxwaHYrJMelVJFV |
| verified | disp-ZqvAuQ1GANGLx8Y8FLz2T (AEGIS-2026-10-21-sandbox-build-banner-v1) |
| verdict | pass |
| verdict event | ev-qHnyFs2tY8EtTvJcEiDXH (ReviewVerdictIssued, emitted) |
| evidence | docs/coordination/tasks/flux/FLUX-2026-10-21-sandbox-banner-verdict-v1.md |
| branch | packet/flux/intent-verdict @ 40b9cdb |
| tested at | packet/mira/knowledge-panel @ dbb51e7 |

## Verdict: PASS

The packet's six criteria all hold: the sandbox renders a permanent, undismissable strip naming the
command, branch, commit and build time; the main build renders nothing at all; missing values are
per-field `未记录` with no guess; and — the criterion this dispatch exists for — a failed command
says so in red rather than quietly reading as a clean main build.

## What I did

15 throwaway assertions across one harness, testing **both** layers of this two-layer claim: the UI
by rendering the real `SandboxBanner` at every identity state, and the backend by source inspection
plus the project's own Rust tests. Then six mutations to prove the assertions bite — and to avoid
duplicating the env-reading logic in TypeScript (a fixture mirror would prove nothing about Rust):

| mutation | result |
|---|---|
| a throwing command → `{kind:'main'}` | 1 assertion fails |
| an unrecognised payload → `{kind:'main'}` | 2 assertions fail |
| a symmetric "主版" strip (packet forbids one) | 2 assertions fail |
| `未记录` → `'unknown'` | 3 assertions fail |
| banner moved inside `ProvenanceFrame` | 1 assertion fails |
| `is_sandbox` → `.is_some()` instead of `== Some("1")` | **the project's own Rust test fails** |

Every mutation restored byte-identically; Mira's worktree is byte-identical to `dbb51e7` afterwards.

The load-bearing detail worth naming: an **unregistered** Tauri command would surface in the UI as
an identity *failure*, not as a silent no-op, so I checked the registration sits inside
`generate_handler!` rather than merely being imported.

## Gates, run by me in Mira's worktree

`tsc --noEmit` clean · `vitest run` **239/239** (19 files) · `vite build` green · `cargo test` **283
passed** · `cargo clippy --all-targets` **0 warnings** · `cargo fmt --all -- --check` clean.
`NODE_OPTIONS` still points at a deleted `.agentd` loader, so every node invocation ran with only
that stale preload unset.

Lint vs `/private/tmp/lwtM`: `SandboxBanner.tsx` 0 errors/prettier clean, `SandboxBanner.test.tsx`
0/clean, `globals.css` prettier clean, `App.tsx` 15 → 15 (identical, all pre-existing).

## Two findings

**1. The delivery note's gate counts are wrong, both understated.** It claims `vitest` 227/227
(16 files, 9 new) and `cargo test` 168 passed (12 new); measured at `dbb51e7` it is 239/239 (19
files) and cargo 283 — the latter matching Mira's own same-day acceptance-card note at this same
HEAD. The substance is fine (the 5 `runtime_cmds` tests really are this patch's, and the
"9 assertions" figure is right), but a verifier cannot reproduce a gate from a number that does not
match the tree.

**2. The banner reads a per-process state, not a build identity.** This is the mechanism the packet
prescribed (to avoid the four localStorage keys the two builds fight over), so it is not a defect
against the spec — but with two windows open, a main build launched from a shell carrying
`SEATLOOM_SANDBOX=1` would claim to be the sandbox, the wrongly-claimed direction. The exactly-`"1"`
rule narrows it and the UI prints `未记录` if the installer injects nothing, but the solid fix is
build-time injection with the launch-time variable as a hint only. That is an operator decision.

## Boundary

No code, backend, DB, or merge change by me. My one probe file was deleted; `git status` and
`git diff` are empty in Mira's worktree. No app was launched and no acceptance card opened — the
scripts and `/Applications` are the operator's, and the screen is locked from a seat shell
(`screencapture` fails, assistive UI scripting denied).

## Artifact index

- Verdict: `docs/coordination/tasks/flux/FLUX-2026-10-21-sandbox-banner-verdict-v1.md`
- Ledger event: `ev-qHnyFs2tY8EtTvJcEiDXH`
- This delivery note.
