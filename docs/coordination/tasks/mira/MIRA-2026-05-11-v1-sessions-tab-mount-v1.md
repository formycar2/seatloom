# Task: Mount SessionsWorkspace in V1 main shell (Sessions tab)

[Lyra -> Mira] B1 Layer B blocked on UI access — Mr. Zhang has tmux sessions ready in `tmux ls` but no path to reach the SessionsWorkspace Attach UI from the V1 main shell. Same R4 gap Aegis flagged in SG-A as "A 6-8 N/A — SessionsWorkspace not mounted in V1". Now blocking B1 Layer B; cannot defer.

| Field | Value |
|---|---|
| template | T4 |
| subtype | implementation |
| id | MIRA-2026-05-11-v1-sessions-tab-mount-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-05-11 |
| to | mira |
| priority | P0 (blocks B1 Layer B → blocks SG-B closure) |
| dispatched_at | 2026-05-11 (after Aegis flag of B1 Layer B blocker) |
| depends_on | A4-β PASS @ `ab672e5` (SessionsWorkspace built mock-first), B1 PROVISIONAL @ `0c0f425` (write path live but unreachable from V1), Aegis 2026-05-11 R4 escalation |
| delivery path | `docs/coordination/tasks/mira/MIRA-2026-05-11-v1-sessions-tab-mount-delivery-v1.md` |
| tags | mira, V1, navrail, sessions-tab, B1-layer-b-unblock, SG-B-gating |

---

## §1 — Context

The full bidirectional v0.0.2 stack is built and verified:
- A3 (`fca4fe0`) — tmux mirror read path, R3-safe attach
- A4-α (`bdac54b`) — SessionsWorkspace UI with mock data
- A4-β (`ab672e5`) — SessionsWorkspace wired to real `cmd_list_tmux_sessions` + `cmd_attach_tmux_session`
- B1 (`0c0f425`) — bidirectional write path via `tmux load-buffer | paste-buffer`
- 008 (`526d25a`) — multi-project schema (parallel)

But `SessionsWorkspace` (the entire Attach UI) is currently **only reachable from the app-v2 / Supervisor IM detached-window path**. The V1 main shell has no `Sessions` tab in its NavRail, so a user opening `pnpm tauri dev` cannot drive the bidirectional B1 path from the main window.

**This packet adds a `Sessions` tab to V1 NavRail and renders `<SessionsWorkspace />` for it.** No other behavior changes.

## §2 — Scope (3 files; bounded; no behavior changes outside Sessions tab)

### 2.1 `ui/src/App.tsx`

Three small edits:

**a) Tab type (line 35)** — extend the union:

```ts
// before
type Tab = 'dashboard' | 'inbox' | 'timeline' | 'workitems' | 'seats' | 'artifacts' | 'playbook' | 'all-projects';

// after
type Tab = 'dashboard' | 'inbox' | 'timeline' | 'workitems' | 'seats' | 'artifacts' | 'playbook' | 'sessions' | 'all-projects';
```

Order: insert `'sessions'` between `'playbook'` and `'all-projects'` (NavRail visual order).

**b) Import** — add at top of file:

```ts
import { SessionsWorkspace } from './app-v2/panel/SessionsWorkspace';
```

**c) Render switch** — in the function that maps `activeTab` to a component (around line 265-285), add the `'sessions'` case:

```ts
if (activeTab === 'sessions') {
  return <SessionsWorkspace />;
}
```

Place the case **before** the `MasterDetail`-using tab block, matching how `'all-projects'` and `'dashboard'` are handled (early return). `SessionsWorkspace` is full-bleed and does not use the master-detail layout.

### 2.2 `ui/src/components/NavRail.tsx`

One edit — add a NavRail entry between `Playbook` and (whatever comes next, currently nothing — `Playbook` is the last):

```ts
// before (line 63)
{ id: 'playbook', icon: BookOpen, label: 'Playbook' },

// after
{ id: 'playbook', icon: BookOpen, label: 'Playbook' },
{ id: 'sessions', icon: Terminal, label: '会话' },
```

Use `Terminal` from `lucide-react` (already imported in V1 if any other tab uses it; if not, add to existing `lucide-react` import line). `'会话'` is the localized label — matches existing label style ('监督概览' / '收件箱' / '工作项' / '时间线' / '席位' / '产出物' / 'Playbook').

### 2.3 (optional) `ui/src/types.ts` or wherever `NavTab` type lives

If `NavRail.tsx` imports a `NavTab` union type from a shared types file, add `'sessions'` to it. If `NavTab` is inferred from the items array via `keyof typeof` or similar, no change needed. Check the existing pattern.

### Out of scope (do NOT touch):

- `app-v2/panel/SessionsWorkspace.tsx` — already built and verified. Do not modify.
- `app-v2/panel/SessionTerminal.tsx` — B1 already wired keystroke forwarding. Do not modify.
- Backend / Tauri commands — none touched.
- DTOs — none touched.
- Any tab logic for the existing 8 tabs — preserved.
- `i18n.ts` — if you want to add a key for `'会话'` later that's a separate packet; for now inline string is fine (matches existing NavRail labels which are inline).

## §3 — Verification

### Layer A (your seat)

```
cd ui
pnpm exec tsc --noEmit          # zero errors
pnpm build                       # exit 0
```

### Layer B (Mr. Zhang via Aegis — runtime)

The 6-step B1 bidirectional smoke (per B1 acceptance §3) becomes runnable. This packet's Layer B is just step 0 of that:

1. `pnpm tauri dev` boot, NavRail shows new "会话" tab between "Playbook" and (nothing else, was last).
2. Click "会话" — SessionsWorkspace renders (Attach dropdown + tab strip + empty-state message).
3. Dropdown lists `*-seatloom` tmux sessions (e.g., `Onyx-data-seatloom`, `Lyra-po-seatloom`, etc.).

If steps 1-3 PASS, B1 Layer B's full 6-step smoke can proceed (typing → Ctrl-C → paste → close-tab → restart-recovery → scrollback resume).

## §4 — Static invariants for verify (will be 5-row matrix in Flux verify packet)

| # | Invariant | Where to look |
|---|-----------|---------------|
| 1 | Scope held to 2-3 files (App.tsx, NavRail.tsx, optionally types) | `git show <sha> --stat` |
| 2 | `'sessions'` added to Tab union | `App.tsx:35` |
| 3 | NavRail items array has `{ id: 'sessions', ... }` entry | `NavRail.tsx` items config |
| 4 | App.tsx render switch has `if (activeTab === 'sessions') return <SessionsWorkspace />` | `App.tsx` render function |
| 5 | No regressions in existing 8 tabs | tsc + build green; manual verification post-Layer-A |

## §5 — Delivery doc requirements

Write to `docs/coordination/tasks/mira/MIRA-2026-05-11-v1-sessions-tab-mount-delivery-v1.md`. Same shape as your A4-β delivery:

1. T3 frontmatter (template, subtype=implementation_delivery, status=delivered, commit=<sha>, packet_ref=this packet).
2. §1 `git show --stat <sha>` verbatim. Expected scope: 2-3 files (App.tsx, NavRail.tsx, optionally types.ts).
3. §2 verbatim `pnpm exec tsc --noEmit` + `pnpm build` outputs.
4. §3 5-row static invariant self-check with line evidence.
5. §4 verdict: `Layer A self-check PASS` / `HOLD` / `FAIL`.

## §6 — Why this packet now (and not absorbed into A4-α/A4-β/B1)

A4-α/A4-β were scoped to `app-v2/panel/SessionsWorkspace.tsx` only — the component itself + IPC wiring. B1 was 5 files for the bidirectional write path. None of those packets touched V1 NavRail because:

1. The "where does SessionsWorkspace mount" question was deferred to "post-SG-A" per Aegis SG-A scope-drift discipline.
2. R4 ("V1 main shell as the canonical UI surface") is an architectural commitment but the packet sequencing kept it out of the A/B series until B1's Layer B exposed the gap.

**This packet closes that gap.** Once it lands, V1 main shell is the canonical entry to the bidirectional tmux mirror, and Mr. Zhang can drive seats from `pnpm tauri dev` end-to-end.

## §7 — Timeline

ASAP. SG-B closure sits on B1 Layer B; B1 Layer B sits on this packet. Aegis flagged it as P0 with explicit "cannot defer further" framing. Target: Mira delivers within next work session, Flux Layer A verify follows, Lyra acceptance, Mr. Zhang Layer B = full B1 6-step smoke.

---

*Dispatched by Lyra · 2026-05-11 · Aegis R4 escalation · gates B1 Layer B · gates SG-B closure*
