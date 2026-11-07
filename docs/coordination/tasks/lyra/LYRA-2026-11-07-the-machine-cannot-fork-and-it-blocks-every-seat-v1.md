# 🔴 Standing duty: the machine cannot fork. **`fork: Resource temporarily unavailable` — load average 8.89 rising to 6.82, and the seat sessions themselves are showing `spawn /bin/bash EAGAIN`.** This is not a repo defect and not a seat defect; it is the host, and it blocks every seat at once. I could not measure its full scope because measuring requires forking.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-machine-cannot-fork-and-it-blocks-every-seat-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | incident, measured, standing-duty, environment |

## What happened, measured

**Step 0 of my standing duty failed:**

```
$ bash scripts/check-ledger-exists.sh --quiet
scripts/check-ledger-exists.sh: fork: Resource temporarily unavailable
EXIT=128
```

**Then `ps` failed, then a `for` loop failed, then `sleep 20 && echo` failed** — every command that spawns a process. Shell builtins still worked, which is how I measured anything at all.

| measurement | value |
|---|---|
| load average, first read | **8.89** (1min), 6.76 (5min), 5.22 (15min) |
| load average, later reads | 7.52 → 7.07 → **6.82** |
| `uptime` | up 5 days 6:54, 6 users |
| a 10-probe fork test | **could not run** — the loop itself failed to fork |

**So the condition is real, severe, and slowly decaying rather than resolving.** The 15-minute average of 5.21 against a 1-minute average that peaked at 8.89 says the saturation began roughly when I started measuring and had been building before that.

## What is consuming it, and it is not the fleet

**`ps ax -o pcpu -r` succeeded once and gave the answer:**

| pid | %CPU | command |
|---|---|---|
| 5149 | **89.6** | Lark Helper (Renderer) |
| 56338 | 41.3 | CorpLink System Extension |
| 64460 | 40.6 | TencentMeeting |
| 18875 | 33.4 | Python |
| 97159 | 13.0 | `saih` — a seat |
| 400 | 11.3 | WindowServer |
| 76288 | 10.1 | Lark Helper (Renderer) |
| 18741 | 8.2 | Lark Helper (Renderer) |
| 74075 | 4.3 | Lark Helper (Renderer) |

**No `cargo`, no `node`, no `vite`, no `postgres` in the top ten.** The seat fleet's own heaviest process is `saih` at 13%. **So the saturation is from the user's desktop applications — Lark alone has three helpers in the top ten — and not from anything the coordination system is doing.** That distinction matters, because it means no seat can fix it and no seat caused it.

## The part that makes this an incident rather than a slowdown

**The seat sessions are failing too.** I read Nimbus's tmux pane directly:

```
spawn /bin/bash EAGAIN
```

**`EAGAIN` is the kernel refusing a fork because a resource limit was hit.** So it is not that processes are slow — it is that new processes cannot be created at all, intermittently. **A seat whose shell cannot spawn cannot run a gate, cannot build, cannot query the ledger, and cannot complete a dispatch.** Every seat is blocked simultaneously, which is the first time this week that has been true for a reason no seat controls.

**And I could not measure the full scope.** My attempts to read Flux's and Aegis's panes failed with the same fork error, so I know Nimbus is affected and I do not know about the others. **I am recording that as unknown rather than inferring it from one observation**, which is the discipline I have had to apply repeatedly.

## What this means for the standing-duty loop, stated plainly

**My standing duty is to drain dispatches and verify work. Both require forking.** With `EAGAIN` intermittent, a dispatch that arrives may not be executable, and a verification may fail for environmental reasons rather than real ones. **That is the dangerous case: a gate that is red because the machine could not fork looks exactly like a gate that is red because the code is wrong.** I have recorded that failure family several times this week in other positions, and this is its most total form — it can make every measurement meaningless at once.

**So the correct behaviour is to stop measuring rather than to record a measurement taken during a fork failure.** The `check-ledger-exists.sh` exit 128 is not evidence about the ledger; it is evidence about the machine. **I did not treat it as a finding about the ledger, and that is the one useful thing I did this round.**

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart anything, replace the sandbox, or write to the live database — partly by choice and partly because most of those require forking. I did not re-run the ledger check to get a number, because a number obtained through a failing fork would be exactly the artifact error I have been correcting all week.

## And it blocks the governance machinery too, which I measured after writing the above

**`git commit` failed.** The pre-commit hook could not run:

```
.git/hooks/pre-commit: fork: Resource temporarily unavailable   (x4)
```

**So this document is staged in the index and not committed.** The work is not lost — it is in the working tree and the index — but the act of recording it requires forking, and forking is what is failing. **A seat that cannot commit cannot deliver, which means the incident blocks the delivery mechanism as well as the work.**

**I also could not run the docs gate.** `scripts/verify-docs.sh` failed with the same fork error. I checked the document for §10 compliance by hand instead — `LC_ALL=C grep -n '[^ -~]'` returns 10 lines, every one of them an em-dash, an arrow, the status marker, or a quoted error string, **and zero CJK characters.** So the document satisfies the rule; it just cannot be certified by the gate that certifies it.

**That is the honest end state of this round: a finding that is real, measured, staged, and unverifiable by the tools that would normally verify it.** I am leaving it staged rather than forcing a commit with `--no-verify`, because bypassing a gate during an incident that is about gates failing is exactly the wrong reflex.

## What the operator needs to know

**This is a host condition, not a coordination defect.** The load is dominated by Lark (89.6% plus three more helpers), CorpLink and TencentMeeting. If the seats need to run, those are what to close — or the machine needs more capacity. **I have no authorization to close another user's applications and would not take that action unsupervised.**
