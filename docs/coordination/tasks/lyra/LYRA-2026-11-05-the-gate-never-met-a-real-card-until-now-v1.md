# Standing-duty measurement of the clickability gate against a **real** branch, which had never happened: the gate landed at 15:16 with zero pending cards, so every test of it used a stubbed row. **I drove it against the real `packet/mira/knowledge-panel`, which is currently 2 commits behind `track`, and it correctly says OK — the first time the operator's ruling has been exercised on real state rather than on a fixture.** And that measurement found one gap: the `behind` note is computed but only printed when something else has already failed, so a fully-green run tells the operator nothing about branches being behind.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-gate-never-met-a-real-card-until-now-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, gate, ruling |

## The gap in the verification, which is not his fault and is real

**`6dbc3af` landed at 15:16. There have been zero pending cards since the operator accepted both at 03:43. So the
gate has never run against a real card** — every test of it, including mine last round, drove it with a stubbed
`podman` returning one synthetic row.

**I am not reporting this as a defect in his work.** A gate cannot be tested against state that does not exist, and
stubbing the row source is the correct way to get the state. **But a stub is a claim about shape, and I had not
checked that claim against the real thing.**

## What I checked, and the stub was right about shape

**The gate's query** (`check-cards-clickable.sh:89-93`): `select a.branch, a.base, a.id from acceptances a where
a.status='pending' order by a.branch, a.id`.

**His stub** returns `branch|base|id` in the same order — measured, three fields, same separator, same order. **And
the real cards that existed earlier today had the same shape**: `packet/mira/knowledge-panel|track/infra-foundation|
acc-disp-Ra5YbN4t78bzdbYhYkQiV`.

**So the shape claim holds.** That closes the possibility I was worried about, which was a fixture that could never
have matched a real row.

## The first real exercise of the ruling, and it behaves

**The real `packet/mira/knowledge-panel` is currently 2 commits behind `track`** — measured with
`rev-list --count packet/mira/knowledge-panel..track/infra-foundation` → **2**.

**That is exactly the state the ruling was about**, and it exists right now by accident, because the two commits that
landed after her merge were mine and Aegis's documentation.

**Driving the gate against that real branch:**

```
$ SEATLOOM_REPO_ROOT=<repo> PATH=<stub>:$PATH bash scripts/check-cards-clickable.sh
gate repo root: ... (is a git repository - the gate can see branches)
OK: all 1 pending cards are clickable
```

**It exits 0 and does not refuse.** Under the old criterion this exact state produced `FAIL: 1 of 1 pending cards are
UNCLICKABLE` — I measured that myself last round when I restored the level refusal and the test reported
`level-restored gate says: FAIL: 2 of 1 pending cards are UNCLICKABLE`.

**So the ruling is implemented and behaves on real state, not only on a fixture.** That is the first time this week
a ruling of mine has been verified against the thing it governs.

## The gap the measurement found

**The `behind` note is computed and then only printed when something else has failed.**

```
:146    [ "$behind" = "0" ] || behind_note="behind ${behind}"
:201    if [ -n "${behind_note:-}" ]; then          <- inside `if [ "$bad" -gt 0 ]`
:202      echo "note (not a refusal): some branches are merely behind — ${behind_note}"
```

**So on a fully-green run the note never prints, and the output is just `OK: all 1 pending cards are clickable`** —
which I measured above.

**Why this matters, and it is small but it is the ruling's whole point:** the note is what teaches the operator that
behind is no longer a refusal. On a green run he sees `OK` and has no way to learn that the branch is behind and that
this is fine. The information exists in the script and is discarded exactly when it is most useful — the run where
nothing is wrong.

**The failure family is one I have recorded repeatedly this week: a value computed correctly and then dropped by the
control flow around it.** It is not a wrong answer; it is a missing one.

## What I am doing about it

**Not editing the gate.** It landed twenty minutes ago, it is verified in both mutation directions, and the gap is a
missing print rather than a wrong decision. Editing another seat's freshly-landed instrument in the same afternoon,
for a cosmetic gap, is how a correct change acquires an unverified one.

**What I am doing is recording it precisely enough to be cheap to fix** — the note should print whenever it is
non-empty, regardless of `bad`, and the wording already exists at `:202-204`.

## What I did not do

I did not edit the gate or any script. I did not merge, push, accept, cancel, or re-open any card, restart any
application, or write to the live database. My measurements were reads plus one stubbed run: the gate's query and its
`behind_note` handling, `rev-list --count` for the real branch's behind count, `git log -1` for the landing time, the
`acceptances` rows for shape, and one run of the gate with a stubbed `podman` returning the real branch and base. The
ledger check is exit 0, `seats` is 10, `canonical_events` is 37.
