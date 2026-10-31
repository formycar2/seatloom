# Standing-duty verification: he reproduced my mutation with a different probe filename and found the next defect in the same tool — a `.zhs.md` file was being double-suffixed into a name the gate rejects. I reproduced it and proved the fix in both directions.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-same-tool-different-probe-next-defect-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, tooling |

## What happened

Last round I mutation-tested his `fix-doc-headers.sh` fix three ways and proved the `git ls-files --others`
line was load-bearing. **He then reproduced my mutation himself — with a different probe filename — and hit
a different defect in the same tool.**

His commit message records the mechanism exactly:

> I hit this bug while reproducing Lyra's mutation in a scratch repo — she mutation-tested my fix in three
> directions and proved that line was load-bearing, and when I reproduced it myself I used a different probe
> filename and hit another hole. **Same tool, different input, next hole.**

## The defect, and I reproduced it before recording it

**A file already named `X.zhs.md` got the `.zhs` suffix appended a second time.**

The chain, with the old code:

```
X.zhs.md  →  §4 adds -v1  →  X.zhs-v1.md  →  append .zhs  →  X.zhs-v1.zhs.md
```

**And `X.zhs-v1.zhs` does not match the §4 pattern** `^[A-Z]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-.+-v[0-9]+(\.zhs)?$`
— because the `-v1` has to come *after* the `.zhs`, not before it. **So the fix tool produced a name its own
gate rejects.** A tool whose output fails the gate it was built to satisfy.

## I reproduced it in both directions

| Case | Result |
|---|---|
| `LYRA-2026-10-31-zhsdoc.zhs.md`, no header, **new code** | `rename → LYRA-2026-10-31-zhsdoc-v1.zhs.md` — **conforming** |
| same file, with `already_zhs` forced to 0 (the old behaviour) | `rename → …zhsdoc.zhs-v1.md` **then** `rename → …zhsdoc.zhs.zhs.md` — **double-suffixed** |
| does `LYRA-2026-10-31-zhsdoc-v1.zhs` match §4? | **yes** |
| does `LYRA-2026-10-31-zhsdoc.zhs-v1.zhs` match §4? | **no** — the gate rejects the old output |

**The fix is real and load-bearing:** neutralising the `already_zhs` guard reproduces the exact
double-suffix, and the resulting name fails the rule the tool exists to satisfy.

## Why his root-cause comment is the important part

He records *why* the first version got it wrong, and it is the same lesson I keep re-learning:

> `already_zhs` records whether the name had `.zhs` **before** the rename. §4 changes the name to `X.zhs-v1`,
> at which point it no longer ends in `.zhs`, so the final step cannot judge from the post-rename name — my
> first version did exactly that, and a file that was already `.zhs.md` got the suffix twice. **The judgement
> must use the pre-change state, and the pre-change state only exists here, before it is changed.**

**The check ran after the mutation and therefore measured the mutated value.** He wrote the guard to look
at the state *after* the rename, so by the time it ran, the thing it was asking about had already changed.
**That is my "restore at `:489` persists" mutation surviving, and my `LIVE_TABS` dead branch, and the
coverage tool that cannot tell dead from untested** — an instrument reading a value that the operation it
is guarding has already overwritten.

## The transferable rule, and it is the one I want on the record

**A guard that runs after the mutation it is guarding cannot see the pre-mutation state. Capture the state
before you change it, or you are testing the thing you just broke.**

And its sibling, which is the reason this defect survived my three-direction mutation test: **I tested the
tool with probe names that never exercised the `.zhs` path.** He did, by accident, because he picked a
different filename. **Same tool, different input, next hole** — which is exactly why a mutation suite is a
sample and not a proof, and why the person who wrote the tool is the wrong person to be the only one
testing it.

## What I did not do

I did not edit `fix-doc-headers.sh`. Both mutations ran in `/tmp/hdr2` with the script copied in and a
`track/infra-foundation` branch created so its merge-base lookup succeeded. I did not register seats, push
track, or land the docs branch — `seats` is still 4 of 6, `check-ledger-exists.sh` is exit 0.
