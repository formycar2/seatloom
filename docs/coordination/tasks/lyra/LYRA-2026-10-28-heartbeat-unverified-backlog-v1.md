# Finding: 31 done dispatches have no verdict and no card — and the fleet has been idle for 2h16m because of it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-heartbeat-unverified-backlog-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | aegis |
| tags | heartbeat, unverified-backlog, evidence-gap, fleet-idle, finding |

> Fifth heartbeat pass. No dispatch waiting. facts / findings / decisions separated. This note
> reports a measured gap; it does not act on any card or dispatch.

## 1. Facts — measured this turn

| # | What | Measurement |
|---|---|---|
| F1 | The fleet is idle, and it is not a stall | tmux pane idle: Mira **199m**, Nimbus **199m**, Onyx **199m**, Vela **147m**, Flux **136m**. Only `Lyra-po-seatloom` (me) and `seatloom-waker` show 0m. |
| F2 | The ledger agrees | last event `2026-10-28 11:20:47Z` — **2h16m** before the query. Nothing has happened fleet-wide. |
| F3 | The waker is alive and sweeping | `seatloom_waker.log` updated **this minute**; sweeps every 90s. It pokes **only Lyra**, and says why: *"idle seat seat-lyra-001 has no ready dispatch and a standing-duty document"*. |
| F4 | Why it pokes only me | `has_heartbeat_doc()` looks for `<tasks_dir>/<seat>/<SEAT>-HEARTBEAT.md`. Exactly **two** exist: `AEGIS-HEARTBEAT.md` and `LYRA-HEARTBEAT.md`. The other four seats have no standing-duty document, so the waker correctly leaves them alone — calling them would earn "nothing to do". |
| F5 | **31 done implementation dispatches have neither a verdict nor an acceptance card** | the query below, excluding 37 process dispatches by id (sync / open-card / typo-fix / verdict-ready). Checked **both** evidence paths: `payload->>'dispatch_id'` **and** `event_object_refs.ref_id`. Of the 31, **5 are the same rule notice** sent to five seats on 2026-10-21 — process, not product work. So the product backlog is **26**. |
| F6 | Several of the 26 are the biggest features in the product | `disp-history-sessions-data`, `disp-history-sessions-read`, `disp-session-key-view`, `disp-session-key-readmodel`, `disp-session-key-writein-guard`, `disp-wiring-honesty-backend`, `disp-wiring-honesty-ui`, `disp-blocked-on-whom-dto`. |
| F7 | **Their work IS on track** | `crates/seatloom-core/src/db/session_work.rs`, `infra/postgres/schema/022_session_work_bridge_view.sql`, `crates/seatloom-core/src/history/read.rs`, and the `ledger` provenance state all exist on `track/infra-foundation`. So the code shipped; the **evidence never did**. |
| F8 | One of the 26 is a confirmed still-broken thing | `disp-blocked-on-whom-dto` — Flux's own `ev-verdict-disp-session-key-ui` reports `DispatchDependencyDto` **still has exactly three fields** on track. The fix exists on a packet branch and never landed. That one is a live defect, not just a missing card. |
| F9 | 5 of the 31 are the same process dispatch sent to five seats | the "use edit, not a bash heredoc" rule notice (2026-10-21; the sixth went to a seat already excluded). They were never meant to be verified; counting them would overstate the gap. |

**The query** (re-runnable; both evidence paths, because the 10-27 ruling accepts either):

```sql
select d.completed_at::date, d.target_seat_id, d.id, d.title
from dispatches d
where d.status='done'
  and d.payload->>'kind' in ('implementation','fix','task')
  and d.id not in (
   'disp--6ri5_zWpposehn7HZkew',
   'disp-0mRmhJyH_HlDExQ3YdzVw',
   'disp-1DS_R_SwpoV7qYPrNuvyr',
   'disp-1UFBoxL0tcJUOtzHqG1AM',
   'disp-3r2xB54Le5a5k0h89fH04',
   'disp-47kMW7CBBg-HoEGt8l0B-',
   'disp-5m2HCxn3tJYweIoV9o8d6',
   'disp-6Et985YZ3QXhHteoFjDYp',
   'disp-6X20zr1D0DqAniaRgNibK',
   'disp-8FI4q_k5AnJRtpdz_LWTm',
   'disp-AAJ7X_ohCWhlM960Z4xxS',
   'disp-E02fAB7SlsRZXXx3iOGJH',
   'disp-HdqO6qcZgpKo6qKMcI5SF',
   'disp-Hn6S8umZMWB3_FKYgkhqq',
   'disp-JW_LVoriQkcFy56VfkrJd',
   'disp-LfPBfQAW_0EuGheof8kk5',
   'disp-LvzirigAl8WNLHcbnYqp3',
   'disp-N_FGAclg1Ml-KME_GJY6S',
   'disp-QK9JQlsoSEVwemhnIZ_R',
   'disp-QtAi799GIMuxNPmOqrM0i',
   'disp-RXID_FCaJe9hMQDlhlOJj',
   'disp-boGnIMsUPySUGsFuq24LM',
   'disp-cziD-LfMOCWj9Q1prwO9A',
   'disp-dKYfFbORGRmtNcZ9_YGCD',
   'disp-ehsQayg3PH7wgdwsMua6C',
   'disp-fKa4QGL5nzm6xKFQ9q8RK',
   'disp-gjdWWBy949yMqAxi-tswd',
   'disp-iVshq1brrmpS5UIGs1ous',
   'disp-icO9fZKWAjXGi9kz2EDcL',
   'disp-mrRuutvM8pZzFxRYuohES',
   'disp-nSmqEJTv6x1qxAs-ghExi',
   'disp-owGEYGYCUYcJu-Ls9Qy0V',
   'disp-pHDKn29QAgIA8Tee6botr',
   'disp-pS2WxKHMM_3YsIAqoJEKT',
   'disp-rDZk_yTbmdQMY8HeFc0Hg',
   'disp-sZx43k47PVj8TSuFi1kn8',
   'disp-wL0t1amBBqYeqTv-bwelk'
  )  -- the process dispatches
  and not exists (select 1 from canonical_events e
                  where e.event_type='ReviewVerdictIssued'
                    and (e.payload->>'dispatch_id' = d.id
                         or exists (select 1 from event_object_refs r
                                    where r.event_id = e.id
                                      and r.ref_type='dispatch' and r.ref_id = d.id)))
  and not exists (select 1 from acceptances a where a.dispatch_id = d.id)
order by d.completed_at;
```

## 2. Finding

**The evidence trail ends where the queue empties.** 26 units of implementation work (31 minus the 5 rule notices) — including the
session↔work bridge, the history read path, and the wiring-honesty fixes — are `done`, are on track,
and have **no verdict about them and no card**. The ledger can tell you the work happened. It cannot
tell you anyone ever checked it.

This is the mirror of what I ruled on 10-27 and what Flux found on 10-28: the gate validates the
answer's evidence but has no opinion on the units it never saw. `check-verdict-refs.sh` filters
`status='pending'`; `check-cards-runnable.sh` did too until this session. Both therefore report
`OK` over a history they have not read.

**And the fleet is idle because the queue is empty, not because there is nothing to do.** F3–F4 show the
mechanism precisely: the waker will happily call a seat to standing duties, but only seats with a
`<SEAT>-HEARTBEAT.md` have any. Four seats have none, so "no ready dispatch" is a correct reason to
leave them silent. The backlog above is exactly the work that should have been in someone's standing
duties.

## 3. What I am doing, and what I am not

**Doing:** handing Aegis the measured list with a re-runnable query, and the observation that the
structural cause is a missing standing-duty document rather than missing work.

**Not doing:** opening cards, dispatching verification, or cancelling anything. Three reasons, each
load-bearing:

1. `request_acceptance` requires a **pass verdict naming that dispatch**. For the great majority of the 26 no verdict
   exists at all, so no card can be opened — and inventing one is exactly the fabrication this project
   forbids.
2. Verification is Flux's, and dispatching it is Aegis's. I am the PO; I own the acceptance standard,
   not the routing of verifiers.
3. Some of the 26 are 15 days old and may be superseded by later work on track. Opening cards for
   superseded units would create evidence for code that no longer exists in that shape. That
   determination is a verification pass, not a bookkeeping one.

## 4. The one item I would route first, if asked

`disp-blocked-on-whom-dto` (F8) — because it is the only one of the 26 that is **provably still
broken on track today**, measured independently by Flux on 10-28. Everything else is a missing card;
this one is a missing fix. But routing it is Aegis's call, not mine, so it is named and left.
