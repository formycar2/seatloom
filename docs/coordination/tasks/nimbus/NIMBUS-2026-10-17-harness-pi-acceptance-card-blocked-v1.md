# Delivery: the harness-Pi acceptance card could not be opened — the verdict names Flux's dispatch, not mine

[Nimbus -> Aegis] The dispatch asked me to open the acceptance card for the
harness-Pi work. **I did not open it, and this note says exactly why, with the
measurements.** Opening it as instructed would have produced a card the operator
cannot accept.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-17-harness-pi-acceptance-card-blocked-v1 |
| status | delivered, blocked on a verdict that does not name my dispatch |
| author | nimbus |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| dispatch | `disp-xAZ5l-CsJa31p7Mc3P7MS` |
| branch | `packet/nimbus/harness-pi` @ `3615395` (synced with track, 0 behind) |
| tags | acceptance-card, verdict-naming, merge-gate, harness-pi, blocked |

---

## 1. What I did do, all of it real

**The branch is synced.** The dispatch's own warning was correct and I verified it
before anything else:

```
git merge-base --is-ancestor track/infra-foundation packet/nimbus/harness-pi
  → NO.  ahead 8, BEHIND 15.
```

Track had moved to `5c2b96f` (Mira's two accepted slices) while I was working, so
`request_acceptance` would have refused with *"branch does not contain base"*. I
merged track into my branch — the sync action the dispatch said was mine — and it
merged **clean, no conflicts**. Now:

```
ahead of track: 9   behind track: 0
Pi in enum Harness: yes   Pi in enum Agent: yes
```

**All gates green after the merge:** fmt clean; clippy `-D warnings` clean on
core / mcp-emit / tauri; core 168 passed; mcp-emit 81; tauri 7;
`cargo check --workspace --all-targets` clean; `tsc --noEmit` clean; vitest 194
passed; `harness_pi_variant_proof.sh` ALL POINTS PASS; `agent_pi_variant_proof.sh`
ALL POINTS PASS.

**The full hydration path measured against live at this tip** (read-only, the same
core methods the Tauri commands wrap):

```
list_projects   39          ← the 39 real projects, including ql_delivery
list_seats       8  OK      ← was FAIL on 2026-10-16 on default_harness='Pi'
list_workitems   7  OK      list_handoffs   3  OK     list_artifacts 15  OK
list_sessions    5  OK      list_events  400  OK      list_documents   0  OK
bindings[seatloom]     7 seats, none of them ql  (aegis, flux, lyra, mira, nimbus, onyx, vela)
bindings[ql_delivery]  1 seat
```

So the branch is ready. That is not what is blocking.

## 2. 🔴 Why the card was not opened — two independent blockers

### Blocker 1: the dispatch_id in the instruction is not mine

```
disp-bsL-wE5cuc8rHfj1zz1t6  target_seat_id = seat-flux-001   status = done
  title: "verify enum Harness/Agent gains Pi (the half of Mr Zhang's ruling that all
          Harnesses become Pi)" — Flux's own VERIFICATION dispatch
```

That is **Flux's verification dispatch**, not my delivery dispatch. My dispatches
for this work are:

| dispatch | title | status |
|---|---|---|
| `disp-4WRVFBHEc0ib_TP0WXFI_` | add `Pi` to `enum Agent` | done |
| `disp-_4-4LQz9T_ienR34isCtD` | `enum Harness` also gains `Pi` | done |

`request_acceptance`'s guard is *"the dispatch must belong to the calling seat"* —
the same guard that refused Aegis when he tried to open a card for Mira (*"only a
done dispatch of the calling seat"*). So the instruction's literal arguments would
have been rejected. Aegis misread Flux's verification dispatch as mine.

### Blocker 2 — the one that actually matters: the verdict does not name my dispatch

Measured with `flux_verdict_facts(ev-Nfy-64T_UWlAa_INtzgSF, <dispatch>)`:

| dispatch | `names_dispatch` | `is_merge_criterion` |
|---|---|---|
| `disp-bsL-wE5cuc8rHfj1zz1t6` (Flux's) | **true** | **true** |
| `disp-_4-4LQz9T_ienR34isCtD` (mine) | false | false |
| `disp-4WRVFBHEc0ib_TP0WXFI_` (mine) | false | false |

Flux's verdict is green and it **is** a merge criterion — **for her own dispatch**.
It is not a merge criterion for either of mine. And a sweep of every
`ReviewVerdictIssued` event in the ledger (37 of them) plus every dispatch-typed
`event_object_refs` row found **no verdict naming either of my dispatches**.

**What that means at the operator's click.** `cmd_accept_delivery`
(`acceptance_cmds.rs:77-83`) calls `flux_verdict_facts(verdict_ref, dispatch_id)`
and then `verdict_gate_message`, which refuses when `names_dispatch` is false:

> *(the system's message is in Chinese; this is the translation)*
> **Not verified green, cannot merge:** the verdict `ev-Nfy-64T_UWlAa_INtzgSF` is
> not about THIS dispatch — it is about `disp-bsL-wE5cuc8rHfj1zz1t6`. Another
> slice's verdict cannot serve as this one's verification basis.

**So a card opened now would be refused the moment the operator clicked ACCEPT.** It
would look actionable and not be. And `cancel_acceptance` is terminal — it never
deletes the row, so the id stays occupied — meaning I could not cleanly take it
back. A card that cannot be accepted is a worse state than no card, and it is the
same defect class this whole round has been about: **a check that passes while the
thing it stands for is absent.**

**I did not open it.** I did not write a card to make the queue look clear.

## 3. What unblocks it, in order

1. **Flux re-issues her verdict** with `dispatch_id` naming my dispatch
   (`disp-_4-4LQz9T_ienR34isCtD`, the Harness half — or both halves via
   `event_object_refs` rows, which is the only channel that can carry two at once,
   exactly as she did for `ev-NFPr-zN36by-i9SE-0FPH`). **This is her act**: `actor_ref`
   is resolved server-side from `SEATLOOM_ACTOR_REF` and never from a parameter, so
   I cannot emit a verdict as Flux. Same boundary as the re-emission dispatch
   earlier today.
2. **I open the card** — the arguments are pre-computed in §4, so it is one call.
3. **The operator accepts**, and the merge gate passes because the verdict now
   names the dispatch.

Until step 1, the product stays unable to read the seats table. That is the honest
state and I am not papering over it.

## 4. The ready-to-use arguments

Everything below is measured, not drafted. The only thing missing is a verdict that
names my dispatch.

```
dispatch_id       = disp-_4-4LQz9T_ienR34isCtD        (mine, done — the Harness half)
branch            = packet/nimbus/harness-pi
base              = track/infra-foundation            (exists locally; branch is 0 behind)
flux_verdict_ref  = <Flux's re-issued verdict, once it names the dispatch above>
how_to_verify     = the operator-facing wording lives in the companion card
                    acceptance/2026-10-17-harness-pi-acceptance-card.zhs.md
                    (Mr Zhang reads Chinese; §10 of the docs rules keeps Chinese out
                    of the English delivery note and in the .zhs card). In English:
                    switch to the live-seats module — SeatLoom shows 7 seats and no
                    ql, ql_delivery shows only ql; the project page shows the 39
                    real projects including ql_delivery and the three fake seed
                    cards are gone.
```

**`acceptance_plan`** — `plan_version: 1`, with Flux's `does_not_assert` carried
into `preconditions` verbatim in substance, because the project has paid for a
conclusion covering what was never verified:

| # | step | expected |
|---|---|---|
| 1 | rebuild and reinstall from `packet/nimbus/harness-pi` | the installed binary contains `Direct, Stepcode, Pi, Custom, Custom(<payload>)` and the Agent equivalent |
| 2 | open the project page | 39 real projects listed, including `ql_delivery`; the three fake seed cards are gone |
| 3 | select SeatLoom, look at the live-seats module | 7 seats, none of them ql |
| 4 | select `ql_delivery` | exactly 1 seat (ql) |

**preconditions — Flux's `does_not_assert`, transferred rather than dropped:**

1. *The rendered page was not observed.* `screencapture` fails (*"could not create
   image from display"*) and AppleScript UI scripting is denied (*"osascript is not
   allowed assistive access"*); the Mac's screen is locked from Flux's seat. Her
   "the app reads the database again" rests on build-carry (the installed binary
   contains the Pi domain strings and the running process is that binary, holding
   live connections) plus the live read path plus the store semantics — **not on a
   screenshot.**
2. *Her verdict was pinned at `ace1b2a`, before this branch was synced with track.*
   At that pin the branch predated Mira's two UI fixes: the store still started
   from the three seeds, and every project still carried every seat. **After the
   sync merge those fixes are present in this branch**, so the silent seed fallback
   is gone and seats are attributed by binding — but Flux did not verify them and
   they are **not** covered by her verdict.
3. *No live write by this branch.* The operator's own scripts made the data
   changes: `fix-seat-harness-to-pi.sh` (all 8 rows now `Pi`) and
   `fix-ql-role-domain.sh`, **which has since been run** — the `ql_delivery` role
   is now `custom(worker)`, inside the `SeatRole` domain, so the data defect Flux
   reported as remaining is **closed**. If the operator re-runs or skips those, the
   page's contents change for reasons outside this branch.
4. *The ruling's cost is recorded, not hidden.* Read strictly, `pi` is an agent
   rather than a harness; the eight seats actually run under the stepcode proxy,
   which is what six rows said; and after the ruling the `seats` table no longer
   records which wrapper layer a seat runs under. The only surviving proxy clue is
   `default_model`. The operator's reading is also defensible and the vocabulary
   boundary is his to draw.

## 5. Known limits, stated rather than hidden

- 🔴 **The card is not open and the product still cannot read the seats table.**
  Blocked on a verdict that does not name my dispatch. Not claimed otherwise.
- **I did not follow the instruction literally, and I am saying so rather than
  quietly substituting my own dispatch.** Substituting would have produced a card
  that fails at accept time, which is worse than the refusal Aegis's literal
  arguments would have got.
- **Flux's verdict is genuine and green.** Nothing here says her work was wrong.
  The gap is purely which dispatch the verdict names.
- **The `request_acceptance` guard does not check verdict naming.** It checks the
  dispatch is mine and done, and that the verdict exists and is a
  `ReviewVerdictIssued`. The naming check lives in the **merge** gate
  (`cmd_accept_delivery` → `verdict_gate_message`), not in the request. So the
  system will happily create a card it will later refuse — which is exactly the
  shape of this whole incident, and is a real slice for someone.
- **No live write, no gate event, no merge.** The only live-touching act was
  read-only measurement.

## 6. Out of scope, unchanged

The live database, gate events (and specifically no announce-type landing gate,
which is the shape that caused today's earlier incident), the merge, and the app
rebuild.

## 7. Addendum — the report event

The report to Aegis is `ev-iDHz0nTjLg3SAEZ_fBTF5` (ArtifactCreated, subject
`disp-xAZ5l-CsJa31p7Mc3P7MS`).

Final state at delivery: branch `packet/nimbus/harness-pi` @ `3615395`, 9 ahead of
`track/infra-foundation`, 0 behind, tree clean, all gates green, both Pi proofs
ALL POINTS PASS, the full hydration path measured against live. **No acceptance card
opened.**
