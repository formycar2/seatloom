---
template: T5
subtype: delivery
doc_id: NIMBUS-2026-10-13-supervisor-intent-tools-surface-delivery-v1
title: Delivery — the Supervisor intent tool surface (slice 1): one sentence in, a typed proposal out, never a write
status: active
author: nimbus
doc_date: 2026-10-13
version: v1
depends_on: LYRA-2026-10-13-supervisor-intent-entry-contract-v1, AEGIS-2026-10-13-supervisor-intent-entry-brief.md, docs/prd-v0.5.md §6.4, docs/interaction-spec-v1.1.md INT-02
tags: delivery, supervisor-layer, intent-entry, slice-1, tool-surface
dispatch: disp-supervisor-intent-mcp-surface-v2
branch: packet/nimbus/gate-vs-announcement
---

# Delivery — the Supervisor intent tool surface (slice 1)

**Governing contract:** `LYRA-2026-10-13-supervisor-intent-entry-contract-v1`
(Lyra, `packet/lyra/milestone-summary` @ `9cde733`), slice 1, whose acceptance
criteria this delivery answers point by point in §4.

**What it is:** a user types one sentence; the system returns a typed,
reviewable result — a proposal, a clarification, or an exact retrieval plan —
and **never a write**. The write happens only on the operator's `Confirm`, and
then through `enqueue_dispatch`, which already existed and already worked.

**What it is not:** no LLM in the parse step, no semantic matching, no default
owner, no constant budget, no composed answer, no second implementation.

---

## 1. Changed files

| File | What |
|---|---|
| `crates/seatloom-core/src/intent/mod.rs` | new — module root, the value-domain contract table, re-exports |
| `crates/seatloom-core/src/intent/schema.rs` | new — the typed schema: verb / object type / dep kind / intent / qualifiers / the three plan outcomes / `Proposal` with every UX-02 field. Hand-written `FromStr`+`Display` on every closed domain |
| `crates/seatloom-core/src/intent/parser.rs` | new — the deterministic parser. Phrase tables in one place, maximal munch, latin word boundaries, verbatim qualifier capture |
| `crates/seatloom-core/src/intent/plan.rs` | new — the pure planner: resolve-or-stop, the routing-rationale computer, the proposal builder, the precondition resolver |
| `crates/seatloom-core/src/intent/facts.rs` | new — `IntentFacts` / `SeatFact` and `named_referents`, the bound on what may be queried |
| `crates/seatloom-core/src/intent/tests_support.rs` | new — one shared fixture so the parser and planner suites test the same world |
| `crates/seatloom-core/src/db/repositories.rs` | `gather_intent_facts` (the ONE gatherer) + `list_seat_routing_facts` (four columns, no harness validation — see §5 finding 3) |
| `crates/seatloom-core/src/lib.rs` | `pub mod intent;` |
| `crates/seatloom-mcp-emit/src/intent.rs` | new — the `plan_intent` MCP tool: reads only, typed rejections, `tools/list` entry |
| `crates/seatloom-mcp-emit/src/main.rs` | `mod intent;`, the `plan_intent` dispatch arm, the argument/render helpers |
| `crates/seatloom-mcp-emit/src/protocol.rs` | 19th advertised tool + the updated count test |
| `crates/seatloom-mcp-emit/proof/intent_tools_surface_proof.sh` | new — six-point end-to-end proof on a scratch DB |
| `src-tauri/src/commands/dispatch_cmds.rs` | `cmd_plan_intent` (thin wrapper over the same MCP tool) + **V5 fixed**: `cmd_enqueue_dispatch` now takes and forwards `dependencies` |
| `src-tauri/src/main.rs` | register `cmd_plan_intent` |

No schema migration, no seed change, no `infra/**` change, no `ui/**` change.
Nothing in `ui/src/components/SupervisorCommandBar.tsx` was touched — it is a
prototype and the contract puts it out of scope.

Pre-existing untracked local files (`.scratch/*`, `.seatloom/rollout-ack.txt`)
were left out of this commit deliberately: they are another session's machine
state, not part of this delivery.

---

## 2. How the three core questions were answered

### 2.1 How is intent parsed into "which dispatch, to whom, with what edges" — and why it stops instead of guessing

`seatloom_core::intent::parser` is a pure function over the sentence. It lifts
the verb, the object type, an owner *hint* and verbatim qualifiers, and reports
what it could not lift as a typed result:

* `UnsupportedVerb` — no supported verb recognised; the five verbs are offered
  with one-line examples.
* `AmbiguousVerb` — two unrelated verbs matched; ask. *(Stated addition to §3.3:
  the table has no "two verbs" row, and picking the first would be guessing.)*
* `ObjectTypeUnclear` — no object type, or more than one.
* `Unparsed` — nothing at all.

Matching is **maximal munch** (`任务单` beats `任务`; `assign to` beats `assign`)
with **word boundaries for latin phrases** (`add` does not match inside
`address`). One stated disambiguation rule handles the phrases that are both a
verb and an object (`派单`, `派活`, `派个活`): when the sentence also carries a
route marker, the shared phrase is read as the *object* and the route marker is
the verb — otherwise every routing sentence that names a dispatch would read as
"create". The rule lives in one comment block next to the table so a reviewer
can read every decision the parser is able to make.

**The owner is selected in exactly two situations**, and in no others:

1. the sentence introduced an owner with a marker (`派给`, `给 … 派`,
   `assign to`) and that name matches **exactly one** seat by name or seat id;
2. no marker, but a latin token in the sentence **is exactly** a seat name or
   seat id.

Capability tags never select. They **order a candidate list** that the user
confirms with one click — which is §3.1 step 7 ("confirm before any write")
rather than a guess. When the owner is omitted, the flow stops at
`contract-3.3-owner-omitted` with the computed candidate list and, per seat,
*why* it is a candidate.

### 2.2 How routing rationale is traceable

`SeatRationale` carries one row per seat in the registry, each with
`matched_capabilities`, `excluding_constraint`, `role`, `authority_docs`,
`constraints`, `in_flight_dispatches`, `selected` and `source`
(`seats + project_role_bindings + dispatches`). Every excluded seat states its
reason as a **fact**: `the intent named owner "nimbus"`, or
`seats.status is "retired", not 'active' (seats row for seat-proof-onyx)`, or
`no project_role_bindings row for seat seat-vela-001 in project "seatloom"`.
No line is a sentence a model composed. This is PRD §6.4's "按需索取足够证据"
made structural: the "why this seat" disclosure is a field, not a paragraph.

### 2.3 Reuse, not a second system

There is one planner (`seatloom_core::intent::plan`, pure) and one gatherer
(`SeatloomDb::gather_intent_facts`). The MCP tool `plan_intent` and the Tauri
command `cmd_plan_intent` both go through them — the Tauri command shells out to
the MCP tool over stdio exactly the way `cmd_enqueue_dispatch` shells out to
`enqueue_dispatch`. The proposal's `suggested_dependencies` are emitted in
`enqueue_dispatch`'s own `{dep_kind, dep_value}` spelling, so on `Confirm` the
caller passes them straight through with no translation.

---

## 3. The V5 fix, and why it was the right half of the "or"

The contract allowed either fixing the wrapper or refusing a precondition
explicitly. The wrapper is fixed: `cmd_enqueue_dispatch` now accepts
`dependencies: [{depKind, depValue}]` and forwards them verbatim — an omitted
list stays omitted rather than being rewritten to `[]`, because the two mean
different things to the readiness rule.

The refusal path is kept anyway, because it covers a case the wrapper cannot:
a precondition clause that names **no existing referent**. See §6.

---

## 4. Acceptance criteria, answered

| Contract §7.2 criterion | Evidence |
|---|---|
| A typed intent schema with the §3.3 ambiguity outcomes as typed results | `intent/schema.rs`; `IntentParse` has four typed non-success variants, none of them a string |
| A deterministic parser, no LLM in the parse step | `intent/parser.rs` is a pure fn; proof POINT 1 shows byte-identical output on repeat (md5 `0f1528…`) |
| A proposal builder filling every UX-02 field from real reads; no hardcoded string survives | proof POINT 3: budget `4096/16384` from `projects.*`, load `1` from `dispatches`; the prototype's `150,000` and `seat-2` do not reappear |
| A routing-rationale computer returning per seat the matched capability, the excluding constraint and the in-flight count | proof POINT 3: "every excluded seat carries a reason: 3 of 4" |
| Both surfaces over the same repository functions | proof POINT 5 (grep-asserted) |
| The Tauri wrapper accepts `dependencies`, or refuses explicitly | `cmd_enqueue_dispatch` gains the parameter; proof POINT 5 |
| `human:supervisor` attribution preserved | `SEATLOOM_ACTOR_REF=human:supervisor` set in the wrapper; proof POINT 5 |
| Self-request Flux verification per the completion protocol | §7 below — gated on `ev-verdict-supervisor-intent-tools` |

Plus the adversarial property Flux will test, proven here first: **for each
§3.3 row the system stops and asks** (proof POINT 2, seven inputs, seven PASS).

---

## 5. Findings — things I measured that somebody should rule on

### 5.1 The contract's §7.1 event table contradicts the standing gate criterion

§7.1 specifies `ev-supervisor-intent-tools-live` as type **`ArtifactCreated`**.
The gate criterion (Aegis ruling 2026-10-11, `crates/seatloom-mcp-emit/src/gate.rs`)
refuses exactly that, and says why:

> `gate rejected: event id "ev-supervisor-intent-tools-live" claims to be a gate
> (it ends in "-live"), so it must be emitted as "SupervisorMessage", not
> "ArtifactCreated". An ArtifactCreated is a delivery announcement; it says the
> code is somewhere, never that the capability runs.`

Measured, on a scratch DB, with the real binary. The ruling wins: a gate must be
a `SupervisorMessage` whose `payload.gate` equals its own id. **§7.1's row must
be corrected to `SupervisorMessage`** (Lyra/Aegis), otherwise slice 4's edge
names a referent that can never qualify as a gate and would hang forever — or
worse, somebody would emit an `ArtifactCreated` under a `-live` id and the
release-side test would read it as an announcement.

### 5.2 The availability gate is downstream of the merge, so I did not emit it

`ev-supervisor-intent-tools-landed` is emitted (it is true: the code is on this
branch at this commit). `ev-supervisor-intent-tools-live` is **not**, because the
statement it would make is not yet true:

* the capability was measured working — see §6;
* but the binary the product path spawns,
  `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit`,
  is the **2026-10-11 16:48 build** and advertises **18 tools, without
  `plan_intent`** (measured: `tools/list` against that exact file). The Tauri
  command shells out to that path, so the UI cannot reach the tool until the
  integration binary is rebuilt — and the rebuild belongs after Flux's verdict
  and the merge, not before.

Emitting the gate now would release `disp-supervisor-intent-ui-wiring` onto a
command the shipped binary does not have — the V8 defect the contract names
("a hand-maintained boolean asserting a wiring state"), wearing a gate's
clothes. So Mira's slice 4 stays blocked, on a measured fact, and Aegis is asked
for the one act that unblocks it (§7).

### 5.3 `list_seats()` cannot be used to read the seat registry — live data defect

`gather_intent_facts` reads four columns through a new `list_seat_routing_facts`
instead of the existing `list_seats`, because the live registry contains
`seat-vela-001` with `default_harness = 'Pi'`, which is outside the `Harness`
domain, and `row_to_seat` therefore fails the **whole** read:

> `seats row "seat-vela-001": column default_harness is out of domain: "Pi" is
> not a valid Harness value (expected one of: Direct, Stepcode, Custom,
> Custom(<payload>))`

Any consumer of `list_seats` fails today because of that row — the seat panel
included. Routing has no opinion about which harness a seat runs, so reading
four columns is both more correct and immune. **The row itself is data debt for
Onyx/Aegis**; I did not patch it.

### 5.4 A `dispatch`-kind dependency edge is satisfied by mere row existence

Observed live at 03:58–04:00 today: Aegis's first `disp-supervisor-intent-mcp-surface`
carried `dep_kind=dispatch` on `disp-supervisor-intent-layer-decompose`, which was
satisfied the moment that row *existed* — while the product contract was still
being written. It was delivered, then cancelled 43 s later, then re-issued as
`-v2` with an `event` edge. The readiness rule ("ready iff every referent
exists") is correct for `event`/`artifact` edges and **wrong for `dispatch`
edges**, where "exists" ≠ "completed". Recorded as a memory (see §8); the fix is
a ruling, not a patch I should make unasked.

---

## 6. Live measurement (read-only)

Against `postgresql://seatloom:seatloom@localhost:5432/seatloom`, at
`2026-10-13T04:45:00Z`, with the binary built from this branch:

```
intent: 给 flux 派个活,等 ev-supervisor-intent-tools-landed 落地再验证 intent 工具面
outcome: proposal
owner:   seat-flux-001 (flux), role verifier
suggestedDependencies: []            <- the event does not exist yet
actions: ['edit', 'cancel']          <- confirm is NOT offered
budget:  8192 / 32768, in-flight 0
rationale: 7 seats, vela eligible=false (no role binding), flux selected
```

The interesting part is the empty dependency list. The sentence carried a
precondition on `ev-supervisor-intent-tools-landed`, which did not exist at the
time of the call, so the planner **refused to invent an edge** and blocked
`Confirm`. That is the no-guessing property failing safe on a real live input,
not on a fixture.

The tool appends nothing: proof POINT 6 counts ledger rows before and after
seven plans (1 → 1).

---

## 7. What Aegis is asked for

1. **Correct contract §7.1**: `ev-supervisor-intent-tools-live` must be typed
   `SupervisorMessage`, not `ArtifactCreated` (finding 5.1).
2. **After Flux's verdict and the merge, refresh the integration binary** at
   `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit`
   (a `cargo build -p seatloom-mcp-emit` in the integration checkout) and then
   emit `ev-aegis-intent-tools-integrated`. I have hung a self-dispatch on that
   pre-declared referent; when it lands I will re-measure and only then emit
   `ev-supervisor-intent-tools-live`, which releases Mira's slice 4.
3. **Rule on the `dispatch`-kind readiness gap** (finding 5.4) and on
   `seat-vela-001`'s out-of-domain harness (finding 5.3).

---

## 8. Gates run, with results

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 146 passed, 0 failed, 2 ignored (pre-existing, need Postgres) |
| `cargo test -p seatloom-mcp-emit` | 76 passed, 0 failed, 14 ignored (pre-existing); waker bin 9 passed |
| `cargo check -p seatloom-tauri` | clean |
| `proof/intent_tools_surface_proof.sh` | ALL POINTS PASS (scratch DB `seatloom_intent`, schema 001-020) |

32 of the new tests are the intent suite itself (parser, planner, facts,
schema round-trips, the MCP tool's schema).

---

## 9. Known limits, stated rather than hidden

* **CJK capability requirements cannot be matched.** The stated-tag rule
  intersects the sentence's latin tokens with the project's tag vocabulary, and
  every seeded tag is latin. A Chinese requirement ("要做 schema 迁移的活")
  therefore produces no stated tag, and the system asks — which is the safe
  outcome, but it is a limit. Fixing it needs a ruling on a Chinese tag
  vocabulary, not a guess in the parser.
* **No budget-exceed check.** The pack engine is still a stub
  (`data_engine/pack_engine`), so there is no real pack cost to compare against.
  The proposal reports the project's real budgets and the seat's real load and
  claims no exceed. §3.3's "budget would be exceeded" row is therefore
  *reported as unverifiable*, not silently satisfied.
* **§3.3 row 11 (a surface's backing command is not callable)** is slice 4's
  concern; this tool's own equivalent is finding 5.2, reported rather than
  papered over.
* **The two-mode input, the cards and the continuity-delta card** are Mira's
  slice 2. This delivery is the tool surface only; nothing in `ui/` changed.
