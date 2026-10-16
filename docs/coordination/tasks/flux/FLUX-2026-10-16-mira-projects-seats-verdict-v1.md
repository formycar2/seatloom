# Verdict — Mira's two 2026-10-16 UI fixes: the Projects page reads the database, and seats are attributed by role binding

- **Verification dispatch**: `disp-nhpxzPaQKm0JzIrD-OYL6` (issued by seat-aegis-001), covering
  - `disp-KnJYxcf7_Zze8MhPRFNJM` — the Projects page reads the database, and a failed read says so
  - `disp-DgFKxw7pNYTM2eWaLIMhm` — seats are attributed by role binding (QL no longer under SeatLoom)
- **Branch under test**: `packet/mira/knowledge-panel` @ `cc0e409` (tip; `f316201` beneath it).
- **Verdict event**: `ev-VV2HvohnhRxRU1YHQRf2C` — the pre-declared id, emitted exactly as required,
  with one `ref_type='dispatch'` context ref per covered dispatch.
- **VERDICT: PASS for the code.** Both fixes do what they claim, every gate I ran is green, and the
  hydration-failure mechanism the first dispatch existed to dig out is now dug out — by me, one level
  deeper than the implementer could reach. **Two things this verdict does NOT cover, stated up front
  because covering them would be the exact sin this project keeps paying for:**
  1. **The runtime acceptance cannot be demonstrated on live.** Two rows in the live database carry
     out-of-domain values (`seats.default_harness='Pi'` ×2; `project_role_bindings.role='worker'` ×1),
     and the app's own hydration read rejects on them. On live as it stands the fixed app shows the
     failure state — correctly and loudly — not the 39 projects. The fixes are right; the data is
     broken; fixing the data is a live write nobody has done and my boundary forbids.
  2. **I could not observe the packaged app's pixels.** The Mac's screen is locked from my seat:
     `screencapture` fails ("could not create image from display") and AppleScript UI scripting is
     denied ("osascript is not allowed assistive access"). Requirement C and D were verified at the
     component and data layers, **not observed in the running app** — the same limit Mira declared
     for herself, now measured rather than assumed.

---

## A. Gates — run by me at `cc0e409`, all green

| Gate | Result |
|---|---|
| `cargo check --workspace --all-targets` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace` | **251 passed / 0 failed** |
| `pnpm typecheck` (tsc --noEmit) | clean |
| `pnpm test` (vitest) | **187 passed / 0 failed** (13 files; includes the 8 new hydration tests and the 5 seat-attribution tests) |
| `pnpm build` (vite) | green |

Every number in Mira's two commit messages reproduces exactly.

## The verification build — the script refused, and the refusal was a false negative

Per the dispatch I ran `scripts/prepare-verification-build.sh packet/mira/knowledge-panel 读不到数据库`
from the main checkout. It built from Mira's worktree, then **refused to install**:
"二进制里找不到 '读不到数据库'". The check is blind: Tauri embeds the frontend assets
**brotli-compressed**, so `strings` on the binary cannot see anything inside the JS bundle (Rust
strings survive; JS strings do not). I proved the build carries the work by linkage instead:

- The binary embeds the asset `/assets/main-DTrXY1mj.js` (the name is plain in the binary).
- That exact file, in the dist the build embedded, contains `读不到数据库` and
  `这个项目下没有绑定任何席位` — and an independent build of the same commit in my own worktree
  produced the **identical content hash** `main-DTrXY1mj.js`.
- The adversarial half: building the **parent** `f316201` produces `main-DuTH0JX4.js` with **zero**
  occurrences of the string. So the embedded bundle is the fixed one, not a stale pre-fix bundle.

I then installed that built app with the script's own steps (stop the old app, copy to
`/Applications`, launch) and it came up (pid 93326). **The must-have check as written fails for any
UI-only fix** — reported for follow-up, same family as last dispatch's brittle proof assertions.

## Requirement 1 — the root cause is dug out, and it is not a guess

Mira established the *mechanism* (the failure was stored in `backendHydrateError` and never rendered,
so the store kept its previous value — the three seeds) but could not see *which of the eight reads
throws* without the webview console. I found it by exercising the app's exact read path from the CLI
against live (`.scratch/flux_hydration_probe.rs` — the same core methods the Tauri commands wrap,
read-only):

```
OK   list_projects          rows=39
FAIL list_seats             seats row "seat-ql-001": column default_harness is out of domain:
                            "Pi" is not a valid Harness value (expected one of: Direct, Stepcode,
                            Custom, Custom(<payload>))
OK   list_workitems         rows=7        OK   list_handoffs   rows=3
OK   list_artifacts         rows=15       OK   list_sessions   rows=5
OK   list_events(400)       rows=400      OK   list_documents rows=0
```

**The failing read is `list_seats`, and the reason is a data value.** `row_to_seat` runs
`check_seat_domains` on every row; `seats.default_harness` must parse as the `Harness` enum
(`Direct` | `Stepcode` | `Custom(<payload>)`). Two live rows carry the bare string `Pi`
(`seat-vela-001`, created 10-10; `seat-ql-001`, created 10-16 10:49). One bad row fails the whole
read (`rows.iter().map(row_to_seat).collect()` — the first `Err` aborts). `api.listSeats()` is one of
the eight `Promise.all` reads in `hydrateFromBackend`, so its rejection abandons the other seven and
lands in `backendHydrateError`. On the parent commit `f316201` I verified the rest of the mechanism
by grep: `backendHydrateError` is declared, set to `null`, and set to `String(e)` in the catch —
and **appears nowhere else in the UI** (`git grep` outside the store returns nothing). The store's
initial value was the three seeds. That is the whole invisible-fallback chain, and it also explains
the operator's original symptom: hydration has been failing since vela's row was created on 10-10.

## Requirement 2 — the mechanism is right; the end-to-end acceptance is blocked by data

- The store now starts **empty** (`projects: []`), the seeds are renamed
  `SEED_PROJECTS_FOR_TESTS` / `SEED_PROJECT_DATA_FOR_TESTS` and are imported **only** by
  `AllProjectsView.hydration.test.tsx` — nothing in the product path. Verified by grep.
- `list_projects` against live returns **39 projects including `ql_delivery`** — the data the page
  must show is there and reachable through the exact function `cmd_list_projects` calls.
- **But the page cannot show them on live as it stands.** The seats read in the same `Promise.all`
  fails on the two `Pi` rows, so hydration aborts before `projects` is ever set, and the page renders
  the failure branch. So: requirement 2 passes at the code layer and at the projects-read layer, and
  **is not met end-to-end on live** — blocked by a data defect, not by this fix. The blocking
  correction is a live write (`UPDATE seats SET default_harness='Custom(Pi)'` — or `'Direct'`; a
  data/governance decision) and it is the operator's or Aegis's to make, not mine and not Mira's.

## Requirement 3 — a failed read says so. Verified at the component layer; not observed in pixels

- `HydrationFailure` (new, mounted once above the shell on every tab): `role="alert"`, red frame,
  "读不到数据库 —— 上面这些不是你的项目", the backend's error **verbatim**, and three ordered next
  steps (`podman ps` → 重读一次 → hand the error text to Aegis). No automatic retry, and it does not
  guess which of the eight reads failed — `Promise.all` rejects with the first rejection, so naming
  one would be a guess; the error text names it.
- `AllProjectsView` distinguishes the two empties: read-failed ("项目还没读出来 …… 上面空着是因为没
  连上数据库,不是因为你没有项目", with the error verbatim) vs database-empty ("数据库里一个项目都
  没有 …… 这次读取是成功的,所以这个空是真的空"). Confusing them was the bug.
- The **real components** are mounted by the vitest suite with a throwing backend and the rendered
  texts are asserted — including the decisive half: `expect(st.projects).toEqual([])` (nothing fake
  replaced what could not be read) and `screen.getByText(/connection refused/)` (the backend's own
  words, not a paraphrase).
- I reproduced the **exact error text** the operator would see, in both worlds: DB stopped → every
  read fails with `connection pool error: Error occurred while creating a new object: error
  connecting to server`; DB up → the domain error above, naming table, row, column, the bad value and
  the expected set. I also relaunched the app with the DB stopped: the process stays alive (no
  crash), and the silent-fallback failure mode is now **structurally impossible** — there are no
  seeds in the store to fall back to, pinned by test.
- **Not observed**: the packaged app's rendering. Screen locked; no screen capture, no assistive
  access. What is verified is the store's recorded value, the components' rendering under the real
  error strings, and the embedded bundle containing exactly that code.

## Requirement 4 — the seeds are out of the Projects path; one mock surface still reaches the user

Judging the two leftovers the dispatch named, as asked:

- `useDataStore.ts` — the constant is now `SEED_PROJECTS_FOR_TESTS`, **imported only by the
  hydration test**. Does not reach the user. ✓
- `ui/src/app-v2/mock-data.ts:291` — a stale *comment* referencing the old `INITIAL_PROJECTS` name,
  annotating `MOCK_GLOBAL_SUMMARY`. **That data does reach the user**: `App.tsx` mounts
  `SupervisorPanel` (keyboard-toggled, persisted), which defaults to `contextMode='global'` +
  `viewMode='dashboard'` → `GlobalDashboard` renders `MOCK_GLOBAL_SUMMARY` — the same three fake
  projects (`p-1 SeatLoom 协调核心`, `p-2 ThoughtOnly 叙事引擎`, `p-3 DeepSpeed 推理优化`) under a
  "3 项目" header. So "no hardcoded seed project data reaches any user-visible surface" is **false**;
  "the Projects page no longer starts from seeds" is **true**. The global-dashboard mock predates
  this change (AD-013) and is outside the four requirements — reported for a separate slice, not
  counted against this fix.

## Dispatch 2 — the seat attribution filter is correct; live has one more broken row

- The fix: seats are attributed by **role binding** (`boundIds` from `list_role_bindings`), the
  `seatsDto.map` inside the per-project loop is gone, and seats that exist but are unbound are
  carried in `unboundSeats` so the empty state explains itself ("这个项目下没有绑定任何席位 …… 不是
 加载失败") instead of rendering a bare `席位 (0)` that reads as broken. Five tests pin the filter
  through the store's own hydration.
- Live bindings (read-only): `seatloom` → **7 seats, none of them ql** (aegis, flux, lyra, mira,
  nimbus, onyx, vela — the operator has run the bind script for vela, `data_engineer`, exactly as
  the dispatch anticipated); `ThoughtOnly` → 0 bindings (empty state, correctly).
- **`ql_delivery` → its binding read FAILS**: `project_role_bindings.role='worker'` for
  `seat-ql-001` is out of the `SeatRole` domain (supervisor, product_owner, architect, verifier,
  designer, data_engineer, custom, custom(<payload>)). The hydration catches this **per project**,
  so ql_delivery would render **0 seats**, not the 1 the acceptance names — again a data defect,
  again a live write to correct (`custom(worker)` or a real role — a governance decision), again
  not mine to make. Note the trap: Mira's `bind-seat-to-project.sh` lists `worker` in its
  KNOWN_ROLES (measured from live), so the script will keep writing a value the core domain rejects
  — the script's vocabulary is measured-from-live, not validated against the domain.

## The two live data defects — the operational finding of this verification

| # | Row | Bad value | Domain rule | Effect on the app | Correction (a live write — operator/Aegis) |
|---|---|---|---|---|---|
| 1 | `seats`: `seat-ql-001`, `seat-vela-001` | `default_harness='Pi'` | `Direct`/`Stepcode`/`Custom(<payload>)` | **the whole hydration aborts** — no projects, no seats, anywhere | `Custom(Pi)` or `Direct` (and decide what the harness actually is) |
| 2 | `project_role_bindings`: `seat-ql-001` × `ql_delivery` | `role='worker'` | the eight `SeatRole` values above | that project's binding read fails (caught) → ql_delivery shows 0 seats instead of 1 | `custom(worker)` or a real role — a governance decision about what ql is in ql_delivery |

Both defects are **surfaced, not hidden, by the fix** — which is the fix working exactly as designed.
Until they are corrected, the operator clicking through will see the red box naming defect 1
verbatim; that is better than the three fake cards, but it is not the 39 projects the dispatch asked
for, and no code change in this branch can make it so.

## Honest limits — what this verdict does not assert

- It does not assert the packaged app's rendering: the screen is locked from my seat (no screen
  capture, no assistive access). Component-level rendering, the store's recorded state, the exact
  error strings and the embedded bundle are all verified; pixels are not.
- It does not assert the runtime acceptance of requirement 2 or dispatch 2 on live: both are blocked
  by the two data defects above, and correcting them is a live write outside my boundary.
- It does not assert that the operator's original hydration failure had no other cause: the `Pi`
  rows fully explain it (present since 10-10, matching the symptom), and the other seven reads all
  answer — but "fully explains" is an inference from the evidence, not an observation of the
  operator's session.
- It does not touch live (read-only `SELECT`s and the two core reads only), restarts nothing, merges
  nothing, and changes no source on Mira's branch.

## Bottom line

**PASS** for both fixes, with the two live data defects named as required follow-ups before the
operator's expectation ("39 projects, ql_delivery visible, ql alone in ql_delivery") can be met, the
mock-data reachability reported for a separate slice, the build script's must-have check reported as
blind for UI fixes, and the unobservability of the running app stated rather than papered over.
