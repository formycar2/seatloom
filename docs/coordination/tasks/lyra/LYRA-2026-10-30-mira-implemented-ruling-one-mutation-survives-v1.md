# Standing-duty verification: Mira implemented the ruling in 26 minutes, 372/372 green, and one mutation survives — the restore exception

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-mira-implemented-ruling-one-mutation-survives-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, verification, mutation, surviving-mutation, closure |

## What she delivered

`073c480` on `packet/mira/knowledge-panel`, 26 minutes after the dispatch was delivered. **Dispatch
`disp-EVFMbz9OxT2gyoFlYfzBE` is `done`.** Her worktree is clean.

| The ruling said | What she did | Verified |
|---|---|---|
| Remove the effect-driven write | the whole write effect (`updateProjectUIState` + `setActiveProjectId`) removed | ✅ |
| One wrapper `chooseTab`, not eight call-site edits | `chooseTab` at `App.tsx:536`, `setActiveTab` + `updateProjectUIState` in one place | ✅ |
| Every operator-driven path goes through it | 7 `chooseTab` call sites; `updateProjectUIState` is called **once**, at line 540, inside the wrapper | ✅ |
| The restore at `:489` stays bare | still `setActiveTab(savedState.activeTab as Tab)` directly | ✅ |
| Do not spread it | the comment at line 530 says why, in the code | ✅ |

**Tests: 12/12 in her contract file, 372/372 across the whole UI suite, 31 files.** Track's line is the one
that is still red — that is a merge question, not a code question.

## I mutation-tested it myself rather than accepting a green suite

**Mutation 1 — the one the dispatch named.** `chooseTab` does not persist:

```
× an operator-driven tab change IS persisted — the wrapper is not decorative
  Tests  1 failed | 11 passed (12)
```

**Caught, by the test she wrote specifically for it.** That test is the difference between a wrapper and a
decoration, and she wrote it because the dispatch told her to. It works.

**Mutation 2 — the direction nobody thought to test.** The restore at `:489` persists:

```
  Tests  12 passed (12)
```

**It survives.** Making the cold-start restore write to disk produces no failure. I restored from backup
and confirmed 12/12 again, and her worktree is clean — `git status --short` is empty.

## Why mutation 2 surviving is the finding, and it is the old finding

**The `:489` exception is the entire point of the ruling and nothing tests it.** The ruling says the restore
must not persist, because that is the exact frame where the operator's saved tab gets overwritten by the
`useState` default. She implemented the exception correctly — I read it — and **the suite cannot tell the
difference between implementing it and not implementing it.**

That is the fourth surviving mutation I have found this week, and every one has been in the same direction:

| Mutation | What it removes | Survived? |
|---|---|---|
| Mira's cold-start gate made permanent | the switch-away-then-back case | **yes** |
| `chooseTab` does not persist | the wrapper's whole purpose | no — caught |
| **the `:489` restore persists** | **the exception the ruling exists for** | **yes** |
| 023's DELETE filter | NULL-path groups, silently | yes, until fixed |

**A green suite is not evidence of correctness. It is evidence that the tests you wrote pass.** The
surviving mutation is always the assertion nobody thought to write, and here it is the one assertion the
ruling was built around.

## What I am asking for, and it is small

**One test: make the restore persist, and confirm something goes red.** Concretely — cold-start into a
project whose saved tab is `artifacts`, and assert that the *write path* was never called on that frame. The
current tests assert the tab is restored; none asserts that restoring it did not write.

That is the assertion that would have caught her round-6 pollution when it first appeared, and it is the
reason her earlier mutation survived for ten rounds. **It is the cheapest test in the file and it is
missing.**

## What is still owed, unchanged

**No acceptance card.** `disp-E06O6IAaWefF-BOW-CNJd` still has no verdict naming it, and the guard is
correct to refuse one about a different dispatch.

**Real-machine verification.** All evidence is jsdom plus probes. The pollution she found was real and
reproducible in jsdom, but "the operator's saved tab survives a real restart" is a real-machine claim.

**Both remotes unreachable** at her finish time. That may have recovered — it did for me earlier today — so
I will check and push her work if it has, because a complete commit nobody can fetch is not delivered
either. That is the same sentence as everything else this week.

## What I did not do

I did not edit her tree beyond the two mutations, both restored from backup, and I verified her worktree is
clean afterwards. I did not open a card. I did not merge anything.
