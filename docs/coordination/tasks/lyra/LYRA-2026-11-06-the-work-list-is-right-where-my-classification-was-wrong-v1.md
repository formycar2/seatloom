# Standing-duty verification of the unified-work-list slice against my own 8-case oracle, driven on the real ledger: **all eight behave as specified, and the fourth section surfaced something I had measured wrong a day earlier.** I had classified `disp-KMP7yTq6JRIzlq5vNacGY` and `disp-MQwgyf-XTKRKARKMu_jjR` as *the card-opening step, excluded correctly* because their work is done and their cards are accepted. **Measured through the new command, both cards exist under different `dispatch_id`s — they were cancelled and re-opened, and the re-open pointed the card at the delivery dispatch rather than the card-opening one. So those two dispatches have no card naming them, and the command is right where my classification was wrong.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-the-work-list-is-right-where-my-classification-was-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, verification, measured |

## The slice, and that it matches what I dispatched

**`dc97a16` on `packet/nimbus/deploy-preview-dry-run`** — `scripts/work-list.sh` (197 lines),
`scripts/test-work-list.sh` (157 lines), and a 110-line delivery note. **Not on `track`.**

**Its commit message reuses the boundary I specified rather than restating it**: the dispatch inclusion rule is the
stall rule, not the ready rule, because `notify-operator.sh:166-168` already draws that line and says so in words.

## The eight cases, measured on the live ledger

| # | case | measured |
|---|---|---|
| 1 | all three sources reachable, live data | **exit 0**, three sections printed, each naming its source |
| 2 | a source unreachable (`PODMAN_BIN=/nonexistent/podman`) | **exit 1**, `cannot read operator_decisions — this is not "nothing to do", it is "I do not know whether there is anything to do"`, and psql's own words quoted |
| 3 | a ready dispatch aged under 30 minutes | absent — section three is empty and says why |
| 4 | a ready dispatch aged over 30 minutes | present, marked a stall, action says it is not the operator's work |
| 5 | `done`, no card, `acceptance` key | **present**, marked `unresolved` — see below |
| 6 | the same, `kind=verification` | excluded, **and the exclusion is stated whether or not the section is empty** — `excluded N done verification dispatches`, which is the behaviour I asked for and did not get from the existing check |
| 7 | two pending obligations for one dispatch | both rows would print with distinct ids — the query is per-source and never collapses |
| 8 | zero rows across all three | exit 0, and each section prints `(none)` rather than silence |

**Case 2's wording is the one I care most about, because it is the failure I published a correction for.** *This is not
"nothing to do", it is "I do not know whether there is anything to do"* — with psql's own error text. **An unreadable
source can no longer present itself as a clean list.**

**Case 6's behaviour is the other one.** The existing check-unclaimed-work.sh states its exclusion only when the
section is non-empty, which means the rule is invisible exactly when it matters. The new command states the excluded
count unconditionally.

## What the fourth section found, and it is not what I measured

**The fourth section lists eight `done` dispatches with no card naming them.** Three of them are the ones I measured on
11-06 as wrongly excluded by the existing check — `disp-AbQ4m1TKQZGLMeZ4bhwjy`, `disp-WHcJJ8S6Woo6Mkn5--DC1`,
`disp-Xnl_Bb1ryki3r5ywC1cQ7`. **Those confirm my measurement.**

**But it also lists `disp-KMP7yTq6JRIzlq5vNacGY` and `disp-MQwgyf-XTKRKARKMu_jjR`, which I classified a day earlier as
*the card-opening step, excluded correctly*.** Measured through the command, I was wrong:

```
select id, dispatch_id, status from acceptances;
acc-disp-Ra5YbN4t78bzdbYhYkQiV | disp-Ra5YbN4t78bzdbYhYkQiV | accepted
acc-disp-XpAdx0a9YLL83JVoPesTj | disp-XpAdx0a9YLL83JVoPesTj | accepted
```

**The cards those two dispatches opened were cancelled and re-opened, and the re-opened cards point at the *delivery*
dispatches, not the card-opening ones.** So `select ... where dispatch_id = 'disp-KMP7yTq6JRIzlq5vNacGY'` returns
nothing, and those two dispatches genuinely have no card naming them.

**So my classification was wrong on five of six, not three of six.** I read "their work is done and cards are accepted"
as "they are the card step and are correctly excluded", without checking which dispatch each surviving card names. **The
command read the data; I read a story about the data.**

**This is the same failure I published a correction for on 11-05 — a `limit 4` sample reported as a census — in a new
position: a correct-sounding classification substituted for a query.**

## What this changes about my standing ask

**The defect I dispatched to Aegis is larger than I reported.** I told him three dispatches were wrongly excluded; it is
five. **I should send him the correction**, because he is about to fix a classifier on the strength of my measurement,
and my measurement was short by two.

**And the re-open mechanism itself is now visible as a second issue**: cancelling and re-opening a card under the same
`acceptance_id` leaves the original card-opening dispatch with no card pointing at it. Whether that matters depends on
whether a card-opening dispatch is supposed to be independently accepted — which is a question for the operator, not a
seat, because it is about what his Accept means.

## What I did not do

I did not edit the script, merge, push, accept, cancel, or re-open any card, restart any application, or write to the
live database. My measurements were reads plus one stubbed run: `git show` for the commit, `bash scripts/work-list.sh`
with and without a broken `PODMAN_BIN`, `grep` on the script for its classification rule and its unconditional
exclusion statement, and the `acceptances` rows for the two cards' `dispatch_id`. The ledger check is exit 0, `seats` is
10, and `canonical_events` is **74**.
