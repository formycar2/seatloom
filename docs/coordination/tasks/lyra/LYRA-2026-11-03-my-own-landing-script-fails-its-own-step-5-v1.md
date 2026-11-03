# Standing-duty finding: my own landing script would fail on its own Step 5 — it checks seven seats but lands only six documents, and `AEGIS-HEARTBEAT.md` is not among them. I proved it by running Step 5 against a tree containing exactly what the script lands.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-my-own-landing-script-fails-its-own-step-5-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, self-correction, defect, proven |

## Why I looked

I have carried "operator must run `land-standing-duty-docs.sh`" as an open ask since 10-31, and last round I
tightened a different finding from "live" to "latent." **The same discipline applied to my own ask says: verify
the script still works against the tree it would actually run against.** I last verified it on 10-31 against a
track that has since moved 96 commits.

**It does not work. It fails on its own verification step, and the failure is mine.**

## The defect

The script lands six files (`DOCS` array, lines 49-56): the five seat HEARTBEATs plus the persist-from-input
ruling. **`AEGIS-HEARTBEAT.md` is not among them** — correctly, because it is not part of this delivery; it was
deleted from track in `4d22315` and restoring it is Aegis's decision, which I documented on 11-02.

**But Step 5 checks seven seats** (`land-standing-duty-docs.sh:184-186`):

```rust
let seats = [
    "seat-lyra-001", "seat-aegis-001", "seat-mira-001", "seat-nimbus-001",
    "seat-flux-001", "seat-onyx-001", "seat-vela-001",
];
```

**And the bash fallback at lines 187-190 and 195-198 has the same list.** So the script's own proof step
requires a file the script does not deliver.

## Proven, not inferred

I built a tree containing exactly what the script lands — the five HEARTBEATs from
`origin/packet/lyra/standing-duty-docs-only` plus `LYRA-HEARTBEAT.md` from the same branch, and nothing else —
and ran the script's own Step 5 Rust program against it:

```
FOUND   docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md
MISSING docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md
FOUND   docs/coordination/tasks/mira/MIRA-HEARTBEAT.md
FOUND   docs/coordination/tasks/nimbus/NIMBUS-HEARTBEAT.md
FOUND   docs/coordination/tasks/flux/FLUX-HEARTBEAT.md
FOUND   docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md
FOUND   docs/coordination/tasks/vela/VELA-HEARTBEAT.md
EXIT=1
```

**Six found, one missing, exit 1.** The script would die with "a seat could not be found" before ever reaching the commit prompt. **The operator cannot run this ask as written, and has never been able to** — the defect has been in
the script since I wrote it on 10-31.

## How it survived my own verification on 10-31

**On 10-31, `AEGIS-HEARTBEAT.md` was still on track.** It was deleted in `4d22315`, which is Aegis's docs-gate
fix — and that commit landed on track *after* I wrote the script. **So the script was correct when I verified
it, and the tree moved out from under it.** I verified the script against a state that no longer exists, which
is the instrument-not-verified-current failure I have documented a dozen times in other people's work, and
which I have now committed in my own artifact for the third time.

**The second reason it survived:** Step 5 is the step that would catch it, and it never ran, because the
script's Step 1 precondition (docs-only must be a descendant of track) fails first — docs-only is 96 commits
behind. **So there are two blockers stacked, and the outer one has been masking the inner one.** That is worth
recording on its own: a failing precondition can hide a second failure, and "the script refused" is not
"the script is otherwise sound."

## What the ask actually is now

**Not "run this script."** The honest sequence:

1. **Fix Step 5's seat list to the six seats this delivery covers**, or add `AEGIS-HEARTBEAT.md` to the
   delivery — and the second is Aegis's decision, not mine.
2. **Rebase `docs-only` onto current track** from origin's `7e06738`, which is 96 behind.
3. **Land the script on track** (it is untracked there).
4. **Then run it.**

**Step 1 is mine.** I wrote the script and the defect is mine, and fixing a seven-to-six in my own script is
not an unsupervised infrastructure action. **I am fixing it this round.**

## What I did not do

I did not run the landing script. I did not rebase anything. I did not commit the script to track — it is
untracked and landing it is a shared-state write. My measurements were the script's `DOCS` array, both seat
lists, `git cat-file -e` for the six files on current track, the grandfather file's line delta (207 vs 223,
exactly the 16-line block, so that part is still a pure addition), and the Step 5 program compiled and run
against a scratch tree. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
