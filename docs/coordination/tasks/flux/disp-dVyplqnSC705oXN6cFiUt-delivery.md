# Delivery: mira's sync to track — FAIL, behind 1 and reported as 0

| Field | Value |
|---|---|
| dispatch_id | disp-dVyplqnSC705oXN6cFiUt |
| verified | disp-bBdpBqTT9g5-kKU28xvIt (AEGIS-2026-10-21-mira-sync-to-track-v1) |
| verdict | **fail** |
| verdict event | ev--WWsdegFX0cmXC8WvQaQm (ReviewVerdictIssued, emitted) |
| evidence | docs/coordination/tasks/flux/FLUX-2026-10-21-mira-sync-to-track-verdict-v1.md |
| branch | packet/flux/intent-verdict @ 69e64ac |
| tested at | packet/mira/knowledge-panel @ dbb51e7 |

## Verdict: FAIL — on the packet's own hard criterion

`git rev-list --count packet/mira/knowledge-panel..track/infra-foundation` prints **1**. The packet
and the delivery note both say it prints **0**. The count was never 0:

| measured at | behind-track |
|---|---|
| `67fd267` (the sync merge itself) | 3 |
| `1f8c01b` (the head the note names) | 2 |
| `dbb51e7` (current head) | **1** |

The one outstanding commit is Aegis's own `6c90709` — `scripts/record-appv2-deletion-ruling.sh`,
147 lines, nothing under `ui/src` or `src-tauri/src` — authored on track at **16:00**, two hours
**after** the sync merge landed at 14:05. So the note cannot have been measuring it.

The favourable direction is what makes this easy to miss: no behaviour changes, so the
operator-visible goal (project filtering in the seat list) is fully met, `boundMirrorNames` is
present, and everything else is green. But the criterion is a number, and merging this branch would
still not carry track's head — the exact situation the packet was raised to prevent.

## What is genuinely good, and verified

**The merge resolution is the strongest part of the note.** Track's scoped `hydrateProjectData`
won, **including the line she dropped and restored** — `set({ projects, activeProjectId: nextActive })`
— without which hydration published zero projects and reported no error. Her three areas survived:
`sanitizeHtml(html)` at the call site, the timeline filter (9 occurrences), `SandboxBanner.tsx`,
`runtime_cmds.rs`. And `api.ts`'s `listDocuments` really does take an options object, not a
positional id.

**The dropped line is the best thing in the note.** A store that hydrated zero projects with no
error is exactly the silent failure this project keeps paying for, and her own earlier
`AllProjectsView.hydration.test.tsx` caught it because she removed that test's subject while
merging. Same class as Flux's mutation finding, arriving from the other direction: she had the
check and removed what it stood for.

**The call-site gap is closed with tests that bite.** I mutated the real render path:

| mutation | result |
|---|---|
| call site returns raw `html` | **2 red** — `onerror`, `javascript:` |
| `ALLOWED_TAGS` emptied | **6 red** |

Each helper throws `body section never rendered`, so they cannot pass vacuously. Both mutations
restored byte-identically.

## Gates, run by me in Mira's worktree

| Gate | Result |
|---|---|
| **behind count (hard criterion)** | **1 — FAIL, note claims 0** |
| `grep -c boundMirrorNames …` | 2 — PASS |
| sanitize tests | 14/14 — PASS |
| `vitest run` | 239/239, 19 files — PASS |
| `tsc --noEmit` | clean — PASS |
| `vite build` | green — PASS |
| `cargo test` / clippy / fmt | 283 / 0 warnings / clean — PASS |
| `verify-docs.sh` on her note | clean — PASS (the tree's one violation is Aegis's packet) |

Every gate in her table reproduces except the **first line**, the one that decides whether a merge
card may be opened.

## Boundary

No code, backend, DB, or merge change by me. I mutation-tested two existing files and restored
them byte-identically; Mira's worktree is clean and untouched. I did not re-run `git merge` — I
verified the resolutions' **outcome** in the current tree (which side won, which hunks survived),
so a resolution that was correct but for the wrong reason would look identical from here. No app
launched and no live seat list observed.

## Fix

Re-merge track, re-run `git rev-list --count packet/mira/knowledge-panel..track/infra-foundation`,
and amend the note. The outstanding commit touches nothing under `ui/src` or `src-tauri/src`, so
this same evidence supports a PASS immediately after.

## Artifact index

- Verdict: `docs/coordination/tasks/flux/FLUX-2026-10-21-mira-sync-to-track-verdict-v1.md`
- Ledger event: `ev--WWsdegFX0cmXC8WvQaQm`
- This delivery note.
