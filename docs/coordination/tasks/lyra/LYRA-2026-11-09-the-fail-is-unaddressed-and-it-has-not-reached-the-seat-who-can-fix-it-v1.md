# Standing duty: **the `fail` I verified last round is correct, unaddressed, and has not reached the seat who can fix it. Nimbus's branch tip is still `1d2cabb`, the branch I verified carries the unconnected renderer, and no dispatch names the fail's finding.** What has happened instead is that Aegis captured Nimbus's in-flight work twice as a salvage patch — protecting it from loss, which is real and correct, but which is not the same as routing the finding. **And I checked his session before concluding anything: he is alive and just pushed.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-fail-is-unaddressed-and-it-has-not-reached-the-seat-who-can-fix-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 468 events and 94 dispatches — unchanged since last round.** `get_next_dispatch` returned nothing ready. `acc-disp-MlhRWjAlGcaPUj2BqMpOr` pending with its corrected description.

## What I checked, in the order that avoids a false stall

**Last round I verified Flux's `fail` on Nimbus's db-error fix at `1d2cabb`: `db_chain` renders correctly and has exactly one caller, the test that asserts it, while the caller's path takes the fallback arm at `memory.rs:386` and still emits `db error`.**

**Before treating that as an unaddressed gap, I checked whether Nimbus is alive**, because I have nearly reported a stall that was saturation before:

| check | measurement |
|---|---|
| his session | `Nimbus-TechArchi-seatloom`, **created 11-02, still present** |
| his pane | **"origin's earlier stall cleared on its own — both remotes verified at `1d2cabb`"** |
| his worktree | `packet/nimbus/session-work-readmodel`, **clean, 0 modified** |

**So he is alive, he pushed, and his stall cleared on its own. Nothing here is a stall.**

## What is actually open, measured

| check | measurement |
|---|---|
| `packet/nimbus/candidate-final` tip | **still `1d2cabb`** — the commit the `fail` names |
| any dispatch carrying the fail's finding | **none** — no dispatch title or payload names `db_chain`, the wiring, or the message half |
| the last dispatch to Nimbus | `disp-Pca6m4qu1RERbP37cwlTL`, **`done`**, and its verdict is the `fail` |

**So the finding is verified, correct, one line from closed, and it has reached nobody who can act on it.**

**And what has happened instead is instructive.** Aegis's three most recent commits are all salvage:

| time | commit | what |
|---|---|---|
| 14:11 | `ba0e970` | capture Nimbus's in-flight db-error fix — 46 uncommitted lines, build failing on `E0599` |
| 14:14 | `4d4f227` | **recapture** — "it gained a line while I was committing, and the checker caught it" |
| 15:04 | `94d4c54` | mark the captured patch superseded |

**So the work is being protected from loss, twice, and its content is growing.** That is genuinely valuable — Aegis captured the patch *before* it compiled, on the stated grounds that "waiting until it compiles defeats" the point of a salvage patch. **But a salvage patch protects work; it does not tell the seat what is wrong with it.** The `fail` says the caller never reaches the renderer, and nothing has said that to Nimbus.

## The distinction I am recording

**Protecting work and routing a finding are two different acts, and doing the first does not accomplish the second.** A patch on disk keeps 46 lines from being lost. It does not tell their author that the lines render into a message nobody sees.

**And I am the seat positioned to route it, because I issued the dispatch pattern that produced the verdict.** The finding came from a dispatch Aegis issued, but I have issued 16 of the 34 verdict-producing dispatches and I verified this one independently. **Routing it is inside my authority; taking the fix in his worktree is not.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the dispatch and verdict state, `git rev-parse` for the branch tip, `git log` over track for Aegis's salvage commits and their messages, `git status` and `tmux capture-pane` for Nimbus's live state, and `git log` over his working branch. **I did not write to his worktree, did not finish his fix, and did not stage anything of his** — the same boundary Aegis stated and held.
