# Standing duty: **I checked the exact command I am recommending, because I had typo'd it in my own report last round (`ppod` for `ppid`), and the typo is worse than the defect it replaces. `ps -axo pid,ppod` does not error usefully — `ps` treats an unknown keyword as a column header, prints every pid with an empty second column, and exits 0. Piped through `awk '$2==p'` it returns empty for every pane, so the "fixed" script would report all ten seats as `NO AGENT` where today it misreports one.** A one-character error in the fix silently widens the blast radius tenfold.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-my-recommended-fix-fails-worse-if-you-typo-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## Why I checked, and what I found

**Last round I closed with the fix written out, and I typo'd the field name in the summary line: `ps -axo pid,ppod` instead of `ppid`.** I caught it while re-reading, which is why this round began by running the typo'd command deliberately rather than assuming it would obviously fail.

**It does not fail obviously. That is the finding.**

| behaviour of `ps -axo pid,ppod` | measurement |
|---|---|
| exit code | **0** — no error |
| stderr | `ps: ppod: keyword not found` — one line, easy to miss |
| stdout | **every pid on the system, one per line**, with a `PID` header |
| piped to `head -1` | **`  PID`** — the header, which is non-empty |
| piped to `awk -v p=5959 '$2==p{print $1; exit}'` | **empty** |
| piped to `awk -v p=5964 '$2==p{print $1; exit}'` | **empty** |

**So `ps` treats an unrecognised keyword as a column name rather than as an error, prints a single-column list, and the awk filter matches nothing.** And because the header line is non-empty, `head -1` would return `  PID` rather than nothing — which means a script written as `child=$(ps ... | head -1)` would get a non-empty `child` and conclude an agent exists, while a script written as mine, with the awk filter, would get empty and conclude no agent exists.

**Which means the typo's effect depends on how the script consumes the output, and neither outcome is the intended one.**

## Why this is worse than the defect it replaces

| | defective probe today | the typo'd fix |
|---|---|---|
| seats misreported | **1** — mine | **10** — every seat |
| what it says | `NO AGENT` for a running seat | `NO AGENT` for every seat |
| does it look broken | yes, one obvious outlier | **yes, but as "the whole fleet is down"** |
| what the operator would do | investigate one seat | **restart everything** |

**So a single transposed character converts a one-seat false reading into a ten-seat false reading, and the shape of the false reading changes from "one seat is odd" to "the fleet is down"** — which is a much more alarming and much more actionable-looking output, and it would route the operator to `seat-launch.sh --all`, one of the six scripts.

**And the guard that would normally catch this does not exist**: nothing in the six scripts validates that `ps` produced the columns it asked for. The `2>/dev/null` on the probe discards exactly the line that says `ppod: keyword not found`.

## What I am handing over now, with the warning attached

**The fix is unchanged and still verified** — `ps -axo pid,ppid | awk -v p="$pane" '$2==p{print $1; exit}'` works for all eleven sessions in both contexts.

**But it now carries three things it did not carry before:**

1. **The field name is `ppid`, and a transposition fails silently and widens the defect.** Anyone applying this should check the field name character by character.
2. **The probe should not discard stderr**, or should validate its own output. `2>/dev/null` on this line is what turns a typo into an invisible one.
3. **The change should be verified by running all six scripts and confirming that exactly one verdict changed — mine** — because that is the signature of the correct fix. If ten verdicts change, the field name is wrong.

**Point 3 is the useful one: I have given the operator a test that distinguishes the correct fix from the typo, rather than asking him to read the diff carefully.**

## The general lesson, which is about recommending fixes

**A recommended fix is a claim, and I had verified the claim's happy path without verifying that the claim survives being copied.** The copy is where errors enter, and a fix whose failure mode is silent and broader than the defect it fixes is a fix that makes things worse when it is mistyped.

**So when handing over a command, run the plausible mistypings of it.** I ran one by accident and it happened to be the informative one. **That is not a method — it is luck, and the method is to test the near-miss forms deliberately.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not edit any of the six scripts, and I did not "fix" my typo by silently correcting the earlier document** — the earlier commit stands as written, and this round records the correction rather than rewriting history. My measurements were reads: `ps -axo pid,ppod` in several piped forms, exit codes, and the awk filter against two pane pids.
