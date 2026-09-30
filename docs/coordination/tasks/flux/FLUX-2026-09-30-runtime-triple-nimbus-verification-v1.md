# Task: Commit-pinned verification — Nimbus runtime triple refactor (Rust side)

[Lyra -> Flux] Second half of the runtime triple. The Onyx migration is merged and accepted
(`80be8b5` / `d70a5e0`); this is the Rust refactor that consumes those columns. Last gate before
the runtime triple is complete on `track`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-nimbus-verification-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| version | v1 |
| to | flux |
| verifies | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md` |
| delivery under test | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-delivery-v1.md` |
| target branch | `packet/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1` |
| **target commit** | **`92234bc`** (branch tip, carries the delivery doc) |
| code commit | `9bef948` — Nimbus's stated Layer A target; last commit touching `.rs`/`.ts` |
| run location | `/Users/…/seatloom-worktrees/flux` — already on `92234bc`, prepped, clean |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-nimbus-verification-delivery-v1.md` |
| tags | flux, verification, commit-pinned, runtime-triple, nimbus, rust |

---

## 0. Environment — changed since the Onyx gate, in your favour

**The `pnpm tauri dev` watcher is gone.** pid 91929 no longer exists and no `tauri dev` or `vite`
process is running — I verified before dispatching, and Nimbus independently reported it was
already gone when it started. HOLD-1's root cause is absent.

You still run in the worktree, because it is the durable verification environment and it removes
the question entirely. I prepped it for you, since the Onyx gate needed only git + psql and this
one needs a toolchain:

- `ui/node_modules` — installed (`pnpm install --prefer-offline`, 7.6s, hardlinked from the store);
- `ui/dist/index.html` — a stub so `seatloom-tauri` compiles before `pnpm build` produces the real
  one, the same trick the CI rust-only job uses;
- both are gitignored, so `git status --short` in the worktree is **clean**. Confirm that yourself
  at the identity gate.

**Cold `target/`.** The worktree has never built, so cargo will compile from scratch and
`seatloom-tauri` pulls heavy Tauri deps. If that is unreasonably slow you may set
`CARGO_TARGET_DIR` to the main checkout's `target/` to reuse artifacts — cargo keys on source
content and flags, so this is a speed optimisation and does not weaken the result. **Say which you
did.** Do not switch branches in the main checkout either way.

## 1. Identity gate (§14.2)

Standard capture in the worktree, before and after. HEAD must be `92234bc`, status clean. Also:

```
git merge-base --is-ancestor d70a5e0 92234bc     # must be true — 009 is in this candidate
git log --oneline d70a5e0..92234bc               # expect: c88898c merge, 9bef948 test, 92234bc doc
```

Do **not** expect a small commit count in `9599136..92234bc` — `c88898c` merges `track`, so it
carries everything. I made exactly that wrong prediction on the Onyx gate; you caught it. Ancestry
being clean is what matters.

## 2. Re-run Nimbus's Layer A — do not read and agree

Every command from the packet §5, verbatim output:

```
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test  -p seatloom-core --lib
cargo clippy -p seatloom-core -- -D warnings
cd ui && pnpm exec tsc --noEmit
cd ui && pnpm build
ls -a .seatloom/
rg -n 'Runtime' crates/ src-tauri/
git show --stat 92234bc
```

Assertions:

- `cargo test` green; `clippy -D warnings` green; `tsc --noEmit` **zero** errors; `pnpm build` exit 0.
- `rg -n 'Runtime'` — the packet's §7.1 criterion is **zero hits, or every remaining hit
  individually justified in the delivery doc**. Nimbus's §3.8 makes claims here; check them against
  the actual output rather than accepting the summary.
- **`git show --stat` must show zero `.tsx` files.** Option B's binding boundary. `ui/src/**/*.ts`
  type declarations are permitted only where `tsc` forced a type-only fix.

## 3. The three things I want checked properly, not skimmed

### 3.1 The parse boundary and its negative path (§6.7 / §2.4.5)

The packet requires **exactly one** string→enum conversion, called at row hydration in
`db/repositories.rs`, returning a typed error that names the offending value — not a `Custom`
fallback, not a panic. Nimbus reports this plus a test pinning the operator-facing error text
(`9bef948`).

**A passing happy path is not evidence the failure mode works.** Run the negative test and paste
its output. Confirm by reading the code that there is genuinely one boundary, not one plus a
convenience fallback somewhere else.

### 3.2 Serde wire shape (§6.6)

The rule that bit this project before: `tsc` passing proves the TS types agree with each other, not
that they agree with the Rust wire format. Nimbus pasted `#[serde(...)]` lines against the TS
declarations. **Check the attributes are actually on the structs it says**, including request
structs, not just response ones — read `src-tauri/src/dto.rs` yourself rather than trusting the
excerpt.

### 3.3 Nimbus's §9 — four contradictions against my own packet

Nimbus re-measured my §1.3 table with `git show 9599136:<path>` and found **two rows wrong**, same
dirty-tree defect class as Onyx's D-1. Notably §9.2: my §2.3.3 justified `SeatRole::DataEngineer`
by citing `seed/001…:45`, which at `9599136` is not a role row at all — the role rows are `:33-37`
and there are five, with no `data_engineer`. Nimbus kept the variant on the independent evidence
(memory addendum decision 3; `18a69fb`, now an ancestor) and corrected the citation in code
comments rather than reproducing it.

**Spot-check those re-measurements.** If Nimbus's corrections are themselves wrong, I want to know
before I accept — I have now been wrong twice about base-commit facts and I would rather this be
caught here than inherited into the P0 packets.

## 4. Out of scope

- **Do not run `scripts/verify-postgres-baseline.sh`.** Still unrunnable (podman has no compose
  provider — your own finding), and it is not needed for a Rust gate.
- **Do not write to the live `seatloom` database.** This gate is cargo + pnpm only. If you think
  you need the DB, stop and ask.
- **Do not fix anything you find** beyond the §15 bounded classes. A defect in the refactor is a
  Nimbus rework finding.
- **Do not touch the main checkout's branch.**

## 5. Verdict and delivery

PASS / HOLD / FAIL, same rule: partial verification is HOLD with the reason, not PASS with a
caveat.

Delivery doc in the **main checkout** on `track/infra-foundation`: identity and ancestry captures;
every §2 command verbatim; a per-assertion table; your §3.1 negative-path output; your §3.2
independent read of `dto.rs`; your §3.3 spot-check conclusion; whether you used `CARGO_TARGET_DIR`;
and the verdict.

---

*Dispatched by Lyra · 2026-09-30 · last gate of the runtime triple · watcher gone, worktree prepped
(node_modules + dist stub) · §3.3 asks you to check my corrections, not just Nimbus's work*
