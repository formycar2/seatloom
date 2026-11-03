# Standing-duty verification: he found the second half of the seat-visibility defect, and I reproduced the mechanism end to end — the live-seats view shows 6 of 7 bound seats because Aegis's tmux session is the only one without the `-seatloom` suffix, and that suffix is a discovery convention being read as an attribution rule

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-the-second-half-of-the-seat-visibility-defect-verified-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, product, finding |

## What he found, and why it is the same defect one layer down

The operator ran the bind script and the binding defect closed: 1 row → 7. **But he then reported seeing 6
seats in the live-seats view rather than 7.** Aegis traced it and found the root cause is different from the
first half.

**I reproduced the whole chain independently rather than taking the trace.**

**Layer 1 — the seats view.** It renders `seats.length` directly, and `seats` is the project-scoped list. With
7 bindings it renders 7. **Verified against the live database:**

```
select s.name from seats s join project_role_bindings b on b.seat_id=s.id
  where b.project_id='seatloom' order by s.name;
aegis, flux, lyra, mira, nimbus, onyx, vela          (7 rows)
```

**Layer 2 — the live-seats view does something different.** `LiveSeatsView.tsx:144` pipes the discovered
sessions through `boundMirrorNames(list.map(parseSeatSession), bindings)`. That function
(`ui/src/lib/project-scope.ts:7`) is a **name intersection**:

```ts
const names = new Set(boundSeats.map((s) => s.name.toLowerCase()));
return discovered.filter((s) => names.has(s.seatName.toLowerCase()));   // name intersection
```

**So a seat appears only if a tmux session parses to its name.** Not if it is bound — if it has a *session*
whose name maps to it.

**Layer 3 — session discovery filters on a suffix.** `pty/mod.rs:383`:

```rust
if !name.ends_with("-seatloom") { continue; }   // skip sessions without the suffix
```

**Measured, the actual tmux sessions:**

```
Aegis-Supervisor              skip    <- the only seat session without the suffix
Flux-Quality&Ops-seatloom     match
Lyra-po-seatloom              match
Mira-UX/UED-seatloom          match
NetworkOrchestrator-seatloom  match
Nimbus-TechArchi-seatloom     match
Onyx-data-seatloom            match
QL-seatloom                   match
Stepaicloud-seatloom          match
Vela-DataWash-seatloom        match
seatloom-waker                skip    <- not a seat
```

**Layer 4 — the intersection.** Nine sessions pass the filter; `parseSeatSession` maps each to a seat name
(`Flux-Quality&Ops` → `flux`, `Lyra-po` → `lyra`, …); the bound set is `aegis, flux, lyra, mira, nimbus, onyx,
vela`. **The intersection is 6** — `aegis` is in the bound set but has no session that survives the filter.

**So the operator sees 6 and every layer is doing what it says.** The seats view shows 7 because it reads the
bindings. The live-seats view shows 6 because it reads sessions. **Two views, two different questions, two
different answers, and neither is wrong.**

## The insight, and it is the load-bearing one

**`-seatloom` is a discovery convention. It is being read as an attribution rule.**

The suffix exists so session discovery can tell SeatLoom's seat sessions from every other tmux session on the
machine. **It is not a statement that a session belongs to the SeatLoom project** — `Aegis-Supervisor` is a
SeatLoom seat that is woken by his own cron rather than by the fleet waker, which is why he was never given
the suffix and why he is deliberately absent from `.seatloom/dispatch-seats.tsv`.

**So a naming convention built for one purpose is silently doing a second job it was never asked to do**, and
the seat it excludes is the one seat whose wake path is different. **That is the same shape as the provenance
badge: a correct rule applied to a set it was not written for.**

**And it is the same shape as the author-privilege defect he found in his own hook on 11-01** — a rule about
a set that did not cover the author. Here the set is "sessions with the SeatLoom suffix" and the member left
out is, again, Aegis.

## The three options he offered, and why option 1 is right

1. **Rename `Aegis-Supervisor` → `Aegis-Supervisor-seatloom`** — small, reversible, but must be synchronised
   with the self-wake script or the wake dies immediately.
2. **Loosen the discovery rule in `pty/mod.rs:383`** — product code, needs a rebuild and reinstall.
3. **Accept 6 and explain in the UI** that Aegis wakes independently and does not take a seat slot.

**I am not choosing between them** — renaming a live session and changing product discovery are both
operator/product decisions, and option 1 in particular has a failure mode he named correctly (the self-wake
script hardcodes the session name, so a rename without a synchronised change kills the wake). **What I verified
is that the three options do address the mechanism, and that the mechanism is what he said it is.**

## What this says about the first half being "fixed"

**The binding defect is genuinely fixed** — 7 rows, 7 seats in the seats view, and the notification channel
went quiet, which he correctly notes is the first proof that check can pass rather than being a permanent
alarm.

**But "the operator can see his seats" was the user-facing goal, and it is 6/7, not 7/7.** I reported the
binding defect as "the one that fixes your screen" on 11-02. **That was true and incomplete** — the screen
needed two fixes, and I found the first. **He found the second, and he found it by looking at what the
operator actually reported seeing rather than at what the database said.**

## What I did not do

I did not rename any tmux session. I did not edit `pty/mod.rs`, `project-scope.ts`, or any view. I did not run
`bind-seatloom-seats.sh` again. My measurements were the live bindings query, `tmux list-sessions`, the
suffix filter applied by hand to each session name, `parseSeatSession`'s stem logic read from source, and the
name intersection computed from both sets. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is
exit 0.
