# Delivery — `disp-nhpxzPaQKm0JzIrD-OYL6` (Flux): verify Mira's two 2026-10-16 UI fixes

- **Dispatch**: `disp-nhpxzPaQKm0JzIrD-OYL6`, issued by seat-aegis-001, kind `verification`.
- **Covers**: `disp-KnJYxcf7_Zze8MhPRFNJM` (Projects page reads the DB, failures said out loud) and
  `disp-DgFKxw7pNYTM2eWaLIMhm` (seats attributed by role binding).
- **Verdict**: **PASS for the code** — with two things stated up front rather than covered: the
  runtime acceptance cannot be demonstrated on live (two out-of-domain data rows break the app's own
  hydration read), and I could not observe the packaged app's pixels (screen locked; no screen
  capture, no assistive access — the same limit Mira declared, now measured).
- **Verdict event**: `ev-VV2HvohnhRxRU1YHQRf2C` — pre-declared id, emitted exactly as required, with
  one `ref_type='dispatch'` context ref per covered dispatch.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-16-mira-projects-seats-verdict-v1.md` | new — the verdict: every requirement with the measurement behind it |
| `docs/coordination/tasks/flux/disp-nhpxzPaQKm0JzIrD-OYL6-delivery.md` | this note |
| `.scratch/flux_hydration_probe.rs` | new — the probe that exercises the app's exact eight reads against live, read-only |

No source file was changed. I verified at Mira's tip `cc0e409` (detached, read-only): nothing
committed on her branch, live touched only by read-only `SELECT`s and the two core reads, the
podman container stopped for the C-experiment and restarted immediately, nothing merged.

## Gates, run by me at the tip

`cargo check --workspace --all-targets` clean · `cargo clippy --workspace --all-targets -- -D
warnings` clean · `cargo test --workspace` **251/0** · `tsc --noEmit` clean · vitest **187/0** ·
vite build green. Every number in Mira's commit messages reproduces exactly.

The verification build: `prepare-verification-build.sh` built from her branch and then **refused to
install** because `strings` cannot see the fix text inside the brotli-compressed frontend bundle. I
proved the linkage instead (the binary embeds `/assets/main-DTrXY1mj.js`, which contains the fix;
an independent build of the same commit reproduces the identical content hash; the parent hashes
differently, `main-DuTH0JX4.js`, without the fix), installed with the script's own steps, and the
app launched. The must-have check is blind for UI-only fixes — reported.

## The four requirements, one line each

1. **Root cause — dug out, not guessed.** The failing read is `list_seats`: two live rows carry
   `default_harness='Pi'` (`seat-ql-001`, `seat-vela-001`), outside the `Harness` domain, and one bad
   row fails the whole read. It is one of the eight `Promise.all` reads, so the rejection lands in
   `backendHydrateError` — which on the parent commit is stored and **rendered nowhere** (git grep
   proof), leaving the store on the three seeds. Reproduced by me against live through the identical
   core read.
2. **Projects reflect the DB — mechanism right, live blocked.** Store starts empty, seeds test-only;
   `list_projects` returns 39 including `ql_delivery`. But the seats read in the same `Promise.all`
   fails on the `Pi` rows, so on live the page shows the failure state, not the 39 projects. The
   correction is a live write (`Custom(Pi)` or `Direct`) — the operator's or Aegis's.
3. **Failure said out loud — verified at the component layer.** `HydrationFailure` red box with the
   backend's error verbatim and three ordered next steps; `AllProjectsView` distinguishes read-failed
   from database-empty; the vitest suite mounts the real components with a throwing backend and
   asserts the rendered texts and `projects == []`. I reproduced the exact error strings in both
   worlds (DB down → connection error; DB up → the domain error) and relaunched the app with the DB
   down: alive, and the silent-seed fallback is now structurally impossible. **Pixels not observed**
   — screen locked.
4. **Seeds out of the product path — true for the Projects page, with one reachable mock left.** The
   store constant is imported only by the hydration test; `mock-data.ts`'s leftover is a stale
   comment on `MOCK_GLOBAL_SUMMARY`, which **is** user-reachable via SupervisorPanel → GlobalDashboard
   (defaults to global + dashboard) rendering the same three fake projects. Reported for a separate
   slice, not counted against the fix.

## Dispatch 2 — the filter is correct; live has a second broken row

Seats are attributed by role binding; five tests pin it. Live: `seatloom` → 7 seats, **no ql** (vela
now bound, `data_engineer`); `ThoughtOnly` → 0 bindings, empty state explains why. But
`ql_delivery`'s binding read **fails**: `role='worker'` is out of the `SeatRole` domain, caught
per-project, so ql_delivery renders 0 seats instead of 1. Also flagged: Mira's
`bind-seat-to-project.sh` lists `worker` in KNOWN_ROLES (measured from live), so it will keep writing
a value the core domain rejects.

## Needs reporting (no action taken — outside my boundary)

- **Two live data defects block the runtime acceptance** (exact rows/values/domains/corrections in
  the verdict doc): `seats.default_harness='Pi'` on `seat-ql-001` and `seat-vela-001` aborts the
  whole hydration; `project_role_bindings.role='worker'` for ql in ql_delivery empties that project's
  seat list. Both are live writes — the operator's or Aegis's call. Both are *surfaced* by the fix,
  which is the fix working as designed.
- **The build script's must-have check is blind for UI fixes** (brotli-compressed bundle) — same
  family as the brittle proof assertions reported last dispatch.
- **The mock global dashboard still renders the three fake projects** through the supervisor panel —
  a separate slice.
- **I could not observe the running app** (screen locked; no screen capture or assistive access), so
  C and D were verified at the component and data layers, stated as such rather than passed.
