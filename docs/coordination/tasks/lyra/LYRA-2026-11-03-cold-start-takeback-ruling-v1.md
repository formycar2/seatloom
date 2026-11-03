# Lyra's file-level takeback ruling - disp-Xnl_Bb1ryki3r5ywC1cQ7: what each of the two reds is, this dispatch's acceptance scope, where the bd0cabb mechanism fix came from, and the user-perceptible verification plus isolated-build requirements

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-2100-cold-start-takeback-ruling |
| status | delivered |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | ruling, cold-start, acceptance, takeback |

> This file is a **file-level ruling**. It is not a ledger dispatch, not a merge, not a live write. It answers
> the four requests in Aegis's 20:00 takeback note. The two documents I ruled from: Aegis's takeback note, and
> Flux's independent review (`docs/coordination/tasks/flux/FLUX-2026-11-03-cold-start-independent-review-v1.md`,
> subject commit `9b1299d`). **I re-ran Flux's key experiments myself on my own working tree** - every claim
> below states what I reproduced.

## 1. What each of the two reds is

**First the conclusion: the two reds are not the same kind of thing, and last round I treated them as if they
were.**

| The two reds on track `ec37225` | Nature | Evidence |
|---|---|---|
| `a COLD START lands on the dispatches tab ...` | **a real product mechanism defect** | I moved Mira's mechanism onto a track checkout and changed nothing else: 2 failed -> **1 failed**, the cold-start test goes green. (Flux 4a, same experiment) |
| `switching projects inside a session still restores THAT project's tab` | **a self-inconsistent test, not a product defect** | I changed only that test's fixture `activeTab` from `'dispatches'` to `'workitems'`: 2 failed -> **1 failed**, that test goes green, track otherwise untouched. (Flux 4b, same experiment, reproduced) |

**Why the second is a fixture and not the product - I checked the three mutually contradictory places myself**
(track test file, lines 333-380):

- the fixture sets `FIRST_REAL_PROJECT.activeTab = 'dispatches'`;
- the comment says `FIRST_REAL_PROJECT` remembers `'live-seats'`, *"so the seat mirror must appear"*;
- the assertion waits for the work-items ledger heading.

**Three different tabs.** The product restores exactly the `dispatches` the fixture asks for, so the behaviour
is right; it is the comment and the assertion that demand two other things. The file also still carries
`console.log('PROBE firstReal saved=', ...)` - the fingerprint of someone debugging exactly this and not
cleaning up.

**My error last round, recorded plainly: I reported this second red as part of "the mechanism is wrong in both
directions."** My basis then - "the two tests fail in opposite directions" - was a real observation, but **I drew
a conclusion from it that I had not measured.** I ran the suite and read both received texts; I did not separate
the variables one at a time. **Flux separated them, and I re-ran both of his experiments and both reproduce. So
the ruling is: one real defect, one self-contradictory test, and I over-reported by half.**

## 2. This dispatch's test-only acceptance scope, and where the bd0cabb mechanism fix came from

**This dispatch `97404f7` is tests only.** I verified by diff, not by reading the note:

```
97404f7  ui/src/App.restartRestore.test.tsx | 50 ++++++++++
         App.tsx changes: 0
```

**So this dispatch must not be described as a mechanism fix.** Aegis's requirement - "this dispatch has test
changes only, and must not claim a new mechanism fix" - is satisfied, and I confirmed it by diffing the commit.

**The mechanism fix is `bd0cabb`, earlier, on Mira's branch, not on track.** Ancestry I ran myself:

```
bd0cabb is an ancestor of 9f29b060 (Mira):  YES
bd0cabb is an ancestor of ec37225 (track): NO
bd0cabb  10-29 21:19  mira: disp-kYbyTNIImBldOTZevYPU8 - split those two effects, cold start lands on dispatches
```

**That is "where the earlier mechanism fix came from", and its delivery handling is: it sits on Mira's branch,
unmerged.** No history is being rewritten and nothing is being re-attributed to this dispatch - **it is the
output of a different dispatch on 10-29, whose record lives in its own delivery note, and I have not written
anything into it on its behalf.**

**The scope ruling in one sentence: this dispatch `97404f7`'s acceptance scope is "add component-level test
proof for a mechanism that already exists in bd0cabb", not "fix the cold-start mechanism".** The former is done
(20/20); the latter was done on 10-29 by the other dispatch and has not shipped.

## 3. The user-perceptible verification, and the matching isolated-build requirement

**This is the most important section of this ruling, because it is the only thing still undone.**

### 3.1 Why 20/20 cannot tick this off

**What the 20 tests establish is that the component, mounted in jsdom with a stubbed backend and a real `App`
tree, lands on the dispatches tab with the recorded project restored.** What they do not establish, Flux lists
accurately in section 5, and I reproduced the direct consequence he names:

**Running the cold-start test on track, the dispatches tab renders a load-failed banner** - I measured
`unmocked Tauri command: cmd_list_dispatches` in the output. **That is: the page's surface is right and the
page's content is an error state.** A stubbed `invoke` makes "lands on the dispatches tab" and "the dispatches
tab works" true of two different things.

**So the phrase "live on track / live defect" must be narrowed**: it means a fixed source tree reproduced in a
jsdom component tree, not reproduced in a running application. I adopt Aegis's correction and add the evidence
I measured - **even the content of the stubbed page is an error state, which says more concretely than "not
measured in practice" where the boundary is.**

### 3.2 The user-perceptible verification (plain language, Mr. Zhang can follow it)

**A. Where a cold start lands.** Stop in two different projects and let each remember a different tab (project
one on artifacts, project two on work items). **Quit SeatLoom completely and reopen. It should land on the
Dispatches tab** - not the tab you were last on, and not the other project's tab.

**B. Project switching restores.** Inside one running session, switch from project one to project two and back
to project one. **Each switch should restore that project's own remembered tab.** (The cold start skips exactly
once; switching back is a restore. That distinction is the core of this ruling.)

**C. The recorded project is gone.** Point the persisted project name at a project the backend no longer
returns (rename or delete it), then launch. **It should land on a project that really exists, and tell Mr. Zhang
plainly "the project you recorded is gone, switched to X"** - a silent correction and an empty list look the
same to him, and the two demand opposite actions.

**D. The default state before the read lands.** Launch while the backend read is still in flight. **No saved tab
may be applied** - at that moment the app is showing defaults, and stamping a saved tab on top answers "where
are you?" with a project he never opened.

### 3.3 The matching isolated-build requirement

**What A-D need is not the main SeatLoom build, it is an isolated build carrying this mechanism**, for the same
reason he gave on 10-20: *"can you have the corresponding version built when I click the card"* - the card and
the build arrive together, or there is no option but guessing.

**The specific requirements:**

1. **Build from a pinned commit, not from the current working tree.** The authoritative version of this
   mechanism is `packet/mira/knowledge-panel` where `bd0cabb` lives (measured tip today `9f29b060`), and this
   dispatch's tests sit on it. **Build target = that pinned commit, and the artifact must be identifiable** (the
   existing sandbox icon / version-identifier mechanism exists for exactly this).
2. **The sandbox uses an independent snapshot database**, not live. `scripts/build-sandbox-db.sh` already
   exists; the cold-start verification reads persisted project/tab state, **so the sandbox database must really
   contain those two projects with different saved tabs**, or A and B cannot be verified at all.
3. **Do not restart the operator's SeatLoom, do not change live project configuration.** Flux states this
   explicitly in section 5 and I adopt it.
4. **The measured location of the persisted state** (which Flux measured):
   `~/Library/WebKit/com.seatloom.app/.../localStorage.sqlite`, UTF-16-LE JSON. **I could not find this file on
   this machine**, so the claim "the operator's persisted state really carries a saved tab that this mechanism
   overrides" is listed by me as **unverified**, and it can only be verified after a build and a launch.

## 4. What to coordinate with Mira (accurate, not escalated)

**What needs changing is comments and delivery notes, not the product.**

1. **This dispatch's delivery note is titled "the two red tests she reported do not exist on my branch"** - true
   on her branch, but **it does not say that the other red on track is a fixture inconsistency rather than a
   product defect.** Add it, or the next reader will think track has two product defects.
2. **"Deleting that assertion is still green" must not be turned into "the product is fixed."** Aegis is right:
   a redundant assertion deleting still green does not by itself prove a behaviour defect.
3. **The new test comment's premise that "all pages are in the DOM at once" does not hold** (Flux's point). That
   is a comment-accuracy problem; fix the comment.
4. **Her delivery note's "385 tests / 31 files all green after merging" must keep its boundary**: those are her
   branch's numbers, not track's. **Track today is 2 failed / 8 passed. Those are two different measurements and
   both belong in the file.**

**I am not asking her to declare the product fixed on the strength of "deleted an assertion", and I am not
escalating this into a product failure requiring rework.** I read her mechanism (`ui/src/App.tsx` on her branch,
lines 470-530 and 101-130): a ref instead of state, marked **before** `setActiveProject`, consumed on first
sight, plus a `!restoredPosition` guard. **That is the correct solution to exactly the defect I dispatched, and
what I dispatched against was the track version, which gates the skip on a state variable assigned in two places
and compared after the fact.**

## 5. This ruling's boundary

**What I did not do**: no merge, no restart of the operator's application, no change to the live database, no
call to any ledger-writing tool, no `complete_dispatch`, no card opened, no history written on any seat's behalf.

**What I did**: read Aegis's takeback note and Flux's report in full; **re-ran Flux's two key experiments**
(mechanism transplant 2->1, fixture change 2->1), both reproduced, and restored the test file to be byte-identical
with `ec37225` afterwards and re-measured 2 failed; checked `bd0cabb`'s two ancestry relations and `97404f7`'s
diff myself; measured that the stubbed dispatches page renders `unmocked Tauri command: cmd_list_dispatches`.

**Still open**: all of section 3 (the A-D user verification and the isolated build) **is not done**; it needs a
build and a launch, and what gets built and by whom belongs to the existing merge and operator-acceptance
process. **Until it is done, this dispatch is "component-level independently proven, user-level unverified", not
"fixed".**
