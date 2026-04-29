# Delivery: Commit-Pinned Infrastructure Baseline

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-commit-pinned-infra-baseline-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| task_ref | NIMBUS-2026-04-29-commit-pinned-infra-baseline-v1 |
| from | Nimbus |
| to | Lyra |
| tags | nimbus, infrastructure, git, branch-management, postgres, verification |

## Delivery Fields

| Field | Value |
|---|---|
| `base_branch` | `main` |
| `base_commit` | `1bf60be` (chore: baseline commit before frontend refactoring) |
| `delivery_branch` | `track/infra-foundation` |
| `delivery_commit` | `2f6c41a4d4c5acae2f7e8f103610e7d45482d15e` |
| `pushed_to_origin` | **yes** — `git push origin track/infra-foundation` succeeded |

## Can Flux Run Commit-Pinned Remote Verification?

**Yes.** The branch is live on `formycar2/seatloom` at the exact commit. Flux can fetch and verify with:

```bash
git fetch origin track/infra-foundation
git checkout 2f6c41a4d4c5acae2f7e8f103610e7d45482d15e
scripts/verify-rust-foundation.sh
# For postgres verification (requires docker):
scripts/verify-postgres-baseline.sh
```

## Included Paths

| Path | Reason |
|---|---|
| `Cargo.lock` | Dependency resolution for tokio-postgres + deadpool-postgres |
| `Cargo.toml` | Workspace manifest (profile.release moved from src-tauri) |
| `rust-toolchain.toml` | Exact 1.95.0 toolchain freeze |
| `crates/seatloom-core/` | All accepted infra: storage layer, db/ module, tests, Cargo.toml |
| `src-tauri/Cargo.toml` | Removed duplicate profile stanza |
| `src-tauri/src/commands/` | Command stubs with `#[allow(dead_code)]` |
| `src-tauri/src/state.rs` | `#[allow(dead_code)]` scaffold annotation |
| `.github/workflows/rust-foundation.yml` | CI workflow |
| `.seatloom/bootstrap/source-map.yaml` | Provenance notes |
| `infra/postgres/` | Docker Compose, schema, real collaboration seed data |
| `scripts/verify-rust-foundation.sh` | Local Rust verification entrypoint |
| `scripts/verify-postgres-baseline.sh` | Local Postgres verification entrypoint |
| `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-*` | All Nimbus infra task + delivery docs |
| `docs/coordination/acceptance/2026-04-29-lyra-nimbus-*` | Nimbus infra acceptance evidence |
| `docs/coordination/acceptance/2026-04-29-lyra-flux-foundation-hardening-verification-acceptance.md` | Flux verification evidence |
| `docs/coordination/tasks/flux/FLUX-2026-04-29-foundation-hardening-verification-*.md` | Flux infra verification packets |
| `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-*.md` | Flux postgres verification packets |
| `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-v1.md` | Remote verification packet |
| `docs/coordination/reviews/2026-04-29-seatloom-branch-management-design-v1.md` | Branch management design (packet dependency) |
| `docs/coordination/tasks/lyra/LYRA-2026-04-29-infra-method-review-gate-v1.md` | Lyra infra gate doc |
| `docs/coordination/tasks/lyra/LYRA-2026-04-29-nimbus-model-routing-protocol-v1.md` | Lyra routing protocol |

**Total: 86 files, 5973 insertions, 132 deletions**

## Excluded Paths (Deliberately Not Staged)

| Path | Reason |
|---|---|
| `ui/**` | Frontend refactor by Aegis + Mr. Zhang — not in allowed change surface |
| `docs/coordination/MEMORY.md` | Lyra updates — not infra scope |
| `docs/coordination/memory/2026-04-29.md` | Lyra daily log — not infra scope |
| `docs/coordination/COORDINATION_RULES.md` | Lyra governance update — not in Nimbus allowed surface |
| `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md` | UI design — not infra scope |
| `docs/coordination/acceptance/2026-04-29-frontend-refactor-runtime-acceptance-decision.md` | Frontend acceptance — not infra scope |
| `docs/coordination/tasks/flux/FLUX-2026-04-29-frontend-refactor-runtime-verification-*.md` | Frontend verification — not infra scope |

## Validation Commands and Results

All run on commit `2f6c41a`:

| Command | Result |
|---|---|
| `$HOME/.cargo/bin/cargo check` | exit 0, 0 warnings |
| `$HOME/.cargo/bin/cargo test -p seatloom-core` | 26 passed, 9 ignored, 0 failed |
| `$HOME/.cargo/bin/cargo fmt --all --check` | exit 0, no diffs |
| `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings` | exit 0, 0 errors |
| `scripts/verify-rust-foundation.sh` | exit 0, all checks passed |
| `scripts/verify-postgres-baseline.sh` | NOT RUN — docker not available on this seat; requires docker-capable fallback seat |

## Safety Compliance

- ✓ Did not merge into `main`
- ✓ Did not force-push any shared branch
- ✓ Did not rewrite or revert unrelated dirty work (UI, Lyra/Mira docs remain unstaged)
- ✓ Branch created from existing HEAD `1bf60be`, not from a rebase
- ✓ No product behavior changes introduced
