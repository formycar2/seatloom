# SeatLoom Engineering Standards

| Field | Value |
|---|---|
| template | T7 |
| subtype | engineering_standards |
| id | engineering-standards-v1 |
| status | active |
| author | aegis |
| date | 2026-10-27 |
| version | v2 |
| depends_on | `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `.gitlab-ci.yml` |
| tags | engineering, ci, code-standards, quality-gates, ratchet, acceptance, verification-gate-class |

> Changelog: v1 (2026-09-29) aegis — initial standard. v2 (2026-10-27)
> lyra — added `## Acceptance gate class` (decision
> `docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`).
> v2 keeps id `engineering-standards-v1` so existing `depends_on` and document
> references stay resolvable; the version row records the content revision.


## Purpose

One authoritative place for SeatLoom's code standards and the CI gates that
enforce them. Before this document, ~90 written governance rules existed and
exactly one was machine-enforced (the `-seatloom` tmux suffix in
`crates/seatloom-core/src/pty/mod.rs`). This turns the enforceable subset into
CI on the internal GitLab instance.

## Guiding principle — ratchet, not big-bang

Build while using. Every gate is one of two classes:

- **FULL** — checks the whole tree. Used only where the baseline is already
  green (verified 2026-09-29): `cargo fmt`, `cargo clippy -p seatloom-core`,
  `cargo test`, `tsc --noEmit`, `cargo check --workspace`.
- **RATCHET** — checks only files changed vs the base ref; pre-existing debt is
  grandfathered. Used where the baseline has debt that would otherwise block
  every MR: `eslint`, `prettier`, and the doc header/naming/English checks.

Debt is deferred, never hidden. When you fix a grandfathered file, remove it
from its grandfather list — the ratchet only ever tightens.

## Repository topology

GitLab (`gitlab.basemind.com:zhangxiaolong/seatloom`, project 11428) is the
primary. GitHub (`formycar2/seatloom`) is a mirror. CI lives in `.gitlab-ci.yml`
and runs on the internal runner (id 705, docker executor, untagged).

Push flow: push to both remotes. GitHub's `.github/workflows/rust-foundation.yml`
is retained as a fast redundant fmt/clippy/test check; GitLab CI is authoritative.

`main` and `track/infra-foundation` are protected (maintainer push/merge, no
force-push). Project settings enforce **pipeline must succeed** and **all
discussions resolved** before merge, with auto-cancel of redundant pipelines.

## Build environment — everything resolves internally

The runner reaches the public internet but the overseas route is throttled
(measured 2026-09-29: npmjs.org at 10–23 KiB/s with ECONNRESET;
`registry-1.docker.io` refused outright; rustup on static.rust-lang.org hung
~700s). Every dependency source is therefore pointed at an internal or domestic
mirror. Do not reintroduce a direct overseas fetch.

| What | Source |
|---|---|
| Base images | `hub.i.basemind.com/zhangxiaolong/ci/*` (Harbor, project is public so CI pulls anonymously) |
| Rust jobs' image | `rust-seatloom:1.95.0` — pinned toolchain + rustfmt/clippy + `postgresql-client` + Tauri v2 Linux libs, all baked in |
| npm | `artifactory.stepfun-inc.com/.../api/npm/npm-public/` (`ui/.npmrc`) |
| crates | Artifactory `cargo-remote` sparse index (`.cargo/config.toml`) |
| apt (image build only) | Artifactory `debian` / `debian-security` |
| rustc toolchain (image build only) | `rsproxy.cn` — Artifactory mirrors crates but not toolchains |

Lockfiles are unaffected by mirror choice: `pnpm-lock.yaml` stores
registry-agnostic integrity hashes, and Cargo's `replace-with` preserves
crates.io checksums. `rsproxy.cn` / `registry.npmmirror.com` are the documented
public fallbacks if Artifactory is ever unreachable.

### Rebuilding the CI image

Build on the amd64 sponsor workspace (native — the laptop is arm64 and qemu
emulation is impractically slow), then push to Harbor:

```
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
# Dockerfile: FROM hub.i.basemind.com/zhangxiaolong/ci/rust:bookworm
#   + Artifactory apt sources, Tauri deps, psql client
#   + rustup toolchain install 1.95.0 --profile minimal -c rustfmt -c clippy
docker build -t hub.i.basemind.com/zhangxiaolong/ci/rust-seatloom:1.95.0 .
docker push hub.i.basemind.com/zhangxiaolong/ci/rust-seatloom:1.95.0
```

Rebuild when `rust-toolchain.toml` or the Tauri system-dep set changes; bump the
tag to the new toolchain version rather than overwriting.


## Rust

- **Toolchain**: pinned to `1.95.0` in `rust-toolchain.toml` (rustfmt + clippy
  components). CI and local must use it; do not float.
- **Formatting**: `rustfmt.toml` restates stable defaults (edition 2021,
  max_width 100, Unix newlines). `cargo fmt --all --check` is a FULL gate.
- **Lints**: `cargo clippy -p seatloom-core --all-targets -- -D warnings` is a
  FULL gate. `src-tauri` and `src-cli` are not yet gated by clippy (pre-existing
  dead-code warnings — matches the prior GitHub CI scope); tracked as debt.
- **Serialization boundary rule** (learned from the runtime-triple work):
  enums that cross the Rust⇄PostgreSQL TEXT boundary MUST hand-write `FromStr` /
  `Display`, never rely on serde derive — serde's tagged representation does not
  match a flat Postgres TEXT column. Reference: the `parse_custom_tagged` pair in
  `crates/seatloom-core/src/objects/session.rs`.
- **Explicit value-domain contract rule** (learned from the same work): any data
  boundary that crosses seats or languages MUST carry a value-domain table —
  Rust type, serialized domain, PG column, nullability — reproduced verbatim in
  both governing packets. This is what kept the Nimbus/Onyx runtime-triple change
  aligned across two independent seats.

## Frontend

- **Package manager**: pnpm only, pinned via `packageManager` in `package.json`
  (corepack). The stray `ui/package-lock.json` was removed; do not reintroduce npm.
  `esbuild` is listed in `pnpm.onlyBuiltDependencies` so CI's frozen install builds it.
- **Types**: `ui/tsconfig.json` is `strict`. `tsc --noEmit` (`pnpm typecheck`) is
  a FULL gate — the tree passes today, keep it that way.
- **Lint / format**: ESLint (flat config, `ui/eslint.config.js`) + Prettier
  (`ui/.prettierrc`). The tree has ~143 eslint errors / ~87 unformatted files
  predating the standard, so these run as a RATCHET via `scripts/lint-changed.sh`
  (changed `ui/src` files only). New and changed files must be clean; fix locally
  with `cd ui && pnpm lint:fix && pnpm format`.
- **Tests**: Vitest (`ui/vitest.config.ts`, jsdom, `passWithNoTests`). No suites
  exist yet; `pnpm test` is a FULL gate that is trivially green until they do.

## Migrations and seed

- **Additive-only**; never drop or semantically rewrite a historical seed row.
- **Idempotent**: a migration must be safely re-runnable (`ADD COLUMN IF NOT
  EXISTS`, backfill guarded, `SET NOT NULL` no-op on already-not-null). Prove it
  in the packet by applying twice.
- **Evidence per seed row**; `NULL` means "not recorded" — never backfill a guess.
- **Apply order is not directory order.** Base schema `001–005` creates the
  tables the seed fills; migrations `≥006` (008 project-isolation, 009 runtime
  triple) are POST-SEED transforms that backfill seeded rows then tighten
  constraints, so they run AFTER seed. `scripts/apply-db.sh` encodes this
  canonical order and is used by CI. Numbers `006`/`007` are reserved (Aegis
  numbering ruling); do not reuse.
- **Known debt**: there is no migration runner; the `001–005 vs ≥006` split is
  convention, and `scripts/verify-postgres-baseline.sh` / `ingest-documents.sh`
  are hardcoded to local docker/podman and stale (stop at 005). A proper ordered
  migration + separate idempotent seed system is an **Onyx follow-up**.

## Git and branches

Per `COORDINATION_RULES.md §13`: `main` (protected, accepted work only),
`track/<domain>`, `packet/<seat>/<packet-id>`, `hotfix/<seat>/<topic>`. Every
code-changing packet declares its base and delivery branch. `main` is never a
scratch branch. Verifier bounded fixes use a `fix(flux):` commit prefix (§15).

**Known structural gap**: the six seats currently share ONE working tree, so a
seat switching to its packet branch switches it for everyone. Per-seat
`git worktree` (one directory per `packet/<seat>/*` branch) is the fix, tracked
as a **Nimbus follow-up**. Until then, only one seat may hold a non-`track`
branch checked out at a time.

## Documents

Per `DOCUMENT_TEMPLATES.md`: every doc under `docs/` begins with the universal
header (`template, subtype, id, status, author, date, version, tags`); template
∈ T1–T7; the doc lives in its template's directory; filenames are ASCII and
match `COORDINATION_RULES.md §4`. New/changed docs must be English (§10);
existing Chinese docs are grandfathered. Enforced as a RATCHET by
`scripts/verify-docs.sh` against `.gitlab/docs-grandfather.txt` (a snapshot of
the 207 currently-nonconforming docs). New docs must be born correct.

**Known debt**: the T7 subtype allow-list in `DOCUMENT_TEMPLATES.md §11.1` (and
its two Rust mirrors in `artifact_store.rs` + `document_parser.rs`) does not yet
include `engineering_standards`; add it when those allow-lists are next touched.
The two Rust allow-list copies should be de-duplicated.

## CI gate summary

| Job | Stage | Checks | Class |
|---|---|---|---|
| `rust:lint` | lint | `cargo fmt --all --check`; `clippy -p seatloom-core -D warnings` | FULL |
| `frontend:lint` | lint | `tsc --noEmit` (FULL) + `lint-changed.sh` eslint/prettier | mixed |
| `rust:test` | test | `cargo check --workspace`; `apply-db.sh`; reconcile; `cargo test --include-ignored --test-threads=1` (postgres:16 service) | FULL |
| `frontend:test` | test | `pnpm test` (vitest) | FULL |
| `docs:verify` | docs | `verify-docs.sh` header/naming/English | RATCHET |

Non-obvious CI facts, all verified 2026-09-29 against a fresh database:
- DB integration tests share one database and race under parallelism — CI runs
  them with `--test-threads=1`.
- Two tests require a `reconcile` run to have populated document rows first.
- The postgres service is reached at host `postgres` via `DATABASE_URL`, which
  `create_pool` reads (`db/connection.rs`). This needs `FF_NETWORK_PER_BUILD:
  "true"`; without it the runner uses legacy container links and the `postgres`
  alias does not resolve — the symptom is a hang, not an error.
- The service wait loop is bounded (60s) and prints a DNS diagnosis on failure.
  Never use an unbounded `until pg_isready` — an unreachable service then burns
  the whole job timeout (observed: 24 minutes).
- `cargo check --workspace` builds `seatloom-tauri`, whose `generate_context!`
  macro embeds `frontendDist` (`../ui/dist`). The rust-only job writes a minimal
  `ui/dist/index.html` stub so the crate compiles; the real frontend build is a
  release concern, not a gate concern.
- The ratchet base is `merge-base(HEAD, origin/track/infra-foundation)`, not
  `origin/main`: GitLab's `main` is an unrelated empty Initial commit, and
  diffing against it marks every file changed. Both verifier scripts resolve the
  base the same way and CI fetches that ref explicitly.

## Acceptance gate class

**Authoritative decision**: `docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`
(v1, lyra, 2026-10-27). That memo owns the rationale, the ruled-out options and
the backlog for the eight blocked dispatches; this section owns the **rule**.

`request_acceptance` requires `flux_verdict_ref` to name an existing
`ReviewVerdictIssued` event whose `payload.dispatch_id` names the very dispatch
being accepted. That guard is correct for implementation work, but it left an
undefined square on the board: a seat can never open an acceptance card for its
**own** completed dispatch, because no one may issue an independent verdict
about their own work. The eight Flux dispatches listed in the memo sat
completed and unaccepted behind this hole.

The rule below closes it without pretending a self-verdict is verification.

### Every completed dispatch carries exactly one class

The issuing seat writes `acceptance_class` into the dispatch payload
(`implementation` or `verification`). The class is declared at issue time, is
never edited in flight, and is settled by the **content of the dispatch**, not
by which seat happened to run it.

| Class | Content | Opens an acceptance card? | Evidence the card carries |
|---|---|---|---|
| `implementation` | Changes product code, config, schema, migrations, seeds, scripts, or any build/CI input | **Yes** | a GREEN `ReviewVerdictIssued` about this dispatch from a seat other than the actor, **plus** the operator runs the PO-authored plan in `acceptance_plan` |
| `verification` | Reads state and issues a verdict; changes none of the above; commits no diff | **Yes** — a `verification` dispatch is a delivery like any other | a pass `ReviewVerdictIssued` about this dispatch, plus its own evidence |
| `review` (non-acceptance read-only) | Investigates, audits, or reports; changes none of the above; opens no acceptance card | **No** | its own durable artifact, named as the work product |

Consequences:

1. **The three classes are about the work, not the seat.** A Flux dispatch
   verifies Mira's code, and that is `implementation` work *for Mira's
   dispatch* and `verification` work *for Flux's dispatch* — each gets judged
   by its own class. A `review` dispatch is defined by having no diff and no
   card, whoever runs it.
2. **A seat never supplies the verdict for its own dispatch.** The independent
   verdict for any `implementation` dispatch is issued by a seat other than the
   actor; a self-issued verdict is invalid even when technically produced.
3. **`review` does not mean free of evidence.** It means *no acceptance card*
   (there is no merge candidate), not *no artifact*. Every `review` dispatch
   still writes its durable artifact and is still open to escalation.

### Who may open which card

| Card's dispatch owner | Card opener | Independent verdict issued by |
|---|---|---|
| Mira / Nimbus / Onyx / Vela (workers) | the owning worker seat | Flux |
| Flux | the operator, from a Flux-issued verification packet | a worker seat that did not do the work |
| Lyra / Aegis (governance) | the governance seat itself | a designated independent reviewer |

Three consequences that need stating once, plainly:

1. **`verdict_exempt` does not exist.** There is no payload value that lets a
   dispatch skip the verdict. A card needs a real pass verdict about itself.
2. **A `verification` dispatch still gets a verdict** — because the verification
   work itself was produced by someone, and its quality is a claim like any
   other. This is what keeps the verifier from becoming the one unchecked seat.
3. **A seat's `review` output is auditable, not exempt.** Any seat may be
   dispatched to re-check a `review` conclusion; that re-check is itself a
   `verification` dispatch and needs its own verdict.

### Ledger facts the gate relies on

- A Flux `ReviewVerdictIssued` names the implementation dispatch in
  `payload.dispatch_id` (or via an `event_object_refs` row with
  `ref_type='dispatch'`). A verdict that names nothing is not about anything.
- A Flux verdict for a Flux dispatch is permitted when the verdict is about the
  *work the Flux dispatch verified* — the dispatch named is the implementation
  unit, and the accepting card belongs to the seat that owns that unit. What is
  forbidden is a verdict whose named dispatch is the issuer's own.
- `request_acceptance` needs a **named local base branch** (the integration
  branch), never a floating workspace state. A `review` dispatch that commits
  nothing opens no card and so needs no branch; if it commits, it is
  `implementation` by definition.

## Follow-ups (not in this packet)

1. **Nimbus** — per-seat `git worktree` so packet branches don't collide.
2. **Onyx** — real migration runner + separate idempotent seed; refit
   `verify-postgres-baseline.sh` / `ingest-documents.sh` off hardcoded docker
   (they are stale: they stop at schema 005 and never apply 008).
3. **Any seat** — burn down `.gitlab/docs-grandfather.txt` (207) and the ~143
   eslint errors, then flip those gates from RATCHET toward FULL.
4. **Governance** — add `engineering_standards` to the T7 subtype allow-list and
   de-duplicate the two Rust allow-list copies.
5. **Ops** — `src-tauri` / `src-cli` are not yet clippy-gated (pre-existing
   dead-code warnings); bring them under `-D warnings` once cleaned.
6. **Governance** — enforce the acceptance gate class above in code, not just in
   prose: (a) reject an acceptance request whose dispatch payload lacks
   `acceptance_class`; (b) add `engineering_standards` to the T7 subtype
   allow-list in `DOCUMENT_TEMPLATES.md` §11.1 and its Rust mirrors; (c) add a
   seat's own dispatch to the forbidden-verdict check.
7. **Lyra** — release the eight blocked Flux dispatches once the memo's
   classification of each one is recorded (memo §6 backlog).
