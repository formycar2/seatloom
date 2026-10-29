# Standing-duty finding: the operator accepted two cards whose verification steps he could not have passed, and nothing told him

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-two-cards-accepted-on-false-premises-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, blocking, acceptance, operator-trust, card-defect |

## What happened

At 05:12:35–05:12:44 the operator accepted **all seven** pending acceptance cards, in nine seconds.

Two of them describe behaviour he did not observe, and I had measured both before he clicked:

| Card | What its step says to expect | What the app actually shows, measured just now |
|---|---|---|
| `acc-disp-first-screen-default-tab` | quit completely, reopen, the highlighted page is **Dispatch** | `activeProjectId = project_delivery/QL`, that project's saved tab = **`live-seats`** → the app opens on **Mirror** |
| `acc-disp-bc20UzOznJQ8hCgbHw876` | count the seats under project `seatloom`: **10** | **7** seats are bound to `seatloom`; the other 3 belong to their own projects |

Both cards were **unchanged** when he accepted them. I verified the text is still the false version:

```
acc-disp-first-screen-default-tab  → 0 occurrences of the corrected wording
acc-disp-bc20UzOznJQ8hCgbHw876 step 2 -> "count the seats: should be 10"
```

And both were accepted with **`feedback` empty** — no note that the premise was wrong, no note that he
saw something different.

## Why this is the finding, and not a footnote

I measured both defects earlier today and reported them:

- `LYRA-2026-10-29-first-screen-card-cannot-pass-v1.md` — read the app's own persisted store, found
  `project_delivery/QL` on `live-seats`, concluded the card cannot pass.
- `LYRA-2026-10-29-seat-count-card-cannot-pass-v1.md` — measured 7 bound versus 10 in the table, found
  the card contradicts itself across its own steps 2, 4 and 5.

Aegis independently proved the first-screen one on the operator's real machine at 10:36 and **told him**:
*"Following that card would have Mr. Zhang verify against a false premise, see the wrong screen, and
conclude the product is broken — when the truth is the card was written against an untested assertion."*

He was right, and that is what happened. The operator opened the app, and the first screen was not what
the card said. **He had been warned, and the card was still there unmodified, and he still clicked
accept.** Nine seconds for seven cards is not reading seven verification plans.

## The three things that failed, separately

**1. The correction never reached the artifact.** Aegis dispatched Mira to fix the behaviour
(`disp-E06O6IAaWefF-BOW-CNJd`) — the right call, since the behaviour is a product decision he correctly
refused to make. But **the card was left in place while its premise was known false.** A false card that
is knowingly left standing is worse than one nobody has checked: it converts a measurement into a
false acceptance.

**2. Nothing gates acceptance on the card being runnable.** `check-cards-runnable.sh` checks that the
refs and script paths a card names resolve. `check-card-verifications.sh` checks that no step asks for a
shell. **Neither checks that the card's expected value is true.** Both of today's cards pass both gates —
they name real surfaces and real commands. The expected value is the part nobody validates, and it is
the part the operator actually relies on.

**3. Seven cards in nine seconds is not verification.** I am not criticising the operator for trusting
the system; that is what the system is for. But the acceptance gate's entire purpose is that a human
checked, and a human cannot check seven plans in nine seconds. Either the plans have to be small enough
to check, or something else has to check them.

## What I am not claiming

I am **not** claiming the work behind those two cards is wrong. The first-screen card's underlying work
is Mira's restore feature, which is real and which the operator asked for. The seat-count card's work is
Onyx's 10-seat seed correction, which is real and delivered. **The work is fine; the verification was
not performed.** Those are different facts and conflating them would be the same mistake as reporting a
card as fixed because its code landed.

I am also **not** claiming the operator was misled deliberately. Aegis told him the truth in a
SupervisorMessage. The failure is that the truth lived in a message while the false claim lived in the
card, and the card is what he was looking at.

## What should change, in order of leverage

1. **A card with a known-false premise must be cancelled, not annotated.** `acceptances` rows are
   immutable once decided, but a *pending* card with a measured-false expected value should not stay
   pending. That is a rule, and it should be written into Aegis's protocol next to step 2d.
2. **The expected value needs a checker.** The shape is measurable: a step that names a count, a tab, a
   project id or a seat list can be checked against live. Today's two would both have been caught — one
   by reading the persisted store, one by counting `project_role_bindings`. This is the third checker
   gap this round, after `check-verdict-refs.sh`'s stale rule and the card-filename conflict.
3. **Seven-in-nine-seconds should be visible as a signal.** Not a gate — the operator's time is his own —
   but the ledger records `decided_at` to the millisecond, so the pattern is measurable and should be
   surfaced rather than discovered after the fact.

## What I did not do

I did not cancel or re-open either card — they are decided, decided rows are immutable, and re-opening
them would be the churn my own ruling R1 forbids. I did not edit them. I did not dispatch a seat.
