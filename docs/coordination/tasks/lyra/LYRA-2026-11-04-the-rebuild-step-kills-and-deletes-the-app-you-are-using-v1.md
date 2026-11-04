# Standing-duty measurement of the step between "accepted" and "the fix is in the app I use": the script that installs a verification build into `/Applications/SeatLoom.app` **kills the running app and deletes it** before installing, and it identifies the app with `pgrep -f "SeatLoom.app"` — a pattern that matches command lines, not executables. I measured both, and while simulating the pattern I matched my own shell. The sandbox is safe from it; the operator's running app is not the thing at risk, the processes around it are.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-the-rebuild-step-kills-and-deletes-the-app-you-are-using-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, measured, live, gate |

## Why I measured this

Last round I established that accepting a card merges the fix into the primary checkout, and that **the operator's
running app will not change** — it was built 10-28. I left the rebuild as "a separate step and yours to order."

**This round measures that step, because "yours to order" is only honest if I know what he would be ordering.**

## The script, and what it does before it installs

`scripts/prepare-verification-build.sh <branch>` — written 2026-10-16 to close a deadlock Mr. Zhang named himself:
cards could only merge on his Accept, but verifying a card needs a build carrying the work, and nobody ever
prepared one. **The rule became: the card and the build arrive together.**

**Its install step (`:122-141`):**

```bash
# (the script's messages are Chinese; the commands are quoted as-is)
if pgrep -f "SeatLoom.app" >/dev/null 2>&1; then
  pkill -f "SeatLoom.app" 2>/dev/null
  ...
  pgrep -f "SeatLoom.app" >/dev/null 2>&1 && fail <the old app would not stop - refusing to install over it>
fi
rm -rf "$APP" || fail <cannot delete the old one>
cp -R "$BUILT" "$APP" || fail <copy to \$APP failed>
```

**It kills the running application and deletes `/Applications/SeatLoom.app` before installing the new one.** There is
no dry-run and no `--apply` — unlike `prepare-sandbox-build.sh`, which defaults to a dry-run and requires an
explicit `--apply`. **Running this script is the action.**

## The `pgrep -f` pattern matches command lines, and I matched myself

**`pgrep -f "SeatLoom.app"` matches the full command line, not the executable name.** I measured this by accident
and then deliberately: running the pattern from my own shell returned **two** pids where only one application was
running — the extra one was **my own bash process**, because my command line contained the string.

```
$ pgrep -af "SeatLoom.app"
19363   <- my own shell, different pid on every invocation
90805   <- the real app
```

**So `pkill -f "SeatLoom.app"` kills every process whose command line contains that string** — which includes any
seat running a check that names the path, any `grep`, and the script's own caller if it names the app.

**This is the exact failure Aegis recorded on 10-28** (*the app-stopped check reported "open" because `pgrep -f
'seatloom-tauri'` matched the checker's own command line*). The fix was applied to a check; **the same pattern is
still load-bearing in an installer that deletes an application.**

**What is safe:** the sandbox. `/Applications/SeatLoom-Sandbox.app/...` does not contain the substring
`SeatLoom.app` — `SeatLoom` is followed by `-Sandbox.app`, not `.app` — so `pkill -f "SeatLoom.app"` does not
touch it. I verified the substring is absent rather than reasoning about it.

## What is correctly in place

| thing | measured |
|---|---|
| the sandbox carries an operator identity | `SEATLOOM_OPERATOR_REF=human:supervisor` |
| the main app carries one too | `SEATLOOM_OPERATOR_REF=human:supervisor` |
| so an Accept clicked in the sandbox can resolve the operator | yes — and `launch-seatloom.sh` exists precisely because an app launched from Finder has no identity and every merge would fail |

**That last one matters for the rebuild**: if the app is reinstalled and started with a bare `open -a`, the
operator identity is lost and Accept stops working. The script prefers `launch-seatloom.sh` for exactly this
reason, and only falls back to `open -a` when the launcher is missing.

## What this means for the operator, in his three sections

**Changed:** I have now measured the rebuild step instead of leaving it as "yours to order."

**What you will see:** running `bash scripts/prepare-verification-build.sh <branch>` will **stop the SeatLoom you
are using right now, delete `/Applications/SeatLoom.app`, install the new build, and start it** — with your
operator identity, because it uses the launcher. It is not a dry-run and there is no confirmation step. **Do it when
you are not mid-way through something in the app.**

**What needs you:** the rebuild is the step that puts the fix in the app you actually use. **Accepting the card
merges the code; only rebuilding puts it on your screen.** And one thing to know before you run it: the script
finds your app by a command-line pattern, so if any other tool on this machine has `SeatLoom.app` in its command
line, that tool gets killed too. It will not touch the sandbox.

## What I did not do

I did not run the script, did not build, install, delete, kill, or start anything. My measurements were all reads:
`prepare-verification-build.sh:122-141` and its header, `launch-seatloom.sh`'s header, `pgrep -af` and
`pgrep -f` run repeatedly to see which pids they return, `ps` on those pids, a substring test of the sandbox path,
and `ps eww` on both running applications for their environment. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 25.
