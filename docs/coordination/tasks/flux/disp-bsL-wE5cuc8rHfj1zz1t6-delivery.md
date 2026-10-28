# Delivery — `disp-bsL-wE5cuc8rHfj1zz1t6` (Flux): verify the enum Harness/Agent `Pi` variant

- **Dispatch**: `disp-bsL-wE5cuc8rHfj1zz1t6`, issued by seat-aegis-001, kind `verification`.
- **Branch**: `packet/nimbus/harness-pi` @ `ace1b2a` (off clean `track/infra-foundation`).
- **Verdict**: **PASS.**
- **Verdict event**: `ev-Nfy-64T_UWlAa_INtzgSF` — the pre-declared id, emitted exactly as required.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-17-harness-agent-pi-verdict-v1.md` | new — the verdict: every requirement with the measurement behind it |
| `docs/coordination/tasks/flux/disp-bsL-wE5cuc8rHfj1zz1t6-delivery.md` | this note |
| `.scratch/flux_hydration_probe.rs` | kept — the probe that exercises the app's exact eight reads against live (its header now records both runs: 10-16 FAIL on harness, 10-17 all OK) |

No source file was changed. I verified at the branch tip, read-only: nothing committed on Nimbus's
branch, live touched only by read-only `SELECT`s and the two core reads. The one write outside my
worktree was `pnpm install` in Nimbus's worktree — the verification-build script's own precheck
message prescribes it ("先在那儿跑一次 pnpm install"), it is gitignored build state, and no tracked
file was touched.

## Gates, run by me at the tip

`cargo check --workspace --all-targets` clean · `cargo clippy --workspace --all-targets -- -D
warnings` clean · `cargo test --workspace` **275/0** · `tsc --noEmit` clean · vitest **159/0** ·
Nimbus's two Pi proofs **ALL POINTS PASS** (7 and 8 points). The named tests
(`pi_serializes_verbatim_on_both_dimensions`, the two domain-error tests, both round-trip tables, the
serde-shape and dimension tests) run individually — all ok.

## The requirements, one line each

- **B — verbatim `"Pi"`, no `rename_all`.** Both enums gain `Pi` with only the plain derive; all
  three `rename_all` occurrences in the file are comments (one is the file's own prohibition).
  `FromStr`/`Display`/`serde_json` all spell `"Pi"`; casing is significant (`"pi"` refused).
- **C — the other variants unchanged.** `Direct`/`Stepcode` arms untouched; `Pi` sits before
  `Custom(String)`, so the escape hatch stays last; `HARNESS_EXPECTED` names the whole domain
  including `Pi`; unknown values are still a typed error, never absorbed into `Custom`.
- **D — `Agent::Pi` the same.** FromStr, Display, `AGENT_EXPECTED`, the round-trip table and the
  verbatim-serialization test all updated.
- **E — live `list_seats` succeeds, measured by me.** My own probe at this tip: all eight hydration
  reads OK, `list_seats` 8 rows, every row in-domain. The `ql_delivery` binding read still fails on
  `role='worker'` — the dispatch's declared precondition, a data defect caught per-project, judged on
  its own side rather than merged into one verdict.
- **F — the app reads the database again.** Built from this branch with
  `prepare-verification-build.sh`, installed, and running (pid 98467) with the operator identity;
  the installed binary contains the updated Rust-side domain strings, so the carry is proven on the
  Rust side (no brotli blindness for this one). Hydration's reads all succeed on live, so the store
  is replaced by the 39 real projects including `ql_delivery` and the three fake cards leave the
  screen. **The rendered page was not observed** — the screen is locked from my seat (screencapture
  fails, assistive access denied), so F rests on build-carry + the live read path + the store
  semantics.

## Needs reporting (no action taken — outside my boundary)

- **`project_role_bindings.role='worker'`** for `seat-ql-001` × `ql_delivery` is still out of the
  `SeatRole` domain; until the operator runs `scripts/fix-ql-role-domain.sh`, ql_delivery renders
  0 seats — a "this project has nobody" illusion that is a data defect, not this branch's.
- **This branch predates Mira's two UI fixes** (it is off clean track): the store here still *starts*
  from the three seeds (overwritten once hydration succeeds — which it now does), every project
  still carries every seat (QL appears under SeatLoom in this build), and there is no
  `HydrationFailure` component. Judge those on their own branch and cards, not against this one.
- **The ruling's cost is recorded** in `enum Harness`'s doc comment, as the dispatch required: the
  table no longer says which wrapper layer a seat runs under; the surviving clue is `default_model`.
