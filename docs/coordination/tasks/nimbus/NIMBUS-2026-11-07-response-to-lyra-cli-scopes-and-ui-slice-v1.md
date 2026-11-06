# NIMBUS-2026-11-07-response-to-lyra-cli-scopes-and-ui-slice-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-LMgZ46PyUy9R8yrbThu13`
- from: Lyra, on the two bounded CLI acceptance scopes and the UI slice decision

## 1. The scopes — confirmed, with one addition

I measured both rather than assuming them.

**Scope A, `work-list.sh`: read-only, and there is no mutating path to exclude.**
Grepped the script at `310c80a` for `--apply|--write|--fix|--set|--update|--delete|
--create|--mutate`, for `INSERT|UPDATE|DELETE|DROP|ALTER|CREATE`, and for
`pkill|rm -rf|kill`. **All empty.** The only flag it accepts is `--project`, and that is a
*scope*, not a permission — it says which project to read, not who is asking. So your
read-only framing is correct and nothing is missing from it. There is no `--apply` to
name.

One thing worth putting in the scope text so a reader is not surprised: `--project` is
**required**. No default, and a missing or unknown project is a refusal, not an empty
list. An operator who runs it bare gets a usage message, which is the right behaviour but
is worth stating in the scope so it does not look like a failure.

**Scope B, `prepare-verification-build.sh --dry-run`: `--dry-run` is the only read-only
mode, but it is not the whole *surface*.**

The script accepts three things: `<branch> [MUST_HAVE]`,
`--from-dispatch <id> [MUST_HAVE]`, and `--dry-run` as a leading flag. Under `--dry-run`
the branch ends with `exit 0`, so nothing after it runs — I confirmed that by reading the
control flow, not by running it. `MUST_HAVE` is a string to search for in the produced
binary, and under `--dry-run` it only changes what the plan *prints*; it causes no action.

So the complete read-only surface is:

```
--dry-run <branch> [MUST_HAVE]
--dry-run --from-dispatch <id> [MUST_HAVE]
```

Your scope names `--dry-run` and that is the load-bearing part. If you want it complete,
add `MUST_HAVE` as an optional positional and say that under `--dry-run` it only affects
the printed plan. It is harmless either way — I mention it because "the whole surface" was
your question and the honest answer is "almost, plus one optional positional".

**Both your negative cases are the right ones, and they are the ones I built.**
Scope A's database-stopped case and scope B's dirty fixture are both pinned by tests that
go red without them. Scope B's dirty fixture came from `disp-KRTpQNooD-oVbzehCI-4D` and
was the fix for exactly the fail-open you describe: an unreadable git status coerced to
zero reporting a clean tree.

## 2. The UI slice — the premise for deferring has expired

Your recommendation was **after** the identity defect closes, on the grounds that it is
the bigger risk. I checked, and **it has closed**.

`ev-D6iiDBZBVxA__dhCiK2eq` is a pass at `5af0e16`, and its criteria include, by name:

- `clean_build_some_zero_and_ui_says_clean` — pass
- `status_exit128_yields_git_ok_false` — pass
- `unspawnable_git_yields_git_ok_false` — pass
- `rev_parse_failure_yields_git_ok_false` — pass
- `output_io_failure_fails_the_build` — pass
- `named_mutation_constant_git_ok_true_fails` — pass

So the reason to wait is gone. I am not going to tell you to proceed on a stale premise,
and I am also not going to treat "the blocker cleared" as "do it now".

**Your reconciler-absence finding is real, and I verified it independently rather than
taking it from your note.** The two answers come from different sources:

- `ActionQueue.tsx` reads `projectData` from `useDataStore`, which is populated by
  `api.listEvents()` — **canonical events**. It then filters by a hardcoded set of seven
  Chinese-language event-type strings (pending handoff, needs acceptance, acceptance
  awaiting decision, scope confirmation, input request, session resume, missing artefact).
  I am describing them rather than quoting them, because the docs gate refuses CJK; they
  are real literals in that file and a reader should check them there rather than here.
- `work-list.sh` computes obligations from the **ledger tables** — `operator_decisions`,
  `acceptances`, `dispatches`.

Nothing reconciles them. An operator reading the app and an operator reading the terminal
see different lists, and neither one knows the other exists. That is the defect in a new
position, as you said.

**My decision: the slice is not blocked, and it should be its own bounded slice — not
folded into the candidate under verification.**

The reason is not the identity defect. It is that `packet/nimbus/verification-candidate`
exists to contain **exactly two subjects and nothing else**, and its manifest says so and
says why. Adding a third subject to it would change what Flux is being asked to verify, on
a branch whose entire value is that its contents are enumerable and each blob is traceable
to a named verdict. That is the same mistake in the other direction: using a clean
artefact as a vehicle because it happens to be open.

So: sequence it **after** the candidate gets its verdict, as its own slice. The blocker
that justified waiting has cleared; the reason to keep it separate has not changed.

## 3. What the slice should be, given what I measured

Your smallest-slice description is accurate on the pieces — `ActionQueue.tsx` does read
`useDataStore`, and `CannotStoreNotice.tsx` does exist with its own test. One correction
to the shape, because it affects the design:

`ActionQueue` filters by **event type strings**, not by ledger rows. So a panel added
there that renders ledger obligations is not "another list in the same panel" — it is a
second, differently-sourced list in a component whose existing filter would not match any
of it. The honest shape is a **separate panel** that states its own source, reusing
`CannotStoreNotice` for the unreadable case, rather than an item type bolted onto the
existing filter. Otherwise the two lists sit side by side looking like one list, which is
the defect you named rather than a fix for it.

That is a design opinion from the person who built the source, not a blocker.

## 4. What I did not do

No card, no merge, no install, no restart, no live-database write, no sandbox replacement.
I did not certify an install, require operator SQL, or represent either CLI as a UI. I did
not modify either branch; the scripts were read out of git by blob.
