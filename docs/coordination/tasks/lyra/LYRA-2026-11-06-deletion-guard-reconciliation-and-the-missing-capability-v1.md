# Coordination for `disp-JLzt98The2hpuaWBB-y9d` — the three build facts reconciled independently, a safe fixture design that needs no ledger writes and no terminal-card reuse, and one concrete missing product capability to report rather than paper over. **The running sandbox cannot state what it is, and nothing in the supported lifecycle can make it.** The process claims `1cee50c`, the repository it points at is at `93ef3d3`, and the branch tip is `bc4c0a4` — three commits. The binary embeds no hash; `strings` finds only the environment variable *names*. **So the positive case this card exists to demonstrate is not checkable today, and the reason is a product gap, not a process gap.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-deletion-guard-reconciliation-and-the-missing-capability-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | coordination, measured, sandbox-identity, product-gap |

## The three build facts, each verified independently

**The dispatch named three things and warned that differing build versus gate-repo HEAD is not alone proof the binary
is wrong. Measured, and I did not accept any of the three from the dispatch's text.**

| claim | my measurement |
|---|---|
| narrow source branch `packet/nimbus/deletion-guard-only` @ `ac3d1f3` | **true** — the branch has `ac3d1f3` (the guard) and `86a65a5` (its note) on top of `track` |
| the sandbox process claims `1cee50c` | **true** — pid 36011 carries `SEATLOOM_SANDBOX_COMMIT=1cee50c`, `SEATLOOM_SANDBOX_BRANCH=packet/nimbus/deletion-guard-accept`, `SEATLOOM_REPO_ROOT=/tmp/sbclick` |
| gate repo `/tmp/sbclick` HEAD `93ef3d3` | **true** — checked out on `track/infra-foundation` at `93ef3d3` |

**And the ancestry, which is what makes the inconsistency real rather than cosmetic:**

```
93ef3d3  05:03:44   (an ancestor of 1cee50c)
1cee50c  05:23:35   (what the process claims)
bc4c0a4  05:36:55   (the branch tip it names)
ac3d1f3  06:06:34   (the branch the card should name)
binary   05:33:31   (mtime)
```

**So the binary's mtime is consistent with a build from `1cee50c`** — after it, before `bc4c0a4` — which is the one
piece of evidence pointing at a specific commit. **And a mtime is not an identity.** Measured: `strings` on the binary
finds `SEATLOOM_SANDBOX_BRANCH`, `SEATLOOM_SANDBOX_COMMIT`, `SEATLOOM_SANDBOX_BUILT` and `SEATLOOM_SANDBOX_DIRTY` as
*names*, and six hex-looking strings, none of which is a commit hash of any of the four.

**So I can say what the process claims, what the repository holds, and what the branch holds — and I cannot say what
the binary contains.** That is exactly Flux's conclusion and I reached it independently.

## The fixture design, safe by construction

**The dispatch asked for positive and negative scratch fixtures through the legitimate lifecycle, with no raw ledger
writes, no terminal-card reuse, and no operator shell or destructive setup. Here is the design, and its precondition.**

### Precondition, and it is the only blocker

**A card must exist.** `request_acceptance` is the supported entry point, and its first guard is that the dispatch is
`done` **and** `target_seat_id` is the caller — measured at `dispatch.rs:1894-1900`. **So only Nimbus can open it.** I
cannot, and neither can Flux, and that is the correct boundary rather than an obstacle.

### The positive case — a safe merge, accepted

| | |
|---|---|
| fixture | a scratch branch containing an **additive** change — a new file, no deletions |
| how it is created | `git checkout -b <scratch> track/infra-foundation`, add a file, commit, push |
| how the card names it | a **new** `request_acceptance` with `branch=<scratch>`, `base=track/infra-foundation`, `how_to_verify` naming the guard's behaviour, and `flux_verdict_ref` naming a verdict |
| what the operator does | clicks Accept in the sandbox |
| what success looks like | `merge_commit` fills in, the card becomes `accepted`, and the additive file is on `track` |
| what proves the guard did not over-refuse | the merge happened at all — a guard that refused every merge would fail here |

### The negative case — a destructive merge, refused

**This is the one the previous plan got wrong, and the wrongness was structural: it told the operator to repoint the
*existing* card at a destructive branch and click Accept. A card is terminal once accepted, and editing the ledger row
by hand is not an acceptance.**

**The safe version needs a second card, not a reused one:**

| | |
|---|---|
| fixture | a scratch branch that **deletes a file tracked on the base** — and a second that deletes the one ruled-exempt path, to prove the exemption still works |
| how it is created | same as above, `git rm <base-tracked file>` |
| how it is named | a **separate** `request_acceptance` with its own id, its own verdict reference, and a `how_to_verify` that says the expected outcome is a **refusal** naming the paths, the branch and the base |
| what the operator does | clicks Accept and **expects to be refused** |
| what success looks like | the click fails, the message names the deleted path and the branch, and **no merge commit appears** |
| what proves the guard bit | the refusal, plus `git log` showing no new merge |

**The exemption fixture is the third and it is not optional**: a branch that deletes only `ui/src/app-v2/AppV2.tsx`,
the one path the operator ruled deletable, must **succeed**. Without it, a guard that refuses everything would pass the
negative case.

### What none of this requires

**No raw ledger writes.** Both fixtures are cards created by `request_acceptance`, which is the supported tool.
**No terminal-card reuse.** The cancelled `acc-disp-AbQ4m1TKQZGLMeZ4bhwjy` stays cancelled and nothing points at it
again; the new cards get new identities from the tool, and the cancelled id is not recycled. **No operator shell work.**
Creating the scratch branches is git, and it is a seat's; the operator's only acts are the two clicks, one expecting
success and one expecting refusal.

## The missing product capability, which is the honest report the dispatch asked for

**The dispatch said: if there is no supported safe lifecycle, report the concrete missing product capability instead of
pretending the operator can test it. There is a supported lifecycle for the fixtures — and there is not one for the
thing the fixtures are supposed to demonstrate.**

**A sandbox build cannot state what it is.** Measured: the binary embeds no commit hash, and its identity comes from
environment variables set at launch. So the only evidence that a sandbox contains a given commit is a build timestamp,
which is not evidence.

**What is missing, concretely**: `SEATLOOM_SANDBOX_BUILT` exists as a variable name in the binary but nothing sets it,
and there is no `--version` or banner surface that prints the commit the binary was built from. **The capability is a
build-time embedded commit, printed by the sandbox banner** — which is exactly what `RuntimeIdentityDto.uncommitted`
was wired for on 11-04, extended one field.

**Until that exists, every runtime claim about the sandbox is unfalsifiable**, including the positive case: an operator
who clicks Accept and sees a merge cannot tell whether the guard refused for the right reason or the merge happened
for the right reason. **I am reporting that as the blocker rather than describing a plan the operator cannot execute
and verify.**

## What I am asking Nimbus for, precisely

**Not the merge, not the card, not the sandbox rebuild.** Three things, all his:

1. **Open a card on `ac3d1f3`**, not `bc4c0a4` — measured, `ac3d1f3` is one commit past `track` with five files, the
   guard and its proof and the shared exemption helper and nothing else, while `bc4c0a4` pulls in Aegis's
   `check-unclaimed-work` rework and two sandbox-script commits that are not the deletion guard.
2. **Write the plan so it reads the commit from the card, never a hard-coded hash.** The cancelled card's plan named
   `1cee50c`, which is neither the branch tip nor what the sandbox is on — a plan that names a bare hash is stale the
   moment the branch moves.
3. **Land the identity capability** — a build-embedded commit, printed by the banner — so the positive case becomes
   checkable. This is the smallest slice that makes the acceptance mean something.

**And what I am asking the operator for**: a decision on whether the current sandbox should be replaced. I found no
authorisation for it, and replacing it deletes a running application, so it is his. **My recommendation is to rebuild
only after the identity capability lands**, because a rebuild without it produces another sandbox that cannot state
what it is.

## What I did not do

I did not edit any code or script, open, cancel, or re-point any card, merge, push, restart any application, or write
to the live database. I did not rebuild or replace the sandbox, and I found no authorisation to. My measurements were
reads: Flux's blocker report in full, the three build facts against the process environment, the repository and the
branches, the ancestry and timestamps of all four commits, `strings` on the sandbox binary for an embedded hash, the
branch's diff and containment, `dispatch.rs:1894-1900` for who may open a card, and the ledger for the cancelled card
and the operator's last decision. The ledger check is exit 0, `seats` is 10, and `canonical_events` is **120**.
