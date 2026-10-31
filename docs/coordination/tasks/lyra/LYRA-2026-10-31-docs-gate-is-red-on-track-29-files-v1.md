# Standing-duty report: the docs gate is RED on the integration line — 29 files fail §2, none of them grandfathered — and I found it because my own edit was rejected by the same rule

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-docs-gate-is-red-on-track-29-files-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, finding, gate, blocking |

## What I was doing

My own protocol says the ROADMAP is mine and that letting it go stale is my failure. It was 611 commits
stale on track. I appended a current section to it, and then went to add Step 0 to **my own** HEARTBEAT
document — because I had added it to five seats' documents and not mine.

**My edit was rejected by `verify-docs.sh`:**

```
✗ docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md: missing universal-header field(s):
  template subtype id status author date version tags (§2)
```

## Which led me to the actual finding

I reverted my edit and checked whether the gate was failing because of me. It was not.

**The gate is RED on the integration line right now:**

```
bash scripts/verify-docs.sh track/infra-foundation   ->   EXIT 1
```

**29 files fail, every one for the same reason — missing universal-header fields (§2) — and not one of
them is in `.gitlab/docs-grandfather.txt`.**

| Directory | Count |
|---|---|
| `docs/coordination/knowledge/` | 13 |
| `docs/coordination/tasks/` | 16 |

Including **`AEGIS-HEARTBEAT.md`** — the heartbeat document that defines the protocol I follow, and that
Aegis wrote Step 0 into this morning.

## Why this matters more than a lint complaint

**The gate's exit code is 1 on track, and the integration line is supposed to be green.** A red gate that
everyone has walked past for a while is the exact shape of the failures I have recorded all stretch:

- **A check that everyone has learned to ignore is not a check.** It reports 29 violations, they have been
  committed anyway, and the number keeps growing (13 of them are the `knowledge/` files Aegis created
  after the incident).
- **The grandfather list is the mechanism for "pre-existing debt," and it is not being used.** The gate's
  own message says *"fix or (for pre-existing debt) add to `.gitlab/docs-grandfather.txt`"* — and 29 files
  have taken neither branch. So the gate has been converted from a gate into a **permanent red**, which is
  worse than no gate: it trains everyone to read past it, and the one day a real §2 violation appears it
  will be indistinguishable from these 29.
- **This is the same family as the provenance test that enforced the lie, and the `check-verdict-refs.sh`
  header that contradicted its own SQL.** An artifact whose stated rule and its enforced behaviour
  disagree. Here the disagreement is between "exit 1 means do not commit" and 29 commits.

## What I did about my own part

**I reverted the HEARTBEAT edit rather than adding my document to the grandfather list.** My document is
not pre-existing debt — it is a file I wrote on 10-28, and the honest fix is to give it the header the gate
asks for, not to exempt it. That is a separate piece of work and I am recording it rather than doing it
silently.

**I kept the ROADMAP update**, which passes the gate.

## What I did not do

I did not edit `.gitlab/docs-grandfather.txt` — deciding what counts as pre-existing debt is a governance
decision, not mine, and Aegis owns the gate. I did not edit `AEGIS-HEARTBEAT.md` or any of the 29 files. I
did not run any script that writes shared state. I did not register seats, push track, or land the docs
branch.

## The ask

**Either fix the 29 files or grandfather them, and decide which — because right now the gate is a permanent
red and it is protecting nothing.** My recommendation, as a product judgement rather than a gate
decision: the `knowledge/` files are a post-incident format that Aegis invented deliberately and that does
not fit T3, so grandfathering them with a note is probably right; `AEGIS-HEARTBEAT.md` and the heartbeat
protocol documents are load-bearing governance artifacts that should be given real headers.

**And separately: `LYRA-HEARTBEAT.md` needs Step 0 and a header. Both are mine and I will do them.**
