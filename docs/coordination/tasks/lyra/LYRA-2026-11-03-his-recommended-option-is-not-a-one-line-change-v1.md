# Standing-duty verification: I checked whether the option he recommends is actually safe, and it is not a one-line change — the session name is referenced by three live scripts, one of which is an installed copy that cron executes, and no script in the repository creates the session at all

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-his-recommended-option-is-not-a-one-line-change-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, risk, option |

## Why I checked

Last round I verified his finding and declined to choose among his three options, saying renaming a live
session is an operator decision. **That was correct as far as it went, but it left the options unexamined, and
an unexamined option is a guess wearing a recommendation's clothes.** He recommends option 1 — rename
`Aegis-Supervisor` → `Aegis-Supervisor-seatloom` — and describes it as small and reversible. **So I measured
what the rename actually touches.**

## Measured: the session name is load-bearing in three live scripts

```
scripts/aegis-self-wake.sh       SESS="Aegis-Supervisor"     (line 33)
scripts/restart-aegis.sh         SESS="Aegis-Supervisor"     (line 36)
scripts/switch-aegis-to-pi.sh    (references the name)
```

**And the one that matters most is not in the repository's execution path.** The cron entry is:

```
0 * * * * /Users/jyxc-dz-0100609/.local/bin/aegis-self-wake.sh >> /tmp/seatloom_aegis_waker.log 2>&1
```

**Cron executes an installed copy at `~/.local/bin/`, not the repo script.** I verified the two are currently
byte-identical, and that `install-aegis-self-wake.sh:87` is what copies one to the other. **So a rename that
changes the repo script without re-running the installer leaves cron executing the old name** — and the old
name no longer exists, so the self-wake goes silent. Not an error: **silent**, which is worse, because the
wake failing quietly is indistinguishable from the wake working until a round is missed.

**He named this risk.** "the rename must be synchronised with the self-wake script and reinstalled, or the wake dies immediately" — the rename must be synchronised
with the self-wake script and reinstalled, or the wake dies immediately. **I am confirming the mechanism behind
his warning rather than adding a new objection**, and the mechanism is worse than "dies": it dies silently,
into a log file nobody reads.

## The part he did not mention, and it changes the option

**No script in the repository creates the `Aegis-Supervisor` session.**

`restart-aegis.sh` only ever *addresses* the session — `tmux list-panes -t "$SESS"`, `tmux send-keys -t
"$SESS"`. It has no `new-session`. I grepped all of `scripts/` for a creation and found none, and
`git log -S "Aegis-Supervisor" -- scripts/` shows the name arriving with the self-wake and restart scripts,
both of which assume it already exists.

**So the session was created by hand, and the rename is therefore also a by-hand action** — `tmux
rename-session`, run from somewhere, at a time when Aegis is not mid-turn. **That is not "small and
reversible" in the way a code change is; it is a manual operation on a live pane that a seat is currently
occupying.**

**And it has a second consequence he did not list:** the session's *creation* is unreproducible. If the
machine is rebuilt or the session is lost, nothing in the repository brings it back, and the next person has
to know the name by hand. **The rename makes that worse, not better**, because it adds a fourth name to the
set of things that must agree while none of them is the source of truth.

## What I am now recommending, and what I am not

**I am not choosing his option.** Renaming a live session and reinstalling a cron target are both operator
actions, and the third option — accept 6, explain in the UI — is a product decision. **All three are his, and
I have verified the mechanism behind each rather than picking one.**

**What I am adding is one measurement that changes the weight:** option 1 is not a one-line change. It is
three script edits plus a reinstall plus a manual `tmux rename-session` on a live pane, and its failure mode
is a silent wake. **That is a bigger change than "small and reversible" suggests, and the operator should
price it that way.**

**And one observation that is not a recommendation:** the deepest problem is not the name, it is that **the
session's name is the join key between three systems and is stored in none of them.** The tmux server knows it,
the cron line knows it, three scripts hardcode it, and the database's `seats.name` is a fourth, separately
maintained copy. **Any of the four can change without the others, and nothing detects it.** That is why a
naming convention ended up doing attribution work — there is no single place where "which session belongs to
which seat" is written down.

## What I did not do

I did not rename any session. I did not edit any script. I did not run the installer. I did not change the
cron entry. My measurements were `grep` on the three scripts for the session name, `git grep -l` across the
repository (25 files, of which 3 are scripts and the rest are documents), `crontab -l`, `diff` between the
installed copy and the repo script, `grep` for a `new-session` anywhere in `scripts/`, and `git log -S` for the
name's origin. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
