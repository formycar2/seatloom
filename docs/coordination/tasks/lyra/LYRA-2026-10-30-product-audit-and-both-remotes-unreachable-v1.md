# Standing-duty report: the product-audit duty, its finding, and both remotes just became unreachable

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-product-audit-and-both-remotes-unreachable-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, product-finding, blocker, remotes |

## Part 1 — the product-audit duty, done

My heartbeat's item 3.1 says: **find where the product is worse than it claims.** I spent the previous many
rounds on delivery-chain and instrument problems, which are real but are adjacent to item 3.2. This round I
did 3.1, and it produced the finding in
`LYRA-2026-10-30-seven-tabs-say-sample-but-read-live-v1.md`.

**The short version, because it is the most consequential thing I have measured this week:**

Seven of the ten nav tabs read live PostgreSQL for real — `hydrateProjectData` calls
`api.listSeats/listWorkitems/listHandoffs/listArtifacts/listSessions/listEvents/listDocuments`, and live
holds **10 seats, 7 workitems, 15 artifacts, 2942 events**. Every one of those seven tabs wears a badge
reading **`SAMPLE DATA — not yet from live capture`**, whose detail text says the rows are *seed fixtures, a
hand-written baseline, not captured activity*, and then **misdirects the operator to look under the Live
Seats tab for live data — while the data in front of him is live.**

**This is worse than a wrong badge.** A wrong badge costs a glance. **A confidently false explanation costs
him the conclusion that the surface is empty when it is not**, and he has been making merge decisions all
day off surfaces that tell him nothing is real.

The fix is two lines in `ui/src/lib/provenance.ts`: add `dashboard`, `inbox`, `workitems`, `seats`,
`artifacts`, `playbook` to `LIVE_TAB_BACKING` as `'database'`, and correct the `seed` branch's detail text,
which describes a surface that does not exist.

**What changed since I first reported this on 2026-10-29, and it matters:** the classification is now keyed
on *what backs the tab* rather than a tab-name list consulted before `backendHydrated`, and a failed
hydration correctly falls to `fixture` instead of wearing a green LIVE badge. **Someone fixed the half that
was lying in the dangerous direction.** Unchanged: the seven live-reading tabs are still in no set. **The
fix stopped exactly where the lie became harmless-looking rather than where it became true.**

## Part 2 — both remotes became unreachable mid-round, and one of them for a new reason

While pushing the finding:

| Remote | Failure | New? |
|---|---|---|
| `gitlab.basemind.com` | `ssh: connect to host gitlab.basemind.com port 22: Operation timed out` | no — the same outage seen repeatedly today |
| `origin` = `git@github.com:formycar2/seatloom.git` | `Permission denied (publickey)` | **yes, never seen before today** |

I diagnosed the origin failure rather than reporting it as "network":

```
ssh -T git@github.com                       -> "Hi formycar2! You've successfully authenticated"
ssh-add -l                                  -> "The agent has no identities"
ssh -o IdentitiesOnly=yes -i ~/.ssh/id_ed25519 -T git@github.com
                                            -> "git@github.com: Permission denied (publickey)"
```

**GitHub accepts the account, and rejects the key.** The key file exists and is `-rw-------`. The agent holds
no identities. So this is not a connectivity problem and not a missing file — **it is the key itself being
refused, which is an operator-side credential matter I cannot repair and should not try to.**

**What is at risk, exactly:** `ff7e922` (this round's finding) is local-only. Both branches are intact
locally, `git fsck` shows no corruption, and the previous 5 rounds of work are still on origin from earlier
pushes. **So nothing is lost — but this round's finding, which is the most consequential of the week, is on
one machine.**

## What I am not doing

I am not generating a new SSH key, not editing `~/.ssh/config`, not adding a remote, and not trying an HTTPS
remote with an embedded token. **Every one of those is an operator-side action on his credentials, and a
seat working around an access refusal is the thing the protocol exists to prevent.** It is reported, not
repaired.

I am also not treating this as a reason to stop the product audit. **The finding is measured, documented,
and committed locally; it does not become less true because a remote refused it.**
