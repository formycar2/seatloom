# Delivery: isolated combined project-filtering candidate

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-19-project-filtering-combined-v1 |
| status | candidate; independent Flux verification required |
| author | nimbus |
| date | 2026-10-19 |
| version | v1 |
| tags | project-filtering, integration, mirror, async-race |
| dispatch | disp-9H3OCmprBFAkiYBa5MDwE |

## Candidate and provenance

Independent worktree: `/tmp/nimbus-project-filtering-combined`.
Branch: `packet/nimbus/project-filtering-combined`.
Base: `98876aa931aa144ab873d9deba5aae9ed972931f` (track tip at creation).
Product commit: `2c69adbfaa5459c7372b9f43248cf9bb6868dbc6`.
Neither original delivery branch was moved or merged into track.

Read Flux's complete `FLUX-2026-10-17-project-filtering-verdict-v1.md`, including
its three-file conflict finding, artifact disclosure gap, and verification exclusions.

Sources, covering every product change:
- Nimbus source `76e2dc3` (`packet/nimbus/project-scoped-reads`), filtering implementation
  originally `387e7aa`: scoped DB seat join, Tauri command arguments, IPC wrappers,
  per-project store reads, supervisor reads, docs workspace and acceptance reads.
  Took only `crates/seatloom-core/src/db/repositories.rs`, the core scope probe,
  project-scope proof, `src-tauri/`, and related `ui/src/` diffs against track.
- Mira source `09326c3`, filtering work originally `bc30966`, verified at `21633ad`:
  copied the project-unresolved guards in WorkItemsView and TimelineView; ported the
  inbox no-project/hydration guard, retaining Nimbus's explicit scoped request.
- Nimbus artifact option: copied only `015_artifacts_project_id.sql`, not applied.
- Candidate-only fixes: `project-scope.ts`, race tests, bound mirror discovery and
  terminal rendering, artifact screen disclosure, hydration failure propagation,
  stale-response checks, supervisor summary readiness and removal of the unscoped
  continuity checkpoint, and explicit-project guards for five residual commands.
- Test adaptations: AllProjectsView hydration mock adds the explicitly global seat
  registry; LiveSeatsView mock models project bindings; existing inbox/seat tests
  model scoped reads. New integration tests preserve Mira's no-project intent.

The exact 38-file change set is reproducible with:
`git diff --name-only 98876aa 2c69adb`.
No original branch was merged wholesale. No unrelated feature branch, memory tool,
lineage DTO or launcher/default-model change was added to the candidate.

## Three-file resolution

`useDataStore.ts`: server-scoped reads win over copying global arrays into a
per-project loop. Mira's project-isolation intent is retained without adding a
second client-side membership authority. During switches, hydration becomes
unresolved; successful/failed old-project reads cannot publish into the selected
project. Binding-read failures propagate instead of silently inventing roles.

`DispatchInboxView.tsx`: explicit project argument and request guards from Nimbus,
no-project/unhydrated notice from Mira. Never call with an undefined project merely
to suppress its resulting default rows afterward. Selection checks reject delayed
responses. Scoped seat lookups and event windows have cleanup guards.

`DispatchInboxView.test.tsx`: retained original dispatch behavior tests, updated
mock signature `(projectId, limit)` and explicit hydration precondition. Added
no-project and failed-hydration tests in `projectFilteringCombined.test.tsx`, with
an actual on-screen artifact disclosure assertion.

## Actual mirror filtering

LiveSeatsView previously discovered all tmux sessions regardless of store seats.
It now reads `listSeats(activeProjectId)` alongside tmux discovery and intersects
parsed session seat names with registered bound seat names, including Aegis-Supervisor.
Only bound sessions and their terminal panes render. Existing attachments are retained
but hidden when outside the selected project: switching projects does not terminate
agents or destroy their mirrors. The registered-name mapping is the current fleet
convention; future arbitrary session names require an explicit registry mapping.

A test switches projects after a real component auto-attach and verifies the old
terminal disappears while its durable connection remains. A separate mapping test
covers Aegis/Nimbus versus QL and no bindings.

## Remaining five fallback commands: changed, outside old verdict

The old Flux verdict explicitly did not verify these beyond naming them. This
candidate removes their default-project fallback, requiring a supplied project:
role bindings, knowledge tree, prompt instances, reconcile and supervisor message.
Relevant UI knowledge/reconcile callers now supply the active project.
These additional behavior changes need independent verification, especially the
write-side reconcile and supervisor-message refusal. No write call was executed.

Global-only reads are explicit exceptions: `listAllSeats` supplies contextual seats
not bound to THIS project; tmux discovery is global before intersection; runtime
artifacts remain global. Continuity no longer substitutes a global checkpoint for
this project's history; it reports no checkpoint instead.

## Artifacts

No live schema change. Runtime artifacts have no project key, so they remain
unfiltered. ArtifactsView now visibly states that runtime records are global and
are not attributed to the selected project; documents are scoped. The optional
operator migration is carried unchanged and has no guessed backfill. Flux previously
noted that this disclosure existed only in comments; a component test now checks
that the operator-facing notice is rendered.

## Verification from candidate

- `cargo fmt --all -- --check`: pass.
- `cargo check --workspace --all-targets`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo test --workspace`: 278 passed, 0 failed; DB-dependent tests ignored.
- `ui`: `tsc --noEmit`: pass; vitest 201 passed, 0 failed.
- `vite build`: pass, with existing chunk-size advisory; built candidate assets only,
  did not install or replace running app.
- `project_scope_proof.sh`: ALL POINTS PASS, including controlled scratch DB tests.
- `git diff --check`: pass.

Read-only real DB probe, run from candidate:

| read | seatloom | ql_delivery | ColossalAI |
|---|---:|---:|---:|
| seats | 7 | 1 | 0 |
| workitems | 7 | 0 | 0 |
| handoffs | 3 | 0 | 0 |
| sessions | 5 | 0 | 0 |
| events (limit 400) | 400 | 0 | 0 |
| dispatches (limit 500) | 206 | 0 | 0 |

Corpus project has no binding rows, hence zero seats. Dispatch counts are observation
snapshots, not permanent assertions. Unscoped seats 8 versus scoped 7; nonexistent
project dispatches 0. No live rows altered.

Deferred-response test loads B before resolving old A and asserts selected project B,
B data present, A result not published, and B hydration still successful. Switching
immediately marks hydration unresolved. These are jsdom/store tests, not observations
of the installed application's screen.

## Boundaries and next step

The old Flux verdict covers two separate sources, NOT this combined commit. No new
pass verdict is claimed. Send candidate to Flux for independent integration review.
No application IPC driver: exact IPC error text remains unverified beyond helper,
types and build checks. No screenshot or operator clicks observed: click-level
acceptance remains unverified. No card opened, no gate event, no merge into track,
no live write, no application installation, no model switch or session restart.

Build-only `ui/node_modules` symlink and `ui/dist` are local dependencies/assets,
not source changes; the symlink is not included in the commit.
