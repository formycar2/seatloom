# Standing-duty correction of last round's own supporting claim: I wrote that `sp-team` is not a pi provider name and offered that as corroboration for the root cause. **That was false, and it was false because I read the wrong directory — I inspected `~/.pi/agent/models.json` while every seat runs `saih`, which reads `~/.saih/agent/models.json`.** `sp-team` and `sp-personal` are both real `.saih` providers. The root cause survives; the corroboration I gave for it does not.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-i-read-the-wrong-config-directory-and-called-it-corroboration-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, correction, measured, provider |

## The claim I published, and why it was wrong

Last round I wrote: *neither `sp-personal` nor `sp-team` is a pi provider name in `models.json`*, and put it in a table
as corroboration that the database and the running processes disagree.

**Measured this round, that is false:**

```
~/.pi/agent/models.json      providers: modelproxy, modelproxy-anthropic, stepplan,
                                        stepplan-anthropic, stepplan-wash, stepplan-wash-anthropic,
                                        subworker, dataworker
~/.saih/agent/models.json    providers: modelproxy, modelproxy-anthropic, sp-personal, sp-team
```

**`sp-team` and `sp-personal` are both `.saih` providers. And the seats run `saih`** — measured, the pane pid's
child process is `saih`. So the directory I inspected was not the directory the failing process reads.

**This is the third time in three days I have published an empty or wrong search as a finding, and this time I
dressed it as corroboration.** A wrong reason attached to a right conclusion is more dangerous than a wrong
conclusion, because the conclusion inherits the false reason's credibility.

## What survives, measured properly

**The root cause holds, and it is better evidenced than last round:**

| source | what it says |
|---|---|
| `.saih/agent/models.json` | `sp-personal` and `sp-team` are both real providers |
| Aegis's active session, `~/.saih/agent/sessions/--...-seatloom--/2026-10-16T03-24-00-504Z_*.jsonl` | **the last 4 records today, at 20:00 UTC, all `sp-team`** |
| the same session, before the switch | **07:00:59 UTC on 11-02 `sp-personal`, 08:00:13 UTC on 11-02 `sp-team`** |
| Flux's pane, measured now | `(sp-personal) step-5-preview` — working |
| `seats.default_model` for aegis, flux, lyra | all three `sp-personal/step-5-preview` |
| Aegis's scrollback line 825 | `sp-personal/step-5-preview → sp-team/step-5-preview (primary restored)` |

**So: Aegis has been on `sp-team` since 11-02 08:00 UTC, `sp-team` is now unavailable, and `sp-personal` — which the
database configures for him and which Flux is running successfully — is his fallback.** That is the finding, and it
does not need the false corroboration.

## The thing I could not determine, stated as undetermined

**I could not find who switched him from `sp-personal` to `sp-team` at 11-02 08:00 UTC.** The waker log starts at
11-02 11:00 — everything before it is gone — so the 07:00 and 08:00 wakes are not in it, and I cannot attribute the
switch to the waker, to an operator action, or to the session itself.

**Last round I did not have this measurement and did not claim it. This round I have the measurement and it says
"unknown", so it says unknown.** The timestamp of the switch is measured; its cause is not.

## What I am correcting, concretely

**The one-line fix I gave the operator still stands** — set Aegis's primary to `sp-personal/step-5-preview`, which
the database already configures and which Flux is running. **What changes is the confidence I should attach to it.**
Last round I presented it with a corroborating argument that was false; this round it stands on the session records
alone, which are sufficient.

**And a second consequence**: `~/.pi/agent/` versus `~/.saih/agent/` is itself a trap. Any future measurement of a
seat's provider must read `.saih`, and I nearly recorded a false conclusion about the provider namespace because I
did not check which binary was running before choosing which config to read. **The order is: identify the process,
then read the config that process reads.**

## What I did not do

I did not change any configuration, restart any application, retry, reassign, or cancel the dispatch, edit any script
or product code, merge, push, accept, cancel, or re-open any card, or write to the live database. My measurements
were reads: both `models.json` files, the pane pid and its child process, the session jsonl with `grep` for provider
and timestamp, the waker log's coverage window, `seats.default_model`, and Flux's pane. The ledger check is exit 0,
`seats` is 10, `canonical_events` is 28.
