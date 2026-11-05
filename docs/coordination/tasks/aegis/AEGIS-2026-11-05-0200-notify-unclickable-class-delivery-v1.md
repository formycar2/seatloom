# Delivery for disp-ZZEb6fpwXPR0zqMQjiHwV — the push channel now says a card is blocked and how to unblock it, and the ruling it was told to wait on has been escalated

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-0200-notify-unclickable-class |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | dispatch, notification, mutation-tested, escalation |

**Dispatch**: `disp-ZZEb6fpwXPR0zqMQjiHwV` (issued by `seat:seat-lyra-001`, claimed via `get_next_dispatch`).
**Changed**: `scripts/notify-operator.sh` (+33 lines), new `scripts/test-notify-unclickable.sh`.
**Ledger writes**: the claim only, plus one `request_operator_decision` (section 3).

---

## 1. Her three measured facts, each re-verified by me rather than accepted

| Her measurement | My re-measurement | Holds |
|---|---|---|
| `grep -c 'clickable\|behind' notify-operator.sh` -> 0 | same command -> **0** | yes |
| class 1 fires while a card is PENDING, which is exactly when cards are blocked | the class-1 data source is `select id from acceptances where status='pending'` | yes |
| class 4's condition is `binds < 7`, measured value now 7, permanently silent | measured **7** | yes |

So class 1 currently tells the operator "a card is waiting for you" at the exact moment the card is blocked. He clicks, is refused, and nothing tells him why.

## 2. The new class 5, and why it will not become noise

The condition reuses `check-cards-clickable.sh`'s exit code — that script already names the branch, the card and the behind count. I did not restate the criterion.

Lyra called her third requirement "the one most likely to be skipped": it must not fire when nothing is blocked. She asked for a mutation proof, the same discipline we applied to Nimbus's guard. Done:

| direction | result |
|---|---|
| clickable (exit 0) | **completely silent** — measured in the real current state |
| blocked (exit 1) | fires, message carries `git merge track/infra-foundation` |
| cannot check (exit 2, my convention) | stays silent **and says it did not check**, never pretends clean |
| **mutation**: disable the silent-when-clickable guard | **turns red** — the guard is load-bearing, not decoration |

These live in `scripts/test-notify-unclickable.sh`, exit 0 reproducible. Without that file, "it will not fire spuriously" is only my word.

## 3. Her third requirement is not satisfied, and the reason is worse than she wrote

She required: do not land this while her ruling on dropping the level-with-base criterion is unapproved, because if the operator approves it, class 2 changes behaviour and this new class would be measuring a gate he just changed.

I checked whether that ruling had ever entered the decision flow. **`operator_decisions` is empty — 0 rows.**

I first proved the query can return a row (a constant probe returned 1), then read the empty result. So it is not "table missing" and not "query failed": **that obligation was never formally put to him.**

Her ruling has sat in `LYRA-2026-11-04-ruling-the-clickable-criterion-requires-chasing-a-moving-mainline-v1.md` since 11-04 23:5x with status `open`. It is exactly the failure she named in the dispatch that asked for this work: *a finding that lives only in a file acts on nobody.*

**What I did**: `request_operator_decision(dec-clickable-criterion-level-with-base)` — two options, consequence stated, evidence chain pointing at the ruling and her cost measurement. It is now `status=pending`, `decider=human:supervisor`, and **`fleet-status.sh` shows it** (measured: "1 item for you to rule on"). That is the only ledger write I made, and it is within my own permission (creators are limited to the operator and `seat-aegis-001`).

On the relationship between class 5 and that ruling: class 5 reads the checker's exit code rather than restating the criterion, so **if the ruling is approved and the checker changes, class 5 follows the new gate automatically.** It is a reading of the gate, not a second copy of it. I verified that, and still handed the timing decision to him rather than choosing for him.

## 4. Errors I made here, recorded

1. **First version used an undefined variable `$REPO_DIR`** (this script only has `$REPO`). `bash -n` cannot catch that; I found it by grep.
2. **The test file was broken twice, both caught by me**:
   - `stub(){ # comment` — bash 3.2 rejects it. The symptom was "line 95 unexpected EOF", which looks nothing like the real cause, and I chased it for a while.
   - four occurrences of `"$WORK/subject.sh --push` **missing the closing quote**. That was the actual syntax error; `cmp` byte-by-byte located it.
3. **My first mutation hit the wrong branch**: I widened the `elif`, but the first `if` still caught rc=0, and the output said "exit code 0, neither 0 nor 1" — the mutation had not produced the intended effect and I nearly recorded it as "mutation survived". Redone properly before concluding anything.
4. **Direction two's second assertion was a false negative**: I grepped the output for the unlock command, but `pop()` prints only the title, never the body. The notification really does carry the command; it just is not in the output. Reading the source fixed it.

Items 3 and 4 are the same error class: **an experiment that did not do what I intended, treated as if it had produced information.** That is precisely what "an empty result must be verified" and "a mutation must be proven clean" exist to prevent.

## 5. What I did not do

- Did not change `check-cards-clickable.sh`'s criterion — that is Lyra's ruling plus the operator's approval, not mine.
- Did not drop the level-with-base requirement (she explicitly said she was not asking for that).
- Did not merge, push, restart any application, or write to the live database beyond that one decision.
- Did not install the new class into a cron-run copy — cron runs only the self-wake script, which invokes this repo copy of `notify-operator.sh`. Verified via `crontab -l`.

## 6. Acceptance

- `bash -n scripts/notify-operator.sh` passes.
- `bash scripts/test-notify-unclickable.sh` -> exit 0, five assertions green including one mutation.
- One real `--push --force` run: only the two "a card is waiting" notifications fired; **class 5 stayed silent**, consistent with the cards currently being clickable.
- `verify-docs.sh` passes for this change.
