# Flux verdict — the Supervisor intent tool surface (slice 1, Nimbus, 9fa5cdc), commit-pinned

- **Dispatch**: `disp-supervisor-intent-tools-verify` (issued by seat-lyra-001, priority 70;
  edge `dep_kind=event, dep_value=ev-supervisor-intent-tools-landed` — measured true)
- **Verdict event**: `ev-verdict-supervisor-intent-tools` (pre-declared id, emitted as required —
  Mira's wiring slice is blocked on exactly this id)
- **Verdict**: **PASS**
- **Verified commit (repeated in every finding below)**: `9fa5cdc665bf902f4b1fb1c92b4d7385988c648c`
  — `feat(intent): the Supervisor intent tool surface — one sentence in, a typed proposal out,
  never a write`, on branch `packet/nimbus/gate-vs-announcement` (parent `6b65b8e`)
- **Contract**: `LYRA-2026-10-13-supervisor-intent-entry-contract-v1` @ `9cde733` on
  `packet/lyra/milestone-summary` — read in full from the shared ref namespace; §7.2 Slice 3
  is the acceptance criteria and §3.3 is the adversarial table. Both answered below.
- **Scope honoured**: `disp-supervisor-intent-mcp-surface-v2`'s delivery announcement
  (`ev-supervisor-intent-tools-landed`) names branch+commit+files and deliberately carries no
  `verified` field; no `-live` gate was emitted because the integration binary still advertises
  18 tools without `plan_intent`. That is correct gate discipline and is not part of this
  verdict — slice 4 owns the live call.

## §14 captures (COORDINATION_RULES §14)

```
$ git rev-parse --abbrev-ref HEAD   -> packet/flux/waker-proof-fix
$ git rev-parse HEAD                -> 1024cc99e34fb6ef32ae9f3cdcc9d26b97976726
$ git status --short                -> ?? .scratch/*, .seatloom/rollout-ack.txt (pre-existing untracked; tracked tree clean)
```

HEAD was not the pinned commit, so I checked out `9fa5cdc` in **detached HEAD** and re-captured:

```
$ git rev-parse --abbrev-ref HEAD   -> HEAD
$ git rev-parse HEAD                -> 9fa5cdc665bf902f4b1fb1c92b4d7385988c648c   (exactly the pin)
$ git status --short                -> same untracked scratch files only (tracked tree clean — no HOLD)
```

The commit's file list matches the announcement exactly: 15 files, all under
`crates/seatloom-core/src/intent/**` (6 new), `crates/seatloom-core/src/{db/repositories.rs,lib.rs}`,
`crates/seatloom-mcp-emit/src/{intent.rs,main.rs,protocol.rs}` + the new proof,
`src-tauri/src/commands/dispatch_cmds.rs` + `src-tauri/src/main.rs`, and Nimbus's delivery note.
No `infra/**`, no `ui/**` (the V1 prototype `SupervisorCommandBar.tsx` is untouched — verified by
an empty `git diff` over `ui/`), no schema migration.

## The seven numbered criteria — each verified against the code at 9fa5cdc665bf902f4b1fb1c92b4d7385988c648c

**(1) Typed intent schema with the §3.3 outcomes as TYPED results — CONFIRMED.**
`intent/schema.rs`: `IntentVerb` / `IntentObjectType` / `DepKind` are closed domains with
hand-written `FromStr`+`Display` (serde's derived form does not match the flat TEXT columns —
the serialization-boundary rule), and an unknown value is a typed `UnknownDomain` rejection that
says "never defaulted". `IntentParse` has four typed non-success variants
(`UnsupportedVerb` / `AmbiguousVerb` / `ObjectTypeUnclear` / `Unparsed`) — none is a string — and
`Clarification` carries `rule_id` / `question` / `reason` / `choices` where every choice carries
its own `why`. `DepKind` deliberately reuses the governed-dispatch domain (schema 012) rather
than inventing a second spelling, because the proposal's edges are passed straight into
`enqueue_dispatch`.

**(2) Deterministic parser, NO LLM in the parse step — CONFIRMED.** `intent/parser.rs` is a pure
function (no I/O, no clock, no randomness — stated and unit-tested). Phrase tables live in one
place; maximal munch (`任务单` beats `任务`) with latin word boundaries (`add` does not match
inside `address`). No LLM/HTTP client appears in either crate's `Cargo.toml`, and the only
"LLM" string in the intent code is the documentation asserting its absence. Observed live:
the same sentence twice produces byte-identical output (md5 `0f15284706bcba1fce7bf63beb8a4b22`,
matching the prefix Nimbus's delivery claims — an independent cross-check).

**(3) Proposal builder fills every UX-02 field from real reads; no hardcoded string survives —
CONFIRMED.** The UX-02 set (Intent / Proposed objects / Ownership / Evidence / Budget / Impact /
Actions) maps to `intent_restated`, `proposed_object`, `ownership`, `evidence`, `budget`,
`impact`, `actions`, plus `routing_rationale` and `actor`. Measured on the proof fixture:
budget `worker=4096 / supervisor=16384` from `projects.*`, load `inFlight=1` from `dispatches`
(one real in-flight row for nimbus); evidence lines name their source tables (`seats`,
`project_role_bindings`, `dispatches`, `projects`); the prototype's `150,000 tokens` and
`seat-2` do not reappear anywhere in the output, and the only occurrences in the source are the
unit-test assertions forbidding them.

**(4) Routing-rationale computer: per candidate seat the matched capability, the excluding
constraint, and the in-flight count — CONFIRMED.** `SeatRationale` carries one row per seat with
`matched_capabilities`, `excluding_constraint`, `role`, `authority_docs`, `constraints`,
`in_flight_dispatches`, `selected`, and `source: "seats + project_role_bindings + dispatches"`.
Observed: 3 of 4 excluded seats each carry a fact-shaped reason (`the intent named owner
"flux"`; `seats.status is "retired", not 'active' (seats row for seat-proof-onyx)`; `no
project_role_bindings row for seat seat-proof-vela in project "intent-proof"
(project_role_bindings)`). No line is a composed sentence.

**(5) Both surfaces over the SAME repository functions — CONFIRMED.** One planner
(`seatloom_core::intent::plan`, pure) and one gatherer (`SeatloomDb::gather_intent_facts`).
`plan_intent` (MCP) calls them; `cmd_plan_intent` (Tauri) shells out to that same tool over
stdio exactly the way `cmd_enqueue_dispatch` shells out to `enqueue_dispatch` — the established
thin-wrapper pattern, no second implementation. Both Tauri commands are registered in
`main.rs`'s handler list; the MCP tool is advertised (19 tools, count test updated).

**(6) The Tauri wrapper accepts `dependencies` — CONFIRMED (V5 fixed, and fixed in the right
half).** `cmd_enqueue_dispatch` now takes `dependencies: Option<Vec<EnqueueDependencyArg>>` and
forwards it verbatim in the tool's own `{dep_kind, dep_value}` spelling; an omitted list stays
omitted rather than being rewritten to `[]`, because the two mean different things to the
readiness rule. The underlying MCP tool always had the argument (`dispatch.rs:86`) — the gap was
the wrapper, exactly as the contract's V5 said. The refusal path also exists for the case the
wrapper cannot cover (a clause naming no existing referent) — see row 6 below.

**(7) `human:supervisor` attribution preserved — CONFIRMED.** The actor is resolved server-side
from `SEATLOOM_ACTOR_REF` (never a parameter — the tool's `inputSchema` has no identity field,
and `tools/list` advertises only `intent` + optional `project_id`); the Tauri wrapper sets
`SEATLOOM_ACTOR_REF=human:supervisor` for the same reason the enqueue path does. Observed: the
proposal's `actor` is `human:supervisor`.

## The §3.3 adversarial table — every row, constructed input, observed output

Run against the proof fixture (scratch DB `seatloom_intent`, schema 001-020, seats: nimbus
active+bound, flux active+bound, onyx retired+bound, vela active+UNBOUND; project budgets
4096/16384; one in-flight dispatch for nimbus; one done dispatch titled `Verify the gate`; one
real event `ev-proof-landed`), driving the REAL binary over stdio. The tool is read-only, so
every "stop" below is a stop before any write.

| §3.3 row | Constructed input | Observed output |
|---|---|---|
| Verb not in the supported set | `把唤醒服务的硬底兜底接上` | STOPPED `contract-3.3-verb-unsupported`; question + the five verbs with one-line examples |
| Owner named, zero seats match | `给 atlas 派个活` | STOPPED `contract-3.3-owner-not-found`; `没有叫做 "atlas" 的 seat。`; choices = the REAL registry (all four seats, each with its `why`) |
| Owner named, more than one match | `给 seat-proof-flux 派个活` with a twin seat whose **name is another seat's id** (`seats.name` is UNIQUE — `seats_name_key` — so a duplicate name is impossible; this is the reachable construction) | STOPPED `contract-3.3-owner-ambiguous`; `the owner name matched 2 seats: seat-proof-flux, seat-proof-twin`; both candidates offered, neither picked |
| Owner omitted and not inferable | `派个活` | STOPPED `contract-3.3-owner-omitted`; `派给哪个 seat?`; candidate list ordered by stated tags with a per-seat `why` |
| Owner omitted but exactly inferable (the complement — must NOT stop) | `create a dispatch for flux to verify the gate` | PROPOSAL for `seat-proof-flux` — an exact latin-token seat-name match is an explicit statement, not a guess; every other seat excluded with a fact |
| Object type unclear (none stated) | `创建一个东西` | STOPPED `contract-3.3-object-type-unclear`; `no object type was stated`; the four object types offered |
| Object type unclear (more than one matched) | `创建一个派单和交接` | STOPPED `contract-3.3-object-type-unclear`; `more than one object type matched: dispatch, handoff` |
| Precondition the write path cannot express — clause names no referent | `给 flux 派个活,等 Lyra 的产品契约落地再动` | PROPOSAL with `suggestedDependencies: []`, `unresolvedReason` naming the missing capability, `blockingClarification.rule_id = contract-3.3-precondition-unresolved`, actions `['edit','cancel']` — **confirm is not offered**; the escape hatch `confirm-without-precondition` is explicit, never silent |
| Precondition naming a referent that does NOT exist | `给 flux 派个活,等 ev-does-not-exist 落地再动` | Same blocking outcome — the waker-time referent was checked and found absent, so no edge was invented |
| Precondition naming a real referent (the allowed direction) | `给 flux 派个活,等 ev-proof-landed 落地再动` | Edge attached in `enqueue_dispatch` spelling `{depKind: "event", depValue: "ev-proof-landed"}`; `confirm` offered. Also verified by exact dispatch id and by **exact** dispatch title (`等 Verify the gate 再动` → `{dispatch, "Verify the gate"}`), while the near-match `等 Verify the gate 落地再动` does NOT resolve — a near-match is a guess |
| No capable seat (retired) | `派给 onyx 一个派单` | STOPPED `contract-3.3-no-capable-seat`; reason `seats.status is "retired", not 'active'`; no choices |
| No capable seat (active but unbound) | `派给 vela 一个派单` | STOPPED `contract-3.3-no-capable-seat`; reason `no project_role_bindings row for seat seat-proof-vela in project "intent-proof"`; no choices |
| Budget would be exceeded | `给 nimbus 派个活,预算 999999 tokens` | PROPOSAL carrying the computed cause before any confirm: `worker=4096, supervisor=16384, inFlight=1, statedHint=999999 tokens`. See the observation below — no explicit "exceeded" predicate exists in slice 1 |
| Evidence insufficient for a why-question | `找出处 关于 xyzzy 的证据` | RETRIEVAL plan (`find-evidence`): the exact reads (`get_knowledge_tree` → `recall l0` → `recall l1/l2` → `query_ledger`) and the note `This tool does not compose an answer… If they return nothing, the result is "Evidence incomplete" plus the list of sources actually searched — never a composed answer`. An answer with no evidence is not producible at this layer |
| Write fails | n/a at this layer | `plan_intent` has no write path: the ledger was counted before and after ~15 plans (1 → 1). The write path is `enqueue_dispatch`, unchanged by this commit; its verbatim-error behaviour is slice 4's end-to-end concern |
| A surface's backing command is not callable | `plan_intent` with an empty intent | Typed rejection returned verbatim with `isError: true`: `plan_intent rejected: the intent is empty — there is nothing to plan`. `cmd_plan_intent` propagates the same text verbatim to the UI (`Err(text)`), so the card shows the measured reason rather than an empty frame |

**The no-guessing property holds adversarially**: every row stops; the two complements that must
NOT stop (an exactly-named seat, an exactly-resolved precondition) resolve; and the near-match
precondition does not resolve. A parser that "usually" stops would have failed at least one of
these constructions — none did.

## Gates — run for real on the pinned commit (9fa5cdc665bf902f4b1fb1c92b4d7385988c648c)

```
$ cargo build -p seatloom-mcp-emit          (built FIRST — see the note below)
$ cargo test -p seatloom-core
running 148 tests
test result: ok. 146 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
$ cargo test -p seatloom-mcp-emit
running 90 tests
test result: ok. 76 passed; 0 failed; 14 ignored; 0 measured; 0 filtered out
   (+ the waker bin: running 9 tests — test result: ok. 9 passed; 0 failed)
$ cargo fmt --all -- --check      (no output — clean)
$ cargo clippy -p seatloom-core --all-targets -- -D warnings     (clean)
$ cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings (clean)
$ cargo check -p seatloom-tauri   (clean — the wrapper crate compiles against the change)
```

146 / 76(+9) match Nimbus's claimed counts exactly.

**The proof script, end to end** (`crates/seatloom-mcp-emit/proof/intent_tools_surface_proof.sh`,
scratch DB `seatloom_intent`, live DB refused by construction and never touched): ALL POINTS
PASS — POINT 1 (typed + deterministic, md5 `0f1528…` on repeat), POINT 2 (seven §3.3 inputs,
seven STOPs, the seat list offered is the real registry), POINT 3 (budget 4096/16384, load 1,
every excluded seat reasoned, prototype strings absent), POINT 4 (edge-or-stop, both
directions, exact-title match only), POINT 5 (one implementation, human:supervisor, V5
forwarding), POINT 6 (ledger 1 → 1 across seven plans).

## Observations reported upward (not verdict-blocking)

1. **§3.3 row 8 has no explicit predicate in slice 1.** There is no "would be exceeded"
   determination anywhere in `seatloom_core::intent` — the proposal carries the computed budget
   facts (project columns + in-flight load + the stated hint) on the card *before* any confirm,
   and `plan_intent` cannot write at all, so the forbidden "confirm anyway" cannot occur at this
   layer. But the explicit determination has nothing to compute against: there is no per-seat
   budget column, and `projects.worker_budget_tokens` is a project-level number. The card layer
   (slices 2/4) owns showing the cause; somebody (Lyra/Aegis) should state what "exceeded" means
   before slice 4 wires the card. Recorded as a memory.
2. **One out-of-domain `project_role_bindings.role` row rejects the WHOLE plan call.** Measured
   live while constructing row 3: a binding with `role='verifier2'` (outside the closed
   `SeatRole` domain) made `plan_intent` fail with
   `project_role_bindings row "seat-proof-twin": column role is out of domain: "verifier2" is not
   a valid SeatRole value…` — a typed rejection, which is the right failure direction, but it is
   the same whole-read failure class as Nimbus's finding 5.3 (`list_seats` dies on
   `seat-vela-001`'s harness) in a *different* column. The intent surface is immune to the
   harness defect (it reads four columns) but not to a role-domain defect. Data debt for
   Onyx/Aegis; not patched here.
3. `seats.name` is UNIQUE (`seats_name_key`), so §3.3 row 3 ("owner matches more than one seat")
   is only reachable when one seat's **name equals another seat's id** — narrow, but real, and
   it stops correctly when reached.
4. The landing announcement's gate discipline is correct and confirmed: `ev-supervisor-intent-tools-landed`
   carries no `verified` field, and no `-live` gate was emitted because the integration binary
   still advertises 18 tools without `plan_intent`. Mira's slice 4 stays blocked on a measured
   fact, which is the contract's V8 defect avoided rather than repeated.
5. **Process note (fails safe)**: `cargo test --bin X` does not rebuild the plain binary
   `target/debug/X`; the proof script needs `cargo build -p seatloom-mcp-emit` first. I hit this
   again on this dispatch's first gate run and rebuilt explicitly before any proof run — the
   same failure mode I recorded as a memory during the waker verification. The proof script
   would be more robust with the build step inside it; that is a script-robustness suggestion
   for Nimbus/Aegis, not something I changed (§15-style discipline: I verify, I do not rewrite
   another seat's proof unasked).

## What I did NOT do

No merge, no integration to track, no daemon restart, no deploy. No product code changed — my
only writes were the two scratch evidence scripts under `.scratch/` (untracked, not committed)
and this delivery note. Nimbus's branch `packet/nimbus/gate-vs-announcement` was not modified;
I worked in a detached HEAD at the pinned SHA and left the tracked tree clean.

## Forward recording

Recorded via `remember` (source_kind `canonical_event`, source_ref
`ev-ThdWJypV37NjPFNeN4xTb`, the `SeatDispatchDelivered` lifecycle event of this dispatch),
topic `topic-aegis-fleet-operations`:

- **unresolved** — §3.3 row 8 (budget "would be exceeded") has no predicate in slice 1 and no
  schema to compute one against; slices 2/4 must own it and somebody must define "exceeded".
- **failure** — a single out-of-domain `project_role_bindings.role` row rejects the entire
  `plan_intent` call (measured with `verifier2`); same whole-read failure class as the vela
  harness defect, different column; Onyx/Aegis data debt.

## Done definition — checklist

- [x] §14 captures reported (detached at the exact pinned SHA, tracked tree clean)
- [x] The §3.3 adversarial table answered row by row with real inputs and outputs (11 rows +
      the two must-not-stop complements, all constructed against the real binary on a scratch DB)
- [x] The seven numbered criteria verified against the code (typed schema, no-LLM pure parser,
      no hardcoded fields, per-seat rationale facts, one implementation, V5 dependencies
      forwarding, human:supervisor attribution)
- [x] Gates run with real output (146 / 76+9 / fmt / clippy ×2 / tauri check) + the six-point
      proof green end to end
- [x] The exact verified commit SHA `9fa5cdc665bf902f4b1fb1c92b4d7385988c648c` repeated in every
      finding and here
- [x] `ReviewVerdictIssued` emitted as `ev-verdict-supervisor-intent-tools` (PASS)
- [x] This delivery note at `docs/coordination/tasks/flux/FLUX-2026-10-13-supervisor-intent-tools-verdict-v1.md`
      plus forward recording
- [x] `complete_dispatch(dispatch_id='disp-supervisor-intent-tools-verify')` — final step
