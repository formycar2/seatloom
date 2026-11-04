# Standing-duty measurement of why Aegis cannot run, and it is not the outage I reported last round: I said the model chain was down and that it might be transient. **Six hours later it has not recovered, and the cause is not an outage at all — it is that Aegis's `primary` provider is `sp-team`, which is unavailable, while `sp-personal` — which is what the database says every seat uses, and what Flux is running successfully right now — is the *fallback* in his chain.** Measured from his own scrollback: `sp-personal/step-5-preview → sp-team/step-5-preview (primary restored)`. **So a working provider is configured for him and the system will not use it, because it is his fallback rather than his primary.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-aegis-s-primary-provider-is-the-unavailable-one-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, provider, dispatch-channel |

## What I got wrong last round

I reported a model outage and said it might be transient. **Two things wrong with that, both mine:**

1. **"Outage" was the wrong noun.** The scrollback does not say the provider is down for everyone; it says *his* primary
   is unavailable. Those are different claims and I published the bigger one.
2. **"Transient" was a guess dressed as caution.** I had one data point — Flux failed and recovered on the same line —
   and I generalised from a different seat with a different provider to the one that was failing. **I should have
   measured which provider was failing before naming the cause.**

**What I measured this round instead, and it is not an outage.**

## The measurement, from his own scrollback

```
801  Model fallback: sp-team/step-5-preview → modelproxy-anthropic/step-5-preview-sys (provider unavailable)
811  Model fallback: modelproxy-anthropic/step-5-preview-sys → sp-personal/step-5-preview (provider unavailable)
821  Error: All fallback models failed. Chain: sp-team → modelproxy-anthropic → sp-personal
825  Model fallback: sp-personal/step-5-preview → sp-team/step-5-preview (primary restored)
839  Error: All fallback models failed. Chain: sp-team → modelproxy-anthropic → sp-personal
```

**Line 825 is the whole finding.** After the fallback exhausts, it comes back to `sp-personal` and then reports
`primary restored` — meaning **`sp-personal` is reachable and is not his primary.** His chain ends by returning to
`sp-team`, which is the one that is unavailable.

**And line 839 proves the retry did the same thing again.** Twenty-one failure lines now, against seventeen last
round. The cron wakes him at the top of every hour, and each wake re-runs the same failing chain.

## The corroborating measurements

| source | what it says |
|---|---|
| the database, `seats.default_model` for aegis, flux and lyra | **all three `sp-personal/step-5-preview`** |
| Flux's pane, measured now | **`(sp-personal) step-5-preview` — working** |
| Aegis's pane, measured now | **`(sp-team) step-5-preview` — failing** |
| `~/.pi/agent/models.json` provider keys | `stepplan`, `stepplan-wash`, `modelproxy`, ... — **neither `sp-personal` nor `sp-team` is a pi provider name** |

**So the database and the running processes disagree about what Aegis's provider is, and `sp-team` is not a name pi
knows from its own config.** That is the same unexplained anomaly I recorded when the operator renamed the providers
(`ae1bb07`): DB says `sp-personal`, session says `sp-team`. **This round shows the anomaly is not cosmetic — it is
why a seat with a working provider configured cannot run.**

## What this costs, concretely

**My dispatch `disp-ZZEb6fpwXPR0zqMQjiHwV` has been in flight for six hours and will never complete on its own**,
because every hourly wake re-runs a chain whose primary is unavailable. It asks for a notification class that would
stop the operator losing five hours a day to unclickable cards.

**And the dispatch lifecycle shows `in_flight`, delivered, healthy.** The ledger's view of this is that Aegis is
working. He is not; he is being woken hourly into the same failure.

## What I am not doing, and it is the same line as last round

**I am not changing Aegis's provider.** It is his session, on the operator's machine, and the rename that created
this divergence was the operator's action — so reversing it is the operator's action too. Changing another seat's
runtime identity to unblock my own dispatch is exactly the unsupervised infrastructure action I have been barred
from since I destroyed the database.

**I am also not re-dispatching to a different seat.** The notification class belongs in `notify-operator.sh`, which
is Aegis's instrument. Routing around him because his session is misconfigured would produce the change while
leaving the misconfiguration in place to bite the next dispatch.

**What I am doing is making the operator's fix one line**: Aegis's primary must be `sp-personal/step-5-preview`, which
is what the database already says for all three of us, and which Flux is running successfully right now.

## What I did not do

I did not change any configuration, restart any application, retry, reassign, or cancel the dispatch, edit any script
or product code, merge, push, accept, cancel, or re-open any card, or write to the live database. My measurements
were reads: `tmux capture-pane -S -800` on Aegis's pane, `seats.default_model` from the database, the same pane probe
on Flux, `~/.pi/agent/models.json` provider keys, `crontab -l` for the hourly wake, and `/tmp/seatloom_aegis_waker.log`
for the wake times. The ledger check is exit 0, `seats` is 10, `canonical_events` is 28.
