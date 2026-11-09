# Standing duty: **I verified the one item I called the only real blocker, and it is real and it is ready. The sandbox is still a 95-byte ASCII text file, and the containment gate that would protect its rebuild is present and load-bearing on the branch the rebuild would use: driven with `--apply` and no environment overrides, it refuses with exit 1, names the protected path, and offers the three safe alternatives. So the rebuild is a single authorisation away, and the fixture that destroyed the binary five times cannot do it again.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-only-real-blocker-is-real-and-the-gate-protecting-it-fires-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## Why I checked this one

**Last round I reduced my escalation list from five recurring items to one real blocker, on the grounds that the other four blocked nobody. That left the sandbox standing alone as the thing genuinely preventing work.**

**A blocker that has been on my list for days deserves the same treatment I gave the others: is it still true, and is it still blocked?** Both answers are yes, and the second one is better news than I had been reporting.

## The item, re-measured

| check | measurement |
|---|---|
| the binary | `/Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox` |
| size | **95 bytes** |
| type | **ASCII text** — `file` says so |
| content | `seatloom-build-commit=314ab7e6e1ed`, `MUST_HAVE_PRESENT_MARKER` |
| mtime | **8 Nov 10:03** — unchanged since the fifth destruction |
| md5 | `7c0a6e2a…` — unchanged |
| backup | **none** |
| main app | md5 `28753f61…`, untouched across all five incidents |

**So the item is exactly what I have been reporting: a fake binary with no backup, needing an authorised rebuild and install.**

## And the rebuild is now protected, which I had not verified

**The containment fix is on `packet/nimbus/candidate-final`, the branch the rebuild would use.** I read the gate and then drove it:

| step | result |
|---|---|
| `prepare-sandbox-build.sh track/infra-foundation --apply` | stopped earlier, at the tree-mismatch guard — *also* correct |
| `prepare-sandbox-build.sh packet/nimbus/candidate-final --apply` | **reached the gate**, refused, **exit 1** |
| the refusal | "拒绝在受保护路径上做破坏性动作:受保护的是 `/Applications/SeatLoom-Sandbox.app`,而目标正是它" |
| what it names | that the step would `rm -rf` the sandbox, and the 2026-11-07 incident by name |
| the three alternatives it offers | `--build-only` to verify a build; `scripts/replace-sandbox.sh` which backs up first; `SEATLOOM_ALLOW_REAL_SANDBOX=1` as explicit confirmation |

**And the gate is the version Flux verdicted at `52cdb8b`: both paths canonicalised through `readlink -f` before comparison, two independent conditions (target under the real path, or target under the protected path), and a refusal that fails closed.**

**Both real applications are byte-identical before and after my probe** — sandbox still 95 bytes md5 `7c0a6e2a…`, main app md5 `28753f61…`. The worktree I used was removed.

## What that changes about the escalation

**I had been reporting "the sandbox needs an authorised rebuild" alongside, for many rounds, "and the fixture that destroyed it five times is still a hazard".** The second half is no longer true. **The hazard is closed and verified at the call site, and I have now confirmed it by driving the destructive path and watching it refuse.**

**So the operator's decision is smaller than I had been implying.** It is not "authorise a rebuild and accept that the fixture may destroy it again" — it is "authorise a rebuild, which is now protected by a gate that refuses by default and offers a backup-first alternative". **The refusal even names `scripts/replace-sandbox.sh`, which backs up before replacing, as the right path for a real swap.**

**And there is a verification path that does not require the install at all**: `--build-only` builds without installing, without replacing, and without starting a process. **So the operator can authorise a build and confirm it produces a real binary before authorising anything that touches `/Applications`.** That is two small decisions instead of one large one, and I had not offered it.

## The pattern, one more time

**This is the fourth item this week where the correct action was to re-measure rather than re-report.** The docs gate was green. The VM was 16 GB. The OOM was 13 minutes. The gitlab gap loses nothing. And now the sandbox hazard is closed.

**In every case the original measurement was true and the escalation had outlived it.** The discipline that found all five is the same one command: ask whether the thing I am about to report is still the thing I measured.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not set `SEATLOOM_ALLOW_REAL_SANDBOX=1`**, which is the one thing that would have made the destructive path reach the real target — and I ran the probe in an isolated worktree with the target left at its default, which is exactly the configuration the gate is designed to refuse. My measurements were reads plus that one gated invocation: `ls`, `file`, `head`, `md5`, `git cat-file`/`git show` for the script, `bash` on the script in an isolated worktree, and `git worktree remove` afterwards.
