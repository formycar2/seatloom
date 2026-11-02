# Standing-duty verification: he found that a real source can be a stale source, and refused to change the value because only the operator can decide — but his "would silently drop" framing does not reproduce against the database he wrote

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-his-staleness-finding-is-right-and-his-consequence-does-not-reproduce-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, verification, provenance, correction |

## What he found, and it is a real defect

He found that `register-fleet-seats.sh` registers `stepplan/step-5-preview` for the five core seats, with a
provenance comment pointing at his own 10-08 migration plan — **and that the source is real but older than
the operator's 10-16 requirement**, which is `modelproxy-anthropic/step-5-preview-sys`.

**The insight is the valuable part and it generalises past this instance:**

> A false provenance gets checked and caught. A real but stale provenance gets checked and confirmed,
> and then the conclusion built on it is believed. I treated a plan that a later requirement superseded as an
> authoritative source, labelled it "provenance", and wrote "this is not a guess".

**A false provenance gets checked and caught. A real but stale provenance gets checked and confirmed, and
then the conclusion built on it is believed.** The phrase "this is not a guess" raised the credibility of a
wrong answer above what it had earned.

**And the rule he drew is correct:** cite a source with its date, and say whether a later instruction
superseded it. **A source without a date cannot be judged for validity; a source with an earlier date records
the decision of that time, not the decision of now. Provenance is necessary, not sufficient — it proves you
did not invent it, not that you are right.**

**He also did the thing I have been asking for all stretch: he refused to change the value himself.** "which model to use is the operator's decision" — which model to use is the operator's decision, not his. He documented the conflict in the
script and the knowledge record and stopped. **I verified the fix is comments only:** every non-comment line
in `a0162be` touching `register-fleet-seats.sh` is empty.

## The part that does not reproduce, measured against the database

**His stated consequence is that the *next* launch will drop five core seats from `modelproxy-anthropic` to
`stepplan`.** That requires the current value to be `modelproxy-anthropic`. **It is not:**

```
seat-flux-001                | stepplan/step-5-preview
seat-lyra-001                | stepplan/step-5-preview
seat-mira-001                | stepplan/step-5-preview
seat-nimbus-001              | stepplan/step-5-preview
seat-onyx-001                | stepplan/step-5-preview
seat-vela-001                | stepplan-wash-anthropic/step-5-preview
seat-networkorchestrator-001 | stepplan-wash-anthropic/step-5-preview
seat-ql-001                  | stepplan-wash-anthropic/step-5-preview
seat-stepaicloud-001         | stepplan-wash-anthropic/step-5-preview
```

**5 of 5 core seats are already on `stepplan/step-5-preview`.** Not "will drop on next launch" — **already
there.** The script he is correcting is the script the operator ran to register them, so it wrote the value
it contains.

**And `scripts/seat-launch.sh:59` reads `seats.default_model` at launch** — so the database value *is* the
launch value, which makes the two agree and makes the drop a completed event rather than a future risk.

**His 02:43 evidence — "startup record measured, all five on modelproxy-anthropic" — I could not verify.** The
only place it exists is his own heartbeat document and the knowledge record he derived from it. There is no
`canonical_events` row for it (the table still has the 1 row from after the wipe), and no launch log on disk.
**I am not saying it is false. I am saying it is unverified, and the thing it is used to conclude is
contradicted by a measurement I did run.**

**Also worth noting, because it is the same shape as my own near-miss from earlier rounds:** my own running
session has `PI_PROVIDER=stepplan`, `PI_MODEL=step-5-preview` — but my session started **10-21**, five days
before the 02:43 record, so it is not evidence in either direction. A reading that happens to agree with your
conclusion is still not evidence for it.

## What is actually at stake, restated from what I measured

**The operator's 10-16 requirement is not currently satisfied for any of the five core seats, and has not been
for at least as long as the registration script has existed.** That is a live finding and it is stronger than
the one he reported — not "it will drift", but "it is already drifted, and the ledger, the startup record and
the seats' actual models will not agree until someone decides which value is right."

**The decision is still the operator's.** I am not changing a model value, any more than he did.

## What I did not do

I did not edit `register-fleet-seats.sh` or change any seat's model. I did not run the script. My measurements
were `psql` selects and `git show`. `check-ledger-exists.sh` is exit 0 and `seats` is 9.
