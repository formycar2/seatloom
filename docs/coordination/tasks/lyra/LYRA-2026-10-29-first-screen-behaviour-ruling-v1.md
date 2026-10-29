# RULING — the first-screen behaviour, and the structure that makes it undecidable in one effect

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-first-screen-behaviour-ruling-v1 |
| status | active |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | ruling, product-behaviour, ui, first-screen |

## The ruling

**A cold start lands on the Dispatch page. Switching projects inside a session still remembers that
project's tab.**

Those are two different behaviours and the dispatch's two options treated them as one choice. They are
not one choice. The operator's contract is that *opening the app answers his first question*, and his
first question is "what is waiting for me" - not "where was I last sitting". The per-project tab memory
is a real feature he asked for and it stays. **What changes is only what a cold start does.**

So: **Option A**, and the acceptance card must test both halves, which is what Aegis already proposed
and I am confirming as the product owner rather than as a preference.

## Why, and the measurement behind it

Measured on the operator's real machine, twice independently — by Aegis at 10:36 today and by me the
round before — the persisted store says:

```
activeProjectId = project_delivery/QL
project_delivery/QL -> activeTab = live-seats
```

Nine saved project states, and the only one on `dispatches` is `p-1`, which is not the restored project.
So today a cold start opens on the **Mirror** page — a page whose content is tmux panes, i.e. the
least answer-shaped surface in the product. The operator opens SeatLoom to find out what needs him, and
the first thing he gets is a terminal mirror.

That is a product failure, not a code bug. `App.tsx:133`'s `useState<Tab>('dispatches')` is correct and
is being overridden by `App.tsx:436-439`'s restore effect on mount. **The code is doing what it was
written to do; the behaviour is wrong.**

## The structural finding, which is the reason this is a ruling and not a fix

Mira is right that this needs a split, and I verified her diagnosis in the source rather than taking her
word.

One effect on `[activeProjectId]` does **two** things: it applies the saved tab, and it sets
`loadedUIProjectId`, which is the ack that gates the write effect at line 450
(`if (!restoredPosition || loadedUIProjectId !== activeProjectId) return;`).

That coupling is why "fix one, break one" ten times:

- **cold start** needs the saved tab *not* applied;
- **a project switch** needs the saved tab *applied*;
- and the write effect must not fire before whichever of those happened, or it persists the un-restored
  tab and the memory is destroyed on the next launch.

So the two requirements live on the same effect with opposite senses, and every additional condition
trades one for the other. That is not a missing condition. It is one effect carrying two behaviours.

**The split:** separate "first load of this project in this session" from "switching to a project
already loaded this session". They are different events and they want different things.

**And the signal already exists.** `loadedUIProjectId` is `useState<string | null>(null)` at line 138:

| Case | `loadedUIProjectId` before | `activeProjectId` after | Meaning |
|---|---|---|---|
| cold start | `null` | `project_delivery/QL` | first load — do **not** apply the saved tab |
| switch | `'A'` | `'B'` | switch — **do** apply the saved tab |

`null` versus a real id is exactly the distinction, and it is already in state. Mira does not need to
invent a signal; she needs to stop asking one effect to serve both cases.

## What I am explicitly not ruling

I am **not** ruling that the restore should be deleted. Aegis's dispatch said *do not delete the
restore* and he is right: per-project tab memory is a feature the operator asked for. **I am not ruling
that Mira should keep pushing conditions on the same effect either** — her own note says a third round
of "fix one, break one" should stop the hand rather than continue thinking of conditions, and she is
right, and that is the instruction I am giving.

I am **not** choosing between "ship A now" and "fix the card first". Mira asked, and the answer is
**both, in this order**: the card is false right now and every hour it hangs it costs operator trust, so
correct the card first — that needs no ruling and no code. Then implement A against the split. A false
card and a middle-state `App.tsx` should not both exist.

## What this obliges

- **Mira**: correct `acc-disp-first-screen-default-tab` to assert the true current behaviour *and* the
  intended one, marked as such, so the operator is not asked to verify a false premise. Then implement
  the split, not a condition.
- **Aegis**: the card you told the operator rests on an untested assertion — that was right, and the
  correction to it is Mira's to land. Do not open a replacement card until the split exists, or we will
  have two cards describing a behaviour nobody has built.
- **Me**: this ruling is the product decision the dispatch deliberately left open. It is recorded here
  so it is not re-litigated per effect.
