# Implementation packet for `disp-8Qmhx6zvPDsYUEO2Jxcy8` — the first unified-work-list slice, bounded to one command over the three existing tables, plus an independent-test oracle. **Before defining any code I measured the five traps the dispatch named, and one of them is a live defect in the check I would have reused**: `check-unclaimed-work.sh` excludes every dispatch whose payload has an `acceptance` key, on the theory that such a payload *is* the card-opening step. **Measured, that key is how `how_to_verify` travels, not a classification — three real product tasks are invisible to that check right now.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-unified-work-list-slice-one-command-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | research, implementation-packet, oracle, measured |

## The five traps, measured before any design

### Trap 1 — payload shape is not task classification, and the existing check gets it wrong

**Measured on the live ledger.** `check-unclaimed-work.sh:69` excludes any dispatch whose payload has an `acceptance`
key, on the reasoning that *the task WAS the card step; a card of its own would be a category error.*

**That reasoning is sound and the key is wrong.** `acceptance` is where `how_to_verify` travels — the operator-facing
acceptance step — not a marker that the dispatch is itself a card request. Measured:

| dispatch | `kind` | has `acceptance` | cards | what it actually is |
|---|---|---|---|---|
| `disp-MQwgyf-XTKRKARKMu_jjR` | (none) | yes | 1, accepted | **the card-opening step — excluded correctly** |
| `disp-KMP7yTq6JRIzlq5vNacGY` | (none) | yes | 1, accepted | **the card-opening step — excluded correctly** |
| `disp-WHcJJ8S6Woo6Mkn5--DC1` | (none) | yes | **0** | product task: the verdict gap — **wrongly excluded** |
| `disp-Xnl_Bb1ryki3r5ywC1cQ7` | (none) | yes | **0** | product task: cold-start fix — **wrongly excluded** |
| `disp-AbQ4m1TKQZGLMeZ4bhwjy` | (none) | yes | **0** | product task: the deletion guard — **wrongly excluded** |

**So the check excludes 2 correctly and 3 wrongly, and it cannot tell them apart** — because it is reading a key that
means something else. All three wrongly-excluded tasks are in fact done: `bd0cabb` on `track` for cold-start, `6e04dae`
on a branch with two Flux `pass` verdicts for the guard, and a Flux verdict for the verdict gap. **So the check's
silence is not hiding live work today. It is hiding the classification rule, which will hide live work the first time
a product dispatch carries an acceptance step and nobody opens its card.**

**This is the trap the dispatch named, and it is live.** It is a defect in `check-unclaimed-work.sh`, which is Aegis's
instrument, so it goes to him as a finding rather than an edit.

### Trap 2 — a ready seat dispatch is seat work, not an operator obligation

**Measured, and the existing code already gets this right.** `scripts/notify-operator.sh:166-168` fires only when a
ready dispatch has sat **more than 30 minutes**, and its wording says *this is a system problem, not your work — I am
telling you so you know.*

**So the rule is: a dispatch becomes an operator obligation only when it has stalled**, and the stall threshold is the
existing 30 minutes. **The unified list must not render a ready dispatch as something the operator owes.** Measured now:
1 ready dispatch (`disp-yINSIW0d6bEOw8LiYBFpy`, issued 03:31, Nimbus's branch sync) — **not an operator obligation.**

### Trap 3 — source failure must be uncheckable or non-zero, never empty

**Measured, all three source tables have rows**: `operator_decisions` 1, `acceptances` 2, `dispatches` 17. So the
command cannot be tested against an empty source and must not report success when a source is unreachable.

**The rule for the implementation**: if any of the three queries fails, the command exits non-zero and names which
source it could not read. **A zero-row result is a valid answer only when all three queries succeeded.** An empty list
is never evidence of health — that is the failure I published a correction for on 11-05, when I reported a ledger
count from a `limit 4` sample.

### Trap 4 — honest runtime/source separation

**Every row the command prints must carry its provenance**: `source` for what the ledger asserts, and `runtime` for
what has actually reached the operator's application. **These differ, measured**: the deletion guard is `source`-true
(on a branch, with verdicts) and `runtime`-false (not on `track`, not in the binary he is running, built 10-28 22:16).
**A list that does not distinguish them would show the guard as done.**

### Trap 5 — no existing branch work to build on

**Measured**: no branch matches `todo|obligation|unified|worklist`; and `grep -rl obligation crates/ src-tauri/ ui/src/`
returns five files, all of which are about `operator_decisions`'s own concept of an obligation — **there is no
unified-obligation read model anywhere.** So this slice is genuinely new code, not a completion of something in flight.

---

## The slice, bounded

### What it is

**One command: `scripts/work-list.sh`**, printing the operator's actionable obligations from the three existing tables,
in one list, each row naming its source table, its owner, its action, and its runtime/source provenance.

### What it is not

- **Not the read model.** No new table, no snapshot service, no pagination API — the 11-02 contract's §2 and §3 specify
  those and they are the next slice.
- **Not a UI.** The dispatch says a CLI may be a diagnostic slice, not a substitute for eventual existing-UI
  integration. This is the diagnostic slice.
- **Not a re-classifier.** It does not invent a task taxonomy; it reports what the ledger says and flags what it cannot
  attribute.

### The three sources, and what each contributes

| source | contributes | owner field | the operator's action |
|---|---|---|---|
| `operator_decisions` | rows with `status='pending'` | `decider` | rule on it |
| `acceptances` | rows with `status='pending'` | the card's seat | click Accept or Reject |
| `dispatches` | rows `status='ready'` **and older than 30 minutes** | the target seat | nothing — this is a stall report, marked as such |

**The dispatch inclusion rule is deliberately the stall rule, not the ready rule**, because of trap 2.

### Actionable fields, per row

`source_table`, `id`, `subject` (title or decision target), `owner` (seat id or decider), `age`, `action` (one
sentence), `provenance` (`source` or `source+runtime`), `unresolved` (set when ownership or classification cannot be
determined from data).

### Inclusion and exclusion, with examples

| row | included? | why |
|---|---|---|
| a pending operator decision | **yes** | only the decider can act |
| a pending acceptance card | **yes** | only the operator can click |
| a ready dispatch, 3 minutes old | **no** | seat work; it has not stalled |
| a ready dispatch, 40 minutes old | **yes**, marked `stall` | the waker may be down; he should know |
| an `in_flight` dispatch | **no** | a seat is working on it |
| a `done` dispatch with no card | **yes**, marked `unresolved` | trap 1: this is exactly the row the existing check loses |
| an accepted card | **no** | closed |

### Adversarial cases the implementation must survive

| case | required behaviour |
|---|---|
| one of the three queries fails | exit non-zero, name the source, print nothing else |
| a decision whose `decider` is not `human:supervisor` | include it, and say who the decider is — do not assume the operator |
| a card whose dispatch no longer exists | include it, `unresolved` set, action names the broken reference |
| a dispatch with no `kind` and no `title` | include it as `unresolved` — **do not infer a class from the payload's shape** |
| two obligations for the same dispatch | print both, with their distinct ids — the contract forbids collapsing them |
| zero rows in all three tables | print `0 obligations` **and** state that all three sources were read successfully |

---

## The independent-test oracle

**Because the slice's whole value is that it does not lie, and a list that cannot be wrong cannot be trusted either.**

| # | case | expected |
|---|---|---|
| 1 | all three sources reachable, live data | exit 0, rows match a hand-run query for each source |
| 2 | `operator_decisions` query forced to fail | exit non-zero, output names `operator_decisions` |
| 3 | a ready dispatch aged under 30 minutes | absent from the list |
| 4 | a ready dispatch aged over 30 minutes | present, marked `stall`, action says it is not the operator's work |
| 5 | a `done` dispatch with no card and an `acceptance` key | present, marked `unresolved` — **the row trap 1 loses** |
| 6 | the same, with a `verification` kind | absent, and the output says why |
| 7 | two pending obligations for one dispatch | both rows present, distinct ids |
| 8 | zero rows across all three | exit 0, `0 obligations`, and a line stating all three sources were read |

**Cases 5 and 6 are the pair that proves the classifier reads values, not shapes** — and case 5 is the case that must
fail if someone reintroduces the `acceptance`-key exclusion.

---

## Owners, and what I am dispatching

| work | owner | why |
|---|---|---|
| implement `scripts/work-list.sh` to this packet | **Nimbus** | it is a read path over the ledger, and he owns the deletion-guard sync already in flight |
| independent verification against the 8-case oracle | **Flux** | he is the verification seat and he is idle after two verdicts |
| fix the `acceptance`-key misclassification in `check-unclaimed-work.sh` | **Aegis** | it is his instrument; I am sending him the measurement, not an edit |

**What I am not doing**: not merging, not deploying, not opening a card, not writing to the live database, and not
routing routine instrument edits to Aegis — the `check-unclaimed-work.sh` item is a defect report with measurements,
not a routine edit, and it is the third thing on his queue this week rather than the first.

**What requires human approval**: binding this into the existing UI, and binding it into the push channel. Both are
separate decisions and neither is in this slice.

## What I did not do

I did not write the command, edit any script or product code, merge, push, accept, cancel, or re-open any card, restart
any application, or write to the live database. My measurements were reads: the six `done` dispatches' payload keys and
card counts, `notify-operator.sh`'s ready-dispatch wording and threshold, the three source tables' row counts, `git
branch -a` for existing work, `grep -rl obligation` across the three source trees, and the contract's §3 snapshot rule.
The ledger check is exit 0, `seats` is 10, and `canonical_events` is **54**.
