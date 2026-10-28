# Delivery: the provenance badge depends on the read backing the tab

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-17-provenance-badge-honest-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-provenance-badge-honest |
| depends_on | LYRA-2026-10-17-next-round-plan-v1 |
| tags | provenance, badge, live-claim, delivery, mira |

---

## 0. The one-paragraph version

`provenanceForTab` no longer returns `'live'` from a tab-name list. Every branch that can
claim LIVE now requires the read backing that tab to have succeeded, and what backs each
tab is declared in a measured table rather than assumed. A test that **fails** the read
proves the badge is not LIVE.

## 1. The defect, and what backs each tab — measured, not assumed

The old line was:

```ts
if (LIVE_TABS.has(tab)) return 'live';
```

It returns before `backendHydrated` is ever consulted, so a surface whose data came from a
failed read still wore a green LIVE badge. That is not hypothetical: right now
`api.listSeats()` rejects (all 8 live seat rows carry `default_harness='Pi'`, outside the
`Harness` domain), hydration's `Promise.all` rejects, the store keeps its seeds — and
`live-seats`, `sessions` and `dispatches` still claim LIVE.

Before changing anything I measured what actually backs each of the three, because the fix
is only correct if it matches reality:

| tab | command | takes Tauri `State`? | backed by |
|---|---|---|---|
| `live-seats` | `cmd_list_tmux_sessions` | **no** | tmux (shells out; no database) |
| `sessions` | `cmd_list_history_projects` → the history read tools | yes | **Postgres** |
| `dispatches` | `cmd_list_dispatches`, polled directly by the view | yes | **Postgres** |

So `live-seats` is genuinely the odd one out, and for a measured reason rather than by
accident: its read never touches the database, and its liveness is already gated on
`discovery` by the `LIVE_MIRROR_TAB` branch (`no-live` / `discovery-failed`).

## 2. What changed

```ts
// What backs each LIVE_TABS entry — measured, not assumed.
export const LIVE_TAB_BACKING: Readonly<Record<string, 'tmux' | 'database'>> = {
  'live-seats': 'tmux',
  sessions: 'database',
  dispatches: 'database',
};

// in provenanceForTab:
if (LIVE_TAB_BACKING[tab] === 'database') {
  // The read backing this tab is Postgres, so a failed hydration means its rows are
  // not live either.
  return backendHydrated ? 'live' : 'fixture';
}
if (LIVE_TABS.has(tab)) return 'live';   // tmux-backed only
```

`live-seats` keeps its discovery-driven branch untouched — a failed Postgres read must not
downgrade a tmux mirror that is genuinely streaming, and an empty discovery must not let it
claim LIVE. Both directions are pinned.

## 3. The test that fails the read

The dispatch required it explicitly: a happy-path test does not test this. Six new
assertions, of which the decisive two are:

- `provenanceForTab('sessions', false)` is **not** `'live'` — it is `'fixture'`, so the
  badge says the read failed rather than staying green;
- `provenanceForTab('dispatches', false)` likewise.

Plus: the same two tabs DO claim LIVE once `backendHydrated` is true; the live mirror is
gated on discovery and not on the database read (both directions); every `LIVE_TABS` entry
declares its backing so no tab can claim LIVE on a tab name alone; and the store-backed and
timeline tabs are unchanged.

## 4. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` | clean |
| `vitest` | **193/193** (6 new) |
| `vite build` | green |
| `cargo test --workspace` | 271 passed / 0 failed (no Rust changed) |
| `lint-changed.sh` | `provenance.ts` **0 → 0** — zero new, measured in a `track` worktree at the real path |
| `verify-docs.sh` | this note clean |

One measurement note, recorded because I got it wrong first: an initial comparison printed
`139` for this file on both sides, which was my own shell typo (`$f` against a variable
named `F`) making eslint lint nothing useful. Re-run correctly: **track 0, mine 0**.

## 5. What is not proven

Nobody has looked at the rendered badge. The rule is pinned by unit tests over the pure
function, which is where the defect lived; whether the amber banner reads as "the read
failed" rather than "this is sample data" to someone who has not been told is the
operator's judgement.

## 6. Boundaries

No re-layout, no new visual treatment, no new affordance — the `fixture` state already
existed with its own copy, so a failed database-backed read now renders the honest banner
that was already designed for it. `ui/src/lib/api.ts` and `ui/src/lib/types-dto.ts`
untouched. No backend change, no live-database write, no gate event, no merge.

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
