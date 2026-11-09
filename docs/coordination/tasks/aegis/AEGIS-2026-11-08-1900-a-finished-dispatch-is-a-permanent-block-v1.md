| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1900-a-finished-dispatch-is-a-permanent-block |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, dispatch, structural-defect, ledger |

# A dispatch whose work is finished but never closed is a permanent block on that seat, and I found the mechanism

## What was wrong

`disp-pTLbm6ciZmDivez6F7rZJ` sat `in_flight` for **16 hours**. Vela's pane showed *"new governed
dispatch available — run your dispatch loop now"* repeating, with the agent idle at a prompt. The
seat was being woken correctly and still could not do anything.

The mechanism is structural, and it is not a seat failing:

1. The database has `uq_dispatches_one_in_flight_per_seat` — a unique index on `target_seat_id`
   where `status = 'in_flight'`. One active task per seat, globally, by design.
2. `get_next_dispatch` honours any in_flight the seat already holds **in any project** by handing
   it back, rather than trying to deliver a second one and hitting the index with an opaque
   rollback. That is the right defensive choice.
3. So a row that is finished-but-not-closed is handed back forever. The seat asks for work, gets
   the same stuck row, and can never receive anything else. Vela had **zero ready dispatches** —
   not because none were issued, but because the slot was occupied by a finished one.

**One missing `complete_dispatch` call disabled the seat permanently.** This is the same defect
the protocol's closing step was written for, and I had only ever seen it from my own side.

## What I did

I did **not** rewrite Vela's ledger row — that is not mine to change, and it would have been the
same laundering I have refused before.

I cancelled the dispatch as its issuer, which is a governed action and which the tool documents as
releasing the one-in-flight slot. Their uncommitted work in the worktree is untouched. Then I
re-issued at priority 96 with everything measured, so the seat can finish properly.

## What I measured about their work, so it is on record

| claim | measured |
|---|---|
| the header exemption works | **63 → 0** files failing the header rule |
| the `author` defect I reported | fixed — now a declared measured default |
| files still failing | 63, on **other** rules: 26 CJK, 37 missing frontmatter keys (`id` 27, `tags` 9, `date` 1) |

And the test cannot see its own success: `scratch_repo()` creates an **empty** knowledge
directory, so its `BEFORE` count is structurally zero and it prints *"the exemption did not reduce
failures (0 → 63)"* when the header failures in fact went 63 → 0.

## The re-issued dispatch says three things

1. **Commit the work.** 85 insertions plus a new test script, currently uncommitted — which is why
   it kept looking stuck.
2. **Fix the measurement bug.** Copy the real knowledge files into the scratch repo, or measure
   BEFORE in the real tree.
3. **Call `complete_dispatch` with the id copied from the ledger.** That single missing call caused
   all of this.

It also says what **not** to do: do not change the CJK rule, and do not add defaults for `id`,
`tags` or `created_at`. Those 37 files need a convention decision from you, and supplying a file's
identity from the gate is not a fix I want a seat making on my say-so.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Zero seats blocked on in_flight. `check-salvage-fresh.sh` green.
- Sandbox: 95-byte fake, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **The convention decision**: are `id`, `tags` and `created_at` required keys of a knowledge
  file? 37 files lack at least one. Yes means fixing the files; no means the gate defaults them,
  which I think is wrong.
- **The sandbox rebuild**, still the only thing between the remaining verified work and you.
