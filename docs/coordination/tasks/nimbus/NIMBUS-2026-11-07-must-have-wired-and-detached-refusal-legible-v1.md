# T3 — Nimbus — 2026-11-07 — MUST_HAVE is a real argument now, and a detached checkout's refusal says how to satisfy it

Dispatch: `disp-JaFOjUXsgouU8Ah-qoJ-j`
Branch: `packet/nimbus/candidate-final`
Commit: **`7d6fa22702ae3ce6528a9001d4d369804409e260`**
Lineage: on the same candidate, `6dd68c5` → `4023e56` (manifest) → `7d6fa22` (this fix)

## The defect, reproduced before fixing

Flux's independent verdict `ev-TWL0d7liGTtboRRnO7B34` passes at `6dd68c5` and records a
real defect. I reproduced it before touching anything.

**By reading the parser:** the catch-all `*)` arm did `[ -z "$BRANCH" ] && BRANCH="$1"; shift`.
Only the *first* positional reached `$BRANCH`; every later one was shifted away and
discarded. `grep -n 'MUST_HAVE='` returned exactly one hit — line 70, the initialiser.
**`MUST_HAVE` was never assigned from the arguments.**

**Empirically:** `prepare-sandbox-build.sh packet/nimbus/candidate-final SOME_UNIQUE_STRING`
exited 0 and printed no mention of the string at all.

**And the worse half, which the reading alone would have missed:** the script's own
suggested commands spliced `$MUST_HAVE` back in (`${MUST_HAVE:+ $MUST_HAVE}`), so the
operator was shown a command that looked like it would run the check — and following it
discarded the value again. An advisory that cannot be satisfied by its own instruction.

A check that cannot run is worse than no check: it tells the operator that section
verified something.

## Fix 1 — MUST_HAVE is a real argument

- The second positional is really captured, positionally counted, and a **third**
  positional is **refused** rather than dropped. Silently dropping an argument is the
  exact shape of this bug, so the opposite had to be pinned too.
- A `--must-have <value>` / `--must-have=<value>` flag form was added, so the value can be
  given without depending on position.
- A value containing a space is refused, because a space-bearing string passed unquoted
  would be split into two positionals and silently become the third.

Measured against a real build from the clean committed tree:

```
bash scripts/prepare-sandbox-build.sh --build-only packet/nimbus/candidate-final seatloom-build-commit=
  FOUND 'seatloom-build-commit=' in the binary (1 occurrence)
  identity check: MATCH
  rc=0
```

And the failing direction:

```
bash scripts/prepare-sandbox-build.sh --build-only --allow-dirty packet/nimbus/candidate-final THIS_STRING_IS_NOT_IN_THE_BINARY_ZZZ
  NOT FOUND 'THIS_STRING_IS_NOT_IN_THE_BINARY_ZZZ' in the binary -- this build does not carry that work
  rc=1
```

## Fix 2 — the detached-checkout refusal says how to satisfy it

The refusal printed `'<branch>' is not any worktree's currently checked-out branch`, then
listed the worktree — but never said *why* it did not match, nor that a detached checkout
needs a matching branch. Flux hit this verifying `6dd68c5` in an isolated clone and had no
way forward.

**A refusal that cannot tell you how to satisfy it only announces failure.**

It now names the detached case and gives the command:

```
     iso-wt -- detached checkout, and its tree is exactly at the commit you want (see below)

     iso-wt is a DETACHED checkout, but its tree is exactly at the commit you want.
     A branch name pointing at this commit is all that is missing, for example:
       git -C /tmp/iso-wt branch <a-branch-name> 6dd68c5
     Then pass that branch name in (this script will not checkout for you, and will
     never quietly build a different tree).
```

It still refuses, and it still builds nothing. The change is only that the refusal is now
actionable.

**A contradiction found while testing:** the listing *also* printed "just pass this branch
name" for that same detached worktree — where the name is literally `HEAD`, and passing
`HEAD` hits the identical refusal again. Two instructions, opposite meanings, printed
together. The detached case now says `see the fix below` instead.

## Fixture sections 8–11 (new)

| Section | Pins |
|---|---|
| 8 | `MUST_HAVE` in the binary → passes and says so with a count; absent → nonzero and says why; not supplied → still succeeds **and states it did not verify**, rather than pretending |
| 9 | a third positional is **refused**, naming which positional — not silently dropped |
| 10 | mutation drops the value again → section 8 must go red |
| 11 | the detached refusal names the cause and gives the branch command |

Section 10's mutation keeps the `case` structure and changes only the second arm to `:`,
then self-checks with `bash -n`.

## Three fixture defects, each of which produced a false verdict

**1. `run_build` appended a hardcoded `MUST_HAVE_ME` tail.** Section 8's marker became
the *second* positional and `MUST_HAVE_ME` the *third* — correctly refused by section 9's
new guard, so every assertion in section 8 was red **for the helper's reason rather than
the code's**. A fixture that interferes with the very argument it is testing measures
nothing about that argument.

**2. Section 10's mutation ran *after* `reset_fixture`.** `reset_fixture` contains
`git checkout -- .`, which erased the mutation before it executed. This is the identical
defect I had already found and fixed for sections 5 and 6 — reintroduced by copying the
old section shape. A probe (`grep -q '2) : ;;' "$MUT"`) now fails loudly if the mutation
did not land, instead of the failure being inferred backwards from "the gate guards
nothing".

**3. The first mutation left a dangling `;;`.** Bash rejected the script outright, so
section 8's red would have come from a *syntax error* rather than from the value being
dropped. A mutation that breaks the program instead of restoring the bug proves nothing.
It now preserves the `case` structure and verifies `bash -n` before being accepted.

## Gates

| Gate | Result |
|---|---|
| `scripts/test-build-failure.sh` | **exit 0, 29 checks** (was 18), three consecutive identical runs |
| `scripts/test-replace-sandbox-failclosed.sh` | exit 0 |
| `scripts/test-worktree-root.sh` | exit 0 |
| `cargo test --workspace` | **387 passed / 0 failed** |
| `cargo build --workspace` | 0 errors (1 pre-existing warning, `state.rs:18`, track-owned) |
| `bash -n` on both touched scripts | clean |
| Side effects | **none** — main app pid 90805 untouched, sandbox mtime unchanged at 6 Nov 05:33:31, zero backups, zero sandbox processes |

## Scope discipline

The dispatch said to keep the other two gaps out of scope unless required to make the
`MUST_HAVE` fix verifiable. They are not required: `MUST_HAVE` is verifiable on its own
(sections 8–10), and the detached refusal is verifiable by grep plus the isolated-clone
run I performed. So no unrelated work was folded in.

## Not done

- No install, no `/Applications` replacement, no restart, no live-database write, no card,
  no track merge.
- No Flux verification of `7d6fa22`. `ev-TWL0d7liGTtboRRnO7B34` verifies `6dd68c5`; this
  is a later commit on the same lineage and needs its own.
- The `--must-have` flag form is covered by the same assertions as the positional form
  (they write the same variable), but no separate fixture drives the flag spelling.
