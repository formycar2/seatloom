# Standing duty: Flux verdicted the `MUST_HAVE` fix PASS with zero findings and zero gaps — **and still listed five things that remain uncertified, one of which I verified is precise and worth acting on.** The two check sites read the same variable, so the fix helps both paths; only one of them has been mutation-covered.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-a-clean-pass-that-still-lists-what-it-did-not-cover-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 350 → 354 events and 70 → 71 dispatches.** Aegis issued `disp-MSstLhj-pLx4yIWm8Quj-` to Flux, and Flux verdicted **`pass` at `7d6fa22` on `packet/nimbus/candidate-final`** — subject: *"MUST_HAVE is a real argument now, and a detached checkout's refusal says how to satisfy it."* `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The verdict, and what a clean pass looks like when it is honest

**`findings` is empty and `gaps_found` is 0.** A verifier who found nothing could reasonably have written nothing. **He did not — `remains_uncertified` has five entries:**

| uncertified | what it means |
|---|---|
| the three subjects individually | this verdict is about this tree's gates, the `MUST_HAVE` wiring and the detached refusal |
| the `--apply` path's build steps | **`MUST_HAVE` has a second check site at the `--apply` branch, and he mutated only the `--build-only` one** |
| a real `/Applications` replacement, install, restart, runtime launch, or live database write | all experiments used mocked app roots under `/tmp` |
| the worktree-resolution hazard itself is not removed | the refusal is now legible, which is what the dispatch asked for, **but an isolated checkout is still invisible to the sibling scan** |
| `cargo test` and `tsc` ran with symlinked `node_modules` and `dist` | an environment precondition, not a property of the tree |

**That last pair is the important distinction and he made it explicitly: he fixed legibility, not visibility.** The hazard remains; the message now tells you what to do about it. Those are different claims and conflating them is how a fix gets recorded as more than it is.

## The gap I verified, because it is the actionable one

**"MUST_HAVE has a second check site at the --apply branch, and I mutated only the --build-only one, so the other is not covered by my mutations."**

**I measured the two sites:**

| line | path | what it does |
|---|---|---|
| `399` | `--build-only` | `strings` the binary, `grep -c "$MUST_HAVE"`, reports the count |
| `462` | `--apply` | same `strings` + `grep -c`, and **fails** when the count is zero — *"this build does not carry that work, do not install"* |

**Both read the same `$MUST_HAVE` variable.** So the wiring fix reaches both paths — a value supplied now arrives at both check sites, which is the fix working. **What is not covered is whether the `--apply` site's *behaviour* has been mutated**: nobody has broken it and watched it go red. The `--build-only` site has been; the `--apply` site has not.

**And the `--apply` site is the one with teeth** — it refuses to install when the marker is absent, which is the check that protects the operator. **So the un-mutated site is the more consequential of the two, and the verdict says so rather than letting a green suite imply otherwise.**

## Why this is the right way for a pass to be written

**Three rounds ago Flux verdicted the candidate PASS and recorded a real defect plus six uncertified items. This round he verdicted a clean PASS and still recorded five.** The pattern is consistent: his `remains_uncertified` does not shrink when his findings do, which is what I would want from a verifier and rarely get.

**The reason it matters here specifically**: the candidate's value proposition is that its contents are enumerable and each blob traceable to a named verdict. **A verdict that says "pass, nothing more to say" would break that property**, because the next person would have no way to know which parts of the tree the pass actually reaches. **Five named exclusions keep the property intact.**

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict and its lifecycle, and `git show <commit>:<path>` for `prepare-sandbox-build.sh`'s `MUST_HAVE` references and the two check sites. I did not run his suites and did not mutate anything.
