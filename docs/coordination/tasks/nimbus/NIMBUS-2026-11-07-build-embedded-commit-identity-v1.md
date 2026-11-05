# The binary can now state its own commit

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-build-embedded-commit-identity-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | disp-41pzdQUapXA-Hz3YUMPfL |
| supersedes | |
| tags | sandbox, identity, build-script, falsifiability, measured |

## Why this slice, and why it is the one that matters

Lyra's reconciliation dispatch measured the consequence precisely: `strings` on the
sandbox binary finds the four `SEATLOOM_SANDBOX_*` env var **names** and six
hex-looking strings, **none of which is a hash of any commit on the branch**. So
nothing in the binary can answer "what are you?".

Until it can, the positive case of an acceptance is unfalsifiable. An operator who
clicks Accept and sees a merge cannot tell whether the guard refused for the right
reason or the merge happened for the right reason, because the running code cannot
state its own commit. This is the smallest slice that changes that.

## What was built

A `build.rs` that captures `git rev-parse --short=12 HEAD` at compile time and writes
it into `OUT_DIR`; a `built_commit` field on `RuntimeIdentityDto` read from it; and a
banner clause that states it and **flags disagreement** with the installer's claim.

Three things are now separately visible, which is the point:

| source | what it states |
|---|---|
| `built_commit` | what this binary was compiled from |
| `commit` | what the installer claimed it launched |
| the card | what the acceptance is about |

Only when all three agree does the click verify what it claims to. When the binary
and the installer disagree, the banner says so loudly rather than silently showing
one of them.

## The bug I nearly shipped, and the test that would have missed it

I first read the captured value with `option_env!("SEATLOOM_BUILD_COMMIT")` while the
build script wrote to `OUT_DIR` with `fs::write`. **Those are two different
compile-time mechanisms**, and the file is invisible to `option_env!`. Measured
directly: the `OUT_DIR` file contained `Some("b9060f7d95c0")` while `option_env!`
returned `None`. The field would have been permanently absent — the banner would
always have said "not recorded" — and the test would still have passed, because it asserted
`is_none() || != Some("")`, which `None` satisfies.

That is a worse failure than a missing feature: it makes "we have this capability"
look true while it is always off. The test now asserts the commit **is** captured,
with the reason written into the failure message — if `git` is genuinely unavailable
in an environment, that is a real failure, not a skip.

A second, smaller trap: `const X: T = include!(...)` is a syntax error, because the
generated file contains `pub const` — an item, not an expression. The include is at
module position instead.

## Verified

| | result |
|---|---|
| `cargo build --workspace` | 0 errors |
| `cargo test --workspace` | **368 passed**, 0 failed |
| `SandboxBanner.test.tsx` | 16 passed |
| `tsc --noEmit` | clean |
| clippy | 0 warnings on my files (the remaining one is track's pre-existing `default_project_id`) |
| the emitted `build_identity.rs` | `Some("b9060f7d95c0")` — a real commit |
| mutation: the field wired to `None` | **RED**, naming the bug |

The three new frontend cases cover agreement, disagreement, and "not captured" —
including that an absent capture must **not** read as agreement.

`cargo fmt` also reformatted `src-tauri/src/commands/history_cmds.rs`, which is
track's own pre-existing fmt debt (measured on `track/infra-foundation`: 1 diff). I
reverted it in its own step rather than folding it into this commit.

## Not done, and why

Items 1 and 2 of Lyra's dispatch — opening the card on `ac3d1f3` with a plan that
reads the commit from the card, and building the safe positive/negative/exemption
fixture set — are separate. The card needs a verdict naming `ac3d1f3`, and none
exists yet: all three verdicts for `disp-AbQ4m1TKQZGLMeZ4bhwjy` name `6e04dae` or
`6f1c566` on `packet/nimbus/project-scoped-reads`. No card is opened, no merge, no
deployment, no live-DB write.
