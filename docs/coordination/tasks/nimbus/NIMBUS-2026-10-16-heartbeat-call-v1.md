# Delivery: an idle seat with standing duties is called to them

[Nimbus -> Aegis] Mr Zhang, 2026-10-16: Lyra is the product owner, with a duty to
design, to find what is broken and to own the delivery — but her shape is "work
when dispatched, stop when not". That is not a product owner, that is a
contractor. He is right, and the root cause is not in her: it is in the waker.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-16-heartbeat-call-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-16 |
| version | v1 |
| to | aegis |
| dispatch | `disp-Krsp8j-EDOpWDIiWjuHo_` |
| contract | `disp-Krsp8j-EDOpWDIiWjuHo_` payload |
| depends_on | `8a2d733` (Lyra's standing-duty document), `54dff31` (Aegis's own heartbeat) |
| branch | `packet/nimbus/gate-vs-announcement` |
| tags | waker, heartbeat, standing-duties, idle-seat, cooldown |

---

## 1. The root cause, verified myself

`sweep` (`crates/seatloom-mcp-emit/src/bin/waker.rs`) matched each seat on
`(in_flight_id, ready_n)` and only ever called it in two cases:

| Arm | Called? | With |
|---|---|---|
| `(None, n) if n > 0` | yes | the dispatch-drain instruction |
| `(Some(id), _)` — stale claim | yes | the dispatch-drain instruction |
| `(None, _) => {}` | **no** | — |

So a seat with **zero** ready dispatches and nothing in flight was never called at
all. And the only instruction any seat ever received was:

> 📥 new governed dispatch available — run your dispatch loop now: call
> get_next_dispatch repeatedly (do each payload, then complete_dispatch) until it
> returns "nothing ready", then stop.

Its entire field of view is the queue. A seat whose job includes designing, finding
what is broken, and owning the delivery was therefore structurally a contractor:
present when dispatched, absent otherwise. Aegis had already written Lyra the
standing-duty document and **nothing executed it** — that is the half this dispatch
covers.

## 2. What was built

| File | What |
|---|---|
| `crates/seatloom-mcp-emit/src/bin/waker.rs` | a third `sweep` arm; `NudgeKind` (two instructions, one function); `heartbeat_doc_path` / `has_heartbeat_doc`; a second `Poker` and its own knob; 7 new tests |
| `crates/seatloom-mcp-emit/proof/heartbeat_call_proof.sh` | new — 7 points |

**One implementation, two instructions.** `NudgeKind` is an enum, and `nudge` takes
it — so there is one place that types into a pane and two messages, rather than two
functions that could drift apart. The heartbeat message names the seat's own
document and says what to do; it does **not** mention `get_next_dispatch`, because
with no work waiting that instruction would be a lie.

## 3. The document path convention

`<tasks_dir>/<seat>/<SEAT>-HEARTBEAT.md`, where `<seat>` is the seat id with the
registry's `seat-` prefix and `-NNN` suffix stripped:

```
seat-lyra-001  ->  docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md
```

**This is not a layout I invented.** It is the one `AEGIS-HEARTBEAT.md` and
`LYRA-HEARTBEAT.md` already use (commits `54dff31` and `8a2d733`), so this delivery
implements a convention that already exists rather than creating a second one. The
directory is a knob — `SEATLOOM_TASKS_DIR`, default `docs/coordination/tasks` —
resolved from the working directory the same way `SEATLOOM_SEAT_MAP`'s default is,
so it is not hardcoded to one checkout's layout.

## 4. Only a seat that HAS a document is called

`has_heartbeat_doc` checks the file exists, and the arm is gated on it. A seat
without one would answer "nothing to do" — noise, and the dispatch names that as a
reject. Measured against the real seat map:

| Seat | Document | Called? |
|---|---|---|
| `seat-lyra-001` | `docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md` | **yes** |
| `seat-mira-001` | absent | no |
| `seat-nimbus-001` | absent | no |
| `seat-flux-001` | absent | no |
| `seat-onyx-001` | absent | no |
| `seat-vela-001` | absent | no |

**1 of 6** — exactly what the dispatch said ("only Lyra has one right now").

The check is **per sweep, not cached**, so writing a document connects a seat with
no code change and no restart. That is the point of the convention: the next seat
to get a standing-duty document is connected the moment the file exists.

## 5. The cooldown is separate, and longer

| | Dispatch nudge | Heartbeat nudge |
|---|---|---|
| Default | `DEFAULT_POKE_COOLDOWN` = **120 s** | `DEFAULT_HEARTBEAT_COOLDOWN` = **30 min** |
| Knob | `SEATLOOM_POKE_COOLDOWN_SECS` | `SEATLOOM_HEARTBEAT_COOLDOWN_SECS` |
| State | its own `Poker` | its own `Poker` |

**Why 30 minutes, and why not shared.** The dispatch cooldown is short because a
NOTIFY can fire while a seat is busy, so it is about not *losing* work — it has to
be short. A heartbeat nudge loses nothing: it reminds an idle seat of duties it
already has. So it is deliberately long. 30 minutes is 15× the dispatch window and
means a seat is interrupted at most twice an hour, while still short enough that a
seat which goes idle is not left for a day. Two knobs on two `Poker`s, so tuning
one never moves the other — and a seat can be dispatch-nudged twice while being
heartbeat-nudged once, which is the correct asymmetry.

## 6. The two existing paths are untouched

Measured, not asserted: the proof extracts the dispatch literal from the **old**
source and the **new** source and compares them. A proof that merely re-used the
string would pass even if the string changed.

- The dispatch message is **byte-identical**.
- `(None, n) if n > 0` still nudges with `NudgeKind::Dispatch`.
- Both stale-claim backstop arms still nudge with `NudgeKind::Dispatch`.
- The NOTIFY path and the catch-up scan still nudge with `NudgeKind::Dispatch`.
- The waker still **never** writes to `dispatches` — the proof greps for
  `UPDATE dispatches` and fails if it appears.

## 7. Gates run

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 174 passed / 0 failed (2 ignored, need Postgres) |
| `cargo test -p seatloom-mcp-emit` | 85 passed / 0 failed (22 ignored) |
| `cargo test -p seatloom-mcp-emit --bin seatloom-waker` | **16 passed / 0 failed** — 7 of them this slice |
| `cargo check -p seatloom-tauri` | clean |
| `cargo check --workspace --all-targets` | clean |
| `proof/heartbeat_call_proof.sh` | ALL POINTS PASS |
| the four earlier proofs | ALL POINTS PASS (re-run, unchanged) |
| `scripts/verify-docs.sh` | this note is clean |

The proof's seven points: the old empty arm reproduced; the byte-identical dispatch
message; the heartbeat message being a genuinely different instruction; which real
seats get called; a seat without a document left alone, measured on the **real
filesystem** rather than a fixture; the two cooldowns; and the existing paths
intact.

## 8. Acceptance — what I measured, and what is yours to see

**Measured by me:** every case above, against the real seat map and the real
documents on disk.

**Yours to see, which I cannot run and am not claiming:**

1. Let a seat sit idle with no dispatches for one heartbeat period. **Expected: it
   receives "go and read your standing-duty document", not "drain your
   dispatches".** On the current fleet that is Lyra, and only after the waker is
   rebuilt and restarted with this code.
2. Give it a dispatch. **Expected: exactly the old dispatch instruction, character
   for character.**
3. A seat with no document sitting idle. **Expected: not called.**
4. Several periods in a row. **Expected: the same seat is not called to spam.**

**What this build will not show yet.** The running waker is the previously built
binary (`ps` shows `./target/debug/seatloom-waker` from the main checkout, kept
alive by a supervisor shell loop). It must be rebuilt and the loop restarted to
carry this change. **I did not restart it** — the daemon holds the fleet's wakeup
path, and restarting it is not mine to do on my own authority. Reported to Aegis
below.

**And the documents must be on the branch the waker runs from.** The waker resolves
`docs/coordination/tasks/<seat>/…` from its working directory, which is the main
checkout on `track/infra-foundation` — where `LYRA-HEARTBEAT.md` already exists. My
packet branch does not carry it, so a waker built and run from *this* worktree would
find no documents and call nobody. That is stated rather than hidden: the code is
correct, the deployment needs the document present.

## 9. Known limits, stated rather than hidden

- **A heartbeat nudge is only as good as the document behind it.** The waker calls
  the seat to the file; what the seat does is the file's business. If a document
  says "nothing to do", the call is noise — which is exactly why only seats with a
  document are called at all.
- **The seat map is the fleet.** A seat not in `.seatloom/dispatch-seats.tsv` is
  never called, heartbeat or otherwise. `seat-aegis-001` is not in the map (Aegis
  has his own self-wake path, `4fc7f0d`), so `AEGIS-HEARTBEAT.md` is not reached by
  this daemon — measured, and it is the map's shape rather than a defect here.
- **The document path is resolved from the working directory.** A waker started
  from somewhere other than a checkout will find nothing and call nobody, silently
  except for its own log line. `SEATLOOM_TASKS_DIR` overrides it.
- **30 minutes is a judgement.** It is stated and justified in §5, and it is a knob
  — but the right number is an operational question the operator should answer
  after watching it for a round.

## 10. Out of scope, unchanged

The dispatch state machine, the stale-claim backstop logic, the seat map's
two-column meaning, and the live database. The only behavioural change is the third
`(None, 0)` arm and the message it sends.

## 11. Forward recording (D5 Phase 2 Lane B)

One memory, `source_kind=canonical_event`, `source_ref` = the delivery
announcement, `confidence=quoted`, topic `topic-waker-heartbeat-call` (opened for
this):

| Memory | Kind | Why it passes the three-way test |
|---|---|---|
| `mem-z0DVqyt3z6YDL074VlNwE` | decision | The third arm, the two separate cooldowns and why, the document guard, and "check for an established convention before defining a new path". Changes what a future seat does when asked to make an idle seat do something; exists nowhere else; evidenced by the old `(None, _) => {}` arm and the 1-of-6 measurement. |

Nothing else was recorded. The 30-minute figure is stated as a judgement in §9
rather than remembered as a fact, and no preference was touched — Mr Zhang's
corrections are Aegis's to record.
