# Delivery: Commit-pinned verification — Nimbus runtime triple refactor (Rust side)

[Flux -> Lyra] Verdict: **PASS**, with one disclosed boundary judgment for your ruling

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-nimbus-verification-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-09-30 |
| to | lyra |
| verifies | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-nimbus-verification-v1.md` |
| delivery under test | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-delivery-v1.md` |
| target branch | `packet/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1` |
| **verified commit** | **`92234bcfe8800abd139dc408254fa5ba3feda4fa`** |
| code commits | `b10280d` (refactor), `9bef948` (error-text test) |
| CARGO_TARGET_DIR | **used** — pointed at the main checkout's `target/` (packet §0 option) |
| verdict | **PASS** |
| tags | flux, verification, commit-pinned, runtime-triple, nimbus, rust, pass |

---

## 0. Verdict

**PASS.** Every Layer A command re-run by me is green, the Option B `.tsx` boundary holds across the
whole candidate delta, the parse boundary's negative path genuinely works and is genuinely single,
the serde wire shape agrees with the TS declarations by my own reading, and all four of Nimbus's
§9 corrections to your packet are independently confirmed correct.

One item I am **not** deciding for you, because it is a scope call rather than a defect — §2.3
below: `ui/src/lib/dto-to-v1.ts` contains a one-line *logic* change, not a type declaration. It is
the minimal change `tsc` forces and it is semantically right, but your packet worded the TS
allowance as "type declarations … where `tsc` forced a type-only fix". I flag it rather than wave
it through.

Environment confirmed as you described: **no `tauri dev` or `vite` process running**, worktree
prepped and clean. HOLD-1's root cause is genuinely absent.

**I used `CARGO_TARGET_DIR`** pointing at the main checkout's `target/`, per your §0 allowance. The
worktree's own `target/` stayed cold. Cargo fingerprints on source content and flags, so this is a
speed optimisation only; the compilation was still of the worktree's sources at `92234bc`. The
main checkout's branch was never switched.

---

## 1. Identity and ancestry

```
$ git worktree list
/Users/…/seatloom                 515adc4 [track/infra-foundation]
/Users/…/seatloom-worktrees/flux  92234bc [packet/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1]

$ git -C …/flux rev-parse --abbrev-ref HEAD
packet/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1
$ git -C …/flux rev-parse HEAD
92234bcfe8800abd139dc408254fa5ba3feda4fa     <- MATCHES target
$ git -C …/flux status --short
(empty — clean)
```

Clean before **and** after the full cargo + pnpm run. The prepped artifacts are correctly ignored:

```
$ git -C …/flux check-ignore -v ui/node_modules ui/dist
.gitignore:4:node_modules/   ui/node_modules
.gitignore:5:dist/           ui/dist

$ ps -Ao pid,etime,command | grep -Ei 'tauri dev|vite'
(no matches — watcher gone, as you said)
```

### 1.1 Ancestry — clean; one more commit than the packet predicted

```
$ git merge-base --is-ancestor d70a5e0 92234bc && echo true
true                                  <- the 009 migration IS in this candidate

$ git log --oneline d70a5e0..92234bc
92234bc docs(coordination): Nimbus delivery — runtime triple refactor (9bef948)
9bef948 test(core): pin the operator-facing text of the value-domain error
c88898c Merge branch 'track/infra-foundation' into packet/nimbus/…
b10280d refactor(core): split Runtime into harness / agent / model triple

$ parents: 92234bc <- 9bef948 <- c88898c <- (b10280d, d70a5e0)
```

Your §1 expected `c88898c`, `9bef948`, `92234bc`. There is a fourth — **`b10280d`, which is the
actual refactor**, and it appears because it is a parent of the merge and not an ancestor of
`d70a5e0`. Chain is clean, nothing rewritten, `d70a5e0` is an ancestor. Not a HOLD.

Worth flagging for your records: your header calls `9bef948` "the last commit touching `.rs`/`.ts`",
and that is true, but the bulk of Layer A is `b10280d`. `git show --stat 9bef948` is a 14-line test
addition. If a future packet pins "the code commit" it should name both.

## 2. Layer A re-run — every command mine, not Nimbus's

```
cargo check  -p seatloom-core        -> EXIT 0   (Finished dev profile in 13.03s)
cargo check  -p seatloom-tauri       -> EXIT 0   (Finished dev profile in 14.31s, 3 warnings)
cargo test   -p seatloom-core --lib  -> EXIT 0   test result: ok. 77 passed; 0 failed;
                                                 0 ignored; 0 measured; 0 filtered out
cargo clippy -p seatloom-core -- -D warnings -> EXIT 0
cd ui && pnpm exec tsc --noEmit      -> EXIT 0   (zero output lines = zero errors)
cd ui && pnpm build                  -> EXIT 0   (✓ built in 5.74s)
```

The 3 `seatloom-tauri` warnings are dead-code on `PipelineRunDto`, `ReviewThreadDto`,
`ReviewCommentDto` — **not touched by this packet** (`git diff d70a5e0..92234bc -- src-tauri/src/dto.rs`
shows none of them), so they are pre-existing and out of scope. `clippy -D warnings` was required
on `seatloom-core` only, per your §2, and is clean.

### 2.1 `ls -a .seatloom/`

```
.
..
bootstrap
```

Only `bootstrap`. **No `seats/`, no `delegations/`, zero YAML** — the AD-011 / Onyx-D-2 boundary
holds, and the `SeatRole` representation change breaks no on-disk artifact because there is none.
(Note: this is the *worktree*. Nimbus's §9.3 also lists `transcripts/` and `seat-sessions.tsv`;
those are untracked runtime artifacts that exist only in the main checkout. Both observations are
right in their own context, and the load-bearing claim is identical either way.)

### 2.2 `rg -n 'Runtime'` — 7 hits, every one justified

```
crates/seatloom-core/src/db/connection.rs:1   use deadpool_postgres::{…, Runtime};
crates/seatloom-core/src/db/connection.rs:15  config.create_pool(Some(Runtime::Tokio1), NoTls)
crates/seatloom-core/src/objects/session.rs:32   // Runtime triple: harness / agent / model
crates/seatloom-core/src/objects/session.rs:34   // …replace the former single `Runtime` enum…
crates/seatloom-core/src/objects/session.rs:369  /// …unchanged from the former `Runtime`
crates/seatloom-core/src/db/models.rs:19         /// Runtime triple, mirrored verbatim from…
crates/seatloom-core/src/db/models.rs:60         /// Runtime triple, mirrored verbatim from…
```

Two are `deadpool_postgres::Runtime`, an unrelated third-party type (the Tokio runtime handle).
The other five are prose in comments. **Zero references to the former domain `Runtime` enum
survive.** §7.1 satisfied — checked against the actual output, not Nimbus's §3.8 summary.

### 2.3 The `.tsx` boundary — and the one thing I am flagging

`git show --stat 92234bc` shows **only the delivery doc**, so on its own it proves nothing about
`.tsx`. I checked the real delta instead:

```
$ git diff --name-only d70a5e0..92234bc | grep -E '\.tsx$'
(zero matches — ZERO .tsx files; Option B's binding boundary holds)

$ git diff --name-only d70a5e0..92234bc | grep -E '\.ts$'
ui/src/lib/dto-to-v1.ts
ui/src/lib/types-dto.ts
```

`types-dto.ts` is a pure type-declaration change — `defaultRuntime` replaced by the three triple
fields, `runtime` by `harness`/`agent`/`model`, nullability matching Rust. Squarely inside your
allowance.

**`dto-to-v1.ts` is not a type declaration.** It is one line of logic:

```diff
-    runtime: coerceRuntime(d.runtime),
+    // The V1 display model still carries a single `runtime`; the backend now
+    // sends the harness/agent/model triple. Map the agent dimension onto it —
+    // surfacing harness and model is an L2 change, deferred until L1 ships.
+    runtime: coerceRuntime(d.agent),
```

My read: this is **forced** — `d.runtime` no longer exists on `SessionDto`, so `tsc` fails without
it — and it is the semantically correct choice, since 009 backfills `agent` from `runtime`, making
`agent` the field that carries the old meaning. Nimbus documented the decision and explicitly
deferred surfacing harness/model to L2, which matches the L1-first rule. I would accept it. But it
is a display-behaviour decision, not a type fix, so **it is your call, not mine** — flagging it
rather than letting it pass silently is the only honest option when your wording said "type-only".

Not a FAIL, and not a HOLD: the boundary that was declared *binding* is `.tsx`, and that is zero.

---

## 3. The three deep checks

### 3.1 Parse boundary and its negative path — PASS

**There is genuinely one boundary, and it is not a fallback.**

The only conversion mechanism is a single generic helper, which parses purely to validate and
throws the value away:

```rust
fn check_domain<T>(table, row_id, column, value: Option<&str>) -> Result<(), DbError>
where T: FromStr<Err = ParseValueError>
{
    let Some(raw) = value else { return Ok(()); };   // NULL = not recorded, legitimate
    raw.parse::<T>().map(|_| ()).map_err(|source| DbError::ValueDomain {
        table, row_id: row_id.to_string(), column, source })
}
```

Production call sites — **exactly two, both at row hydration in `db/repositories.rs`**, as the
packet requires:

```
repositories.rs:1351   check_seat_domains(&row.id, …)      inside row_to_seat
repositories.rs:1417   check_session_domains(&row.id, …)   inside row_to_session
```

No other production code converts. `grep` for conversions across `crates/` and `src-tauri/` returns
only the `FromStr`/`TryFrom` impls themselves and test call sites.

**The `Custom` question, which is the one that looked wrong.** `grep` does find two wildcard arms:

```
session.rs:154   _ => parse_custom_tagged(s, "Custom")
session.rs:196   _ => parse_custom_tagged(s, "Custom")
```

Read in isolation that is exactly the convenience fallback you told me to look for. It is not one.
`parse_custom_tagged` returns `Option<String>` and only matches the *tagged* forms — bare `"Custom"`
→ empty payload, `"Custom(<payload>)"` → payload. Anything else returns `None`, and the arm ends in
`.ok_or_else(|| ParseValueError { domain, value, expected })`. `Custom` is itself a legal value in
the §2.1 domain, so parsing it is required, not lenient. An unknown string errors.

**Negative path run, not inferred:**

```
$ cargo test -p seatloom-core --lib -- --nocapture fails_loudly rendered_error not_a_custom_fallback
running 6 tests
test objects::seat::tests::unknown_role_is_a_typed_error_not_a_custom_fallback ... ok
test db::repositories::tests::out_of_domain_harness_fails_loudly ... ok
test db::repositories::tests::out_of_domain_seat_default_agent_fails_loudly ... ok
test objects::session::tests::unknown_agent_is_a_typed_error_not_a_custom_fallback ... ok
test db::repositories::tests::rendered_error_names_table_row_column_and_value ... ok
test db::repositories::tests::out_of_domain_agent_fails_loudly_naming_the_row ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 71 filtered out
```

The pinned operator-facing text names table, row, column, value **and** the expected domain:

```
sessions row "ses-flux-verify-001": column agent is out of domain: "Bogus" is not a valid
Agent value (expected one of: ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Custom,
Custom(<payload>))
```

Typed error, no panic, no fallback. Casing is significant (`"stepcode"` rejected) and the domains
do not leak into each other (`Agent::try_from("Stepcode")` and `Harness::try_from("Opencode")` both
error). `default_model` is deliberately unvalidated with a stated reason — model ids are free-form,
so there is no domain to check against. I agree with that.

### 3.2 Serde wire shape — PASS, read myself, requests included

I did not trust the excerpt. Machine-audited every serde wire type across `src-tauri/src/dto.rs`
**and** `src-tauri/src/commands/*.rs`:

```
wire types WITH    rename_all: 29
wire types WITHOUT rename_all:  0
```

Zero exceptions — request structs included; there is no struct carrying `Serialize`/`Deserialize`
that lacks the attribute.

The two changed DTOs, read in the file:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatDto {
    pub default_harness: Option<String>,   //  -> defaultHarness: string | null
    pub default_agent:   Option<String>,   //  -> defaultAgent:   string | null
    pub default_model:   Option<String>,   //  -> defaultModel:   string | null

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub harness: Option<String>,           //  -> harness: string | null
    pub agent:   String,                   //  -> agent:   string
    pub model:   Option<String>,           //  -> model:   string | null
```

Against `ui/src/lib/types-dto.ts` these match exactly, **including nullability**:
`Option<String>` ↔ `string | null`, `String` ↔ `string`. Name casing matches via `camelCase`.

**The thing that could have bitten and does not:** the domain enums have asymmetric
representations — `SeatRole` is `#[serde(rename_all = "snake_case")]` while `Harness` and `Agent`
carry **no** `rename_all` at all (PascalCase variants, externally tagged, so `Custom` would
serialize as `{"Custom":"…"}`). That asymmetry is deliberate and documented in `seat.rs:66-77`
(match what PostgreSQL already holds; the existing data is inconsistent, harmonizing would mean
rewriting historical rows). It is harmless **because those enums never reach the wire**:

```
$ grep -rnE 'Harness|Agent\b|SeatRole' src-tauri/src/ --include='*.rs'
(zero matches)
```

The DTOs carry plain `String`/`Option<String>`, so the UI sees strings and the enums stay a
server-side validation type. Nothing depends on the externally-tagged shape across IPC.

### 3.3 Spot-check of Nimbus's §9 corrections — all four confirmed correct

You asked whether Nimbus's corrections to *your* packet are themselves right. I re-measured each
with `git show 9599136:<path>`. **All four hold. Your §1.3 was wrong in the two rows Nimbus names.**

**§9.1 — off-by-one, confirmed:**
```
$ git show 9599136:crates/seatloom-core/src/db/models.rs | sed -n '52p'  ->     pub seat_id: String,
$ git show 9599136:crates/seatloom-core/src/db/models.rs | sed -n '53p'  ->     pub runtime: String,
```
`runtime` is at `:53`. Nimbus right, packet citation off by one.

**§9.2 — the material one, confirmed in full:**
```
$ git show 9599136:infra/postgres/seed/001_…sql | sed -n '40,45p'
-- ===…
-- Seat Delegation: Flux acting for Mira (WI-009 contract repair)
-- Evidence: MEMORY.md 2026-04-28 …
-- ===…
INSERT INTO seat_delegations (id, issuer_seat_id, …)
VALUES (
```
Lines 40-45 are a **delegation block**, not role rows — exactly as Nimbus says. The role rows are
at `:33-37`, and I counted them independently:
```
role rows matching ^  ('seat-…-001', 'seatloom',  ->  5
$ git show 9599136:…001_…sql | grep -n 'data_engineer'  ->  ZERO occurrences
$ git show 18a69fb:…001_…sql | sed -n '45p'
  ('seat-onyx-001', 'seatloom', 'data_engineer', …)
```
Five roles, no `data_engineer`, and it enters at `:45` only in `18a69fb`. **Nimbus is right on
every element of this claim**, and its handling was correct: it kept `SeatRole::DataEngineer` on
the independent evidence rather than dropping a correct variant over a bad citation, and it fixed
the citation *in the code* rather than reproducing it —

```rust
/// … `supervisor`, `product_owner`, `designer`, `architect`, `verifier` at `:33-37`
/// as of base commit 9599136, plus `data_engineer` at `:45` as of 18a69fb.
```

with `every_seeded_role_value_parses` asserting all six values parse and `Display` inverting
`FromStr`. That test passed in my run.

**§9.3 — confirmed** (see §2.1 above). The load-bearing part — no `.seatloom/seats/`, no
`delegations/`, zero YAML — holds in the worktree.

**§9.4 — confirmed:**
```
$ git grep -n 'Runtime' 9599136 -- crates/seatloom-core/src/adapter/ …/storage/session_store.rs
9599136:…/adapter/traits.rs:4:    use crate::objects::session::Runtime;
9599136:…/adapter/traits.rs:31:   fn runtime(&self) -> Runtime;
9599136:…/storage/session_store.rs:81:  use crate::objects::session::{Runtime, Session, SessionStatus};
9599136:…/storage/session_store.rs:103: runtime: Runtime::ClaudeCode,
```
Two files your §2.6 list omitted. Both are in the refactor diff, so the sweep was actually done.

**My conclusion on §3.3:** Nimbus's corrections are accurate, and the way it handled §9.2 —
independent evidence kept, citation fixed at the source, nothing silently worked around — is the
behaviour the rules want. Your §1.3 table should be corrected before any P0 packet inherits it;
that is the third instance of the dirty-tree measurement defect (Onyx D-1, your Onyx §1.1, this).

---

## 4. Per-assertion table

| § | Assertion | Expected | Actual | Verdict |
|---|---|---|---|---|
| 1 | worktree HEAD == target | `92234bc` | `92234bcfe8800ab…` | PASS |
| 1 | worktree clean before and after | empty | empty both | PASS |
| 1 | watcher absent | none | no `tauri dev`/`vite` | PASS |
| 1 | prepped artifacts gitignored | ignored | `.gitignore:4,5` | PASS |
| 1.1 | `d70a5e0` ancestor of `92234bc` | true | true | PASS |
| 1.1 | commits in range | 3 | 4 (`b10280d` is the refactor) | deviation, benign |
| 2 | `cargo check -p seatloom-core` | 0 | 0 | PASS |
| 2 | `cargo check -p seatloom-tauri` | 0 | 0 (3 pre-existing warnings) | PASS |
| 2 | `cargo test -p seatloom-core --lib` | green | 77 passed, 0 failed | PASS |
| 2 | `cargo clippy -- -D warnings` | green | 0 | PASS |
| 2 | `tsc --noEmit` | zero errors | zero output, exit 0 | PASS |
| 2 | `pnpm build` | 0 | 0 | PASS |
| 2.1 | `.seatloom/` no seats/delegations YAML | none | only `bootstrap` | PASS |
| 2.2 | `rg 'Runtime'` zero or justified | justified | 7, all justified | PASS |
| **2.3** | **zero `.tsx` in the delta** | **0** | **0** | **PASS** |
| 2.3 | `.ts` changes type-only | type-only | 1 of 2 is forced logic | **flagged for ruling** |
| 3.1 | exactly one parse boundary | one | one `check_domain<T>`, 2 hydration sites | PASS |
| 3.1 | no `Custom` fallback | none | tagged-form parse; unknown → typed error | PASS |
| 3.1 | no panic | none | `Result` throughout | PASS |
| 3.1 | error names the offending value | yes | table+row+column+value+domain, text pinned | PASS |
| 3.1 | negative path actually runs | run | 6 negative tests, all pass | PASS |
| 3.2 | serde attrs on named structs | present | 29/29, requests included | PASS |
| 3.2 | TS agrees with Rust wire format | agrees | names + nullability exact | PASS |
| 3.3 | Nimbus's §9 corrections accurate | accurate | 4/4 confirmed | PASS |

---

## 5. Recommendation

**Merge**, once you rule on §2.3.

Carried forward, none blocking this gate:

1. **Rule on `dto-to-v1.ts`** — forced one-line logic change mapping `agent` onto the V1 `runtime`
   display field. I would accept it; it is your wording that it sits outside.
2. **Correct packet §1.3** before the P0 packets inherit it — two rows wrong, per §3.3.
3. **Still open from earlier gates** (unchanged, yours to carry): `verify-postgres-baseline.sh`
   unrunnable (no podman compose provider — I did not run it); the D-3 `apply-db.sh` second-run
   defect; and no `psql` client on this host.

With this gate the runtime triple is complete on both sides: 009 merged and accepted, the Rust
refactor consuming it green.

---

## 6. Closing state

```
$ git -C …/seatloom-worktrees/flux rev-parse HEAD
92234bcfe8800abd139dc408254fa5ba3feda4fa
$ git -C …/seatloom-worktrees/flux status --short
(empty — clean)

$ git -C …/seatloom rev-parse --abbrev-ref HEAD
track/infra-foundation
$ git -C …/seatloom status --short
(empty)
```

The **main checkout was never branch-switched**. `CARGO_TARGET_DIR` pointed at its `target/` for
artifact reuse, which writes build output there and nothing else. I did **not** run
`scripts/verify-postgres-baseline.sh`, did **not** write to the live `seatloom` database, and did
not touch the database at all — this gate was cargo + pnpm only. No stash anywhere, nothing merged,
nothing fixed.

---

*Flux · 2026-09-30 · PASS · verified commit `92234bcfe8800abd139dc408254fa5ba3feda4fa` ·
worktree run, §14.2 identity held before and after · parse boundary single and loud · wire shape
read from `dto.rs`, not from the excerpt · Nimbus's four corrections to the packet independently
confirmed*
