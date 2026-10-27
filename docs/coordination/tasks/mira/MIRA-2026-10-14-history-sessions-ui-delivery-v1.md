# Delivery — `disp-history-sessions-ui`: historical sessions by project, slice 3 (the UI)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-14-history-sessions-ui-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-history-sessions-ui |
| depends_on | LYRA-2026-10-14-history-sessions-in-ui-contract-v1 |
| tags | history-sessions, ui, slice-3, delivery, mira |

> The dispatch named this file with its own issue date; the `date` field above is the
> day the work was actually finished. Both are recorded rather than silently
> reconciled.

---

## 0. The one-paragraph version

All three screens are built inside the existing `sessions` tab: project list → session
list → the conversation itself. Every empty and missing state says **why** and **what is
next**. The three read commands are really called, their rejections are shown verbatim,
and there is **no** hand-written switch in the source asserting that a command is wired.
Gates: frontend 93/93 tests (21 new), `tsc` clean, `eslint` + `prettier` zero new issues,
`vite build` green; Rust 251/251 tests, `clippy -D warnings` clean.

## 1. Gate checked before touching anything

- The dependency edge `ev-history-sessions-read-live` **is in the ledger**
  (SupervisorMessage, 2026-10-16T05:04:08, `seat:seat-aegis-001`) and it carries a
  `verified` field — a real call to `list_history_projects` against the live database
  returning 38 / 183 / 36680. An announcement would not have been enough, and this is
  not one.
- I re-measured it myself against the same binary (see §6), and the numbers agree.
- I read the contract in full (§1–§8), the original task statement of
  `disp-history-sessions-read`, and Nimbus's slice-2 delivery before writing any code.

## 2. What was built

| File | Change |
|---|---|
| `ui/src/app-v2/panel/HistorySessionsPanel.tsx` | **new** (983 lines). The three screens, every §3.2 state, every §3.3 failure path |
| `ui/src/app-v2/panel/HistorySessionsPanel.test.tsx` | **new** (21 assertions) |
| `ui/src/app-v2/panel/TmuxMirrorWorkspace.tsx` | **new**. The tmux mirror, moved verbatim (see §3) |
| `ui/src/app-v2/panel/SessionsWorkspace.tsx` | **now a two-mode shell**: an in-tab switch between the history browser and the tmux mirror, defaulting to history |
| `ui/src/app-v2/panel/__fixtures__/history-measured.json` | **new**. The measured snapshot (38 / 183 / 36680 / 49 / 14 / 14) |
| `scripts/refresh-history-ui-fixture.sh` | **new**. Re-runnable: drives the same binary, re-measures, rewrites the snapshot |
| `ui/src/lib/types-dto.ts` | **additive only**: 7 history interfaces, mirroring `crates/seatloom-core/src/history/schema.rs` field for field |
| `ui/src/lib/api.ts` | **additive only**: 5 entries (3 reads + 1 local slice read + 1 open) |
| `src-tauri/src/commands/history_cmds.rs` | **new**. Nimbus's 3 thin wrappers (verbatim) + my 2 slice-3 commands |
| `src-tauri/src/commands/mod.rs`, `src-tauri/src/main.rs` | **additive only**: 6 registration lines, **0 deletions** |

## 3. About that move (stated up front so it does not read as a rewrite)

`SessionsWorkspace.tsx` went from 212 lines to 70 — not because the tmux mirror changed,
but because it moved **verbatim** into `TmuxMirrorWorkspace.tsx`. The moved code is
semantically identical apart from the component name; every difference is prettier's
line wrapping. Reproducible check:

```bash
diff <(git show track/infra-foundation:ui/src/app-v2/panel/SessionsWorkspace.tsx \
        | sed -n '/^export const SessionsWorkspace/,$p' | sed 's/SessionsWorkspace/TmuxMirrorWorkspace/g') \
     <(sed -n '/^export const TmuxMirrorWorkspace/,$p' ui/src/app-v2/panel/TmuxMirrorWorkspace.tsx)
```

Reason for the move: two workspaces now share one tab, and neither should have to
re-layout for the other — which is the contract's "do not re-layout" in practice.

## 4. Where I crossed a line, and why (three items, all checkable)

**4.1 I integrated Nimbus's three Tauri thin wrappers.** They exist, compile, and are
his slice-2 deliverable on `packet/nimbus/gate-vs-announcement` @ 0505636 — but they are
**not on track**. Without them the UI can call nothing and acceptance-card step 1 cannot
pass. The integration is path-limited (6 registration lines, 0 deletions) and **I checked
the deletion side first**: taking his whole `main.rs` would delete `repo_root()`'s HOME
fallback, which is the exact failure a path-limited integration already caused once on
2026-10-14 (his branch predates that fix). The code is his, not mine; the file header says
so.

**4.2 I added two Tauri commands the contract does not mention.**
`cmd_read_history_context_slice` (read a local context slice) and `cmd_open_local_path`
(open with the OS). The reason is direct: slice 2 returns **pointers only** by 011's rule,
and nothing in the stack could dereference them — so acceptance-card step 3 ("the
operator can read the content") was impossible. Both read the operator's own local file; **nothing is written to the
database and nothing crosses MCP**, so 011's rule stands. The slice command is confined to
files under `<repo root>/resume-contexts` and refuses anything else with the path named
(pinned by Rust unit tests).

**4.3 I added one env line inside his `call_tool`.**
`SEATLOOM_HISTORY_CONTEXTS_DIR` now points at `state.repo_root/resume-contexts`. Before,
the child inherited the parent's cwd, so launching from `/Applications` made the contexts
directory `/resume-contexts` — the same failure mode `repo_root()`'s HOME fallback already
guards against for the binary path. With this line both follow the same root, and a missing
directory is reported as that path rather than as an empty list.

## 5. One interpretation for Lyra/Aegis to confirm

Contract §4 says the entry is "the existing project switcher and the existing `sessions`
tab". I did **not** wire the history projects into the NavRail's `ProjectSwitcher`, and the
reason is measured: migration 021 wrote the 38 home paths into the `projects` table, so
that switcher already lists them — but selecting one changes the whole V1 application's
active project (inbox, work items, everything re-scopes). That would blend "browse my
history" with "switch working project" into one gesture. So the history project list lives
in the sessions tab's own three columns (column one *is* the switcher for this corpus): no
new top-level navigation, no new route, no new window. If the contract meant the former,
this is the implementation to change; I chose the latter on the evidence.

## 6. Measured, not reported

Driving the **same binary the Tauri app spawns** (`target/debug/seatloom-mcp-emit`) with
the env `call_tool` sets:

```
list_history_projects                 -> projectCount 38 / sessionTotal 183 / messageTotal 36680 / corpusLoaded true
list_history_sessions(OfficeBench)    -> sessionCount 49 / undatedCount 0
list_history_sessions(seatloom)       -> sessionCount 14 / undatedCount 1
get_history_session_turns(claude-code, 621ae382…) -> turns 75, sourcePresent true,
                                                      countsAgree true, first turn at original line 7
```

Cross-checked against the **source corpus**, independently of the database:
`resume-contexts/by-session/` holds 183 files, `SESSIONS-FLAT.csv` has 183 data rows, the
per-file message-count headers (the source's own message-count line, parsed as a
number and never re-typed) sum to 36680, and there are 38 distinct home paths — all four
agree. The failure path was measured too: pointing `contexts_dir` at a missing directory
returns a refusal naming the missing directory and the env var that points at the right
one, and the UI shows exactly that text.

The application binary `target/debug/seatloom-tauri` was rebuilt (13:35); all five command
names are present in it (`strings` verified), the process starts with a clean log and exits
cleanly. **What I could not do:** click through the window — I work from a shell and cannot
see the interface. So "open SeatLoom and see 38 projects" rests on compile-level evidence,
unit-test-level evidence, and a measured read path, not on a click. Mr. Zhang's click will
be the first one.

## 7. §3.2 / §3.3, item by item

| Contract item | Implementation |
|---|---|
| A project with no sessions | Renders with `sessionCount === 0` and says why + what is next. Defensive by design: the read only returns projects with at least one session, so this state should not normally occur |
| A session whose harness produced no conversation | The project row says how many sessions have no extracted conversation and why (e.g. cursor's workspace record); the session row repeats it; the detail uses the tool's own `emptyReason` |
| A session with no timestamps (all 14 cursor) | The literal operator-visible string for "timestamps not recorded" (pinned verbatim in `HistorySessionsPanel.test.tsx`) plus an explanation that this harness records no timestamps, so there is no date and it is not sorted as the oldest; placed last and labelled, never interleaved |
| The 8 repository-root sessions | Their own group, titled with the literal operator-visible "unclassified (repository root)" label (pinned verbatim in the test file) and the reason: it strips to the empty string, so any parent would be a guess. **Moved to the end**, never folded into a neighbour |
| A message whose original file is gone | Says the file is no longer on disk, and that the line number survives while the content does not — **the turn is still shown** |
| The read command failing | The measured error text, marked as the backend's own words; never a forever-spinner, never a silent empty list |
| The corpus not loaded yet | The literal operator-visible "the historical sessions are not loaded yet" (pinned verbatim in the test file) plus the next step (apply migration 021); no empty project list, because an empty list reads as "you have no history" |
| A count that disagrees with the source | `countsAgree === false` says it is a defect, not a rounding difference, and that the contract says to reject it; `null` (one side absent) says the two cannot be compared rather than pretending they were |

## 8. Wiring honesty (the hard constraint)

- **No hand-written switch.** There is no `…_REGISTERED` constant. One test reads the
  source and forbids that shape, and asserts the panel goes through `api.*` rather than a
  bare `invoke`.
- **An unreachable backend says so.** Outside Tauri (browser `pnpm dev`) the panel says the
  database and the disk are both on this machine only, and names the command to run. It
  never renders an empty list that reads as "you have no history".
- **An unloaded corpus says so.** `corpusLoaded: false` → the not-loaded string + the next
  step, and no project list at all.
- **Rejections are verbatim.** Whatever text the backend returns is what is shown, labelled
  as measured rather than paraphrased.

## 9. Gates (all green; zero new issues in my files)

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **93/93** — 21 new: the six numbers; per-project sums equal the totals; project ordering with the root group last; 49 rows for OfficeBench; dated/undated never interleaved (checked across all 38 projects); 75 turns in source order with original line numbers and the real text; a missing source file still shows its turns; a count disagreement reported as a defect; an empty session explained with no empty frame; four failure paths; the browser state; no hand-written switch |
| `scripts/lint-changed.sh track/infra-foundation` | eslint 0 errors, prettier clean |
| `vite build` | green |
| `cargo test --workspace` | **251 passed / 0 failed** — 3 new `history_cmds` unit tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | one red: `src-tauri/src/main.rs:88` — **already red on track** (the HOME fallback block's indentation), not introduced here; I did not re-indent someone else's block |

## 10. Artifact index

- Branch: `packet/mira/knowledge-panel` (with a merge of `track/infra-foundation` so the
  read path is present for slice 3 to compile and be measured against; re-synced to
  `a587f57` immediately before the acceptance request, because the gate refuses a branch
  that does not contain its base).
- Measured snapshot and the script that refreshes it:
  `ui/src/app-v2/panel/__fixtures__/history-measured.json`,
  `scripts/refresh-history-ui-fixture.sh`.
- Ledger: see `emit_event(ArtifactCreated)`.
- Acceptance card: requested with contract §6 verbatim; merges only on the operator's
  Accept click.

## 11. One ordering fact worth recording

The dispatch's done-definition lists the acceptance-card request *before*
`complete_dispatch`. The tool will not accept that order: `request_acceptance` refuses a
dispatch that is not `done` (013/014: only a done dispatch of the calling seat may be
handed to the gate). So the actual order had to be **`complete_dispatch` first, then
`request_acceptance`** — the guard is the constraint, and the prose cannot be followed
literally. Recorded here so the next seat does not lose time discovering it, and so a
reader of the ledger does not mistake the sequence for a seat that skipped its card.
