# Standing duty: the sandbox is still destroyed and no dispatch covers its rebuild — because rebuilding is an install and an install is the operator's to authorize. **What I can do is measure the exact prerequisite, and it is smaller than the incident looks: the candidate has a `--build-only` path that builds and proves without installing, replacing, or launching anything.** So the operator's decision is one authorization, not a repair project.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-sandbox-rebuild-is-one-authorization-and-i-measured-the-prerequisite-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, operator-attention |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 361 events and 72 dispatches — unchanged since last round.** `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

**And the incident from last round is unchanged**, which I re-measured rather than assumed:

| thing | now |
|---|---|
| `/Applications/SeatLoom-Sandbox.app/.../seatloom-sandbox` | **still 95 bytes** — not restored |
| `/Applications/SeatLoom.app` | **INTACT** — 12,713,776 bytes, mtime 28 Oct 22:16 |
| main process | **pid 90805, up 5 days 5 hours** |
| `seatloom_sandbox` database | intact |

**No dispatch mentions rebuilding, restoring, or the 95-byte file.** The three dispatches that mention "sandbox" are all `done` or `cancelled` and predate the incident.

## Why nobody has picked it up, and why that is correct

**Restoring the sandbox is a build plus an install, and both write shared state.** My rule after 10-31 is that an unsupervised infrastructure action writing shared state is not a seat's to take, and a seat dispatching itself into one would be the same violation in a new costume. **So the correct state is exactly what exists: measured, reported, and waiting for one authorization.**

**The seats have context headroom** — Nimbus 35.1%, Flux 14.3%, Aegis 36.1% — so this is not a capacity problem. It is a boundary.

## What I measured, so the authorization is informed

**The question that matters is not "can it be rebuilt" but "what does rebuilding cost and what does it prove".**

| question | measured answer |
|---|---|
| does the candidate carry the identity capability? | **yes** — `build.rs` on `candidate-final` has 7 references to the capture and watch declarations |
| is there a path that builds without installing? | **yes** — `--build-only`, described in the script as *"build only + prove the build carries that work, no install, no replacement, no process"* |
| does `--build-only` conflict with `--apply`? | **no, and the conflict is refused** — line 126 refuses both together |
| what does `--build-only` require? | a branch that **some worktree has checked out** — line 218 refuses otherwise, and the refusal now names the detached case and the exact `git branch` command to fix it |
| is the binary's identity checkable afterwards? | **yes** — `MUST_HAVE` is wired on both paths, and the `--apply` path refuses to install when the marker is absent |

**So the rebuild is: check out the candidate branch in a worktree, run `--build-only` with a marker, and read that the built binary carries it.** That is a build and a proof. **The install step is separate and is the one that needs saying yes to.**

**And the one piece of good news, which is the reason this incident is recoverable at all**: the destroyed binary predates the identity capability — it was built 11-06 05:33 and could not state its own commit. **A rebuild from the candidate produces a binary that can.** So the replacement will be strictly better than what was lost, which is not true of most destruction.

## What I am not doing, stated so it is not mistaken for an oversight

**I am not dispatching the rebuild.** A dispatch to a seat to rebuild and install would be asking a seat to take the action I have been refusing to take myself, and the constraint I wrote into Nimbus's last dispatch — "no install, no `/Applications` replacement" — is the same constraint that binds me. **The correct move is to hand the operator a measured prerequisite and let him authorize, not to route around the boundary by delegation.**

**I am also not restoring from any source**, because there is none: build artifacts are not tracked in git, and I confirmed that with `git log --diff-filter=A` over `*.app/**`.

## What I did not do

I did not modify, restore, rebuild, install or replace anything. My measurements were reads: `ls -l` on both application binaries, `pgrep` and `ps` for the processes, a database check for `seatloom_sandbox`, the ledger for dispatches mentioning the sandbox, and `git show <branch>:<path>` for the candidate's `build.rs`, the `--build-only` semantics, the flag conflict refusal, and the branch-checkout requirement.
