# Verdict — the integration round, attacked adversarially: "the app reads real data and says honestly when it does not"

- **Verification dispatch**: `disp-integration-round-verify` (issued by seat-aegis-001), acceptance
  criteria = Lyra's plan §5.2 slice 3 (`LYRA-2026-10-17-next-round-plan-v1.md`).
- **Verdict event**: `ev-verdict-integration-round` — the pre-declared id, emitted exactly as required.
- **VERDICT: FAIL — narrow, and scoped to the product, not to any seat's branch.** The claim under
  test is false in the product the operator is running: **on `track/infra-foundation` the app still
  cannot read the seats** (`list_seats` fails on all 8 live rows — exact output below), and
  **`sessions` and `dispatches` still claim LIVE while the read failed** (exact matrix below). Every
  individual piece that would make the claim true is built and verified — the badge fix landed on
  Mira's branch *during* this verification and passes my adversarial matrix — but none of it is in
  the product. Integration is the missing step, and it is not mine.

## What I attacked, with the exact input and the exact output

### 1. A read that fails — on TRACK (the current product)

`provenanceForTab` driven directly (the real function, via a temporary vitest probe, since deleted),
`backendHydrated=false`:

| tab | discovery | output |
|---|---|---|
| `sessions` | unknown | **`live`** ← claims LIVE over a failed read |
| `dispatches` | unknown | **`live`** ← claims LIVE over a failed read |
| `sessions` / `dispatches` | ok / empty / failed | **`live`** (all four discovery states) |
| `live-seats` | unknown / ok | `live` |
| `live-seats` | empty / failed | `no-live` / `discovery-failed` |
| `timeline`, `workitems`, `seats`, `artifacts`, `playbook`, `inbox`, `all-projects`, `dashboard` | any | `fixture` — correct |

So the thing the dispatch said to verify above all — *a surface whose data came from a failed read
must not claim LIVE* — **is still false on track for `sessions` and `dispatches`.** The eight
store-backed tabs are badged honestly (`fixture`), but those two are not.

### 2. The same attack against the FIXED function (Mira's branch, `7a1b12e`, landed 04:02Z during this verification)

| tab | input | output |
|---|---|---|
| `sessions` | `backendHydrated=false` | **`fixture`** — the lie is closed |
| `dispatches` | `backendHydrated=false` | **`fixture`** — the lie is closed |
| `sessions` / `dispatches` | `backendHydrated=true` | `live` — still LIVE where it is true |
| `live-seats` | any hydrated, discovery ok/unknown | `live` |
| `live-seats` | discovery empty / failed | `no-live` / `discovery-failed` |
| everything else | false / true | `fixture` / `seed` (timeline: `ledger` on her branch) |

The fix is `LIVE_TAB_BACKING` — a measured declaration of what backs each of the three tabs
(`live-seats: tmux`, `sessions: database`, `dispatches: database`) — and the database-backed branch
returns `backendHydrated ? 'live' : 'fixture'`. **I had independently derived the same distinction
before reading her commit** (`cmd_list_tmux_sessions` takes no `State` and shells out to tmux, so
live-seats is legitimately independent of the Postgres read; the other two reach Postgres), so the
fix matches the measured reality rather than papering over it. Her 6 new tests fail the read and
assert non-LIVE; the full suite on her tip is **193/193**, tsc clean, Rust **271/0**.

### 3. A read that succeeds — the badge says LIVE where it is true

`backendHydrated=true`: `sessions`/`dispatches` → `live` (fixed function and track alike),
`live-seats` → `live` with discovery ok. True claims survive. ✓

### 4. The three LIVE_TABS individually — data paths measured, not assumed

- **`live-seats`** — `LiveSeatsView` reads `useLiveSessionsStore` (tmux mirror), **not**
  `useDataStore`; its liveness is discovery-gated (HOLD-3). A failed Postgres read must not
  downgrade it, and an empty discovery must not let it claim LIVE. Both directions pinned by her
  tests and by my matrix. Correct as it stands, on track and on her branch.
- **`sessions`** — `SessionsWorkspace` = `HistorySessionsPanel` (its own read through the history
  tools, own `MeasuredError` render) + `TmuxMirrorWorkspace` (tmux). No store data. On track its
  badge claims LIVE regardless of the read — wrong signal, now fixed on her branch.
- **`dispatches`** — `DispatchInboxView` polls `api.listDispatches()` **directly** (not through the
  hydration) and renders its own error (`加载派单失败:{error}`). So its rows are not fixtures when
  hydration fails — but on track it wears a LIVE badge even while showing its own error, which is
  the inconsistency the fix closes.

### 5. Does the app read real data? — the harness blocker, measured on live right now

My probe (the same core methods the Tauri commands wrap, read-only) at **track's tip** against live:

```
OK   list_projects          rows=39
FAIL list_seats             seats row "seat-aegis-001": column default_harness is out of domain:
                            "Pi" is not a valid Harness value (expected one of: Direct, Stepcode, Custom, Custom(<payload>))
OK   list_workitems 7   OK list_handoffs 3   OK list_artifacts 15   OK list_sessions 5
OK   list_events(400) 400  OK list_documents 0
OK   bindings[seatloom    ] count=7 ql_bound=false
OK   bindings[ql_delivery ] count=1 ql_bound=true      ← the role defect is FIXED in live (custom(worker))
OK   bindings[ThoughtOnly ] count=0
```

**The product still cannot read the seats.** Live is `default_harness='Pi'` ×8 (the operator's
ruling, script run); track's `Harness` has no `Pi`; one bad row fails the whole read; hydration's
`Promise.all` rejects; the store keeps its seeds; 8 of 10 tabs render fixtures. The two ways out
both exist and neither has happened: the code fix (`packet/nimbus/harness-pi`, **verified PASS by
me on 2026-10-17**) is not integrated, and the data fix (`af0275a`'s `fix-seat-harness-domain.sh`,
which sets `Stepcode` after Nimbus's measurement that the true value is Stepcode) has not been run
against live. Meanwhile `af0275a` records the ruling's reversal — the harness domain was right and
the rows were wrong — so which of the two closes the blocker is now a live-data question, not a
code one. The role defect (`role='worker'`) **is** closed in live: `ql_delivery` reads 1 binding.

### 6. The two re-emitted events — one correct, one absent, and the id contract violated again

- **Nimbus's half is correct.** `ev-blocked-on-whom-dto-landed` exists (03:57:42Z), its
  payload says explicitly it is a re-emission (`reissue_of: ev-blocked-on-whom-dto-delivered`,
  `not_a_second_verification: "This is the SAME measurement re-stated under the declared id"`,
  original timestamp, boundary honoured), and it **did** release its slice: `disp-blocked-on-whom-ui`
  went `blocked` → `ready` (measured; the edge resolves on `SELECT 1 FROM canonical_events WHERE
  id=$1`).
- **Mira's half was never emitted.** `ev-first-screen-default-tab-landed` is absent from the ledger;
  her dispatch (`disp-first-screen-default-tab`) is `done` — closed without its required
  announcement, exactly the detection gap the plan documented.
- **Both dispatches' own announcements went out under server-minted ids** —
  `ev-J0i6eYQzPB2eT5ppNnYm3` and `ev-49Uizr1yc3D85D84CNqyf` — not the ids the plan's §5.1 declared
  (`ev-reemit-stalled-announcements-landed`, `ev-provenance-badge-honest-landed`). Nimbus's payload
  records the gap himself: no tool compares a dispatch's required announcement against what was
  actually emitted.
- **The consequence, measured:** `disp-integration-round-verify-v2` — the verifier correctly edged
  on the two fix announcements — is **`blocked`**, on two event ids that do not exist. That is the
  fifth instance today of a check satisfied while the thing it stands for is absent, and the second
  one that is Lyra's own.

### 7. The empty states — unchanged, and pinned

On Mira's tip: the seat empty state ("这个项目下没有绑定任何席位 … 不是加载失败") and the two project
states (read-failed vs database-empty, explicitly distinguished) are the 10-16 verified ones;
`seatAttribution.test.ts` + `AllProjectsView.hydration.test.tsx` **13/13** pass, full suite 193/193.
Nothing in this round moved them. ✓

## What this verdict does not assert

- It does not assert the rendered badge: nobody has looked at the pixels (the screen is locked from
  my seat — screencapture fails, assistive access denied). The rule is pinned over the pure
  function, which is where the defect lived; the amber banner's readability is the operator's
  judgement.
- It does not assert the individual branches are wrong: the badge fix passes my adversarial matrix,
  the harness fix passed yesterday, Mira's UI fixes passed on 10-16. The FAIL is about the
  **product** — the claim is false in what the operator is running.
- It does not assert the harness question is settled: `af0275a` re-ruled the data toward Stepcode
  while live still holds Pi; whichever way it closes, the closing act (integrate the variant, or run
  the data script) has not happened and is not mine.
- No live writes, no merges, no restarts by me; the podman container was not touched this round.

## Bottom line

**FAIL, narrow.** The claim "the app reads real data and says honestly when it does not" is false in
the product today, with exact inputs and outputs: `list_seats` fails on all 8 live rows (hydration
dead, 8 tabs fixtures), and `sessions`/`dispatches` claim LIVE on a failed read. The fixes that
would make it true exist and are verified on branches — the badge fix landed during this
verification and closes the lie exactly — and the round's own announcement-id contract was violated
a second and third time, leaving the correctly-edged verifier blocked on ids that do not exist.
What remains is integration and two operator acts, named with their owners above.
