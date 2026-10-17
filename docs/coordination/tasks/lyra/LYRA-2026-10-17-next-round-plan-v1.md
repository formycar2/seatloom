# Next-round plan — the app is a mockup right now, and the fix is already built

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-17-next-round-plan-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| depends_on | LYRA-2026-10-16-next-round-plan-v1.md, LYRA-2026-10-16-first-screen-contract-v1.md |
| tags | next-round-plan, decomposition, product-truth, integration, ql |

**Owner:** Lyra (PO). **Raised by:** Aegis's dispatch `disp-C2a6xeJUDA08dEAycc3WK` (2026-10-17
11:41 heartbeat). **Measured on:** `track/infra-foundation` @ `575d85d`, the live database, the
live ledger (1310 events, 180 dispatches), and the installed application binary. Every claim below
carries a file:line, a command, or a measured value. Where I could not measure, it says so.

---

## 0. The one fact that gates everything, and it is not on anyone's list

**The application cannot read the database, so every screen that reads through the data store is a
mockup. This started this morning and it is a regression, not a long-standing defect.**

The chain, each link measured:

| # | Link | Evidence |
|---|---|---|
| 1 | The operator ran `scripts/fix-seat-harness-to-pi.sh` this morning, per his own 2026-10-17 ruling. **All 8 live seat rows now carry `default_harness = 'Pi'`.** | `select id, default_harness from seats` → `Pi` × 8 |
| 2 | `Harness` on track is `Direct, Stepcode, Custom(String)`. There is no `Pi`. | `crates/seatloom-core/src/objects/session.rs:55` |
| 3 | `Harness::from_str("Pi")` falls through to `parse_custom_tagged("Pi", "Custom")`, which requires the literal `Custom(...)` shape. A bare `Pi` does not start with `Custom`, so it returns `None` → **`Err`**. | `session.rs:126-134`, `:151-160`; `HARNESS_EXPECTED = "Direct, Stepcode, Custom, Custom(<payload>)"` at `:145` |
| 4 | Every seat read validates that column. `row_to_seat` → `check_seat_domains` → `check_domain::<Harness>` → `Err`. | `repositories.rs:2899-2916`, `:2855-2875` |
| 5 | So `list_seats` fails on **every row**, not one. | consequence of 3+4 |
| 6 | `hydrateFromBackend` puts `api.listSeats()` inside one `Promise.all` with seven other reads. One rejection rejects the whole thing. | `ui/src/stores/useDataStore.ts:1589-1598` |
| 7 | The `catch` sets `backendHydrated: false` and leaves the store holding `INITIAL_DATA` — the in-memory mock. | `useDataStore.ts:1713-1718`, `:1392-1395` |
| 8 | **Therefore 8 of the 10 NavRail tabs render fixtures**, and have since the operator ran the script. | measured below, §1 |

**Corroborated independently, and called rather than read off code:** Nimbus's
`packet/nimbus/harness-pi` records *"`list_seats` really succeeds — called, not read off the code"*
with his `Pi` variant in place, against the same live rows. So the failure is exactly the missing
variant, and nothing else.

**Consequence for this plan.** Aegis wrote that until the Harness change is merged, no amount of
other seats' work moves the product — and told me not to write a plan that waits on someone else's merge. He is right about the blocker
and the plan below therefore does not propose new surfaces. It proposes **integrating what is
already built and verified**, which is the only thing that moves the product this week.

---

## 1. Q1 — surfaces that look wired but aren't

Measured surface by surface. "Mock now" means the tab renders `INITIAL_DATA` because of §0.
"On track" means in the build the operator is running; "on branch" means built and verified but
not integrated, therefore **not in the product**.

| # | Surface | Looks like | Actually | Evidence | State |
|---|---|---|---|---|---|
| 1 | 8 of 10 NavRail tabs — timeline, workitems, seats, artifacts, playbook, inbox, all-projects, dashboard | real data | `useDataStore`, which is `INITIAL_DATA` | §0 links 6–7; `grep -oE "api\.[a-zA-Z]+\|useDataStore" ui/src/views/*.tsx` | **mock now** |
| 2 | The **LIVE** badge on `live-seats`, `sessions`, `dispatches` | this surface is live | `provenanceForTab` returns `'live'` **before** consulting `backendHydrated`, so the badge never depends on the read succeeding | `ui/src/lib/provenance.ts:65` — `if (LIVE_TABS.has(tab)) return 'live';` precedes `:66` | **the lie is on track** |
| 3 | The project switcher / any project's seat list | seats belonging to this project | **all 8 seats under every project**, by a v0.1 compromise whose own comment says multi-project partitioning lands when the schema gets `project_id` FKs | `useDataStore.ts:1623` — `const seats = seatsDto.map(...)` inside `for (const p of projectsDto)` | fixed **on branch** (`f316201`) |
| 4 | A seat's role | the role it is bound to | an out-of-domain role is **silently swallowed** and the seat renders with a coerced default. Live has `seat-ql-001 → role 'worker'`, outside `SeatRole` | `useDataStore.ts:1610` `catch { set(p.id, {}) }`; `repositories.rs:2978-2983` validates `SeatRole`; live `project_role_bindings` | **the swallow is on track**; the domain fix `575d85d` is on track but **not applied to live** |
| 5 | `ql_delivery` in the app | QL is adopted and visible | **QL is not in the app at all.** With hydration failed the store shows `INITIAL_PROJECTS` = `p-1/p-2/p-3`, so the real project list — 39 rows including `ql_delivery` — never renders | `useDataStore.ts:1392`; live `select count(*) from projects` → 39 | fixed **on branch** (`cc0e409`) |
| 6 | The QL project in the switcher | one project | **two rows for one directory**: `project_delivery/QL` (corpus-derived, 021) and `ql_delivery` (adopted), identical `home_path` | live `select id, home_path from projects where home_path='…/project_delivery/QL'` → 2 rows | unresolved |
| 7 | The ⌘K panel's work graph | who is doing what | fed by `MOCK_CHANNEL_DATA[channelId]`, keyed `ch-p1/p2/p3`, while the only live project id is `seatloom` — so it renders `{nodes: [], edges: []}` forever | `ProjectDashboard.tsx:454` | **on track** |
| 8 | The four write forms (workitem / handoff / seat / delegation) | saved | zero `api.*` calls in all four; no create command exists in `main.rs` | `grep -c "api\." ui/src/components/{WorkItemForm,HandoffForm,AddSeatDialog,DelegationOverlay}.tsx` → 0, 0, 0, 0 | fixed **on branch** (`d4a82dd`, adds `CannotStoreNotice`) |
| 9 | The acceptance merge gate | merges only on a green verdict about this work | `event_or_artifact_exists` — existence only, while its own refusal message promises a green verdict | `acceptance_cmds.rs` on track; my 2026-10-16 measurement | fixed **on branch** (`c63d7af`) |
| 10 | My own two slices' downstream | the chain runs | **both stalled on event ids that will never exist.** Nimbus emitted `ev-blocked-on-whom-dto-delivered` (he chose `-delivered`); Mira emitted `ev-vrg-SFOoMv9BBj2jx_uSG` (server-minted). Both dispatches are `done`. The edges name `ev-blocked-on-whom-dto-landed` and `ev-first-screen-default-tab-landed`; **neither exists.** | `query_ledger` for both ids → absent; `get_dispatch` → `done` | **a known failure mode, recurred twice** |

**On #10, stated plainly because it is mine.** The ledger already carries the lesson, recorded
verbatim: a dispatch that names an event id must emit with that id, because the dependency edge
recognizes the id and not the artifact id. It happened twice in one chain today. `complete_dispatch` closes a dispatch whether or not its required announcement was emitted
under the required id, and nothing detects the mismatch. That is the same shape as §0: **a check
that passes while the thing it stands for is absent.** Two slices are dead until someone re-emits
under the right ids, and no tool will tell anyone.

---

## 2. Q2 — what the first screen still lacks

He ruled the direction on 10-16 — *what is the whole fleet doing / what is stuck / what needs my
decision*. The default landing is
still `live-seats` on track (`App.tsx:92`); the change to `dispatches` is on Mira's branch
(`5c5437f`). `LiveSeatsView` shows tmux terminals — it answers *what is each seat's terminal doing
right now*, which is a different question from *what is the fleet working on*. A seat can sit idle
at a terminal holding an in-flight dispatch, or type busily with nothing dispatched.

| Missing piece | What he does today instead | Worth doing? |
|---|---|---|
| **What each seat is working on** — which dispatch, what status, since when | switches to the dispatch tab and reads a table | **Yes — this is his question, and the data is already read there every 5s.** `LiveSeatsView` has zero dispatch awareness (`grep dispatch` → nothing) |
| **Whether the app is reading the database at all** | reads a per-tab badge that lies on exactly 3 tabs (§1 #2) | **Yes, and it is the cheapest high-value item on this list.** One line: `provenanceForTab` must consult `backendHydrated` before claiming live |
| **What needs his decision, in one place** | knows to open the dispatch tab for cards; product rulings live only in the ROADMAP and in SupervisorMessages | **Yes — the K3 gap from the 10-16 contract.** Nothing new needed if the timeline's existing type filter is allowed to express it; that is his call, not mine |
| **The fleet at a glance, including idle seats** | the seats tab, currently mock (§1 #1) | Yes, but strictly downstream of §0 |
| **What is blocked and on whom** | the dispatch tab's dependency column — raw `event:ev-…` on track, translated on Mira's branch | Partly done. The remaining half — *is anyone working on the thing I am blocked on* — is not answerable from the data (10-16 §7 gap 1) |

---

## 3. Q3 — QL's shortest path to actually being used

Measured state:

| Fact | Evidence |
|---|---|
| The directory exists and holds real work — 30+ entries, Huawei OceanStor REST docs, StepMind integration docs, JS services, crawlers | `ls …/project_delivery/QL` |
| It is **not a git repository** | `git -C … rev-parse --is-inside-work-tree` → *fatal: not a git repository* |
| Its MCP wiring already exists and is correct | `.mcp.json`: `SEATLOOM_ACTOR_REF=seat:seat-ql-001`, `SEATLOOM_PROJECT_ID=ql_delivery`, pointing at the shared binary |
| The DB rows exist: project `ql_delivery`, seat `seat-ql-001`, binding role `worker` | live SQL |
| **The binding's role is out of domain**, so every read of that project's bindings fails | `repositories.rs:2978`; live `project_role_bindings` |
| **The acceptance/merge path cannot work there** — `request_acceptance` requires `base` and `branch` to name existing local branches | the tool's own contract |

**The shortest path, in order:**

1. **Run `scripts/fix-ql-role-domain.sh` against live.** Without it, every read of QL's role
   bindings fails and the seat's role is silently wrong. **Only the operator can do this** — it is
   a live-database write, and Aegis is permission-capped on exactly that class of action.
2. **Decide whether QL becomes a git repository.** **Only he can decide this.** It is the fork:
   - *If yes* — the acceptance path works unchanged: branch, commit, card, merge on his click.
   - *If no* — QL still works, with one step honestly absent. Capture works today (`emit_event`
     needs only a `project_id`), memories and topics work, dispatches work. What does not transfer
     is the merge, because there is no branch to merge. The product should say *"this project has no
     repository, so there is nothing to merge"* rather than offering a merge button that cannot
     succeed — which is the same discipline as `CannotStoreNotice`.
3. **Resolve the duplicate project row** (§1 #6). One of `project_delivery/QL` and `ql_delivery`
   is the corpus's auto-derived row; keeping both means the project appears twice and every
   project-grouped count is wrong. **A data decision, therefore his.**
4. **Then, and only then, dispatch work to `seat-ql-001`.** Today it has zero dispatches
   (`select … from dispatches where target_seat_id='seat-ql-001'` → empty), so the seat exists in
   the registry and has never been given anything.

**A correction to Aegis's brief, with evidence.** He wrote that the out-of-domain role's
consequence is that ql_delivery shows zero seats. That is not what the code does. `useDataStore.ts:1610`
catches the failure and substitutes an empty map, and `seatFromDto` coerces a missing role rather
than dropping the seat — so QL would show **all 8 seats** (the v0.1 compromise, §1 #3) with ql's
role silently replaced by a default. The real consequence is quieter and worse: a wrong role with
no signal. And right now neither happens, because §0 means the app is showing `p-1/p-2/p-3` and QL
is not in it at all.

---

## 4. The decision: what the next round is

**The next round is integration, not new build.** One milestone, and it is the one he can perceive:

> **Open SeatLoom and see the real fleet, not demonstration data.** Open the app and see the real fleet: real
> dispatches with real statuses, real acceptance cards, real seats — and where the product has no
> record, it says so.

Everything beneath it is plumbing and already exists:

| Step | What | State |
|---|---|---|
| 1 | `Pi` joins `enum Harness` | built and verified on `packet/nimbus/harness-pi`; **Flux is verifying now** (`disp-bsL-wE5cuc8rHfj1zz1t6`, issued 03:41) |
| 2 | Mira's 11 UI commits — default landing, five honest surfaces, projects page reads the DB, seats by role binding | built; **two acceptance cards opened, waiting on his click** |
| 3 | The role domain fix applied to live | on track (`575d85d`), **not applied** — needs the operator |
| 4 | The provenance badge stops lying | one line, on Mira's branch |
| 5 | Re-emit my two stalled announcements under their declared ids | nobody has done it; no tool detects the omission |

**What I am deliberately not proposing.** No new surface, no new partition, no new interaction — he
forbade it on 10-16 and nothing since has lifted that. No new contract for the work graph, the
producer mapping, or the K3 home: all three are named gaps with their own decisions pending on him
or Aegis, and writing contracts for them now would be manufacturing work.

---

## 5. The decomposition

Edges hang only on event ids verified to exist in the ledger.

| # | Slice | Seat | Dispatch | Edges |
|---|---|---|---|---|
| 0 | This plan | Lyra | `disp-C2a6xeJUDA08dEAycc3WK` | — → emits `ev-next-round-plan-1017-landed` |
| 1 | Re-emit the two stalled announcements under their declared ids | Nimbus + Mira | **`disp-reemit-stalled-announcements`** (new) | none → ready |
| 2 | Make the provenance badge depend on the read | Mira | **`disp-provenance-badge-honest`** (new) | none → ready |
| 3 | Verify the integration round adversarially | Flux | **`disp-integration-round-verify-v2`** (re-issue) | `ev-reemit-stalled-announcements-landed` **and** `ev-provenance-badge-honest-landed` → blocked; pre-declared verdict `ev-verdict-integration-round-v2` |

**Why slice 1 is two seats in one dispatch.** It is one mechanical fix with one cause, and the two
halves are independent: Nimbus re-emits `ev-blocked-on-whom-dto-landed`, Mira re-emits
`ev-first-screen-default-tab-landed`. Splitting it into two dispatches would create two things to
track for one sentence of work. If Aegis prefers two, that is his call.

**Why slice 3 is edged on the announcement, not a gate.** Flux verifies code, so an announcement is
the right edge. The click-level check is the operator's acceptance card.

### 5.1 The event contract

| Event id | Emitter | Type | Required payload |
|---|---|---|---|
| `ev-next-round-plan-1017-landed` | Lyra | `ArtifactCreated` | `contract`: this file's path + commit. Announcement; no `verified` field. |
| `ev-reemit-stalled-announcements-landed` | Nimbus | `ArtifactCreated` | `branch`, `commit`, which ids were re-emitted, and the observation that both blocked slices left `blocked`. Announcement; no `verified` field. |
| `ev-provenance-badge-honest-landed` | Mira | `ArtifactCreated` | `branch`, `commit`, `files`, the exact one-line change, and the failing-read test that proves the badge is not LIVE. Announcement; no `verified` field. |
| `ev-verdict-integration-round` | Flux | `ReviewVerdictIssued` | Pre-declared at enqueue time; Flux must emit exactly this id. |

> **Corrected 2026-10-17 04:0x, and the correction is itself the lesson.** The first version of
> this plan edged Flux's verification slice on `ev-next-round-plan-1017-landed` — the plan's own
> announcement. Because the plan event is emitted *before* the slices are enqueued, that edge was
> satisfied the instant it was written, and Flux's slice came out `ready` while neither fix existed.
> A verifier needs the code to exist, so a verifier's edge belongs on the *fix's* announcement, not
> on the plan that asked for it. The two announcement ids above are declared for exactly that
> reason, and `disp-integration-round-verify` is re-issued edged on both.
>
> This is the fifth instance today of an edge or a check that is satisfied while the thing it
> stands for is absent, and the second one that is mine. See §1 #10 and §0.

### 5.2 Done definition, per slice

**Slice 1 — re-emit under the declared ids.** Each seat re-emits its already-delivered announcement
with `event_id` set **explicitly** to the id its contract named, and the payload states that it is a
re-emission of the earlier event (id, when, why). No re-measurement, no new conclusion. Then the
blocked slices wake on their own.

**Slice 2 — one line.** `provenanceForTab` must consult `backendHydrated` before any branch returns
`'live'`. A surface whose data came from a failed read may not wear a LIVE badge. Prove it with a
test that fails the read and asserts the badge is not LIVE.

**Slice 3 — attack the claim "this reads real data".** At minimum: a read that fails (the badge must
not say LIVE, and the surface must say what is missing); a read that succeeds; the three LIVE_TABS
individually; the empty states. Report exact input and exact output.

---

## 6. What needs the operator, and what needs Aegis

**Operator only:** run `scripts/fix-ql-role-domain.sh` against live; decide whether QL becomes a
git repository; decide which of the two QL project rows survives; click the two acceptance cards
Mira has open (whose builds must arrive with them — the 10-16 lesson).

**Aegis only:** integrate `packet/nimbus/harness-pi` once Flux passes it; integrate Mira's branch
once he accepts her cards; rule on whether slice 1 should be one dispatch or two.

**Mine, already decided:** the round is integration; the milestone is "open the app and see the real
fleet"; no new surface is proposed.
