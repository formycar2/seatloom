# Delivery: waker hardening — a stale in_flight claim no longer silences the fleet

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-10-waker-stale-claim-hardening-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| tags | nimbus, waker, dispatch, deadlock, stale-claim, hot-reload |
| governing dispatch | `disp-q__RsEv13FNQ78nCbGd8c` (blocking defect) |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/d5-mcp-recall` @ `e65d438` |
| proof | `crates/seatloom-mcp-emit/proof/waker_stale_poke_proof.sh` (5/5 PASS, re-runnable) |

---

## 0. Summary

The waker had a guaranteed deadlock, and it fired: a seat whose turn ended
WITHOUT `complete_dispatch` (context exhaustion, an error, an interrupt, a
session restart, a kill) leaves its dispatch `in_flight` forever. The waker's
"busy → it self-drains" rule then never called that seat again — one stuck claim
stopped three seats for twelve hours (Onyx visibly idle, two ready dispatches
queued behind it, and nimbus/lyra's work depending on them).

`in_flight` is a **claim, not a liveness fact**. The hardening makes the waker
act on that distinction:

| # | Fix | Default |
|---|---|---|
| 1 | **Stale-claim detection** — an `in_flight` older than the threshold whose pane shows no agent activity is poked anyway | `SEATLOOM_STALE_AFTER_SECS=900` (15 min) |
| 2 | **Periodic catch-up** — the idle+ready scan now repeats, instead of running once per (re)connect | `SEATLOOM_SWEEP_SECS=90` |
| 3 | **Seat-map hot reload** — the map is re-read when the file changes (every sweep) and once on demand for an unmapped seat | `SEATLOOM_SEAT_MAP` (unchanged) |

Plus the pane-liveness rule the dispatch specified: **no bare `ps | grep pi`**
(pi is a 22-byte symlink; a bare grep would read the whole fleet as down). The
pane's processes are inspected directly, and the log names **which** agent it saw
— because the pane may host a different one than expected.

**What it deliberately does NOT do:** rewrite a stuck `in_flight` to `failed` or
`ready`. That is ledger semantics (Onyx/Lyra's lane) and it would hide the real
cause. The waker only "goes and calls the seat"; the seat decides how to close
its own task. Poking is safe by construction: the seat's AGENTS.md rules that an
"already in_flight" answer from `get_next_dispatch` IS its current task, so a
nudge resumes it rather than duplicating it.

---

## 1. How the stale decision is made

Per sweep, per mapped seat, one round trip:

```
ready_n, in_flight_id, claim_age_secs (computed by Postgres)
```

- **idle + ready work** → poke (the lost-NOTIFY case: the NOTIFY fired while the
  seat was busy, so nothing replays it).
- **in_flight, age ≥ threshold** → interrogate the pane:
  - a recognised agent in the pane → **do not poke**, and log which agent;
  - no agent → **poke**, logging the claim's age and the pane state.
- **in_flight, age < threshold** → leave alone (the healthy case).
- **in_flight with no timestamp** → leave alone. An unknown age is not evidence
  of a stuck seat.
- A per-seat nudge cooldown (`SEATLOOM_POKE_COOLDOWN_SECS`, default 120s) keeps a
  sweep from spamming a seat that was just called.

### Pane liveness

```
pane pid (tmux list-panes) -> ps -o comm= -p <pid>  (the pane's OWN process)
                            -> ps -ax -o pid=,ppid=,comm= filtered by ppid  (its children)
match either against SEATLOOM_AGENT_PROCS (default: pi,node,claude,gemini,
codex,opencode,cursor,stepcode,stepgo), on the BASENAME
```

Both the pane's own process and its children are considered: a pane created with
`tmux new-session <cmd>` runs the agent as the pane process with no shell in
between, so checking only the children would call a working agent idle.

---

## 2. Two real bugs found by the proof, fixed here

The proof exists to demonstrate the fix; it immediately earned its keep.

**Bug A — the claim age mixed two clocks.** The age was computed in the daemon
(`Utc::now() - stored_at`) from a timestamp stored by Postgres. The local
Postgres container runs **~2h17m behind the host**, so the age was wrong by that
much in whichever direction: a 40-minute-old claim read as ~3 hours, and a claim
taken a moment ago could read as stale. Fixed: the age is now
`EXTRACT(EPOCH FROM (NOW() - claimed_at))::float8` **computed by Postgres**, so
both operands share one clock. A negative age (the disagreement's other
direction) is not treated as stale — a claim cannot be from the future.

**Bug B — `pgrep -P` is unusable for this on macOS.** It silently skips the
calling process's own ancestors, so a seat's agent is invisible *exactly when the
checker runs inside that seat's subtree* (verified live: `pgrep -P <pane_pid>`
returned nothing while `ps` showed the `pi` child). Fixed: `ps` with an explicit
ppid filter, which also returns the command names in the same pass — which the
agent check needs anyway.

A third finding, reported rather than changed: the nudge message contains an
emoji and em-dashes, and `tmux send-keys` with a non-ASCII character does **not**
reach a bare zsh pane (verified: `send-keys "a—b"` produces nothing at all). The
agents' input handles it fine — that is how seats receive these nudges today —
but it means the nudge cannot be verified by watching a pane's text, and a pane
left at a bare shell would not display it. The message is left byte-identical
(changing a cross-seat trigger phrase is a coordination decision, not this
lane's), and the proof therefore observes the tmux **call** through a shim.

---

## 3. Validation

Commands (Rust 1.95.0; the proof uses scratch DB `seatloom_waker_proof`, never
live):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 findings) |
| compile | `cargo check --workspace --all-targets` | PASS |
| unit + DB tests | `cargo test --workspace` | PASS (core 113, mcp-emit 54 + waker 6) |
| proof | `proof/waker_stale_poke_proof.sh` | PASS 5/5 (re-runnable) |

The proof runs the real daemon against a scratch database with a **throwaway tmux
session** as the only mapped seat, so no live seat is ever nudged, and observes
the tmux calls through a PATH shim that logs and forwards. Its five points:

```
POINT 1  STALE claim (in_flight 40 min, pane idle) -> POKED
         waker: seat-waker-proof-001 STALE claim: in_flight disp-stale for 2400.60507s
               with pane idle — poking anyway; the seat resumes its own task
         tmux:  send-keys -t waker-proof-seat Enter
POINT 2  FRESH claim (in_flight since NOW)         -> not poked
POINT 3  idle + 1 ready dispatch (no NOTIFY sent)  -> poked by the periodic sweep
         waker: sweep: idle seat seat-waker-proof-001 has 1 ready dispatch(es) -> ...
POINT 4  a seat added to the map while running     -> nudged, no restart
         waker: seat map loaded ... (1 seat(s)) ... (2 seat(s))
POINT 5  a recognised agent in the pane            -> not poked, and named
         waker: ... in_flight disp-ready for 2407.08s (pane running node) — an agent
               is working; not poking
```

New unit tests: the seat-map parse/change/detect cycle (including "the file
vanished → keep the previous map"), the pane-liveness classes (including "a
DIFFERENT agent still counts as working"), agent basename matching against full
paths, the nudge cooldown, the config fallbacks on nonsense values, and the
claim-age rules (no timestamp → never stale; a future timestamp → never stale).

---

## 4. Escalations and notes

1. **The daemon must be restarted to pick up this build.** The running
   `seatloom-waker` (in the `seatloom-waker` tmux session) is the old binary;
   the hardening only takes effect once it is rebuilt and relaunched — an ops
   action outside this lane, flagged rather than performed.
2. **The seat map lives in the main checkout**
   (`.seatloom/dispatch-seats.tsv`, 6 seats incl. `seat-vela-001`), and the
   running waker predates the vela entry — which is exactly the "new seat needs a
   restart" symptom this dispatch reported. After the restart the hot reload makes
   that a non-issue.
3. **The nudge text's non-ASCII characters** (see §2): reported, not changed. If
   the fleet wants a nudge that a bare shell would also display, that is a
   one-line change to the message — a cross-seat coordination call.
4. **Forward recording for this dispatch** (per the protocol now in AGENTS.md):
   the memories table is not live yet (018 pending), so `remember` cannot be
   called from this session's toolset. Recorded here instead of skipped:
   - *decision* (changes future action): "an `in_flight` row is a claim, not a
     liveness fact — any busy-check built on it must be paired with an age +
     pane check". Not already verbatim elsewhere; evidence = this delivery's §1.
   - *failure* (changes future action): "computing an age in the daemon from a
     DB timestamp mixes clocks — the local container was 2h17m behind; compute
     durations in SQL". Evidence = §2 Bug A and the proof's POINT 1 age reading.
   - *unresolved*: none beyond the escalations above.
5. No `infra/**` change, no whole-branch merge, no live-DB write.
