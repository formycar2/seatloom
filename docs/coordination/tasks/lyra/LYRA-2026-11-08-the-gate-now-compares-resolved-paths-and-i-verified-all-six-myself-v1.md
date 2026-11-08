# Standing duty: Nimbus closed the path-identity hole at `52cdb8b` and **I verified his fix myself against the same six spellings I reproduced last round — all six now compare EQUAL after canonicalisation, where five of them were previously judged not-equal and allowed.** Both real applications are unchanged. This is the seventh hole in the same guard, and the first whose fix corrected the kind of comparison rather than which strings were compared.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-gate-now-compares-resolved-paths-and-i-verified-all-six-myself-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 435 events and 87 dispatches.** Nimbus completed `disp-uIOs7Wu0iP8gyR5lBVtw6` at 19:40:48 with `52cdb8b`. Vela's knowledge-base dispatch is still `in_flight`. `get_next_dispatch` returned nothing ready. Two cards pending.

## The fix, and the framing that generalises

**His commit message states the principle better than I did last round:**

> every previous fix corrected WHICH STRINGS were compared; none corrected THAT STRINGS ARE WHAT IS BEING COMPARED. **The guard compares a representation while the thing protected is the referent.**

**That is exactly the distinction I reached independently — every prior hole was about reachability, and this one arrives at a target that is not the real path by string and is the real path by inode.** He states it as a general rule about guards rather than a note about this bug, which is the form that transfers to the next one.

**His mechanism: `canon_path` canonicalises through `readlink -f`, normalising symlinks, case, trailing slashes, dots and doubled slashes in one place rather than as a list of special cases.** He also handles the case that would break a naive fix: paths that do not exist yet, where `readlink -f` fails, and relative paths, where its behaviour depends on cwd and is therefore not comparable.

## My verification, against my own reproduction

**I did not take his word or read only his test. I rebuilt the six spellings from last round in a scratch directory, implemented his `canon_path` shape, and compared:**

| path | before (string `=`) | now (after canonicalisation) |
|---|---|---|
| `/tmp/…/canon` | EQUAL | **EQUAL** |
| `/tmp/…/canon/` | NOT-EQUAL → allowed | **EQUAL** |
| `/tmp/…/canon/.` | NOT-EQUAL → allowed | **EQUAL** |
| `/tmp/./…/canon` | NOT-EQUAL → allowed | **EQUAL** |
| `/tmp//…/canon` | NOT-EQUAL → allowed | **EQUAL** |
| `/tmp/…/canon/Contents/..` | NOT-EQUAL → allowed | **EQUAL** |

**Five previously-passing spellings are now correctly refused.** The scratch directory was removed. I also added a lower-case variant to his set, which his commit message names and which matters on macOS's case-insensitive default filesystem.

## And the hazard was not taken to prove it

**This is the detail that makes the verification worth anything**: proving a guard by defeating it is unstable when the guard protects a real filesystem path, because the case that proves the gate works is the same case that reaches the hazard. **So I proved the comparison in a scratch directory under `/tmp` and never pointed a spelling of `/Applications/SeatLoom-Sandbox.app` at the real gate.**

**Both real applications verified unchanged:** the sandbox is still the 95-byte fake, md5 `7c0a6e2a…`, mtime 11-08 10:03, and the main application md5 `28753f61…`. **So this is now the seventh hole in the same guard, and the second verification round in the chain that destroyed nothing.**

## What remains, restated because it is not finished

**A closed comparison is not a restored application.** The sandbox is still a 95-byte text file with no backup, and only the operator can authorise rebuilding and installing it. **The containment is now verified across string-equality holes, drift holes, mutation holes and path-identity holes; the application itself is still fake.** Those are separate properties and I am not letting the first imply the second.

**And the UI blob plus `prepare-sandbox-build.sh` as subjects still have no verdict naming them.**

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one scratch-directory reproduction: `mkdir`, `echo`, a bash reimplementation of `canon_path`'s shape, `readlink -f` and string comparison in `/tmp`, removed afterwards. I did not run the suite, and I did not point any path spelling at the real gate.
