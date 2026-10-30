# Product finding: three cards are waiting on the operator, and the app he would verify them with is 43 hours older than the fix — the project's own "card and build arrive together" rule is not being followed

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-installed-app-predates-the-fix-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, product-finding, blocking, acceptance, build |

## The rule that exists

`scripts/prepare-verification-build.sh` opens with the reason it was written, and it is the clearest statement
of this problem in the repository:

> The rule for acceptance cards is "merge only when he clicks Accept" — that is right, it is what
> keeps from deceiving him. But verifying a card needs a program **carrying the commands**, and
> nobody ever prepared one. So the card is open, the work is done, he is sitting at the machine,
> and there is no way to try before clicking. Blind accept was his only option. **That is not
> caution, that is a deadlock.**
>
> So the rule became: **the card and the build arrive together.** Any card opened to pending must
> first run this script, install the program carrying the work, start it, and only then tell him
> the card exists. **A card that arrives without its build has not been told to him at all.**

*The rule for acceptance cards is "merge only when he clicks Accept" — that is right, it is what keeps from
deceiving him. But verifying a card needs a program carrying the commands, and nobody ever prepared one. So
the card is open, the work is done, he is sitting at the machine, and there is no way to try before
clicking. Blind accept was his only option. **That is not caution, that is a deadlock.** So the rule became:
**the card and the build arrive together.** Any card opened to pending must first run this script, install
the program carrying the work, start it, and only then tell him the card exists. **A card that arrives
without its build has not been told to him at all.***

The script even has a `MUST_HAVE` argument whose comment records the failure it was built to prevent: a
build that installs but does not carry the work is worse than no build.

## What is actually installed

```
/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri                    10-28 22:16
/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri.bak-20261028-221634 10-28 22:16
/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri.bak-20261028-221533 10-28 22:15
```

**The last install was 10-28 22:16.** The fix the first card exists to demonstrate:

```
d2fea84  mira: open the card + close Flux M-4 gap   10-30 17:50
branch tip                                             10-30 19:02
```

**The installed app is about 43 hours older than the fix.** I also probed the binary directly for
`chooseTab`, `restoreFrameWrites`, `activeTab`, `projectUIStates` — **zero occurrences**, and the embedded
frontend is brotli/zstd-compressed so a plain `strings` search is not conclusive. **The mtime evidence is
conclusive on its own: nothing has been installed since two days before the fix existed.**

## What this means for the three pending cards

| Card | What its steps require | What the installed app can show |
|---|---|---|
| `acc-disp-faEWBt8kSKgv-6pJs_Rqw` | quit and reopen → tab must be **Dispatch**; switch projects → that project's remembered tab; switch back → still there | **step 1 fails** — the write effect is still there, so the saved tab is overwritten |
| `acc-disp-E06O6IAaWefF-BOW-CNJd` | cold start must land on Dispatch, not the last page | depends on the same code path |
| `acc-disp-TbGRGr5cvpJFgCeaayu7S` | honest: says he **cannot** verify it today, and gives a refusal condition | correct as written |

**So the first card's own verification steps will fail on the app he has.** He would quit, reopen, see the
wrong tab, and correctly conclude the work is broken — **when the work is done and verified at 375/375 on a
branch he has not been given a build of.**

## Why this is the finding and not a footnote

This is the third independent reason the same three cards have not been clicked, and each one compounds the
last:

1. **The panel that lists them is closed** (`seatloom.supervisor.open = false`), with no badge anywhere else.
2. **The panel scopes to his active project** `project_delivery`, which has **zero** acceptances — so opening
   it shows "nothing to accept" while three wait under `seatloom`.
3. **The app he would verify them with predates the fix by 43 hours** — so following the card's steps
   produces a failure, not a verification.

**Any one of those is enough to stop him. All three are present.** And the project already knew about the
third one, wrote a script for it, wrote the reason into the script's header in plain language, and **the
script was not run for these cards.**

## The ask

**Run `scripts/prepare-verification-build.sh packet/mira/knowledge-panel` before telling him the card is
waiting**, with a `MUST_HAVE` string that only the fix would contain. The script exists, it stops the app
first, it refuses to install a failed build, and it verifies the process started. **It is one command and it
is the difference between a card he can act on and a card that will fail when he tries.**

And the first two reasons are the ones I asked for last round and they are still unfixed — a pending-count
badge that does not depend on panel state or active project.

## What I did not do

I did not run `prepare-verification-build.sh` myself. **It stops the running app, installs to
`/Applications`, and rebuilds from a branch** — that is an operator-side action on his machine and his
session, and a seat doing it unasked is the same overreach I have declined three times today. I did not edit
any of the three files I have asked about. I did not grant, cancel, or re-open any card. I did not change his
persisted localStorage.

## A note on my own evidence

The `strings` probe returning zero for `chooseTab` is **weak evidence** and I am labelling it as such: the
frontend assets are compressed inside the binary, so identifier search proves nothing — the same
dead-stripped-binary mistake I made on 10-29. **The mtime is the load-bearing evidence and it is enough: no
install since 10-28 22:16, fix authored 10-30 17:50.**
