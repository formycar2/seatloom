# Delivery: the Projects page reads the database, and a failed read says so

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-projects-page-hydration-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-KnJYxcf7_Zze8MhPRFNJM |
| tags | hydration, projects-page, silent-fallback, delivery, mira |

---

## 0. The one-paragraph version

The store no longer starts from three hardcoded seed projects, so the product path
cannot show them; a failed hydration now renders a red notice carrying the backend's own
error text and the next step; and the Projects page distinguishes "the read failed" from
"the database is empty". I could **not** reproduce which of the eight reads throws — I
establish below why that was invisible for so long, and the fix makes it self-reporting.

## 1. What I could establish, and what I could not

**Established — and it is the reason this survived.** `hydrateFromBackend` has always
caught its failure and stored it in `backendHydrateError`. I grepped the whole UI for that
field: **nothing ever rendered it.** `backendHydrated` feeds only the provenance badge. So
a failed hydration was structurally invisible — the store simply kept its previous value,
which until now was three hardcoded seed projects. That is why the operator saw three
plausible project cards and concluded the product had three projects, while `ql_delivery`
sat in the database the whole time.

**Not established — which read throws.** The dispatch asked me to dig it out. I tried, from
a shell, and could not:

- ran the installed app and captured stdout/stderr — 0 bytes (a release Tauri webview does
  not forward `console` to either stream);
- queried the macOS unified log filtered to the process — only AppKit/XPC noise, nothing
  from the frontend;
- checked the alternative the dispatch ruled out anyway (commands registered, tables
  readable, entry point calls hydrate) — all confirmed fine, so the failure is a runtime
  one I cannot observe without the webview console.

I am not going to guess which command it is. The dispatch's own rule is that a guess
dressed as a finding is worse than an admitted gap, and this project has paid for that
shape repeatedly. What the fix does instead is make the error **appear**, so the next
person with the console — or the operator reading the screen — sees the exact text.

## 2. What changed

| # | Requirement | Change |
|---|---|---|
| 1 | no silent fallback | `hydrateFromBackend`'s catch already stored the error; **the error is now rendered** |
| 2 | page reflects the DB | store starts **empty**, so `projects` is only ever what the database returned |
| 3 | failure is loud | new `HydrationFailure`, mounted once above the shell, on every tab |
| 4 | seeds off the product path | `INITIAL_PROJECTS` / `INITIAL_DATA` renamed to `SEED_PROJECTS_FOR_TESTS` / `SEED_PROJECT_DATA_FOR_TESTS` and **removed from the store's initial value** |

**`HydrationFailure`** states that the read failed, that what is above is not the
operator's projects, prints the backend's error verbatim, and gives three ordered next
steps: check the container is up (`podman ps` then `bash scripts/bootstrap.sh`), press the
re-read button (a transient connection blip), and if neither works hand the exact error
text to Aegis. It deliberately does **not** retry automatically or guess which of the eight
reads failed — `Promise.all` rejects with the first rejection and abandons the other
seven, so naming one would be a guess.

**`AllProjectsView`** now has two distinct states instead of one empty grid:

- read failed -> a notice saying the projects have not been read yet and that this is not
  because the operator has none, plus the error text;
- read succeeded, zero rows -> a notice saying the database holds no projects and that this
  read did succeed.

Those are different facts and confusing them is exactly the failure being fixed.

**The browser-dev branch** of `hydrateFromBackend` no longer sets
`backendHydrateError: null` with the comment "Not an error". It now records the reason:
there is no Tauri backend in a browser, so there is nothing to hydrate from.

## 3. The seeds

Kept for tests, exported under names that say what they are. The store's initial value is
`projects: []` / `projectData: {}`. A test pins both halves: that the shipped initial
value is empty, and that the seeds are still reachable under their new names with their
original ids (`p-1`/`p-2`/`p-3`) and paths.

## 4. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **187/187** (8 new) |
| `vite build` | green |
| `cargo test --workspace` | 251 passed / 0 failed (no Rust changed) |
| `scripts/verify-docs.sh` | this note clean |

**Lint, measured both ways.** track vs mine: `useDataStore.ts` 2/2, `AllProjectsView.tsx`
0/0, `App.tsx` 15/15 — **zero new**.

## 5. Acceptance

- Projects page shows `ql_delivery` (and `ThoughtOnly`, `DeepSpeed`, …), total matching the
  database. Reject: still three, or `ql_delivery` missing.
- With the container stopped, opening the app says plainly that the database is
  unreachable, what is missing, and what to do. Reject: the three fake cards reappear.

**One caveat on the first criterion, stated rather than hidden:** I verified the page
renders whatever hydration returns, using the real mapper (`projectFromDto`), in a test
that mounts the view. I did not verify it against the live 39-row set in the running app,
because I cannot observe the running app from here.

## 6. Boundaries

No skeleton re-layout. No live-database write. No gate event. No merge.
`ui/src/lib/api.ts` and `ui/src/lib/types-dto.ts` untouched.

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
