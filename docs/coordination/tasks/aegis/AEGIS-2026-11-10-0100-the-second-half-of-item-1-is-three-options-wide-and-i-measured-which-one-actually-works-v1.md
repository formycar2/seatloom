| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0100-the-second-half-of-item-1-is-three-options-wide-and-i-measured-which-one-actually-works |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, standing-backlog, session-discovery, option-2-rejected, measurement |

# The second half of item 1 is three options wide, and I measured which one actually works

## The item, restated

The operator's SeatLoom project shows 7 seats in the "seats" view but 6 in the "live seats" view.
`aegis` is the missing one. The mechanism, from the backlog:

- `crates/seatloom-core/src/pty/mod.rs:383` filters tmux discovery to names ending `-seatloom`
- the six seat sessions are `<Name>-<Role>-seatloom`; **aegis is `Aegis-Supervisor`, no suffix**
- so aegis is filtered out before the UI ever intersects

## What I measured today

The first half is fixed and holding: **7 bindings** for `seatloom`, and `seat-aegis-001` **is** in the
`seats` table.

For the second half I simulated the actual intersection rather than reasoning about it. The UI's
`boundMirrorNames` lowercases both sides and intersects discovered sessions with bound seat names:

| discovery | sessions returned | intersection |
|---|---|---|
| as it is now (`-seatloom` only) | 9 | **6** — flux, lyra, mira, nimbus, onyx, vela |
| all tmux sessions, unfiltered | 11 | **7** — aegis joins them |

So relaxing the filter does produce 7, exactly as option 2 claimed. And `parseSeatSession` handles the
no-suffix form correctly: `Aegis-Supervisor` → seatName `aegis`, role `Supervisor`.

## Why option 2 is worse than the backlog thought

The backlog priced option 2 as "product code, needs a rebuild and reinstall". That is true but it is not
the real cost. **The real cost is a cross-project leak, and it is invisible from the filter's own line.**

`cmd_list_tmux_sessions` has **two** consumers. `LiveSeatsView` intersects with bound seat names, so it
would be safe. **`TmuxMirrorWorkspace` and `AttachSessionDialog` do not filter at all** — they render
whatever discovery returns. Today the `-seatloom` filter is the only thing keeping them scoped.

And the machine has **three other projects' tmux sessions carrying the same suffix**:
`NetworkOrchestrator-seatloom`, `QL-seatloom`, `Stepaicloud-seatloom` — real sessions under
`~/Documents/GitHub/project_delivery/`. Their parsed seat names are `networkorchestrator`, `ql` and
`stepaicloud`, and all three **exist in the `seats` table** (`seat-ql-001`, `seat-stepaicloud-001`,
`seat-networkorchestrator-001`). They are simply not bound to `seatloom`.

So the `-seatloom` suffix is doing double duty: it is a discovery convention **and** the de-facto project
fence for two surfaces that have none. Relaxing it in `pty/mod.rs` to admit `Aegis-Supervisor` would also
admit all three other projects into the mirror dialog, in a product whose stated requirement includes
**project isolation**.

**That is the same shape as a mistake I have already made once in this project**: a guard that looks like
it enforces one rule turns out to be the only thing enforcing a different one, and removing it for the
visible reason breaks the invisible one.

## The three options, re-priced

| option | what it costs, measured |
|---|---|
| 1. rename the session to `Aegis-Supervisor-seatloom` | still needs the self-wake script updated in the same change, or aegis stops waking. Small, reversible, **touches nothing shared** |
| 2. relax `pty/mod.rs:383` | a rebuild and reinstall, **plus three other projects leak into two unfiltered surfaces**. Should also add a project filter to those surfaces first |
| 3. accept 6 and explain | zero cost, but the number stays wrong and every future reader has to know why |

My recommendation changes to **option 1**, with one addition the backlog did not have: the rename and the
waker update must land **together**, because a rename without the waker change kills the only thing that
wakes me — and my self-wake is a cron entry, so it would die silently rather than loudly.

## What I did not do

I did not rename the session, did not touch `pty/mod.rs`, and did not update the waker. Renaming my own
session changes how I am woken, which is the one mechanism whose failure is invisible — a dead waker
looks exactly like a quiet hour. That needs your authorisation, and it needs to be done as one atomic
change rather than by me mid-heartbeat.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0, plus `check-unclaimed-work` at 0.
- Three pending cards awaiting you.
- Push: local and remote both at `a47678a`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.
- My own session is `Aegis-Supervisor` and the waker pid is 94069 — both unchanged.

## Needs you

- **Option 1, 2 or 3 for the live-seats count.** My recommendation is 1, and I have not acted on it.
- **Three cards are waiting**, each separate work. Mira's is missing one statement you should read first.
- **The sandbox decision unblocks five dispatches at once**, not one.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed.
