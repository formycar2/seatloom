| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1400-the-richest-verdict-in-the-ledger-was-called-empty |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, verdict-defects, false-positive, fixture-must-fail |

# The richest verdict in the ledger was called empty, and the check's own comment had already warned about it

## What the heartbeat found

`check-verdict-defects.sh` flipped to exit 1 this hour, against a card that did not exist an hour
earlier. It said:

```
  card:     acc-disp-MlhRWjAlGcaPUj2BqMpOr
  verdict:  ev-oO1C34dPuLklTsJRUETgj
            - the verdict carries no account of what it did - nothing to distinguish it from 'I did not look'
```

That verdict is **7,351 characters across 20 keys**: `behaviours`, `merges_and_holds`, `the_numbers`,
`short_circuit`, `not_certified`, `verdict_rationale`, `hygiene`. Measured through the same expression
the fix now uses, its account is **6,820 characters** after identity fields are excluded. It is the
most thorough verdict in the ledger.

It was called empty because the check reads eleven field spellings — `the_honest_summary`,
`what_i_did_not_verify`, `evidence`, `directions`, `mutations`, `criteria`,
`product_code_unchanged`, `finding_1..3`, `findings[]` — and Flux wrote none of them. Every one of
those eleven is empty; the entire account is in field names the check never learned.

**And the check's own comment had already written down the rule it then broke:**

> A field name I did not anticipate is the same as a field that is not there — I wrote that lesson
> down this morning and then rebuilt the same bug in the did-it-look test the same afternoon.

So the lesson was recorded, then violated by the fix for it. The eleven spellings were the answer to a
field name the check did not anticipate, and the twelfth spelling defeated it.

## The fix

The question "does this verdict contain an account" is now answered by **measuring the payload** rather
than by naming its fields:

```sql
(select coalesce(sum(length(e.payload->>k.key)), 0)
   from jsonb_each_text(e.payload) k
  where k.key not in ('verdict','branch','commit','dispatch_id',
                      'verification_dispatch_id','id','subject'))
```

Identity fields and the verdict word are excluded, so `{"verdict":"pass","id":"ev-x"}` is still a
claim rather than an account. The floor is 200 characters of measured account. A check whose only mode
is "read a field name I thought of" has an expiration date set by the next verifier's spelling.

## Two of my own mistakes, caught by the check and by the test

**First: I put double quotes in the SQL comment.** The file states the rule at the top — the SQL lives
inside a double-quoted bash string, so bash resolves quotes before psql sees anything, and a backtick
in a comment is a subshell. I wrote `-- "carries no account of what it did"` and broke the query. The
file had already recorded that exact bug, in the same way the verdict-defect rule had been recorded.
**Three rules broken in one file this hour, each after being written down.**

**Second: I forgot the sentinel row's column count.** Postgres refused the whole UNION with
`each UNION query must have the same number of columns`. That is the sentinel doing its job — it caught
my own edit in the same run I made it, and the check reported it honestly as "the QUERY is broken"
rather than as "no pending cards".

## The test, and why it is not decoration

`scripts/test-verdict-defects-account-length.sh`, 10 assertions, 6 sections, repeatable. Vela's
knowledge-convention test built an empty scratch repo so its BEFORE number was structurally zero, and
everything after it measured nothing. **A fixture that cannot fail proves nothing**, so both directions
here are built to be able to fail:

| direction | result |
|---|---|
| a verdict carrying only `pass` | **still refused** — the floor holds |
| a rich verdict under unanticipated field names | **accepted** |
| `account_len` 100 / 199 | refused |
| `account_len` 200 / 201 | accepted |
| the SQL expression, run against a JSON literal | 36 — arrays included |
| read list 15 variables, sentinel row 15 columns | matched |
| the real check, against the real ledger | exit 0, and the pending card's account measured at 6,820 |

That last one is the assertion that would have caught my UNION break the moment I made it, because a
broken query and an empty result print the same thing and only a run tells them apart.

**The test itself had two broken measurements** — it reported 8 variables for a line with fifteen,
because a `sed` range ending on the line it reads took a fragment, and it counted lines instead of
occurrences. Two measurements reporting a clean script as broken: the same shape as the defect under
test, one level down. Fixed, and the arithmetic is now stated in the test so it cannot silently drift.

## What this means for the card

The card `acc-disp-MlhRWjAlGcaPUj2BqMpOr` is real and now passes every check. Its verdict is the
strongest evidence in the ledger. The check did not find a defect in the work — it found a defect in
itself, and it found it because it was pointed at a verdict written in a way it had not anticipated.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- One pending card, `acc-disp-MlhRWjAlGcaPUj2BqMpOr`, on `packet/vela/washing-pipeline`. Clickable.
- Push: local and remote both at `7a71780` at the start of this hour. The work of this hour is
  committed and pushed below.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **The card is now waiting for you.** Vela's doc-gate change, verified by Flux with the numbers
  reproduced independently. Accepting it is your call, not mine.
- **The 26 CJK files and the 11 incomplete-frontmatter files**, now measured separately for the first
  time. The 26 cannot simply be renamed — 29 inbound references, 24 in the knowledge index.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
