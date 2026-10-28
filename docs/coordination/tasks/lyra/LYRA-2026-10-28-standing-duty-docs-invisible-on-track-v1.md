# Standing-duty finding: my five documents have no operational effect until they land on track

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-standing-duty-docs-invisible-on-track-v1 |
| status | open |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| tags | standing-duty, governance, waker, delivery, blocking |

## Facts — measured, with the command that shows each

**1. The waker nudges Lyra ten times and the other five seats zero times.**

```
$ grep -c 'standing-duty document' /tmp/seatloom_waker.log
10
$ grep 'standing-duty document' /tmp/seatloom_waker.log | sed 's/.*seat seat-/seat-/;s/ .*//' | sort -u
seat-lyra-001
```

The log is live — its mtime was `23:53:48` against a clock reading `23:54:47` at the moment of
measurement.

**2. The mechanism works. It is looking in the wrong place for the other five.**

```
$ grep -n 'tasks_dir' crates/seatloom-mcp-emit/src/bin/waker.rs
112:  const DEFAULT_TASKS_DIR: &str = "docs/coordination/tasks";
172:              tasks_dir: std::env::var("SEATLOOM_TASKS_DIR")
173:                  .unwrap_or_else(|_| DEFAULT_TASKS_DIR.to_string()),
```

That is a **relative** path, so it resolves against the waker's working directory:

```
$ lsof -p 68013 | grep -i cwd
seatloom- 68013 jyxc-dz-0100609 cwd DIR ... /Users/.../GitHub/seatloom
```

That is the **primary checkout**, and it is on:

```
$ git rev-parse --abbrev-ref HEAD   # in the primary checkout
track/infra-foundation
```

**3. Which is where my five documents are not.**

```
  mira: on_track=0 on_my_branch=1
  nimbus: on_track=0 on_my_branch=1
  flux: on_track=0 on_my_branch=1
  onyx: on_track=0 on_my_branch=1
  vela: on_track=0 on_my_branch=1
```

So: `packet/lyra/milestone-summary` carries the documents that would give five seats a standing
duty, and the only process that would ever call them is reading a tree that does not contain them.

## Why this is the more important half of last round's work

Last round I delivered `disp-8ekIOxWQQE852HwLwCX8y` and reported it complete, and by its literal
terms it was: five documents written, every check proved able to fail, gate green, pushed to both
remotes. **Operationally, nothing changed for those five seats.** The waker was running the whole
time, on the whole tree, and never saw them.

This is the same defect I wrote a standing check for in Nimbus's document last round — *nothing you
verified is stranded off track* — and I then committed it in the same hour. The check was right; the
seat it was written for was me.

**A governance artifact is not delivered when it is pushed to a branch. It is delivered when the
process that consumes it can read it.** Those are different events, and last round I reported the
first as if it were the second.

## What I did and did not do

I did **not** merge my branch into track. That is the operator's call, not mine, and
`COORDINATION_RULES.md` is explicit that merging another seat's branch into the integration line
is an operator action. I did not move files into the primary checkout either — that is the same act
with a different name.

## What this needs

1. **The operator (or Aegis, with the operator's acceptance) merges `packet/lyra/milestone-summary`
   into `track/infra-foundation`.** That single act is what makes the five standing duties real.
   Until it happens, the fleet has one seat with standing duties, not six.
2. **Or the waker is pointed at a tree that has them** (`SEATLOOM_TASKS_DIR`), which would be the
   wrong fix — it would make the waker read a seat's packet branch, which is not the integration
   line and must not be.
3. **The waker's relative `tasks_dir` is itself a latent trap.** Any future standing-duty document
   has the same silent failure mode, and the failure is invisible: the waker logs nothing about a
   seat it skips. A seat that has no heartbeat document is skipped in silence, which is exactly the
   behaviour that made five seats go quiet for a week and looked like a scheduling problem.

## A second observation, offered as fact and not yet a decision

The waker's log has 26 lines for a process that has been running since 19:05 and sweeps every 90s —
roughly 192 sweeps. The log lines carry no timestamps, so I cannot tell whether it was restarted,
rotated, or is genuinely this sparse. I am recording that as **unmeasured** rather than guessing at
it, because this project has already paid for "looked right, was actually a guess".
