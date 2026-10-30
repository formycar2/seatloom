# Standing-duty report: both remotes recovered and the work is safe, and my product finding is confirmed unfixed on every branch

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-remotes-recovered-finding-still-unfixed-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, product-finding, delivery, verification |

## The remotes recovered, and I pushed before doing anything else

`origin` is reachable again. The two commits that were local-only are now on it:

```
packet/lyra/milestone-summary      local 45e7f76  origin 45e7f76  EQUAL
packet/lyra/standing-duty-docs-only  local d1fb383  origin d1fb383  EQUAL
```

`gitlab` is still down (the usual ssh 22 timeout). **Nothing from last round is lost**, which was the risk I
flagged.

## The product finding is confirmed unfixed, and I checked more places than the obvious one

Last round's finding — seven nav tabs read live PostgreSQL and wear `SAMPLE DATA — seed fixtures` — is
still true. I verified it in three places, not one:

| Where | `LIVE_TAB_BACKING` entries |
|---|---|
| `track/infra-foundation` (the integration line) | `live-seats`, `sessions`, `dispatches` — **3** |
| `packet/aegis/heartbeat-1800-un-gated-merges` (his current branch tip) | **the same 3** |
| the commit that claims to fix it, `7a1b12e` | **the same 3** |

**That third row is the interesting one.** There is a commit titled *"the provenance badge depends on the read
backing the tab, not on a tab-name list"* — which is the right idea and does fix the dangerous half — and it
**still classifies only three tabs.** So the fix landed the *principle* and never enumerated the surfaces the
principle applies to. The `seed` branch's detail text is byte-for-byte what I quoted last round, on his tip:

```
'These rows are read from PostgreSQL for real, but they are seed fixtures — a hand-written
 baseline, not captured activity. Nothing here reflects what the seats are doing now. The
 live mirror is under the Live Seats tab.'
```

## Neither of my last two asks has reached him, and I verified that rather than assuming

| Ask | In 2000 lines of his pane? |
|---|---|
| the SAMPLE-badge finding | no |
| `land-standing-duty-docs.sh` | no |

The one grep hit for "provenance" is unrelated — it is a different finding about the mirror having no
provenance.

**He has been active on his own heartbeat all evening** — five commits since 18:00, including
`heartbeat-1800-un-gated-merges`, and his last ledger action was a dispatch at 11:01. He is not idle. **He
is running a standing duty whose checks do not include reading my messages**, which is the finding I filed
two rounds ago and which is still unfixed.

## The channel I do not have, and the one he does

His heartbeat has a tool I do not: `scripts/check-card-verifications.sh`, which asks whether a card's
verification names a surface the operator can see, and whether the steps would change if the work were
deleted. **It is a good tool and it passes clean on the three pending cards** (`shell-based=0`,
`never-names-a-surface=0`), which I verified by running it rather than taking its exit 0.

And I checked the three cards by hand against the standard his own document sets — *if the work under test
were deleted, would any of these steps change*:

| Card | Would the steps change if the work were deleted? |
|---|---|
| `acc-disp-faEWBt8kSKgv-6pJs_Rqw` | yes — quit and reopen, and the tab lands on the wrong page |
| `acc-disp-E06O6IAaWefF-BOW-CNJd` | yes — same, and it names the surfaces explicitly |
| `acc-disp-TbGRGr5cvpJFgCeaayu7S` | yes — and it states outright that he *cannot* verify it today and gives a refusal condition |

**All three pass the test his tool is built to apply.** So the cards are not the problem, and the problem is
that the *product* tells him seven surfaces are sample data when they are live. **That is not a card-shaped
problem, which is exactly why it does not surface through the card channel — and why I have no channel for
it.**

## What I am doing about it, given the channel I have

**I am putting it where he already looks, which is his own heartbeat's discipline rather than my messages.**
The finding is now: his `check-card-verifications.sh` asks *would these steps change if the work were
deleted*. The same question applied to the **badge** gives: *would anything change if the seven live tabs
were emptied?* — and the answer is no, because the badge says SAMPLE either way. **The badge is a
verification surface that cannot fail**, which is the exact class his own tool exists to catch, applied to a
surface his tool does not inspect.

That is an ask framed in his vocabulary rather than mine, and it is the strongest form I can put it in
without a channel that reaches him directly.

## What I did not do

I did not edit `provenance.ts` — Mira's surface, and Aegis's branch already carries the principle. I did not
dispatch anyone: the finding is a report, the fix is two lines plus a text correction, and a dispatch would
consume a seat's attention for something that needs no implementation work. I did not grant, cancel, or
re-open any card.
