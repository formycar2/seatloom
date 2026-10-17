# Delivery: both harness-Pi acceptance cards are open — the branch is synced and both would pass the merge gate

[Nimbus -> Aegis] Flux re-issued her verdict naming both of my implementation
dispatches, so the blocker from earlier today is gone. I synced the branch, re-ran
every gate, and opened **two** cards — one per implementation dispatch, as instructed.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-17-harness-pi-cards-opened-v1 |
| status | delivered, awaiting the operator's decision |
| author | nimbus |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| dispatch | `disp-5m2HCxn3tJYweIoV9o8d6` |
| branch | `packet/nimbus/harness-pi` @ `baf8255` (synced, 0 behind track) |
| tags | acceptance-card, harness-pi, agent-pi, verdict-reissue, merge-gate |

---

## 1. The re-issued verdict, verified before using it

```
ev-AbuUI51WIx1Ks-Eb_4FZp   ReviewVerdictIssued   2026-10-17 05:14:46 UTC
  verdict    = pass
  reissue_of = ev-Nfy-64T_UWlAa_INtzgSF
  dispatch refs: disp-4WRVFBHEc0ib_TP0WXFI_   (mine — enum Agent gains Pi)
                 disp-_4-4LQz9T_ienR34isCtD   (mine — enum Harness gains Pi)
                 disp-bsL-wE5cuc8rHfj1zz1t6   (Flux's own verification dispatch)
```

Measured with `flux_verdict_facts` — the same predicate the merge gate uses — for
each dispatch the cards gate:

| dispatch | verdict | `names_dispatch` | `is_merge_criterion` |
|---|---|---|---|
| `disp-4WRVFBHEc0ib_TP0WXFI_` | pass | **true** | **true** |
| `disp-_4-4LQz9T_ienR34isCtD` | pass | **true** | **true** |

So both cards would **pass** the merge gate on the operator's click. This is the
thing that was false earlier today and is now measured true, not assumed.

Her re-issue reason, as quoted in the dispatch: *"The first verdict named only the
verification dispatch ..., so it was not the merge criterion for either
implementation dispatch and Nimbus's acceptance card would have been refused by the
merge gate. This re-issue points the SAME verified conclusion at the two
implementation dispatches, one `event_object_refs` row each."*

**The re-issue is not a second verification** and both cards say so in their
preconditions: no test was re-run and no conclusion changed. Her original
`does_not_assert` carries over unchanged.

## 2. The sync, done first and re-measured rather than assumed

The dispatch's warning was correct. Before the sync:

```
track tip        = fa93d0f  (chore: ignore .saih/ project config)   ← not 5c2b96f
ahead / behind   = 12 / 2
merge-base --is-ancestor track/infra-foundation packet/nimbus/harness-pi → NOT an ancestor
```

`git merge --no-edit track/infra-foundation` → **clean, no conflicts**, one file
(`.gitignore`). Now **13 ahead, 0 behind**, and the branch does contain track.

**Every gate re-run after the merge, none assumed:**

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-tauri --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 168 passed / 0 failed (2 ignored, need Postgres) |
| `cargo test -p seatloom-mcp-emit` | 81 passed / 0 failed (22 ignored) |
| `cargo test -p seatloom-waker` | 16 passed / 0 failed |
| `cargo test -p seatloom-tauri` | 7 passed / 0 failed |
| `cargo check --workspace --all-targets` | clean |
| `tsc --noEmit` (ui) | clean |
| `vitest run` (ui) | 194 passed / 0 failed |
| `harness_pi_variant_proof.sh` | ALL POINTS PASS |
| `agent_pi_variant_proof.sh` | ALL POINTS PASS |

## 3. The two cards, both open and pending

| card | dispatch | gates | branch / base | verdict |
|---|---|---|---|---|
| `acc-disp-4WRVFBHEc0ib_TP0WXFI_` | `disp-4WRVFBHEc0ib_TP0WXFI_` — enum Agent gains Pi | pending | `packet/nimbus/harness-pi` / `track/infra-foundation` | `ev-AbuUI51WIx1Ks-Eb_4FZp` |
| `acc-disp-_4-4LQz9T_ienR34isCtD` | `disp-_4-4LQz9T_ienR34isCtD` — enum Harness gains Pi | pending | same | same |

**Two cards, not one, exactly as instructed.** Accepting either merges the whole
branch, so the second card is not a second merge — it is the ledger's record that
the other dispatch was also handed over. Both cards say this in their preconditions
so a future reader does not read two pending cards as two pending merges.

Two `AcceptanceRequested` events appended: `ev-tVA2j1O_-VmHAgYEnhhKZ` and
`ev-wcfa0g7bFLD8zXW_-oWBh`.

## 4. The acceptance plans, in plain language

Each card's `how_to_verify` and `expected` are the measured facts, not aspirations:

- rebuild and reinstall from `packet/nimbus/harness-pi`
- the installed binary carries `Direct, Stepcode, Pi, Custom, Custom(<payload>)`
  (Harness) and `ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Pi, Custom,
  Custom(<payload>)` (Agent) — Pi verbatim, no `rename_all`, per AD-011
- the project page lists **39 real projects including `ql_delivery`**, and the three
  fake seed cards are gone
- under SeatLoom the live-seats module shows **7 seats and none is ql**; under
  `ql_delivery` exactly **1** (ql)
- an unknown harness value still yields the typed error naming the whole domain, and
  `pi` in lower case is still refused — casing is significant, unknown values are
  never absorbed into `Custom`

**The caveat the dispatch insisted on, written into both cards' `preconditions` as
the first line:**

> NOT COVERED BY THIS CARD: whether the modules below the project switcher filter by
> project. That is a separate slice (Nimbus's project-scoped-reads plus Mira's UI
> work) and must not be read as verified here.

So "verified" cannot stand in for "not verified" on that point.

## 5. Flux's `does_not_assert`, transferred verbatim in substance

Both cards carry it, unchanged:

1. **The rendered page was not observed.** `screencapture` fails (*"could not create
   image from display"*) and AppleScript UI scripting is denied (*"osascript is not
   allowed assistive access"*); the Mac's screen is locked from Flux's seat. Her
   verdict rests on build-carry (the installed binary contains the Pi domain strings
   and the running process is that binary, holding live DB connections) plus the
   live read path plus the store semantics — **not on a screenshot.**
2. **Her verdict was pinned at `ace1b2a` and re-issued** to point the same conclusion
   at the two implementation dispatches. The re-issue is not a second verification.
3. **This branch contains Mira's two accepted UI slices only after the sync merges**
   (`3615395`, `baf8255`). Flux verified the Pi enums, not those slices.
4. **No live write by this branch.** The operator's own scripts made the data
   changes: `fix-seat-harness-to-pi.sh` (all 8 rows now `Pi`) and
   `fix-ql-role-domain.sh` (the `ql_delivery` role is now `custom(worker)`, inside the
   `SeatRole` domain — so the data defect Flux had reported as remaining is closed).
5. **The ruling's cost is recorded, not hidden.** Read strictly, `pi` is an agent
   rather than a harness; the eight seats actually run under the stepcode proxy,
   which is what six rows said; and after the ruling the `seats` table no longer
   records which wrapper layer a seat runs under. The only surviving proxy clue is
   `default_model`. The operator's reading is also defensible and the vocabulary
   boundary is his to draw.

## 6. Known limits, stated rather than hidden

- 🔴 **The cards are pending the operator's decision.** Nothing is merged and the
  running app is still the previously installed binary. The product does not read
  the seats table until he accepts and the app is rebuilt. **I am not claiming the
  operator's checklist.**
- **Accepting either card merges the whole branch**, including the sync merges. The
  second card is the ledger record, not a second merge — said in both cards.
- **The project-filtering slice is not covered** by these cards and is called out as
  such in both. Under this branch the backend is project-scoped, but that work is on
  a different branch (`packet/nimbus/project-scoped-reads`) and is not in this one.
- **The `request_acceptance` gap Aegis said he would record is real and still
  open.** `request_acceptance` does not check verdict naming; that check lives in the
  merge gate. So the system will still happily create a card it later refuses. It
  caused today's earlier incident and it is a real slice for someone. I verified the
  cards would pass *before* opening them precisely because of it.
- **No live write, no gate event, no merge.** The only live-touching acts were the
  two governed `request_acceptance` calls and read-only measurement.

## 7. Out of scope, unchanged

The live database, gate events, the merge, and the app rebuild.
