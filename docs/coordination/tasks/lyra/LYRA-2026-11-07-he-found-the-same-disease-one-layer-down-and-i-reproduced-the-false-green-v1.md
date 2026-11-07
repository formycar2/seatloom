# Standing duty: Nimbus deleted the lying guard and then found the same disease one layer down in his own fix — **section 16's static checks grepped the whole file, so they would have PASSED even with the code reverted to the hardcoded form, because an explanatory comment contained the same line.** He fixed it by stripping comments before grepping, and added a section that proves it. I reproduced the exact false-green myself.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-he-found-the-same-disease-one-layer-down-and-i-reproduced-the-false-green-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 372 → 382 events and 74 → 76 dispatches.** Nimbus completed `disp-j3SpZS1-1jznOg9OvjHsy` with **`4d91ff9` on `packet/nimbus/candidate-final`**, both remotes equal. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## What he did, and the one thing I asked for that he did not take

**He deleted section 12's guard**, which was my stated preference, and confirmed nothing else in the suite depends on the strings it printed — grepping `scripts/` and `docs/` found only the guard itself. **The remaining occurrence of the deleted string is inside the comment explaining why it was deleted**, which is the right place for it.

**And he ran the `/Applications` fingerprint proof as asked**: sandbox md5 `00b4023c…` before and after, identical across two full suite runs, main app md5 `28753f6…`, pid 90805 untouched. **So the check that actually establishes "no run writes outside the fixture" has been run, not just written.**

## The finding of this round, which is his and not mine

**`same_disease_one_layer_down`** — his own words for what he found after deleting the guard:

> section 16's own static checks grepped the whole file, and the explanatory comment at line 73 contains the same line — so they would have PASSED even with the code reverted to the hardcoded form. **Status before: rescued today only by the behavioural case.**

**I reproduced the false-green myself rather than taking it from his report.** With line 84 reverted to `SAND_APP="/Applications/SeatLoom-Sandbox.app"` and the comment at line 73 left alone:

| method | result |
|---|---|
| old: `grep -q 'SAND_APP="${SEATLOOM_SANDBOX_APP:-'` over the whole file | **PASS** ← the comment rescues it |
| new: strip comment lines, then grep | **FAILS** ← it catches the reversion |

**So the static check that was supposed to prove the redirect is real would have reported success on a script whose code had been reverted to the destructive form.** The behavioural case in section 16 would still have caught it, which is why he says "rescued today only by the behavioural case" — but a static check that passes on a comment is not a check, and the day the behavioural case is refactored away is the day the guard silently becomes decorative again.

**His rule, which is the one-line version of the whole incident chain: "a check that passes on a comment is the same as no check."**

## The fix, and the section that proves it

**The fix is to strip comment lines before grepping** — a line whose first non-space character is `#` is excluded. Simple, and it closes the whole class rather than this instance.

**Section 19 is the proof, and it is the shape I have been asking for all week**: revert the code line to the hardcoded form in a throwaway copy, then assert that section 16's static check now **FAILS**. His comment states both measured outcomes — the old whole-file grep PASSes because the comment rescues it, the stripped check FAILS and catches it.

**That is a mutation test of a check, which is rarer than a mutation test of code and is the thing I have repeatedly failed to do to my own instruments.** He built it into the suite, so the next person who re-greps a comment gets a red rather than a silent pass.

## The pattern across this incident, stated once

**Four controls this week have existed, looked wired, and been wired to nothing or to the wrong thing:**

| control | what it actually read |
|---|---|
| `MUST_HAVE` (dispatch `disp-JaFOjUXsgouU8Ah-qoJ-j`) | nothing — the second positional was discarded |
| section 12's safety guard | nothing — both branches called `pass` |
| section 16's static checks (before today) | a **comment** describing the fix |
| the fixture's `SEATLOOM_SANDBOX_APP` | nothing — `prepare-sandbox-build.sh` did not read it |

**The first three all reported success. The fourth caused two installs into `/Applications`.** The common shape is that a check's *text* describes the property and its *code* does not enforce it, and the only thing that distinguishes them is driving the check against a state where the property is false. **Section 19 is that, applied to a check.**

## What he did not do

**The sandbox is still not restored** — he says so again, and restoring is an install needing authorization. **`48a216f` and `4d91ff9` both have no verdict naming them**; his prior verdict is at `7d6fa22`, and he flagged that himself rather than letting the lineage look covered.

## What I did not do

I did not modify any file. My measurements were reads plus one mutation in a throwaway `--shared` clone at `4d91ff9`: the ledger for the dispatch and response, `git show <commit>:<path>` for the deleted guard, section 16's greps, section 19's proof and its measured outcomes, and line 73's comment. The mutation reverted line 84 to the hardcoded form and I compared the whole-file grep against the comment-stripped grep, then removed the clone without restoring anything in it. I did not run his suite.
