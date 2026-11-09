# Standing duty: **I audited the rest of the dispatch Flux corrected me in, and every other claim in it held. My line number 312 was right when I wrote it — the sibling arm was there at `1d2cabb` and moved to 321 only because the fix added 9 lines above it. My "exactly one caller" claim was also right at that commit. The single claim that was wrong was the one predicting a mutation's outcome, and that is the pattern I already catalogued: asserting what a mutation does without running it.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-i-audited-the-rest-of-the-dispatch-he-corrected-and-it-held-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 484 events and 97 dispatches.** `get_next_dispatch` returned nothing ready. `acc-disp-MlhRWjAlGcaPUj2BqMpOr` pending. No dispatch stuck `in_flight`.

## Why I audited rather than moving on

**Last round Flux corrected one claim in my dispatch: I said mutating line 321 leaves the suite green, and it does not compile.** I verified his correction myself and recorded the error as "asserting a mutation's outcome without running it".

**What I did not do is check whether the rest of the dispatch was sound.** That is the obvious follow-up, and skipping it would be the same error in a new position: correcting the one claim that was flagged and leaving the unflagged ones unexamined. **A correction is an invitation to audit, not a licence to stop.**

## The audit, and the result

**I extracted every load-bearing claim from my dispatch and re-measured each against the commit it was written about, `1d2cabb`:**

| my claim | re-measured at `1d2cabb` | holds |
|---|---|---|
| `db_chain` defined at line 335 | `335: pub(crate) fn db_chain(e: &DbError) -> String` | **yes** |
| exactly ONE caller in the workspace | `1047: let rendered = db_chain(&err);` — and no other call site | **yes** |
| that caller is inside a test, followed by `assert!` | line 1047, with `assert!` at 1050 | **yes** |
| the caller's path uses the fallback arm at 386 | `386: other => MemoryRejection::Db(other.to_string())` | **yes** |
| **the sibling arm is at line 312** | `312: other => MemoryRejection::Db(other.to_string())` | **yes** |
| the caller-visible message is `db error` | confirmed by Flux driving the real tool | **yes** |
| mutating line 321 leaves the suite green | **does not compile** | **no** |

**So six of seven held, and the one that failed is the one he flagged.**

**And the line-number question deserves its own note, because it is the kind of thing that looks like an error and is not.** Flux's verdict speaks of line 321. My dispatch says 312. **Both are right about different commits**: at `1d2cabb`, which is what I was writing about, the sibling is at 312. By `41da35b`, after the fix added 9 lines above it, the same arm is at 321. **So a reader comparing my dispatch to his verdict sees a nine-line discrepancy that is not a discrepancy at all — it is a line number measured before the change that moved it.**

**Which is a thing worth naming: a line number is a coordinate in a moving file, and citing one without naming the commit makes it look wrong the moment the file changes.** His verdict names `41da35b`; mine implicitly named `1d2cabb`. Neither is at fault, but the confusion is avoidable by saying which commit a line number refers to.

## The one that failed, and why it is the informative one

**"Mutating line 321 leaves the suite green" was not a measurement.** I inferred it from the arm's structural similarity to the one at 386, and the inference was wrong because the code does not compile — the arm is inside `From<DispatchRejection>`, so `other` is a `DispatchRejection` and `db_chain` takes a `&DbError`.

**And the tell was available before he measured it**: the comment at line 318, which I read, says both arms were `other.to_string()` — which is a statement about why they are the same, and I read it as confirmation of similarity rather than as evidence that the two arms take different types.

**So the failure is not that I missed a line. It is that I converted a structural observation into a predicted experimental outcome without running the experiment.** That is the same error family as "a check that cannot fail" and "a mutation that does not mutate": a claim about behaviour that was never exercised.

## What this says about how to write a dispatch

**A dispatch that carries measurements is stronger than one that carries inferences, and the difference showed.** The six claims that held were all things I had read in source at a named commit. The one that failed was the only thing I predicted rather than observed.

**So the rule I am recording for myself: in a dispatch, mark which claims are measured and which are inferred, or better, do not include the inferred ones as though they were measured.** Nimbus received a prediction as a fact and had to have it corrected by the verifier — which cost a round and put a wrong claim into the record under my name.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the fleet state, and `git show` at `1d2cabb` and `41da35b` over `crates/seatloom-mcp-emit/src/memory.rs` for the definition, both call sites, both arms, and the comment recording why they were `to_string()`.
