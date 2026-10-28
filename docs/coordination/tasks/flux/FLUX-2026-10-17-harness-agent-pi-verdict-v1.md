# Verdict — `Pi` joins `enum Harness` and `enum Agent` (the operator's 2026-10-17 ruling)

- **Verification dispatch**: `disp-bsL-wE5cuc8rHfj1zz1t6` (issued by seat-aegis-001).
- **Branch under test**: `packet/nimbus/harness-pi` @ `ace1b2a` — six commits off clean
  `track/infra-foundation`: `1b791a8` (Agent gains `Pi`), `1549299` (Harness gains `Pi`), the
  delivery doc, and three proof/doc commits.
- **Verdict event**: `ev-Nfy-64T_UWlAa_INtzgSF` — the pre-declared id, emitted exactly as required.
- **VERDICT: PASS.** The code does what the ruling requires, every gate I ran is green, and the
  acceptance criterion that matters most — `list_seats` against live — **succeeds, measured by me**:
  all eight rows read, all in-domain. One live data defect remains and it is **not** this branch's:
  `project_role_bindings.role='worker'` for ql in ql_delivery, exactly the precondition the dispatch
  named. Judged on its own side, as instructed.

## A. Gates — run by me at `ace1b2a`, all green

| Gate | Result |
|---|---|
| `cargo check --workspace --all-targets` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace` | **275 passed / 0 failed** (up from track's 251 — the new Pi tests) |
| `tsc --noEmit` | clean |
| vitest | **159 passed / 0 failed** (this branch predates Mira's UI suite — fewer tests, consistent) |
| Nimbus's `agent_pi_variant_proof.sh` | **ALL POINTS PASS** (7) |
| Nimbus's `harness_pi_variant_proof.sh` | **ALL POINTS PASS** (8) |

The named tests, run individually so the verdict can cite them: `pi_serializes_verbatim_on_both_dimensions`,
`pi_serializes_verbatim_as_the_database_spells_it`, `the_harness_error_names_the_whole_domain_including_pi`,
`the_agent_error_names_the_whole_domain_including_pi`, `harness_domain_round_trips`,
`agent_domain_round_trips`, `unknown_harness_is_a_typed_error`,
`unknown_agent_is_a_typed_error_not_a_custom_fallback`, `serde_shape_is_pascal_case_and_externally_tagged`,
`dimensions_are_not_interchangeable` — all ok.

## B. `Harness::Pi` serializes verbatim as `"Pi"` — and there is no `rename_all`

`enum Harness` gains `Pi` between `Stepcode` and `Custom(String)`, carrying only the plain
`#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` — **no attribute of any kind**.
All three `rename_all` occurrences in the file are comments: line 42 is the file's own prohibition,
and two more are the new tests asserting "No `rename_all`". `FromStr` maps `"Pi" => Ok(Self::Pi)`,
`Display` writes `"Pi"`, and `serde_json::to_string(&Harness::Pi)` is asserted to be `"Pi"` — the
spelling PostgreSQL already stores (AD-011). The casing is significant: `"pi"` is refused, not
absorbed into `Custom`.

## C. The other three Harness variants behave as before; `Custom` is still the last-resort escape hatch

`Direct` and `Stepcode` arms are untouched; `Pi` is inserted before `Custom(String)`, so the
catch-all stays last. `HARNESS_EXPECTED` reads `"Direct, Stepcode, Pi, Custom, Custom(<payload>)"`,
and the new test asserts an unknown value still produces that typed error rather than being absorbed
into `Custom`. `dimensions_are_not_interchangeable` (Stepcode-is-a-harness / Opencode-is-an-agent)
still passes.

## D. `Agent::Pi` — same discipline

`enum Agent` gains `Pi` after `Opencode`, before `Custom(String)`; `FromStr`, `Display`,
`AGENT_EXPECTED` (`"ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Pi, Custom, Custom(<payload>)"`),
the round-trip table and the verbatim-serialization test are all updated. Two live rows already
spelled `default_agent='Pi'` (`seat-ql-001`, `seat-vela-001`); they are now legal.

## E. `list_seats` against live — succeeds, measured by me, not read off his proof

I re-ran my own probe (`.scratch/flux_hydration_probe.rs` — the same core methods the Tauri commands
wrap, read-only) at this branch's tip against live:

```
OK   list_projects          rows=39
OK   list_seats             rows=8        <-- was FAIL yesterday on default_harness='Pi'
OK   list_workitems         rows=7        OK   list_handoffs   rows=3
OK   list_artifacts         rows=15       OK   list_sessions   rows=5
OK   list_events(400)       rows=400      OK   list_documents rows=0
OK   bindings[seatloom    ] count=7 ql_bound=false seats=[aegis, flux, lyra, mira, nimbus, onyx, vela]
FAIL bindings[ql_delivery ] role "worker" is out of the SeatRole domain
OK   bindings[ThoughtOnly ] count=0
```

All eight seats read, all in-domain (`harness=Some("Pi")` ×8; `agent` 6×`ClaudeCode` + 2×`Pi`) —
Nimbus's committed `probe_seats` example prints the same. The `ql_delivery` failure is the dispatch's
declared precondition: the operator has not run `fix-ql-role-domain.sh`, the role `'worker'` is
outside `SeatRole`, and the hydration catches it **per project** — so the consequence is
ql_delivery showing 0 seats, a data illusion, **not** Nimbus's code and not this branch's defect.
Judged on its own side, as the dispatch instructs.

## F. The app reads the database again — built, installed, launched; pixels not observed

- Built with `scripts/prepare-verification-build.sh packet/nimbus/harness-pi` (from the main
  checkout, as the script requires). Its precheck first failed because Nimbus's worktree had no
  `node_modules`; the script's own message prescribes `pnpm install` there, which I ran (gitignored
  build state, no tracked file touched), then the build completed.
- **The carry is proven on the Rust side this time** (no brotli blindness here — the fix is Rust):
  the installed binary contains `"Direct, Stepcode, Pi, Custom, Custom(<payload>)"` and
  `"ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Pi, Custom, Custom(<payload>)"`. The running
  process (pid 98467) is `/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri` — that binary.
- The app launched with the operator identity (`SEATLOOM_OPERATOR_REF=human:supervisor` confirmed in
  its environment) and holds live DB connections (`pg_stat_activity`: 41 backends for `seatloom`).
- **What follows for F, at the layer I can reach**: hydration's eight reads all succeed on live
  (E above), so `set({ projects })` replaces whatever the store held with the **39 real projects
  including `ql_delivery`**, and the three fake cards are off the screen. The projects page maps
  `store.projects`, which is now only ever what the database returned.
- **What I could not do, stated plainly**: I still cannot see the app. `screencapture` fails
  ("could not create image from display", again after `caffeinate`) and AppleScript UI scripting is
  denied ("osascript is not allowed assistive access") — the Mac's screen is locked from my seat, the
  same wall measured yesterday and recorded in memory. So F is verified by build-carry + the exact
  read path + the store semantics, **not by observing the rendered page**.

## The branch scope — read it before judging this build against anything else

`packet/nimbus/harness-pi` is off **clean `track/infra-foundation`**, so it predates both of Mira's
UI fixes (her acceptance cards are not merged). In THIS build: the store still *starts* from the
three seeds — they are overwritten the moment hydration succeeds, which it now does, but the
silent-fallback itself is still in this code; every project still carries every seat, so QL appears
under SeatLoom here; and there is no `HydrationFailure` component. Those belong to
`disp-KnJYxcf7_Zze8MhPRFNJM` / `disp-DgFKxw7pNYTM2eWaLIMhm` and must be judged on their own branch,
not against this one. (The dispatch's precondition note — "if ql_delivery shows 0 seats, that is the
data defect" — describes the build **with** Mira's filter; in this build ql_delivery would show all
eight seats with no role.)

## The ruling's cost is recorded, as the dispatch requires

`enum Harness`'s own doc comment now carries the cost: read strictly, `pi` is an agent rather than a
harness, the eight seats actually run under the `stepcode` proxy (what six rows said), and after the
ruling the table no longer records which wrapper layer a seat runs under — the only surviving clue
is `default_model` (`modelproxy-anthropic/…`, `stepplan-anthropic/…`). The comment states the
operator's reading is also defensible and that the vocabulary boundary is his to draw. That is the
account the dispatch asked to be kept, and it is kept in the source rather than in a commit message
alone.

## Honest limits

- The rendered page was not observed (screen locked); F rests on build-carry, the live read path and
  the store semantics.
- Live was read, never written, by me; the operator's own scripts made the data changes. The
  `'worker'` role is flagged, not fixed — `fix-ql-role-domain.sh` is his to run.
- This verdict says nothing about Mira's two UI fixes; this branch does not contain them.

## Bottom line

**PASS.** `Pi` joins both enums verbatim, with no `rename_all`, `Custom` still last, the other
variants unchanged, `list_seats` succeeding against live measured by me, the app rebuilt from this
branch and running, and the ruling's cost recorded where a future reader will find it. The one
remaining live defect (`role='worker'`) is on the data side, named, and outside this branch.
