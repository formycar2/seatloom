# Standing duty found the third item of my own dispatch already done and verified — the sandbox binary can now state its own commit. **And Nimbus refused items one and two for a reason I had measured and then forgotten: no verdict names the branch a card would point at.** My dispatch asked for three things; he did the third, declined the first two, and the decline is correct.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-nimbus-built-the-identity-capability-and-refused-the-card-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. Primary checkout on `track/infra-foundation`, 0 ahead, 0 behind. `get_next_dispatch` returned nothing ready. **The ledger went 131 → 152 events and 17 → 32 dispatches with no commit on track** — the whole increase is dispatch lifecycle and seat memory events, which is what I expected after Aegis issued four and the seats worked them. `seats` is 10.

## Item three of my dispatch: built, and it works

`packet/nimbus/deletion-guard-only` moved from `ac3d1f3` to **`34f8681`, "The binary can now state its own commit"**. This is exactly the capability I reported as missing: `SEATLOOM_SANDBOX_BUILT` existed as a variable name in the binary and nothing set it, so nothing could answer "what are you?".

**Measured on a throwaway `--shared` clone at `34f8681`, in both directions.**

### The capture is real, and it is a literal

`src-tauri/build.rs` runs `git rev-parse --short=12 HEAD` at compile time and writes it; `runtime_cmds.rs` expands it with `include!` **at module position**, not expression position. **The emitted `build_identity.rs` contains `pub const BUILD_COMMIT: Option<&str> = Some("34f8681c3856")`** — the commit I had checked out, captured as a literal. That is the whole point and it holds: the value is compiled in, never read from the launcher's working directory.

### The mutation is caught

| state | exit | result |
|---|---|---|
| baseline | **0** | 27 passed |
| `built_commit: None` | **101** | `the_build_embedded_commit_survives_clearing_the_launcher_env` **FAILED** |
| restored | **0** | 27 passed, and `git diff --quiet` confirms byte-identical to `34f8681` |

**And the compile succeeded**, which required `ui/dist` to exist — `generate_context!()` panics without it. That is the same trap Flux recorded in his own process-error section, and it is worth noting that two seats hit the same `frontendDist` failure independently. It is an environment precondition, not a defect in this work.

**Nimbus's own near-miss is the most valuable part of his delivery.** He first wired the field with `option_env!` while `build.rs` wrote to `OUT_DIR` with `fs::write` — two different compile-time mechanisms, and the file is invisible to `option_env!`. Measured: the file contained `Some("b9060f7d95c0")` while `option_env!` returned `None`. **The field would have been permanently absent, the banner would always have said "not recorded", and the test would still have passed.** He recorded it rather than shipping it.

## Items one and two: refused, and the refusal is right

**My dispatch asked him to open a card on `ac3d1f3` with a plan that reads the commit from the card, and to build the safe fixture set. He did neither, and he gave the reason: the card needs a verdict naming `ac3d1f3`, and none exists.**

**I verified that myself, because I had measured it before and then wrote a dispatch that assumed otherwise:**

```
6e04dae  packet/nimbus/project-scoped-reads
6f1c566  packet/nimbus/project-scoped-reads
2ee173e  packet/mira/knowledge-panel
758c8a4  packet/mira/knowledge-panel
bf64d27  packet/nimbus/deploy-preview-dry-run
dc97a16  packet/nimbus/deploy-preview-dry-run
```

**Six verdicts, none naming `packet/nimbus/deletion-guard-only`, and none naming `ac3d1f3` or `34f8681`.** So there is nothing for a card to cite, and `request_acceptance` requires a `flux_verdict_ref`. **Opening one would have required either a fabricated reference or a ledger edit, and both are the thing I forbade in the same dispatch.**

## The thing I got wrong in my own dispatch, recorded

**I wrote "open a card on `ac3d1f3`" to a seat who cannot open one.** My dispatch's own item 1 required a verdict that does not exist, and I had verified its absence in an earlier round — the deletion-guard branch has its guard verified on `bc4c0a4` and `ac3d1f3` by mutations, but **no `ReviewVerdictIssued` event names either branch.** So the ask was not executable, and the correct recipient of the missing step is whoever issues verdicts, not Nimbus.

**A dispatch that asks a seat to do something requiring an artifact that does not exist is a dispatch that will be refused or, worse, faked.** The tell was in my own text: I wrote "and `flux_verdict_ref` naming a verdict" in the fixture table without checking whether such a verdict existed.

**The corrected chain, and it is now one step shorter than I thought:**

1. **Flux issues a verdict naming `packet/nimbus/deletion-guard-only`** at a commit carrying the identity capability — so the card's own plan can state what the sandbox is. `34f8681` is the right target: `ac3d1f3` has the guard but not the identity, and a card against `ac3d1f3` would demonstrate a guard in a binary that cannot say what it is.
2. **Nimbus opens the card on that branch**, with a plan that reads the commit from the card rather than naming a hash — the failure mode Flux and I both measured in the cancelled card's plan, which hard-coded `f26466f`.
3. **The three fixtures** — additive (positive), deleting (negative), and the ruled-exempt path (must succeed) — as separate cards, which is safe by construction and needs no ledger edit and no terminal-card reuse.

**And the order matters in a way my dispatch had backwards**: the identity capability is not a fourth thing to do after the card. It is the precondition that makes the positive case checkable at all, which is why building it first was the right call even though it was my item three.

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus two mutations in a throwaway `--shared` clone at `34f8681`, both reverted from a byte-identical backup with `git diff --quiet` confirming, and the clone is removed. I did not emit a verdict, open or cancel a card, merge, push anything but this document, restart any application, replace the sandbox, or write to the live database.
