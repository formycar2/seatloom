# Verdict — branch synced to track: **FAIL** on the packet's own hard criterion (behind 1, claimed 0)

- **Verification dispatch**: `disp-dVyplqnSC705oXN6cFiUt`.
- **Implementation dispatch**: **`disp-bBdpBqTT9g5-kKU28xvIt`** (`verify_dispatch_id` in the payload);
  packet `AEGIS-2026-10-21-mira-sync-to-track-v1`.
- **Branch under test**: `packet/mira/knowledge-panel` @ `dbb51e7` (the branch's current head; the
  delivery note names `1f8c01b`, which is 3 commits behind the head it claims to describe).
- **Verdict event**: `ev--WWsdegFX0cmXC8WvQaQm` — emitted as required.
- **VERDICT: FAIL** — the sync work itself is real and good, but **the branch is still behind
  `track`, and the delivery note reports it as 0.**

## Scope

One dispatch, its own verdict; nothing else merged in.

## The finding

The packet states its first acceptance criterion as a literal command output:

> `git rev-list --count packet/mira/knowledge-panel..track/infra-foundation` 输出 **0**
> (落后 0 个才算同步干净)

I ran that command. It prints **1**, not 0.

**How the note got it wrong, and why it matters.** The note claims the branch is 0 behind at
`1f8c01b`. Both parts are wrong, and neither is a rounding error:

| measured at | behind-track count |
|---|---|
| `67fd267` (the sync merge itself) | **3** |
| `1f8c01b` (the head the note names) | **2** |
| `49a9ea2` / `331f533` | 2 / 1 |
| **`dbb51e7` (current head)** | **1** |

The count was **never 0**. The single outstanding commit is `6c90709` — *aegis: record the AppV2
deletion ruling's verification script*, `scripts/record-appv2-deletion-ruling.sh`, authored at
16:00:43 on track, which is **after** the sync merge `67fd267` landed at 14:05. So the note could
not have been measuring it; it appears to have read `0` for a state the branch never occupied, or
to have measured a different predicate.

**This is exactly the failure the dispatch exists to prevent.** `packet`'s own premise was that a
stale branch produces a false verification: *"现在开卡合并,等于把你的项目往回拖 19 个提交"*. The
sync removed the substantive gap — the project-filtering code is genuinely in — but the branch is
still 1 behind, so the situation the packet describes is **reduced, not removed**, and the note
reports it as solved. Merging this now would still carry a branch that does not contain track's head.

**Direction of the error is favourable, which is what makes it easy to miss.** The outstanding
commit is Aegis's own script, 147 lines, and it touches nothing in `ui/src` or `src-tauri/src` — it
does not change the operator-visible behaviour at all. The project-filtering motivation is fully
satisfied. But the criterion is a command output, it is `0`, and the measured value is `1`.

## What is genuinely good, and verified

I am not grading the whole dispatch as bad — the merge resolution is the strongest part of it.

**Track won where it should have, and her three areas survived intact.** `useDataStore`'s
`hydrateProjectData` is track's scoped-per-project read (verified present, including the very line
she dropped and restored — `set({ projects, activeProjectId: nextActive })` is back, so hydration
publishes projects). Her `sanitizeHtml(html)` call site is intact in `DocumentsWorkspace.tsx`; the
timeline filter is hers (`hiddenTypes`, `COLLAPSED_BY_DEFAULT`, 9 occurrences); `SandboxBanner.tsx`
and `runtime_cmds.rs` survived. The `api.ts` claim is right: `listDocuments` takes an options
object, not a positional id.

**The line she dropped and the test that caught it is the best thing in the note.** A store that
hydrated zero projects and reported no error is precisely the silent failure this project keeps
paying for, and `AllProjectsView.hydration.test.tsx` — her own earlier test, written to pin
"the Projects page reads the database" — caught it because she removed that test's subject while
merging. It is the same class as Flux's mutation finding, arriving from the opposite direction:
she had the check and removed what it stood for. Recorded as an honest failure, not buried. It is
worth a memory entry of its own.

**The call-site gap is really closed, and I proved it bites.** Two mutations against the real
render path:

| mutation | result |
|---|---|
| call site returns raw `html` (exactly Flux's mutation) | **2 red** — `onerror` and `javascript:` caught |
| `ALLOWED_TAGS` emptied | **6 red** (2 from this file, 4 from the normal-document direction) |

Both restored byte-identically. The tests render the real component and assert on the DOM the
browser built, and each helper throws if the body section never rendered, so they cannot pass
vacuously — the failure mode they were added to prevent.

## Gates, run by me in Mira's worktree

| Gate | Result |
|---|---|
| **behind count (hard criterion)** | **1 — FAIL, note claims 0** |
| **`grep -c boundMirrorNames ui/src/views/LiveSeatsView.tsx`** | **2** — PASS |
| sanitize tests | **14/14** — PASS |
| `vitest run` | **239/239**, 19 files — PASS (matches the note) |
| `tsc --noEmit` | clean — PASS |
| `vite build` | green — PASS |
| `cargo test` | **283** — PASS (matches the note) |
| `cargo clippy --all-targets` | 0 warnings — PASS |
| `cargo fmt --all -- --check` | clean — PASS |
| `verify-docs.sh` on her delivery note | clean — PASS (the one violation in the tree is Aegis's packet, not her note) |

So every gate in her table reproduces except the **first line**, which is the one the packet named
as the merge criterion.

## Honest limits

- **I did not re-derive the conflict resolutions** from the merge's original inputs; I verified
  their *outcome* in the current tree (which versions won, which of her hunks survived) rather
  than re-running `git merge`. A resolution that was correct but for the wrong reason would look
  identical to a correct one from here.
- **The 3 conflicted test files' doubles** I checked for the `listDocuments(options)` shape only.
  I did not run those files in isolation to confirm the assertions were left standing unchanged —
  the note's claim about that, though the total 239/239 includes them.
- **No app launched, no seat list observed.** `boundMirrorNames` being present is verified by grep
  and by tests, not by looking at a live seat list following the project.

## Bottom line

**FAIL, on one criterion, and it is the one that decides whether a merge card may be opened.** The
sync's substance is sound — track's scoped project filtering is in, her three areas survived intact,
the merge conflict where she dropped the hydration line was caught by her own test and fixed, and
the call-site sanitation gap is closed with tests I mutation-proved bite. But
`git rev-list --count packet/mira/knowledge-panel..track/infra-foundation` prints **1**, the
delivery note says it prints **0**, and the dispatch exists precisely because a branch behind
track makes every other verification on it unreliable. One more track merge — which is only Aegis's
docs/scripts commit, and touches nothing in `ui/src` — closes it, after which this same evidence
supports a PASS. Re-merge, re-run that one command, and amend the note.
