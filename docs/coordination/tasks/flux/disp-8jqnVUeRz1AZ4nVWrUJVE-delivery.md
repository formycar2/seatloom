# Delivery — `disp-8jqnVUeRz1AZ4nVWrUJVE` (Flux): the project-filtering fix, both layers verified

- **Dispatch**: `disp-8jqnVUeRz1AZ4nVWrUJVE`, issued by seat-aegis-001, kind `verification`.
- **Covers**: `disp-_Xfb_jysuptYTOT4pCBx_` (Nimbus, backend) **and**
  `disp-RUnXBR3uICuWmmFizJ7vX` (Mira, UI) — both verified, both named in the verdict's refs.
- **Verdict**: **PASS for both**, with three named findings.
- **Verdict event**: `ev-UKSo1P85Leoi8bm0hRjfB` — the pre-declared id, emitted exactly as required.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-17-project-filtering-verdict-v1.md` | new — the verdict: every requirement with the measurement behind it |
| `docs/coordination/tasks/flux/disp-8jqnVUeRz1AZ4nVWrUJVE-delivery.md` | this note |

No source file was changed. No live writes, no gate events, no merges. The two verification builds
came from each seat's own branch via `prepare-verification-build.sh`, as the dispatch required.

## What was verified, per requirement

- **A** — gates on both branches: fmt/clippy clean, Rust 278/0 (Nimbus) and 275/0 (Mira), tsc
  clean, vitest 194/194 and 199/199, Nimbus's proof 10/10 (run by me).
- **B** — the five reads called by me against live, project by project: SeatLoom 7 seats without QL
  / 7 / 3 / 5 / 400 / 200; ql_delivery exactly 1 seat and zeros; a corpus project 0 with the reason
  measured (no binding rows). Negative control: unscoped 8 vs scoped 7; unknown project → 0.
- **C** — absent `project_id` is an error, never a default: the helper's 3 unit tests, tsc (ten
  signatures now require it), and grep (no fallback in the eight scoped commands). Residual:
  `cmd_list_role_bindings` + four others still carry the same fallback — latent, named.
- **D** — seats filter through the `project_role_bindings` JOIN (a WHERE on `seats` would be a
  syntax error); the live numbers above are the proof.
- **E** — artifacts: he chose "state it cannot be partitioned" (no `project_id` param, no project
  field on the DTO, 15/0/4/11 measured, migration script for the operator). Right at the code layer;
  the Artifacts tab itself says nothing on screen — the "明说" reaches the code reader, not the
  operator.
- **F** — Mira: the project is passed on every call that accepts one (dispatches, with the reload
  deps), the four the backend refuses are filtered at the store by the row's own `projectId` (her
  per-module table matches the code), and no-project/hydration-failed render explicit states rather
  than another project's rows. Hole in her accounting: the supervisor continuity card and probe are
  not in her table and still read the silent default on her branch (Nimbus fixed those call sites).

## The integration finding

The two branches **conflict on three files** (`useDataStore.ts`, `DispatchInboxView.tsx`,
`DispatchInboxView.test.tsx`) — they are two answers to the same second layer, so the merge is a
design decision (Nimbus's server-side store as the base, Mira's guards merged in), not a mechanical
one. Aegis's "different files" premise does not hold.

## Not verified, stated

The rendered apps (screen locked from my seat — screencapture fails, assistive access denied; the
click-level acceptance is the operator's), and the IPC-level error text (no app-IPC driver from a
seat; the helper, the types and the grep are the reachable layers).
