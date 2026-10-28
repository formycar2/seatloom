# Flux verdict — the integration round, adversarially verified

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-28-verify-integration-round-v2-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | verification, integration-round, provenance-badge, negative-control, adversarial, seat-flux |

Dispatch `disp-integration-round-verify-v2`.
Plan `docs/coordination/tasks/lyra/LYRA-2026-10-17-next-round-plan-v1.md` §5.2 slice 3 (on
`packet/lyra/milestone-summary`, not in this tree).
Verdict event: **`ev-verdict-integration-round-v2`**, emitted with exactly that id.

## Verdict: PASS

The claim under test is **"the app reads real data and says honestly when it does not"**. Two
things were measured broken on 2026-10-17 and both are fixed on track. I attacked both rather
than reading them off code.

## The one thing above all: a failed read does not claim LIVE

This is the claim that decides whether the operator can tell a working product from a mockup,
so I proved it with a **negative control** rather than only with a passing case.

I restored the pre-fix line — `if (LIVE_TABS.has(tab)) return 'live'` — as the **first** check in
`provenanceForTab`, which is exactly the shape Lyra measured. Four of my ten assertions went
red, including the headline one:

```
× THE ONE THING ABOVE ALL — a failed read may not claim LIVE
× sessions: failed read vs succeeded read produce different badges
× dispatches: failed read vs succeeded read produce different badges
× the fixture wording is the contract string, verbatim
```

So the fix is load-bearing and my probe bites. With the fix in place:

| input | output |
|---|---|
| `backendHydrated = false` | `sessions = MOCK`, `dispatches = MOCK` |
| `backendHydrated = true` | all three `LIVE_TABS` entries read `LIVE` |

## Defect 1 — `default_harness='Pi'` rejected every seat row

**The defect.** All live seat rows carry `default_harness='Pi'`, which was outside
`Harness {Direct, Stepcode, Custom(String)}`. `parse_custom_tagged` needs the literal
`Custom(...)` shape, so a bare `Pi` fell through to `Err`, `list_seats` failed on every row,
`hydrateFromBackend`'s single `Promise.all` rejected, and the store kept `INITIAL_DATA` — so
**8 of 10 NavRail tabs rendered fixtures**.

**The fix, on track.** `crates/seatloom-core/src/objects/session.rs:72` adds the `Pi` variant
and `FromStr` at `:180` maps the literal `"Pi"`.

**How I verified it.** A probe placed inside the enum's own test module (applied and reverted,
tree byte-identical afterwards), driving the real parser:

```
FLUX parse("Pi")            = Pi
FLUX parse("Nonsense")      correctly refused
FLUX parse("Custom(gemini)") = Custom("gemini")
```

The middle line is the one that matters. A widening that admits the fleet's actual harness
without becoming a free-for-all is the whole point, so I checked that an unknown value is still
refused rather than assuming it.

I also confirmed the live rows directly: `select id, default_harness from seats` returns `Pi`
for all **10** rows — aegis, flux, lyra, mira, networkorchestrator, nimbus, onyx, ql,
stepaicloud, vela. (Lyra's plan says 8; there are 10 now, which is growth, not drift.)

## Defect 2 — the provenance badge did not depend on the read

**The fix, on track.** `ui/src/lib/provenance.ts:114-120`:

```rust
if (LIVE_TAB_BACKING[tab] === 'database') {
    // The read backing this tab is Postgres, so a failed hydration means its rows
    // are not live either. This line used to be
    // `if (LIVE_TABS.has(tab)) return 'live'` — a tab-name list, consulted before
    // `backendHydrated`, so a failed read still wore a green LIVE badge.
    return backendHydrated ? 'live' : 'fixture';
}
```

**How I verified it.** Ten adversarial assertions over the real `provenanceForTab`, each with
its exact input and exact output recorded:

| case | input | exact output | |
|---|---|---|---|
| a read that fails, every `LIVE_TABS` entry | `backendHydrated=false` | `live-seats=LIVE` (exempt), `sessions=MOCK`, `dispatches=MOCK` | ✅ |
| a read that succeeds | `backendHydrated=true` | all three read `LIVE` | ✅ |
| each `LIVE_TAB` individually, both states | flag false then true | `live-seats` LIVE/LIVE (identical, discovery-gated); `sessions` MOCK/LIVE; `dispatches` MOCK/LIVE | ✅ |
| the mirror is gated on tmux discovery | `discovery` empty/failed/unknown × both DB states | `NO LIVE SEATS` / `DISCOVERY FAILED` / `LIVE`, each pair identical across DB states | ✅ |
| the fixture wording is the contract string | failed read, each database-backed tab | headline `SAMPLE DATA — not yet from live capture` verbatim | ✅ |
| `LEDGER_TABS` behave the same way | `timeline`, both states | `MOCK` then `LIVE` | ✅ |
| a store-backed tab | `dashboard`, both states | `MOCK` then `SAMPLE` | ✅ |
| the two tab sets are disjoint | intersection | `[]` | ✅ |

On the mirror row: `live-seats` keeps its LIVE badge under `backendHydrated=false`, and that is
**correct**, not a leak. Its backing is `tmux` (`LIVE_TAB_BACKING['live-seats'] = 'tmux'`), so
its liveness is decided by discovery, not by the database read. I verified that discovery
decides and the DB flag does not, by holding discovery fixed and flipping the flag.

## A trap I hit and am recording, because it nearly produced a false failure

The non-live badges legitimately **contain** the letters LIVE: `NO LIVE SEATS` and
`DISCOVERY FAILED`. My first assertion was `expect(badge).not.toMatch(/LIVE/)`, which fails on
correct behaviour. I rewrote the assertions to compare **exact badge strings**.

This is the same shape as the three empty-list cases in the first-screen verification, where
all three shared the phrase `NO DISPATCHES YET` and only the qualifier distinguished them. The
general lesson: when a negative assertion fails, check whether the thing you are asserting
absent is a substring of the correct answer before concluding the code is wrong.

## The two re-emitted events

Both edges exist, both previously-blocked slices left `blocked`, and both re-emissions say they
are re-emissions:

| id | emitter | `reissue_of` | |
|---|---|---|---|
| `ev-reemit-stalled-announcements-landed` | Nimbus | `ev-J0i6eYQzPB2eT5ppNnYm3` | ✅ |
| `ev-provenance-badge-honest-landed` | Mira | `ev-49Uizr1yc3D85D84CNqyf` | ✅ |

Both carry a `reissue_reason`. Nimbus's payload is explicit that it is a mechanical id
correction and **not** a second verification — "No re-measurement of my own delivered work",
"No test was re-run, no proof was re-run, no gate was re-measured, and no conclusion changed" —
and he states that he did **not** emit Mira's two ids, because `actor_ref` resolves server-side
and emitting them would have put his name on her work. That is the right call and I confirmed
both of Mira's events carry `actor_ref = seat-mira-001`.

He also records four things his event does not assert, including that the installed binary
predates all of this work and that the mirror pipe-guard fix (`9a67ea5`) is **not** on track.
Both are accurate.

## Gates

| check | result |
|---|---|
| `ui` `vitest run` | **26 files / 321 tests** passed — including my 10 probe assertions |
| `ui` `tsc --noEmit` | clean |
| `cargo test --workspace` | **322 passed / 0 failed** |

My probe file was deleted before the worktree was removed, so the product tree carries nothing
of mine.

## What I did not verify

- **I did not open the built app**, and the installed binary predates this work. A rebuild plus
  reinstall is needed before the operator sees any of it — the acceptance card and its build
  have to arrive together, which is exactly the gap Aegis's `f52aa46` and `dd7ba34` address.
- **I did not drive the full `App` render**, because that needs the Tauri bridge. I verified the
  badge decision by driving the real `provenanceForTab` directly, which is the function the badge
  reads, and the Rust fix by driving the real parser.
- **I did not re-verify Mira's or Nimbus's own delivered work** beyond the two fixes' presence
  and behaviour on track.
- **The `list_seats` end-to-end path against live** is verified at the parser level plus the live
  column values, not by running `list_seats` against the live database from this process.

## Hygiene

One detached worktree at the merged track HEAD, removed afterwards. Two mutations and one
negative control, each reverted with the tree verified byte-identical. No live-DB writes, no
files of another seat modified, no merge into track.

## Files

`packet/flux/intent-verdict`. The verdict event is `ev-verdict-integration-round-v2` in the
ledger.
