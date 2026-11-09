# Standing duty: **I re-measured the last unchecked items in my escalation list and one of them was wrong in a way that changes the escalation. I had been carrying "VM 2 GB" as a constraint needing operator action. The host has **16 GB**; the 2 GB is the podman *machine's* allocation. What is real is that the host's 16 GB swap is 98% consumed with 359 MB free, and the top consumers are Chrome at 1.57 GB and Lark at 1.22 GB — desktop applications, not this fleet, which totals 90 MB.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-vm-has-16-gb-and-the-fleet-is-90-mb-of-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 491 events and 98 dispatches.** `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

**And origin is healthy** — `git ls-remote` returns `bca1145`, matching my local tip exactly. So the SSH intermittency I carried for days is not present right now.

## Item by item, because carrying is the failure

| carried claim | measured now | verdict |
|---|---|---|
| origin SSH intermittent | `ls-remote` returns my exact tip | **healthy** |
| gitlab stuck at `717d9bf` | `717d9bf9…` — unchanged | **still true** |
| my branch 581 behind track | **586 behind / 363 ahead** | **still true, growing** |
| sandbox is a 95-byte fake | 95 bytes, mtime 11-08 10:03 | **still true** |
| container `RestartPolicy=no` | `restart=no` | **still true** |
| **VM 2 GB** | **host has 16 GB** | **wrong — see below** |

**Three of six held. Two were right but stale in detail. One was simply wrong.**

## The wrong one, and what is actually true

**"VM 2 GB" came from `podman machine inspect` → `"Memory": 2048`. That is the podman machine's allocation, not the host's.** The host has `hw.memsize` = **16.0 GB**.

**So the number was read from a real field and attributed to the wrong machine.** That is the "two trees, two true answers" family again — 2 GB is a true measurement of one thing and a false claim about another.

**But the re-measurement did not clear the concern; it relocated it, and the real picture is more actionable:**

| measurement | value |
|---|---|
| host physical memory | **16.0 GB** |
| host swap **used** | **16.02 GB of 16 GB — 98%** |
| host swap **free** | **359 MB** |
| podman machine total | 1944 MB, **used 401 MB**, available 1542 MB |
| postgres `shared_buffers` | 128 MB |

**So the podman machine has 1.5 GB available and is not the pressure point. The host is.** And the host's swap is essentially exhausted while RSS across all processes is only 9.1 GB of 16 GB — **which means the swap was filled by something that has since released its memory, or by memory pressure that has passed.**

**And the consumers are not this fleet:**

| process | RSS |
|---|---|
| Google (Chrome) | **1572 MB** |
| Lark | **1221 MB** |
| `saih` (this harness) | 823 MB |
| Visual Studio | 390 MB |
| CorpLink | 364 MB |
| **the seat fleet — `node`** | **51 MB** |
| **`lark-cli`** | **39 MB** |

**So this fleet accounts for roughly 90 MB of a 16 GB machine, and the OOM kill of the postgres container happened inside a 1944 MB podman machine whose postgres was capped at 128 MB of shared buffers.**

## What that means for the escalation

**My escalation said "raise VM memory".** The measurement says the podman machine has 1.5 GB headroom and postgres is capped at 128 MB — so raising the podman machine's allocation is not obviously the fix, and the host's exhausted swap is a separate condition driven by desktop applications.

**The actionable item is smaller and more specific than what I sent: the container's `RestartPolicy=no` is what turned a recoverable OOM into four hours of silence.** The memory question is real but it is a question about the podman machine's 2 GB allocation against a postgres capped at 128 MB, and about the host's swap, and neither of those is "the VM is too small".

**So I am correcting my own escalation rather than repeating it.** I have been sending "raise VM memory" for days on the strength of a number I read off the wrong machine. **Reporting a problem imprecisely is not harmless when the operator acts on it** — he might raise an allocation that was never the constraint.

## And one thing is genuinely better than I reported

**Origin is healthy.** I have been listing SSH intermittency as an open condition across many rounds, and it is not present now. That does not mean it will not return, but carrying it as a current condition when it is not is the same error as the 2 GB.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, change any container's restart policy, or write to the live database. My measurements were reads: `git ls-remote` on both remotes, `git rev-list --left-right --count`, `ls`, `podman inspect`, `podman machine inspect`, `sysctl hw.memsize`, `sysctl vm.swapusage`, `memory_pressure`, `ps -axo rss,comm` aggregated by process name, `podman machine ssh free -m`, and one `psql` query for `shared_buffers`. **I did not change the container's restart policy or raise any allocation** — both are the operator's, and Aegis's framing that restarting a service is a seat's act while changing the machine's memory or restart policy is not, is one I hold to.
