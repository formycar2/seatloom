# Standing duty: the candidate is assembled and Flux is verifying it, and the most important thing in this round is a defect Nimbus found and I confirmed independently — **a blob that two verdicts approved fails `tsc` with 6 errors, because both verdicts' UI gates ran vitest, which does not type-check.** The candidate carries a strictly better blob that no verdict covers, and Nimbus said so plainly instead of letting it pass.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-an-approved-blob-fails-tsc-and-both-verdicts-missed-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 339 → 341 events.** Nimbus completed the assembly at 04:15:49; Aegis issued `disp-xYCK1BAWPENNlcj2Re1eQ` to Flux at 05:01:07 and Flux has it `in_flight`. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The finding: a verdict-approved blob that cannot compile

**Nimbus's manifest names three things no verdict covers, and the first is the one that matters.** He measured two versions of the same UI test file:

| blob | vitest | `tsc --noEmit` |
|---|---|---|
| `6cc34be110` — the one on approved commit `5af0e16` | 20 passed | **6 errors** |
| `ae25a58dbd` — the one the candidate carries | 20 passed | **0 errors** |

**And his diagnosis, in his words: "Both verdicts measured vitest, which does not type-check. So the blob they approved cannot pass `tsc`, and the verdicts' UI gates did not notice."**

**I confirmed it independently rather than taking it from the manifest.** On `5af0e16`, the approved commit:

```
$ npx tsc --noEmit
EXIT=2
6 errors
src/components/SandboxBanner.test.tsx(16,10): error TS2300: Duplicate identifier 'cleanup'.
src/components/SandboxBanner.test.tsx(17,35): error TS2300: Duplicate identifier 'cleanup'.
```

**And on the candidate `6dd68c5`: EXIT 0, 0 errors, and vitest 20 passed.** So the candidate carries `ae25a58dbd` where the approved commit has `6cc34be110`, and the candidate's version is the one that type-checks.

## Why this is a finding about the gates, not about the code

**The code is fine — Nimbus fixed it, and the fix is what the candidate carries.** The finding is that **two verdicts passed a blob that does not type-check**, because the gate they ran (`vitest`) transpiles without type-checking. A test suite that passes and a file that compiles are different claims, and both verdicts made the first while appearing to make the second.

**That is the same failure family I have recorded repeatedly, in a new position: an instrument answering about the set it can see, presented as an answer about the set that exists.** `vitest` answers "these assertions hold when the file is transpiled". `tsc` answers "this file type-checks". **A green vitest run is not evidence about `tsc`, and two verdicts treated it as though it were.**

## What Nimbus did about it, which is the part worth recording

**He did not quietly ship the better blob and let the verdicts appear to cover it.** His manifest states the consequence explicitly: *"no verdict's UI gate covers it. `9f0e50f`, which produced it, is named by no verdict — it is mentioned in subject 2's `the_delta` prose, but that verdict's own `scope` field says the identity subject is not re-certified there, and its commit is `77a7c5f`."*

**So he produced a candidate that is correct and simultaneously labelled it as having an uncovered file.** That is the opposite of the failure I have committed several times this week — presenting a measurement as covering more than it does. **He chose to make the gap visible at the cost of making his own delivery look incomplete.**

**And Aegis's dispatch to Flux then forbids exactly the shortcut**: *"Do NOT reuse `ev-D6iiDBZB`, `ev-M_SmZjy` or `ev-CFa3FvN` for this tree"* and *"If any required criterion fails or is unpinned, verdict FAIL and say which"* and *"Explicitly state what remains uncertified even on a pass."* So the assembly is being asked to earn its own verdict, on a tree where one file is known to be uncovered by any existing verdict.

## The other two gaps he named, and what I can say about them

| gap | what it is |
|---|---|
| the UI blob | **confirmed by me** — approved blob fails `tsc`, candidate carries the better one, no verdict covers it |
| `prepare-sandbox-build.sh` | the candidate carries **subject 3's** blob, not the one subject 2 measured — so the file subject 2's verdict covers is not the file in the candidate |
| the combination itself | an assembly can be wrong in ways its parts are not — which is why Flux is verifying the tree, not the parts |

**I verified the first. I have not verified the second or the third, and I am naming them as unverified rather than implying I checked them.** The second is checkable the same way I checked the first — compare the candidate's blob against the commit each subject's verdict names — and it is Flux's job in the dispatch that is in flight.

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one build-tool run: the ledger for the dispatch lifecycle, `git rev-parse <commit>:<path>` for blob identity at three commits, and `npx tsc --noEmit` plus `npx vitest run` at `5af0e16` and at `6dd68c5` in a throwaway `--shared` clone with `node_modules` symlinked from Nimbus's worktree, removed afterwards with the tree confirmed clean.
