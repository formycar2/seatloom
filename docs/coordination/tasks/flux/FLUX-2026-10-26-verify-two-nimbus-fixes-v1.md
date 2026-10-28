# Flux verdict — two Nimbus fixes, verified independently

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-26-verify-two-nimbus-fixes-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-26 |
| version | v1 |
| depends_on | AEGIS-2026-10-26-flux-verify-two-nimbus-fixes-v1 |
| tags | verification, mutation-testing, negative-control, seat-flux |

- **Verifying dispatch**: `disp-i_ZLKc4E2S0Bm78K-Zmdy` (Aegis → Flux)
- **Packet**: `docs/coordination/tasks/aegis/AEGIS-2026-10-26-flux-verify-two-nimbus-fixes-v1.md`
  (landed on track as `523e69e`; I worked from an isolated worktree at `track/infra-foundation`)
- **Pre-declared verdict event**: **`ev-hh9xak8wlN5fJst6107cy`** — emitted with exactly that id
- **Subject commits**: `793040c` (tauri bundle) and `9a5a715` (tilde path), both merged via
  `b3d1ec8` and `66faf60`

## Headline

**Both fixes are correct in source, and I found one error in Nimbus's reasoning and one
deployment gap he did not report.**

| Fix | Verdict | Basis |
|---|---|---|
| Tauri bundle (`disp-VWK5z1k7Jcu8-iQFrGVrb`) | **PASS** — the conclusion is right, one provenance label in his script is now wrong | three negative controls, all discriminating correctly |
| Tilde path (`disp-NMcTvureVxwPh_3bFAGxx`) | **PASS** — the fix is real and load-bearing | mutation verified; live DB confirms the source of the string |
| **Not reported by Nimbus** | **the operator's running app still carries the bug** | measured, see §4 |

---

## 1. Tauri bundle — his conclusion is right; the criterion is genuinely fixed

### What I verified, in order

**Control 1 (positive): the operator's app against the main checkout's dist.**

```
$ scripts/verify-tauri-bundle.sh target/release/bundle/macos/SeatLoom.app ui/dist
  Contents/Resources: icon.icns
  258 blobs decompressed from 7 build director(ies)
  executable: 12562704 bytes
  OK: assets/SupervisorPanel-65vuChzU.css (54374 bytes; compressed bytes present in the executable)
  OK: assets/SupervisorPanel-B8572Sh_.js (351623 bytes; ...)
  OK: assets/main-CFbL2ovg.css (4635 bytes; ...)
  OK: assets/main-CIxBkYnl.js (618064 bytes; ...)
  OK: assets/supervisor-ZKEgQeoe.js (857 bytes; ...)
  OK: index.html (642 bytes; ...)
  OK: supervisor.html (589 bytes; ...)
  PASS: all 7 frontend files are embedded in this executable, verified byte-for-byte.
```

**So Nimbus's conclusion is correct**: `Contents/Resources` holding only `icon.icns` is
Tauri v2's normal behaviour. The frontend is embedded, brotli-compressed, in the
executable. Aegis's original bug report was a false alarm caused by a check written
against the wrong directory, and Aegis was right to retract it.

**Control 2 (negative, one byte changed):** I copied the dist and appended one comment
line to `main-CIxBkYnl.js`. Result: exit **1**, and it named the exact file:

```
  NOT EMBEDDED: assets/main-CIxBkYnl.js — no blob in any build holds its content
FAIL: 1 of 7 frontend files are not embedded in THIS executable (assets/main-CIxBkYnl.js)
```

**Control 3 (same bytes, different filename):** I renamed `main-CIxBkYnl.js` to
`main-ZZZZZZZZ.js` and updated `index.html`'s reference. The JS still reported **OK** —
proving the match is on **content**, not on the blake3 filename — while `index.html`
correctly reported NOT EMBEDDED because I had edited it. This is the control that rules
out the name-match shortcut Nimbus explicitly said he avoided.

**Control 4 (negative, a genuinely different build):** I built a fresh dist from
`track/infra-foundation` (which contains both fixes) and pointed the script at the
operator's old app. Result: **6 of 7 files NOT EMBEDDED, exit 1.**

```
  NOT EMBEDDED: assets/SupervisorPanel-1a494UXw.js — no blob in any build holds its content
  NOT EMBEDDED: assets/main-Cyii3zfb.js — no blob in any build holds its content
  NOT EMBEDDED: assets/supervisor-BXSu2rbx.js — no blob in any build holds its content
  NOT EMBEDDED: index.html — no blob in any build holds its content
  NOT EMBEDDED: supervisor.html — no blob in any build holds its content
  NOT EMBEDDED: vite.svg — no blob in any build holds its content
```

**The criterion discriminates.** A one-byte change fails; a same-content rename passes;
a different build fails on almost everything. A check that passes everything proves
nothing, and this one does not.

**The favicon half is also real.** The same run's step 5 reported `DANGLING: index.html
references /vite.svg` against the old dist, and reported **`none`** against the fresh
dist — because `ui/public/vite.svg` (`793040c`) is now copied in. So the 404-on-every-launch
defect Nimbus found is fixed in source and the fix is observable through the same script.

### Where Nimbus is wrong: a provenance label that has gone stale

His script's step 4 prints:

```
  isComposing          x11  <- @xterm/addon-fit — NOT SeatLoom
```

**That was true when the original bundle check was written, and it is false now.** Mira's
shift fix (`e7e327e`, 11:20) merged to track at **16:25**, and Nimbus wrote this script at
**20:01** — so at the moment he wrote the label, `isComposing` was already in SeatLoom's
own source:

```
ui/src/hooks/useGlobalShortcuts.ts:31:  if (event.isComposing) return true;
```

The shipped dist (built 18:52, after the 16:25 merge) contains 11 occurrences, at least one
of which is SeatLoom's. `proposeDimensions` **is** still xterm-only (confirmed: present in
`@xterm/addon-fit/lib/addon-fit.js`, absent from SeatLoom's source), so that half of the
label stands.

This does not change the verdict — the load-bearing part of his criterion is the
byte-for-byte content match, which is sound and which I verified with four controls — but
the label is an interpretive claim that will mislead the next person who reads the output.
It should be re-worded to something like "isComposing — appears in `@xterm/addon-fit` AND,
since 2026-10-26, in `useGlobalShortcuts.ts`; either proves the bundle shipped, neither
alone identifies the author."

## 2. Tilde path — the fix is real, and the tests are load-bearing

### The cause, confirmed at source

```
ui/src/lib/dto-to-v1.ts:153   path: '~/' + d.name,   // backend has no path; synthesise a label
```

and on `track/infra-foundation`:

```
ui/src/lib/dto-to-v1.ts:158   path: d.homePath ?? null,
```

### The live ledger is the source of the string, exactly as Nimbus said

```
 id                                  |        name         |                    home_path
-------------------------------------+---------------------+------------------------------------------------
 project_delivery/networkOrchestrator | networkOrchestrator | /Users/…/Documents/GitHub/project_delivery/networkOrchestrator
```

`'~/' + 'networkOrchestrator'` = the string the operator reported. **Reproduced.** The
operator saw a real thing, and it came from a real line.

### All four layers are plumbed

| Layer | Change |
|---|---|
| SQL | `SELECT id, name, created_at, worker_budget_tokens, supervisor_budget_tokens, home_path` (both queries) |
| Row model | `ProjectRow.home_path: Option<String>` + `row_to_project` reads index 5 |
| DTO | `ProjectDto.home_path: Option<String>` + `From<ProjectRow>` carries it |
| TS types | `ProjectDto.homePath: string \| null`; `Project.path: string \| null` |
| Mapper | `path: d.homePath ?? null` |
| Three call sites | `{project.path ?? <no-home-path label>}` in `ProjectOverview.tsx:53`, `ProjectSwitcher.tsx:107`, `AllProjectsView.tsx:125` |

### Mutation-verified — the tests do catch the defect

| Mutation | Result |
|---|---|
| restore `path: '~/' + d.name` | **2 red** (`a hydrated project renders the ledger path…`, `the ledger home_path is rendered verbatim…`) |
| `path: d.name` (the lazy fix — strip the tilde, still synthesised) | **2 red**, same two tests |

The second mutation matters more than the first: the tests assert the **real path appears
verbatim**, not merely that `~` is absent. A fix that just deleted the tilde would fail.

### The pre-fix code, run against the new test file

I checked out `9a5a715~1` and copied the *new* test file onto it: **2 red**, the same two.
So the tests are genuinely pinned to the defect and not merely to the current code.

### The no-home-path branch

The label the operator sees for a project with no recorded home path is present in all
three call sites. Live has **39
projects and zero with `home_path IS NULL`** (021 §2b backfilled the seeded `seatloom`
row), so the branch is reachable only through the test — which is exactly what the test
does, with a `proj('no-home', 'no-home', null)` row.

### Gates

- Full UI suite on `track/infra-foundation`: **278 passed (23 files)**, `tsc --noEmit`
  clean. Baseline at `9a5a715~1`: **206 passed (16 files)**. The delta includes Mira's
  track merge; the tilde fix's own file went **8 → 9** tests.
- 14 unhandled errors are pre-existing (`App.restartRestore.test.tsx:84`'s deliberate
  unmocked Tauri invoke), unchanged.
- I ran the suite with the **main checkout's** `node_modules`, not a symlink into another
  seat's — my first attempt symlinked Nimbus's, and two suites failed to resolve
  `dompurify` because pnpm's store layout differs per worktree. That was my harness's
  fault, not the code's, and I re-ran rather than reporting it.

## 3. What I did not do

- I did **not** open the app and look at it. The bundle script says so itself and it is
  right: that is the only step that proves a human sees the frontend.
- I did **not** rebuild the operator's app. Rebuilding and reinstalling touches what he is
  running, which is his call.
- I did **not** apply anything to the live database.
- I did **not** verify `sessions`-row invariants — not applicable to these two fixes.

## 4. Finding — the operator's running app still has the bug (P1 for him, P3 for the repo)

Nimbus's delivery notes state the fix and its gates. Neither states what is **deployed**.

| Object | Time |
|---|---|
| tilde fix committed (`9a5a715`) | 2026-10-26 **21:10** |
| bundle fix committed (`793040c`) | 2026-10-26 **20:01** |
| `/Applications/SeatLoom.app` binary | 2026-10-26 **19:12** |
| the running `seatloom-tauri` process | started **19:46** |

**Both fixes post-date the installed binary.** I measured the running app's embedded
assets by finding which `tauri-codegen-assets` blobs appear verbatim in its executable and
decompressing them:

```
blobs embedded in the RUNNING binary: 7
tilde synthesis in running app: True
no-home_path string in running app: False
```

So the operator's app — the one he is looking at right now — carries `path:"~/"+` and does
not carry the no-home-path string. **The symptom he reported at 19:51 is still on his
screen.** He will reasonably read "merged to track" as "fixed for me", and it is not.

The second half is the same shape: `ui/dist` in the main checkout was built at **18:52**,
before the favicon commit at 20:01, so `ui/dist/vite.svg` does not exist and the script
still reports `DANGLING: index.html references /vite.svg` against that dist. A fresh build
resolves it — I built one and step 5 reported `none`.

**Neither is a code defect.** Both are the deployment gap, and it is the same lesson Aegis
recorded today at `eb47ec8`: *a process running does not mean it runs the new code.* The
fix is correct; it has not been rebuilt, reinstalled, or restarted. I am reporting it
rather than acting, because rebuilding and replacing what the operator is running is his
decision.

## Reproduction

```bash
# ── bundle: four controls ───────────────────────────────────────────────────
cd ~/Documents/GitHub/seatloom
scripts=../seatloom-worktrees/nimbus/scripts/verify-tauri-bundle.sh   # or the copy on track
bash "$scripts" target/release/bundle/macos/SeatLoom.app ui/dist     # PASS, 7/7

cp -R ui/dist /tmp/d1 && printf '\n// x\n' >> /tmp/d1/assets/main-*.js
bash "$scripts" target/release/bundle/macos/SeatLoom.app /tmp/d1     # FAIL exit 1, names the file

cp -R ui/dist /tmp/d2 && mv /tmp/d2/assets/main-CIxBkYnl.js /tmp/d2/assets/main-ZZ.js
sed -i '' 's|main-CIxBkYnl.js|main-ZZ.js|' /tmp/d2/index.html
bash "$scripts" target/release/bundle/macos/SeatLoom.app /tmp/d2     # JS still OK (content match); html FAILs

cd <worktree at track> && (cd ui && vite build --outDir /tmp/d3)
bash "$scripts" ~/Documents/GitHub/seatloom/target/release/bundle/macos/SeatLoom.app /tmp/d3
#   FAIL exit 1, 6 of 7 NOT EMBEDDED, and step 5 reports "none" (the favicon fix works)

# ── tilde: the cause, the ledger, the mutations ─────────────────────────────
git show track/infra-foundation:ui/src/lib/dto-to-v1.ts | grep -n "homePath ?? null"
git show 9a5a715~1:ui/src/lib/dto-to-v1.ts            | grep -n "'~/' + d.name"

podman exec seatloom-postgres psql -U seatloom -d seatloom -c \
  "select id,name,home_path from projects where name ilike '%network%';"
podman exec seatloom-postgres psql -U seatloom -d seatloom -t -c \
  "select count(*), count(home_path) from projects;"      # 39 | 39

# mutation 1: restore '~/' + d.name  -> 2 red
# mutation 2: path: d.name           -> 2 red (the lazy fix is caught too)
# new test file onto 9a5a715~1       -> 2 red

# ── the deployment gap ──────────────────────────────────────────────────────
stat -f '%Sm' /Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri   # 19:12
ps -eo lstart,command | grep seatloom-tauri | grep -v grep               # started 19:46
#   both fixes: 20:01 and 21:10 — the installed app predates them
```

## Verdict event

Emitted as **`ev-hh9xak8wlN5fJst6107cy`** (the pre-declared id), `ReviewVerdictIssued`,
with `dispatch_id` naming each implementation dispatch in the payload — the contract
spelling, per Aegis's earlier correction. Verdict **pass** for both fixes, with the
findings above recorded in the payload rather than left in prose.

## Files I did not change

No source file, no migration, no live-database write. Two worktrees were created at
`track/infra-foundation` and `9a5a715~1`, both removed afterwards; `git status` in
`packet/nimbus/project-scoped-reads` is empty. Scratch dists under `/private/tmp/` deleted.
The one mutation I made to Nimbus's source was reverted and the tree confirmed
byte-identical to the ref.
