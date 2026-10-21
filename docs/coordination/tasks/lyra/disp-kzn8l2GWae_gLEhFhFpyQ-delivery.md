# Tool-use rule adoption delivery

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | lyra-disp-kzn8l2GWae_gLEhFhFpyQ-delivery |
| status | completed |
| author | lyra |
| date | 2026-10-22 |
| version | v1 |
| tags | tool-use, token-cost, dispatch |

## Decision

Adopt the governing packet `docs/coordination/tasks/aegis/AEGIS-2026-10-22-use-edit-not-heredoc-v1.md`, read from the primary checkout because it is absent from this worktree. Use `edit` for targeted code changes and `write` for new files or complete rewrites. Do not embed code modifications in bash heredocs or prepend redundant absolute-path `cd` commands. Preserve all verification requirements. Batch disjoint edits to one file in one `edit` call, as required by the active harness instructions.

## Changes and tool counts

This dispatch has no code changes. The only new artifact is this delivery note. Its creation uses `write`; any verification-result update uses `edit`. Authored bash code-modification heredocs: **0**. Authored absolute-path `cd` prefixes: **0**. Code-modification `edit`/`write` calls: **0**, since this is a process-only task. These counts cover calls in this dispatch, not historical session activity. The next actual code task must provide its own evidence; this note does not claim that future acceptance has already passed.

## Validation

- `git diff --check`: failed on pre-existing whitespace in the two October 19 documents; no unrelated repairs made.
- `bash scripts/verify-docs.sh HEAD`: failed only on this note's dispatch-id filename, which is mandated by the current completion protocol and conflicts with the older task naming gate. No gate weakening or grandfather-list change made.
- Targeted staged whitespace check is run before commit. No runtime tests apply to this documentation-only adoption.


## Scope and escalation

The worktree was already dirty on entry: two modified October 19 coordination documents, one untracked acceptance document, and `.saih/`. They are not owned by this dispatch and will not be edited, staged, or committed here. Delivery branch on entry: `packet/lyra/milestone-summary`; base commit: `b927269c1eabce680c842fddb9c15c0ecd149741`. No branch switching or infrastructure action is authorized. Only this dispatch's artifact will be committed. The required dispatch-id delivery filename takes precedence over the older generic task filename convention. Shared memory logs remain untouched.
