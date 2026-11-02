# Standing-duty verification: the docs gate is RED on track right now, he caught himself producing a false green with the wrong baseline, and the operator-visible defect he named is real — eight core seats exist and the UI shows one, because only one is bound to the project

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-the-gate-is-red-and-the-ui-shows-one-seat-of-eight-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, verification, gate-red, product |

## The gate is red on track, right now, and I reproduced it

**`docs/coordination/knowledge/BUDGET_POOL_MODEL_MAP.md` — the operator's own 17:50 commit — fails §2:**

```
$ bash scripts/verify-docs.sh            # the default path CI runs
✗ docs/coordination/knowledge/BUDGET_POOL_MODEL_MAP.md:
  missing universal-header field(s): template subtype id status author date version tags (§2)
EXIT=1
```

**And running it with the tip as a baseline hides it entirely:**

```
$ bash scripts/verify-docs.sh 3e3c8bc    # tip as baseline
all conform
EXIT=0
```

**Same file, two answers, and the difference is the baseline.** With a tip argument the file is inside
`tip..HEAD` as *already present* rather than *new*, so it is not in the checked set. **Aegis caught this in
himself before publishing it** — his delivery says he ran the tip form, was about to report the gate green,
then checked, saw the file had no header fields at all, re-ran the default path, and got the failure. **His
own words: "this is a false green I caused with the wrong baseline, not a gate defect."**

**That is the right attribution and I am recording it because the distinction is the whole point.** A check run
with a baseline that cannot fail and a check that is broken look identical from the outside. **He did not
report the green, and he wrote down the near-miss.** This is the third time in this stretch that a false green
was almost published, and the first time the seat that nearly published it caught itself first.

**The gate is still red.** The operator's document needs a header, and it is the operator's document.

## The operator-visible defect, and it is the one that matters most today

**He named it and I verified it independently. The operator sees one seat in the UI. There are eight.**

```
project_role_bindings WHERE project_id='seatloom':
  seat-vela-001 | data_engineer          <- one row, total

seats WHERE id='seat-aegis-001': 0 rows
```

**The eight core seats exist in `seats`** (flux, lyra, mira, nimbus, onyx, vela, plus aegis missing entirely),
**but only vela is bound to the project.** And `useDataStore.ts:1927` documents why that is the whole story:

> `listSeats(projectId)` is already project-scoped server-side — the join through project_role_bindings does
> it, so no client-side filter is needed. Seats not bound to THIS project are contextual, not members of it.

**So the seat list the operator sees is the bindings, not the seats.** Eight registered seats, one row, and
the UI is telling the truth about what it was asked. **That is the same shape as the provenance badge: not a
broken view, a correct view of an incomplete table.**

**The fix exists and is verified but unrun:** `scripts/bind-seatloom-seats.sh`, dry-run by default, inserts
`seat-aegis-001` and six role bindings taken verbatim from seed 001, leaves vela's row and every existing row
alone (`ON CONFLICT DO NOTHING`). **It writes the live database, so it is the operator's to run** — and Aegis
says so explicitly: it is a live-ledger write and he is permission-capped.

## Why this is the most important finding of my stretch so far

**Every other finding this stretch has been about instruments — checks that lie, guards with author
privilege, scripts whose headings contradict their loops.** This one is about **the operator's own screen**.
He opens the product and sees one seat of eight. **No instrument would ever have told him, because every
instrument is green and every green is honest about the set it can see.**

**And it is the third operator-visible defect in two days** — the SAMPLE badge on live tabs, the installed
build predating every fix, and now the seat list showing one of eight. **All three are the same failure: a
correct view of an incomplete table, which reads as a working product.**

## What I did not do

I did not run `bind-seatloom-seats.sh --apply` — it writes the live ledger and the operator runs it. I did not
edit `BUDGET_POOL_MODEL_MAP.md`, which is the operator's own document and needs his header. I did not change
any binding. My measurements were the gate's own two runs, `psql` selects on `project_role_bindings` and
`seats`, the bind script's dry-run, and `sed` on the store. `seats` is 9, `canonical_events` is 1,
`check-ledger-exists.sh` is exit 0.
