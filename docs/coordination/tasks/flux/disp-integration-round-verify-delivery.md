# Delivery — `disp-integration-round-verify` (Flux): adversarial verification of the integration round

- **Dispatch**: `disp-integration-round-verify`, issued by seat-aegis-001, kind `verification`.
  Acceptance criteria: Lyra's plan §5.2 slice 3 (`LYRA-2026-10-17-next-round-plan-v1.md`).
- **Claim under test**: "the app reads real data and says honestly when it does not."
- **Verdict**: **FAIL — narrow, scoped to the product, not to any seat's branch.**
- **Verdict event**: `ev-verdict-integration-round` — the pre-declared id, emitted exactly as required.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-17-integration-round-verdict-v1.md` | new — the verdict: every attack with its exact input and exact output |
| `docs/coordination/tasks/flux/disp-integration-round-verify-delivery.md` | this note |
| `.scratch/flux_hydration_probe.rs` | kept — the live read-path probe (its header records both runs) |

No source file was changed. Live was read, never written; the podman container was not touched this
round; nothing merged or restarted.

## The attacks, and their exact results

1. **A failing read, on track** (the real `provenanceForTab` driven through a temporary vitest probe,
   since deleted): `backendHydrated=false` → `sessions` **`live`**, `dispatches` **`live`** — the
   badge claims LIVE over a failed read, in all four discovery states. The eight store-backed tabs
   answer `fixture` (honest). **The thing the dispatch said to verify above all is still false in
   the product.**
2. **The same matrix against the fixed function** (Mira `7a1b12e`, which landed at 04:02Z *during*
   this verification): `sessions`/`dispatches` → `fixture` on a failed read, `live` on a succeeded
   one; `live-seats` discovery-gated both ways. I had derived the same tmux-vs-database distinction
   independently before reading her commit (`cmd_list_tmux_sessions` takes no `State`), so the fix
   matches measured reality. Her suite on that tip: **193/193**, tsc clean, Rust 271/0.
3. **A succeeding read**: LIVE where true, on both versions. ✓
4. **The three LIVE_TABS individually, data paths measured**: `live-seats` = tmux mirror via
   `useLiveSessionsStore`, discovery-gated, no store data; `sessions` = history-panel read + tmux
   mirror, own `MeasuredError`; `dispatches` = direct `api.listDispatches()` poll, own
   `加载派单失败` error render. None of the three renders store fixtures — but on track the two
   database-backed ones wear LIVE even over their own error state.
5. **Does the app read real data? No — measured on live at track's tip**: `list_seats` FAILS
   (`"Pi" is not a valid Harness value`, first failing row `seat-aegis-001`); the other seven reads
   OK; `ql_delivery`'s binding now reads 1 row (the `worker` role defect is fixed in live). The
   blocker has two open exits, neither taken: the verified `packet/nimbus/harness-pi` code fix is
   not integrated, and `af0275a`'s Stepcode data script has not been run against live — and
   `af0275a` re-ruled the data toward Stepcode, so which exit closes it is now a live-data question.
6. **The re-emitted events**: Nimbus's `ev-blocked-on-whom-dto-landed` is an explicit re-emission
   (`reissue_of`, `not_a_second_verification`, boundary honoured) and it **did** release its slice
   (`disp-blocked-on-whom-ui`: blocked → ready). Mira's `ev-first-screen-default-tab-landed` was
   **never emitted** (her dispatch closed without it). Both dispatches' own announcements went out
   under server-minted ids, not the declared ones — so `disp-integration-round-verify-v2`, the
   verifier correctly edged on the fixes, is **blocked on two ids that do not exist**.
7. **The empty states**: unchanged and pinned (13/13 in the two files; 193/193 full suite). ✓

## Needs reporting (no action taken — outside my boundary)

- **The product still cannot read the seats and two tabs still lie on a failed read.** Integration
  (Aegis) plus one operator act (resolve the harness question: integrate the variant or run the
  Stepcode script against live) are what close the claim; neither is mine.
- **The announcement-id contract was violated a second and third time today** (server-minted ids
  for both slice announcements; one required re-emission never emitted at all). The consequence is
  measured: the correctly-edged verifier sits blocked. Nimbus's payload names the missing guard —
  nothing compares a dispatch's required announcement against what was emitted — and that guard is
  a real slice for someone.
- **The rendered badge was not observed** by anyone (the screen is locked from the seats' shells);
  the rule is pinned over the pure function, and the banner's readability is the operator's
  judgement.
