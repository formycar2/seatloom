# Task: Flux A4-β Layer A commit-pinned verify (SessionsWorkspace real-API wire-up)

[Lyra -> Flux] A4-β delivery landed at `ab672e5`. Lyra pre-flight audit clean (scope 3/3 files, wire shape verified against Rust struct, MOCK removed, banner/pill preserved). Per three-layer gate protocol you run commit-pinned verify before Lyra acceptance. This packet is Layer A only; Layer B runtime defers to Mr. Zhang or a tauri-dev seat.

| Field | Value |
|---|---|
| template | T4 |
| subtype | verification |
| id | FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-05-09 |
| to | flux |
| priority | P0 (SG-B gating) |
| deadline | 2026-05-10 |
| depends_on | Mira delivery `docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-delivery-v1.md` @ `ab672e5`; packet `docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-v1.md` |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-delivery-v1.md` |
| verify target commit | `ab672e53e2c55c93d1e13f8ce6b4cbdc07c546cc` (checkout this sha, no merge) |
| tags | flux, A4-β, verification, SG-B-gating, tmux-mirror, commit-pinned |

---

## 1. Context

SG-A closed UNCONDITIONAL on 2026-05-09 afternoon. A4-β is the last packet in the SG-A → SG-B transition: it replaces the A4-α mock IPC (`MOCK_TMUX_SESSIONS` + `console.warn('A3 not yet available')`) with real calls to A3's `cmd_list_tmux_sessions` + `cmd_attach_tmux_session`. Once A4-β clears Flux Layer A + Lyra acceptance, Lyra sends "B1 GO" to Nimbus, and SG-B begins.

Mira's delivery reports three files, 56 insertions / 42 deletions, tsc zero errors, pnpm build green. Lyra pre-flight confirms wire shape vs Rust struct is correct (snake_case for `TmuxSessionInfo`, camelCase for `AttachTmuxRequest` envelope).

Your job: commit-pinned Layer A verify against `ab672e5`, static invariant audit, and a written delivery doc. Runtime Layer B is out of scope for this packet (Mr. Zhang / tauri-dev seat will own it).

## 2. Verify steps (Layer A)

Run all from a checkout of `ab672e5` (detached HEAD is fine).

```bash
git fetch origin
git checkout ab672e53e2c55c93d1e13f8ce6b4cbdc07c546cc
git log -1 --format='%h %s'   # must show ab672e5 feat(ui): A4-β — …

# A.1 — scope audit
git show --stat ab672e5

# A.2 — static checks
cd ui
pnpm install --frozen-lockfile    # if lockfile out of date, report it; don't update
pnpm exec tsc --noEmit
pnpm build

# A.3 — backend untouched
git show ab672e5 -- 'crates/**' 'src-tauri/**'   # must be empty

# A.4 — i18n untouched
git show ab672e5 -- 'ui/src/i18n/**' 'ui/src/**/*.i18n.*'   # must be empty

# A.5 — SessionTerminal untouched
git show ab672e5 -- 'ui/src/app-v2/panel/SessionTerminal.tsx'   # must be empty

# A.6 — MOCK removal invariant
git grep -nE 'MOCK_TMUX_SESSIONS|A3 not yet available|TODO\(A4-β\)' -- ui/   # must be empty

# A.7 — real IPC presence invariant
git grep -nE "api\.listTmuxSessions|api\.attachTmuxSession" -- ui/   # must be 2+ matches (1 api.ts def + usages)
git grep -nE "invoke\('cmd_list_tmux_sessions'\)|invoke\('cmd_attach_tmux_session'" -- ui/src/lib/api.ts   # must be 2 matches

# A.8 — read-only banner + v0.0.1 pill preserved
git grep -nE "Read-only mode \(v0.0.1\)" -- ui/src/app-v2/panel/SessionsWorkspace.tsx   # must be 1 match at line 185-187
git grep -nE "v0.0.1 attach-only · read-only mirror" -- ui/src/app-v2/panel/SessionsWorkspace.tsx   # must be 1 match at line 123

# A.9 — disableStdin preserved (not in A4-β diff, but confirm at HEAD)
git grep -nE "disableStdin: true" -- ui/src/app-v2/panel/SessionTerminal.tsx   # must match line 66

# A.10 — wire shape alignment (cross-check against Rust)
git grep -nE "pub struct TmuxSessionInfo|rename_all" -- crates/seatloom-core/src/pty/mod.rs
git grep -nE "session_name: string|created_at: number|attached: boolean" -- ui/src/lib/types-dto.ts
git grep -nE "rename_all = \"camelCase\"" -- src-tauri/src/commands/session_cmds.rs   # for AttachTmuxRequest at line 84
```

## 3. Static invariant matrix (13 checks)

Copy verbatim into delivery §3, fill PASS/FAIL + line evidence:

| # | Invariant | Target | Expected |
|---|-----------|--------|----------|
| 1 | Scope bounded | `git show --stat ab672e5` | 3 files only: SessionsWorkspace.tsx + api.ts + types-dto.ts |
| 2 | Insertion/deletion tally | stat | 56 insertions, 42 deletions |
| 3 | Backend frozen | `git show ab672e5 -- crates/** src-tauri/**` | empty |
| 4 | i18n frozen | `git show ab672e5 -- ui/src/i18n/**` | empty |
| 5 | SessionTerminal frozen | `git show ab672e5 -- ui/src/app-v2/panel/SessionTerminal.tsx` | empty |
| 6 | MOCK_TMUX_SESSIONS removed | `git grep MOCK_TMUX_SESSIONS -- ui/` | zero matches |
| 7 | A3-not-yet-available warn removed | `git grep 'A3 not yet available' -- ui/` | zero matches |
| 8 | TODO(A4-β) comments removed | `git grep 'TODO(A4-β)' -- ui/` | zero matches |
| 9 | Real IPC wrappers present | `api.ts` | `listTmuxSessions` + `attachTmuxSession` exported |
| 10 | Arg envelope correct | `api.ts:153` | `attachTmuxSession` passes `{ request }` envelope |
| 11 | TmuxSessionInfo wire shape | `types-dto.ts:209-213` | snake_case fields match Rust struct at `pty/mod.rs:114-118` (no rename_all) |
| 12 | Read-only banner preserved | `SessionsWorkspace.tsx:185-187` | `⚠️ Read-only mode (v0.0.1)` text intact |
| 13 | v0.0.1 pill preserved | `SessionsWorkspace.tsx:123` | `v0.0.1 attach-only · read-only mirror` text intact |

## 4. R-rule / compliance matrix

Copy verbatim into delivery §4, fill PASS/FAIL:

| Rule | Required behavior | Verification approach |
|------|-------------------|-----------------------|
| R1 attach-only | UI never calls `cmd_launch_session` path; only `cmd_list_tmux_sessions` + `cmd_attach_tmux_session` | `git grep "cmd_launch_session\|launchSession(" -- ui/src/app-v2/panel/SessionsWorkspace.tsx` must be zero |
| R3 failure-isolation | Close-tab doesn't propagate tmux kill; uses local state only | `closeTab` body at `SessionsWorkspace.tsx:72-78` must not call `api.killSession` (deliberate per A3 backend guarantee; note this is a mirror-leak concern carry-forward, not R3 violation) |
| v0.0.1 read-only | `disableStdin: true` at `SessionTerminal.tsx:66` untouched | check above |
| Browser-dev degradation | `!isTauri()` returns empty list + user-visible error | inspect `SessionsWorkspace.tsx:26-28` (list), `:49-52` (attach) |
| Cancellation safety | useEffect uses `cancelled` flag + clears interval | inspect `SessionsWorkspace.tsx:23-46` |

## 5. Known non-blockers (do not fail the verify on these)

- **Mirror leak on close-tab**: `closeTab` doesn't call `api.killSession`, so backend FIFO + `tmux pipe-pane` stays open until app exit. This matches A4-α behavior, does not violate R3 (A3 backend `kill_session` is already R3-safe, but more importantly the non-call means the FIFO leaks, not a kill propagation). Flag as carry-forward in your delivery §6; Nimbus B1 or a follow-up cleanup can address.
- **Unused optional fields in `attachTmuxSession` request**: `seatId` + `sessionId` are optional in the TS type but unused by the current call site. Not a bug; kept for future seat attribution.
- **pnpm install lockfile**: if `--frozen-lockfile` fails because Mira's dev env drifted the lockfile, treat as a yellow flag in delivery §7; do not auto-update the lockfile.

## 6. Delivery doc requirements

Write to `docs/coordination/tasks/flux/FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-delivery-v1.md`. Must contain:

1. T3 frontmatter (template, subtype=verification_delivery, status=delivered, commit=`ab672e5`, packet_ref=this packet).
2. `git log -1 --format='%H %s'` output confirming checkout at `ab672e5`.
3. All §2 verify-step outputs verbatim (stdout + stderr + exit codes).
4. §3 static invariant matrix filled with PASS/FAIL + line evidence. Any FAIL blocks acceptance.
5. §4 R-rule matrix filled with PASS/FAIL + line evidence.
6. §5 carry-forwards acknowledged in a "Known non-blockers" section.
7. Layer B note: "Runtime verification deferred to Mr. Zhang / tauri-dev seat. Not part of this Layer A delivery."
8. Verdict: PASS / CONDITIONAL PASS / FAIL with rationale.

## 7. Acceptance criteria (Lyra)

1. Delivery doc posted at the path above.
2. 13/13 static invariants PASS.
3. 5/5 R-rule entries PASS.
4. tsc zero errors + pnpm build exit 0 (reproduce Mira's claim at commit-pinned checkout).
5. No unexpected FAILs outside the §5 non-blocker list.

On Flux PASS → Lyra posts acceptance doc → Lyra sends "B1 GO" to Nimbus.

## 8. Timing

Dispatched 2026-05-09 evening. Target: Flux delivery within the next work session. If blocked on anything (lockfile drift, unexpected repo state, tooling gap), post a comment on this packet rather than partial-delivering.

---

*Dispatched by Lyra · 2026-05-09 evening · Gates SG-B entrypoint B1 dispatch*
