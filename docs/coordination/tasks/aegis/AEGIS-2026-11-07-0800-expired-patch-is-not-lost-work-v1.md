| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-07-0800-expired-patch-is-not-lost-work |
| status | delivered |
| author | aegis |
| date | 2026-11-07 |
| version | v1 |
| tags | aegis, salvage, honesty |

# Salvage caught a second expired patch, and this time the work had already landed

## What this round did

`check-salvage-fresh.sh` reported that
`nimbus-session-work-readmodel-20261031-1302.patch` no longer reverse-applies to any
worktree. The protocol's instruction is to rename it to `.superseded.patch` and update the
manifest, and I did both, committed as `1810655` on local `track/infra-foundation`.

## But "the patch expired" is not a conclusion — it has two opposite meanings

The protocol only says "reverse-apply fails = expired = the manifest is lying". Executing
that literally yields a result whose conclusion is right and whose reason is empty:

1. the work was committed, so the patch should retire;
2. the work was discarded or overwritten, so the patch is the last copy and must be salvaged.

These two demand different next actions, and **reverse-apply failure alone cannot tell
them apart**.

## Measured: it is the first kind, on three pieces of evidence

| Evidence | Command / result |
|---|---|
| reverse-apply fails | `git apply --check --reverse` -> `patch does not apply` (content is not in the tree) |
| the repeat-write refusal exists | `git log -S "MemoryRejection::Duplicate"` -> `ee2d03b` |
| the dedup key exists and the dead transaction pair is gone | `git log -S "pub fn memory_dedup_key"` -> `57b7737`; the same commit's subject says "drop the dead transaction pair" |

So Nimbus made the decision the README left to him, and he made it on the
"delete the original two lines" side rather than the "finish the patch" side.
**This is not a loss. It is a success that is now recorded.**

## The rule written into the README

An expired patch is not automatically lost work. **Measure which kind it is before
renaming**, or a success gets written down as a loss — and the other direction is worse,
because work that genuinely needed salvaging would be silently let go.

## Other health items measured this round

- Ledger `public` table count: **35** (>= 30)
- `check-salvage-fresh`, `check-cards-clickable`, `check-verdict-refs`,
  `check-verdict-defects`, `check-card-verifications`: all **exit 0**
- `verify-docs.sh` before the change, and the pre-commit docs gate after it, both pass
- All six seats' pane child processes present; self-wake `--check` healthy, `:00` POKED
- `git status --porcelain` contains only the known untracked `scripts/bind-seatloom-seats.sh`
- **No** merge, install, application replacement, restart, or live-database write

## Not resolved (left as they were, not decided unilaterally)

- The deletion guard is still not part of the verified candidate; its card remains
  `cancelled` and is neither reopened nor rewritten.
- UI integration is still undelivered; the CLI tools are diagnostics only.
- `scripts/bind-seatloom-seats.sh` is still untracked, left for the operator to decide.

## A note on my own process

While running the docs gate I wrote `bash scripts/verify-docs.sh | tail; echo $?`, which
reports `tail`'s status and not the script's — the exact failure shape my own instructions
warn about. It printed a misleading `exit=0` while the gate had actually refused. The
committed evidence is the pre-commit refusal itself, which was correct and which stopped
the commit until the doc was rewritten in English.
