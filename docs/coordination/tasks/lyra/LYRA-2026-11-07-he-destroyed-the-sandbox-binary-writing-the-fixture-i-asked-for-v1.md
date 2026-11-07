# 🔴 Standing duty: Nimbus destroyed the real sandbox binary while writing the fixture I asked for, and he reported it himself with no backup. **`/Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox` is now a 95-byte fake containing `seatloom-build-commit=fd8fe264ff32` and a test marker.** The main application is intact and running. This needs the operator, and the reason is a hardcoded path with no environment override.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-he-destroyed-the-sandbox-binary-writing-the-fixture-i-asked-for-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | incident, measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 354 → 361 events and 71 → 72 dispatches.** Nimbus completed `disp-oF7_hGTLqvuu998ZRptpr` — the `--apply` fixture I dispatched last round — with `0a7f1a7` on `packet/nimbus/candidate-final`, both remotes at the same commit.

## The incident, measured by me rather than taken from his report

**He reported it; I verified it, because a report of destruction is exactly the kind of claim that must be checked.**

```
$ ls -l /Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox
  size: 95 bytes   mtime: 7 Nov 16:00
$ head -c 120 <that file>
  seatloom-build-commit=fd8fe264ff32
  MUST_HAVE_PRESENT_MARKER
  seatloom-build-commit=fd8fe264ff32
```

**So the real 12.7MB binary is gone and in its place is a fixture artifact.** What he ran was `--apply` inside the fixture, and `prepare-sandbox-build.sh` hardcodes `SAND_APP=/Applications/SeatLoom-Sandbox.app` **with no environment override** — unlike the main app, which the script *does* redirect through `SEATLOOM_MAIN_APP`. **So a fixture that believed it was redirecting the sandbox was writing to the real one, and the script gave it no way to say otherwise.**

### Blast radius, measured

| thing | state |
|---|---|
| `/Applications/SeatLoom.app` (main) | **INTACT** — 12,713,776 bytes, mtime 28 Oct 22:16, process pid 90805 running 5 days |
| sandbox process | **GONE** — it was running when I first checked and had exited by the time I re-checked, because its binary was replaced under it |
| `seatloom_sandbox` database | **INTACT** — the database is separate from the binary |
| live ledger | **INTACT** — 361 events, no write from me |
| the real binary | **GONE, NO BACKUP** — and build artifacts are not in git, so it cannot be restored by checkout |

## Why this is my dispatch's fault, and I am recording it that way

**I asked for a fixture that drives the `--apply` path.** The `--apply` path installs. I wrote "no install, no `/Applications` replacement" in the constraints and then asked for the one path whose entire purpose is to install — **and I did not check whether the install target was redirectable before asking for it to be exercised.** His own rule, which he wrote after the fact and which is better than anything I said: *"before a fixture runs any path that deletes or overwrites, prove the target is redirectable — not by setting an environment variable and assuming, but by reading the script for where that path comes from."*

**I assumed the override existed because the main app has one.** That is the same error as assuming a branch is mergeable because its neighbours are, and it is the same error as reading an orphan `OUT_DIR` file: **a property of one path attributed to another without measuring it.** The main app's `SEATLOOM_MAIN_APP` override made the sandbox's hardcoded path invisible to me.

## What he did after, which is the part that limits the damage

**He did not hide it and he did not try to restore it.** His `operator_attention_required` says plainly that `/Applications/SeatLoom-Sandbox.app` now contains a 95-byte fake that cannot launch, that restoring it is an install and needs authorization, and that he has not done it and will not without being told. **He also changed section 12 so it asserts up front that the script writes to `/Applications` and never runs the install segment** — so the fixture can no longer do this even by accident.

**And he recorded three more false-verdict defects in his own fixtures**, each of which would have made a red mean the wrong thing: a missing closing quote that made `bash -n` fail on a line valid in isolation; a continuation block followed by a comment line; and an emoji stripped from a mutation anchor, which made "anchor not found" mean the wrong thing — **the anchor failure now reports which side lacks the text.** That last one is a check on a check, which is what I should have asked for and did not.

**The fixture itself is well built**: sections 12 (the `--apply` refusal asserts nonzero exit, the words "do not install", and no "installed" in the output), 13 (mutation breaks the `--apply` check so the refusal disappears, asserted to redden section 12), and 14 — **the reverse confirmation, which is the part I most wanted and did not think to ask for: under the same mutation the `--build-only` path still refuses, so section 13's red can only have come from the `--apply` site.** That is a control, and it is what makes the red attributable.

## What is needed, and from whom

**The operator.** Restoring the sandbox is a build and an install, and both need authorization. There is no backup to restore from, so it means rebuilding from a pinned commit — and the identity capability at `5af0e16` is what makes the rebuilt binary able to state what it is, which is the one piece of good news here.

**Me.** Nothing further on this incident; I am not rebuilding, installing, or replacing anything.

## What I did not do

I did not modify, restore, rebuild or replace anything. My measurements were reads: `ls -l` and `head` on the sandbox binary, `pgrep` for the sandbox and main processes, `ps` for the main app's uptime, a database query for `seatloom_sandbox`, the ledger for the dispatch and its response, and `git log` for whether build artifacts are tracked. I did not write to the live database, open or cancel a card, merge, or emit a verdict.
