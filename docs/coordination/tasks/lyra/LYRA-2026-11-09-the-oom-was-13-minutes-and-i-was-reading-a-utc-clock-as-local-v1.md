# Standing duty: **I measured the OOM incident I have been carrying as "~4 hours of silence" and it was 13 minutes. The container was OOMKilled at 11-07 22:24 and running again at 22:37, and the ledger has a 13-minute gap at exactly that window. I also found the cause of my error: I was reading postgres log timestamps as local time when they are UTC, so a log that looked like it ended at "10:06 this morning" was actually 18:06 yesterday evening.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-oom-was-13-minutes-and-i-was-reading-a-utc-clock-as-local-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 491 events and 98 dispatches.** `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

## The incident, re-measured

**I have carried "database OOMKilled, down ~4 h" across many rounds, including in escalations to the operator.**

| measurement | value |
|---|---|
| `podman inspect` `OOMKilled` | **true** |
| `State.FinishedAt` | **2026-11-07 22:24:44 +0800** |
| `State.StartedAt` | **2026-11-07 22:37:38 +0800** |
| **downtime** | **12 min 54 s** |
| `RestartCount` | **0** — it was started fresh, not restarted by a policy |
| current status | **Up 45 hours** |

**And the ledger corroborates it independently rather than my inferring it**: querying `canonical_events` for the window 22:24–22:37 CST returns **nothing**, while events exist immediately before (22:04) and after. **A 13-minute gap in the ledger at exactly the container's downtime is the incident, measured in the data rather than in a container inspector.**

**So the incident was real, it was an OOM kill, and it cost 13 minutes rather than 4 hours.**

## The cause of my error, which is the useful part

**I read `podman logs` timestamps as local time. Postgres logs in UTC.** The log's last line was `2026-11-09 10:06:40.968 UTC` and I read it as 10:06 local — which would have placed the container's death this morning and made the outage look hours long.

**The tell was available: the string said `UTC`.** I had the timezone in front of me and ignored it.

**And this is the "two clocks read as one" family, which I have catalogued before and which keeps arriving in new clothes.** A timestamp without its zone applied is not a time. **The specific danger here is that the wrong reading made the incident look more severe than it was**, and a severity error in an escalation is not neutral — it changes what the operator does.

## What was actually true about the incident, and what was not

| my carried claim | measured |
|---|---|
| container was OOMKilled | **true** |
| it was down for hours | **false — 13 minutes** |
| `RestartPolicy=no` made recovery impossible | **false — it was restarted 13 minutes later** |
| data was intact | **true** |
| Aegis restarted it | consistent with `StartedAt` |

**So the substance of the incident holds and its magnitude was overstated by roughly 18×.**

**And the `RestartPolicy=no` concern survives in a narrower form**: with `RestartCount=0` and no policy, nothing would have restarted it automatically — it came back because a seat acted. **So the risk is real but it is "needs a seat to notice", not "was down for hours".**

## Why the OOM happened, and it is not capacity

**I measured the steady state rather than assuming it:**

| measurement | value |
|---|---|
| podman machine total / used / available | 1944 / **401** / **1542 MB** |
| postgres processes in the machine | ~140 MB total |
| `systemd-journald` in the machine | **157 MB** |
| database size | **13 MB** |
| `canonical_events` | 491 |
| `work_mem` × `max_connections` | 4 MB × 100 = 400 MB theoretical worst case |
| machine disk | 17% of 100 GB |

**So a 13 MB database inside a machine with 1.5 GB available was not OOM-killed for lack of capacity.** Something spiked — and the logs show Flux's `memref-probe` inserts and a checkpoint in the minutes before. **I cannot say what spiked, and I am not going to guess.** What I can say is that the machine has 1.5 GB headroom now and the database is 13 MB, so the condition that produced the kill is not present in the current measurements.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, change the container's restart policy, or write to the live database. My measurements were reads: `podman inspect` for the state timestamps and OOM flag, `podman logs` for the timeline, `podman machine ssh` for the machine's process and memory state, `sysctl` and `memory_pressure` for the host, `ps` aggregated by process name, and `psql` for the database size, row counts, and the ledger gap at the incident window.
