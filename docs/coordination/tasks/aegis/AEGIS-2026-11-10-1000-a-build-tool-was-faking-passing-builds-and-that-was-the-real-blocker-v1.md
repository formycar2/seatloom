| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-1000-a-build-tool-was-faking-passing-builds-and-that-was-the-real-blocker |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, build-toolchain, sandbox-precondition, dispatch, verification |

# A build tool was faking passing builds, and that — not the sandbox file — was the real blocker

## What `check-unclaimed-work` surfaced

It flipped to exit 1 with one new candidate: `disp-fkLMc9KBolrwtEyoKi2EZ`, a dispatch Lyra issued to Nimbus
titled *"your build toolchain is faking passing builds: three fixes, I tested each one"*.

That title is the most consequential thing I have read this week, because it reframes the blocker I have
been reporting to you for days.

## The defect

Lyra measured that `prepare-sandbox-build.sh --build-only` **reported success on a 48-byte ASCII text
file**. The cause:

- both build paths resolve `tauri` through `PATH="$WT/node_modules/.bin:$PATH"`
- on that worktree that path held a **330-byte bash stub** which writes a text file and computes the commit
  label from live `HEAD`
- the script had **zero Mach-O checks**, and all four of its checks were satisfied by the stub's output —
  because the label comes from live `HEAD`, it always matches

**So the build tool would have happily certified a fake build as installable.** That is the same class as
the sandbox being a 95-byte text file: a thing that looks like a control and verifies nothing.

## What I verified myself, rather than taking the report

All three fixes are present on `fb83cd6` and correctly ordered:

| fix | measured |
|---|---|
| `assert_macho` defined | line 87 — refuses on missing file, missing `file` command, unreadable type, and any non-`Mach-O` output |
| called on **both** build paths | lines 529 and 594 |
| called **before** the other checks | line 529, with a comment saying a stub can also satisfy those four |
| `tauri-build.log` kept on success | lines 519–531, with the reason that it is the only evidence distinguishing "tauri really ran" from "something wrote a file at that path" |

The ordering is the part that matters. A Mach-O check placed after the existing four would never have run,
because the stub satisfies them.

## What this changes about the decision in front of you

I have been reporting the sandbox as "a 95-byte fake needing your authorisation to rebuild". **That was
incomplete in a way that mattered:** rebuilding it with the old toolchain would have produced another fake
certified as real. The toolchain had to be fixed first, and it now is.

So the honest statement is: **the toolchain is now real, and that is the precondition for a rebuild — it is
not the rebuild.** Nimbus ran a real end-to-end build producing a 12.7 MB Mach-O, per Lyra's verification.
The sandbox is still a 95-byte fake, and I have not touched it.

## What I dispatched, and what I did not

The fix has **no verdict and no card**, so it cannot reach you. I dispatched
`disp-LuC5D3g7M9i3BZbbWvYoI` to Flux at priority 97 with three requirements that go beyond "does it work":

1. **run the mutation yourself** — disable `assert_macho` and confirm the suite goes red. A test that passes
   with the guard disabled proves nothing about the guard, and that is the exact defect class here.
2. **state explicitly that the toolchain is real and that no sandbox was rebuilt** — because an ambiguous
   verdict would let a rebuild be read as done.
3. **fingerprint both applications before and after** — `/Applications/SeatLoom.app` and the sandbox — the
   same discipline Lyra used.

I did not open a card. Nimbus owns that work and a card is opened by the seat that owns it.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0, plus `check-unclaimed-work` at 0
  after classifying the new candidate with a real reason.
- Nine declared candidates, each with a reason; three aged out and were pruned.
- Sandbox: **still a 95-byte ASCII text file**, mtime Nov 8, no backup. Not touched.
- Main application: pid 90805, untouched.
- DB reliability: 10/10 this hour.
- Push: local and remote both at `9737785` at the start of this hour.

## Needs you

- **The build toolchain is fixed and needs verification before a card can reach you.** That verification is
  in flight.
- **The sandbox rebuild is still yours to authorise**, and now it is actually possible rather than
  self-defeating.
- **Three cards are waiting**, none blocked by being behind track.
- **Option 1, 2 or 3 for the live-seats count** — my recommendation is 1, unchanged.
