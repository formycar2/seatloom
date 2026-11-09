| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1200-the-328-docs-are-three-different-things |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, docs-gate, classification, honesty |

# The 328 docs are not one problem. They are three, and only one of them is a defect

## What I said last hour, and what is wrong with it

Last hour I reported "328 docs fail the header rules, are declared nowhere, and are invisible."
That was true as a count and wrong as a conclusion. I reported a number without classifying it,
which is the same error as reporting a green light without asking what it covers.

I have now classified all 328.

## The actual breakdown

| count | what it is | is it a defect? |
|---|---|---|
| **63** | `docs/coordination/knowledge/` — use YAML frontmatter with `id`/`title`/`curation`/`confidence`/`source_kind` | **No.** This convention is deliberate and documented in `INDEX.zhs.md`: the fields align to the `memories` table columns so the pipeline can load them without conversion. The gate does not know the convention exists. |
| **221** | task and memory docs with no header table at all — prose deliverables | **Partly.** These predate the template rule. They are real debt, but they are historical records, not contracts anyone reads for metadata. |
| **28** | header tables using a *different* schema — `doc_id`/`doc_date` instead of `id`/`date` (23), or translated key names (5) | **Yes, and this is the interesting one.** These are attempts to follow the rule using the wrong key names. Someone meant to conform and had no way to know they hadn't. |

## The finding that matters

**The gate's rule and the knowledge-base convention disagree, and the gate wins by silence.**

`INDEX.zhs.md` documents the frontmatter shape on purpose, with a stated reason — the fields
align to the `memories` table so nothing needs converting later. 63 files follow it exactly. All
63 fail the gate. None of them are declared as debt, because nobody noticed they fail: the gate
only looks at changed files, and these were written once.

So a contributor writing a knowledge doc today has two documented instructions that contradict
each other, and no signal which one is enforced. That is worse than either convention alone.

The 28 wrong-key docs are the same failure from the other side: the rule says `id`, the person
wrote `doc_id`, and nothing told them. **A rule that cannot be checked until you already know it
is not a rule.**

## What I am not doing

I am not mass-editing 328 files, and I am not adding 328 lines to the grandfather list to make
the number look tidy. Both would be the same move: make the dashboard green without changing
what is true.

## What I recommend, for your decision

1. **Teach the gate the knowledge convention.** It is documented and intentional; the gate should
   accept it. That is a small change to `check_file` and it removes 63 false failures.
2. **Declare the 221 prose docs as debt** in one commit, so the number is honest. They are
   history, not contracts.
3. **Fix the 28 wrong-key docs.** Small, and it is the only group where the intent was to comply.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Push **confirmed**: remote and local both at `ea93003`. The intermittent network resolved on
  the fourth attempt of the verify loop.
- Both pending cards clickable.
- Sandbox: 95-byte fake, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Dispatched this hour

`disp-i9M3ZtGSqZJC_jRa8J_ic` to the verification seat at priority 97, for the containment fix
at `33909f1` — which is the load-bearing one and has no independent verdict. The dispatch
explicitly forbids running any mutation that could reach the real path, and states that my
previous instruction to do exactly that was the error. It asks the verifier to look for the
*next* instance of the pattern, because every fix so far has covered the instance it was shown
and left the next one.
