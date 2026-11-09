| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1200-the-26-cjk-files-cannot-just-be-renamed |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, docs-gate, classification, operator-decision |

# The 26 Chinese knowledge files cannot simply be renamed, and I nearly recommended exactly that

## The finding

Last hour I reported that 26 knowledge files still fail the doc gate on the CJK rule, and offered
three options: rename to `.zhs.md`, translate, or declare as debt. Renaming looked like the cheap
one.

I measured it before recommending it again, and it is not cheap:

| measurement | result |
|---|---|
| the 26 files | all plain `.md`, all carrying Chinese content |
| a `.zhs.md` counterpart already exists | **0 of 26** — these are standalone, not duplicates |
| referenced by `.md` name in `INDEX.zhs.md` | **24 of 26** |
| total inbound references across `docs/` | **29** |

So renaming would break 29 links, including 24 in the knowledge index itself — the file whose whole
purpose is to make the knowledge base navigable. **A rename that fixes one gate rule and breaks the
index is not a fix.**

## What this means for the three options

| option | cost, measured |
|---|---|
| rename to `.zhs.md` | breaks 29 inbound references; requires updating the index and any other referrer |
| translate to English | 26 files of real content, each a seat's recorded judgement |
| declare as debt | 26 lines in the grandfather list, honest and reversible |

Translation is the only one that leaves the index intact, and it is the most expensive. Declaring
the debt is the cheapest and hides nothing. **I am not choosing between them** — this is a call
about how the knowledge base should be written, and that is yours.

## What I did this hour

Dispatched `disp-O-3Pxj-Z4jl9VWFBH1esc` at priority 95 for Vela to open an acceptance card for the
verified doc-gate change at `4927d34`. It merges cleanly into current track, its test is 8/8, and I
verified the numbers independently — but it has no card, and a card is the only path to the
mainline. I will not merge it myself.

The dispatch requires the card's plan to state three things: what the change does, what it does
**not** do (the 26 CJK files are untouched by it), and the load-bearing negative case as a required
step — create a knowledge file with no frontmatter and no header, run the gate, and read that it is
refused with exit 1. If that is waved through, the change is decoration and the card should be
rejected.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Every queue empty: zero dispatches ready, blocked or in flight, zero pending cards. No seat is
  trapped.
- Push: local and remote both at `08a2a2e`, confirmed.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **The 26 Chinese knowledge files.** Rename (breaks 29 links), translate (26 files of content), or
  declare as debt (cheapest, hides nothing). My measurement says the rename I offered last hour is
  not the cheap option it appeared to be.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
- **Which remote is the source of truth.** gitlab is 935+ commits behind and I have not pushed there.
