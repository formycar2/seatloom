# Verdict — the project-filtering fix, both layers: Nimbus's backend scoping and Mira's UI

- **Verification dispatch**: `disp-8jqnVUeRz1AZ4nVWrUJVE` (issued by seat-aegis-001), covering
  - `disp-_Xfb_jysuptYTOT4pCBx_` — Nimbus: backend filters by project (`packet/nimbus/project-scoped-reads` @ `94daf79`)
  - `disp-RUnXBR3uICuWmmFizJ7vX` — Mira: the UI passes the project (`packet/mira/knowledge-panel` @ `21633ad`)
- **Verdict event**: `ev-UKSo1P85Leoi8bm0hRjfB` — the pre-declared id, emitted exactly as required,
  with one `ref_type='dispatch'` context ref per covered dispatch. **Both dispatches are covered: I
  verified both halves.**
- **VERDICT: PASS for both** — with three findings that belong in the record: one residual silent
  default in a command outside the five, one module missing from Mira's per-module accounting, and
  the fact that the two branches **conflict on three files**, so the integration is a design
  decision, not a merge.

## A. Gates — run by me on each branch

| Gate | Nimbus @ `94daf79` | Mira @ `21633ad` |
|---|---|---|
| `cargo fmt --all -- --check` | clean | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean | clean |
| `cargo test --workspace` | **278/0** | **275/0** |
| `tsc --noEmit` | clean | clean |
| vitest | **194/194** | **199/199** (7 new) |
| his/her proof | `project_scope_proof.sh` **10/10** (run by me) | — |

## B. The five commands against live, called by me

I built and ran Nimbus's committed probe (`project_scope_probe`) myself against live, project by
project — not read off his proof:

| read | seatloom | ql_delivery | ColossalAI (corpus) |
|---|---|---|---|
| seats | **7** — aegis, flux, lyra, mira, nimbus, onyx, vela — **no QL** | **1** — `seat-ql-001` only | 0 |
| workitems | 7 | 0 | 0 |
| handoffs | 3 | 0 | 0 |
| sessions | 5 | 0 | 0 |
| events | 400 | 0 | 0 |
| dispatches | 200 | 0 | 0 |

Negative control: unscoped seats = 8 vs scoped 7; a project that does not exist → 0 dispatches. The
numbers MOVE, which is the operator's report inverted.

## C. No project_id → an error, never a default

`require_project_id` refuses with a message naming the command and the reason ("An unscoped read
would return every project's rows at once, which is the defect being fixed"). Verified at three
layers: the helper's **3 unit tests** (absent → error; supplied → passthrough; the message names the
reason); **tsc** — all ten `api.ts` read signatures now require `projectId`, so an unscoped call site
does not compile; and **grep** — no fallback remains in the eight scoped commands (the
`acceptance_cmds.rs` hit is the comment recording the removed one). The IPC-level error string is
not exercisable from my seat (no app-IPC driver); the three layers above are what is reachable.
**Residual, named by his own proof and confirmed by me:** `cmd_list_role_bindings`
(`seat_cmds.rs:62`) plus `supervisor_cmds` / `reconcile_cmds` / `knowledge_cmds` / `prompt_cmds`
still carry the same `unwrap_or_else(default_project_id)` fallback. Latent, not active — the UI
always passes an id — but the same shape, unfixed, in five more commands.

## D. Seats are filtered by JOIN, because `seats` has no `project_id`

`list_seats_for_project` joins `project_role_bindings` (a `WHERE` on `seats` would be a syntax
error, not a filter). Measured on live: SeatLoom 7 without QL, ql_delivery exactly 1
(`custom(worker)`), and the historical-corpus projects 0 **with the reason measured** — zero
binding rows, not an error and not a broken-looking empty list.

## E. Artifacts: he chose "state it cannot be partitioned" — honest at the code layer, silent at the screen

Measured (by him, re-verified in his proof's POINT 9, and consistent with my own read of the
schema): 15 live artifact rows, 0 with `source_session_id`, 4 with `source_workitem_id`, 11 with
neither — a derived join would attribute 4 and silently drop 11. So the command **takes no
`project_id`** and the DTO carries no project field: no caller can believe it was scoped, and the
operator's option is a migration script on the branch, marked as his act. **Judgment: the right
choice, honestly recorded — with one gap.** The store comment says "the tab must not imply
otherwise", but the Artifacts tab itself shows no note: the same 15 rows appear under every project
with nothing on screen saying they are unscoped. The "明说" reaches the code reader, not the person
looking at the screen — which is the audience this bug was reported by.

## F. Mira's half — the project reaches every call that accepts one, and the unresolved state says so

- **Dispatches** (the one whose backend accepts a project): the call site now passes
  `activeProjectId`, and `activeProjectId` joined the load callback's deps, so switching reloads
  instead of showing the previous project's rows until the next 5s poll.
- **The four the backend does not accept** (seats/workitems/handoffs/sessions, plus events and
  documents): filtered at the store by each row's own `projectId` before the mapper — her delivery's
  per-module table says exactly this, and the table matches the code.
- **No silent global**: with no project resolved, or hydration failed, the inbox renders an explicit
  amber state ("还没有选定项目 —— 下面这些不是你的派单"); WorkItemsView and TimelineView say
  "请选择一个项目". Pinned by 3 new tests; the two key filtering tests ("work items, handoffs,
  sessions and events land under their OWN project only"; "documents are filtered too, and the
  runtime artifacts are NOT — and that is stated") pass by name.
- **Finding — one hole in her accounting.** Her per-module table lists six modules; the supervisor
  panel's **continuity-delta card and its reachability probe are not in it**, and on her branch they
  still read globally (`api.listSeats()`; `api.listDispatches(undefined, 200)` +
  `listHandoffs()` + `listEvents(200)`) — which on her branch's backend is the **silent default**,
  i.e. seatloom's rows presented as the fleet view. Nimbus's branch fixed exactly those call sites
  (both now take `activeProjectId` and say "no project selected" when there is none), so the
  integrated product is fine — but her "一个不漏" claim is not complete as delivered, and the hole
  is in the one surface the supervisor lives in.

## The app layer — built from each branch, running; the pixels remain unobservable

Per the dispatch I built from **each** branch with `prepare-verification-build.sh` (not from main):
Nimbus's build installed and running (pid 42710) — the binary carries the guard strings
("project_id is required", "An unscoped read would return every project's rows at once"); Mira's
installed and running (pid 43960) — carry proven by the asset linkage (`main-DulPlQhu.js` embedded
in the binary, and it contains the new strings). **I could not see either app**: `screencapture`
fails ("could not create image from display", measured a third time, also after `caffeinate`) and
assistive access is denied — the screen is locked from my seat. So the operator-facing acceptance
("切到 SeatLoom:7 个没有 QL … 切到 ql_delivery:只有 ql 一个") is **his to see**; what I verified is
the read path under it.

## The integration finding — the two branches are not file-disjoint

Aegis's dispatch said the two branches' changes are in different files. They are not: a test merge
of the two branches **conflicts on three files** — `ui/src/stores/useDataStore.ts`,
`ui/src/views/DispatchInboxView.tsx`, `ui/src/views/DispatchInboxView.test.tsx`. The two halves are
two different answers to the same second layer (Mira: filter client-side in the per-project loop;
Nimbus: read one project server-side and re-read on switch), so the merge is a **design decision**,
not a mechanical resolution. The direction is clear — Nimbus's store and backend as the base, with
Mira's no-project guards merged in (his SupervisorIntent is already the project-aware version of
hers) — but someone must own that resolution, and neither branch alone is the product.

## What this verdict does not assert

- The rendered app under either build (screen locked — the operator's click-level acceptance).
- The IPC-level error text of `require_project_id` (no app-IPC driver from a seat; the helper, the
  types and the grep are the reachable layers).
- Anything about the five other commands' fallbacks beyond naming them.

## Bottom line

**PASS for both halves.** Nimbus: the five reads scope correctly against live (measured by me), an
absent project is an error rather than a default at every reachable layer, seats filter through the
binding join, and artifacts refuse to pretend. Mira: the project reaches every call that accepts
one, the unresolved state says so instead of showing another project's rows, and her per-module
accounting matches the code — with the supervisor continuity card missing from it. The record also
carries: five more commands with the same latent fallback, the Artifacts tab's silence about its
unscoped half, and the three-file conflict that makes the integration a decision rather than a
merge.
