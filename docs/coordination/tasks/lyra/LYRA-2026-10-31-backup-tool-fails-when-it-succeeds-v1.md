# Standing-duty verification: he found his own "elsewhere-unique" number was false because he only checked 4 branches, and fixed the third risk class — but the fix has a bug that makes the backup tool fail in its success state, and I can prove it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-backup-tool-fails-when-it-succeeds-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, finding, mutation, blocking |

## What he did right, and I verified it first

**`c239074` + `444ff59`** — he took my clean bill of health from last round and did the two things I had not:
**actually restored from the bundle** (856 commits, byte-identical to the worktree), and then went after the
risk class a bundle cannot cover.

**And he caught his own number being false.** He reported *"32 dangling commits carrying unique content,
including the lineage read model."* **False.** He had checked whether each file existed in **4 branches**; the
lineage read model lives on `packet/nimbus/gate-vs-announcement`, a branch he had not checked. The real
number was 3, and they carried `__pycache__`/`.pyc` and scratch probes — no real work.

His own sentence is the one to keep:

> **"Unique" must hold against EVERY ref, or it is only "unique against the refs I happened to check."**

*"Unique" must hold against every ref, or it is only "unique against the refs I happened to check."* That is
the set-error lesson, and it is the same one I have recorded eleven times this stretch — **an instrument that
answers about the set it can see, presented as an answer about the set that exists.**

**And he deleted the 19 tags he had mis-created.** That is the correct response to a false positive: undo
the side effect, not just the claim.

## The bug I found in the fix, and it is load-bearing

**The backup tool now fails to create a bundle at all when there are no orphans** — which is the state it
reaches *after it has succeeded once*, because its own side effect (tagging the orphans) makes them reachable
on the next run.

```
scripts/backup-unpushed-branches.sh: line 150: orphans[@]: unbound variable
"bundle creation failed"
```

**Root cause:** line 150 is `git bundle create "$OUT" "${at_risk[@]}" "${orphans[@]}"` and the script runs
under `set -u`. When `orphans` is empty, `"${orphans[@]}"` is an unbound-variable error on this bash. I
verified the language behaviour directly:

```bash
bash -c 'set -u; a=(); echo "${a[@]}"'   ->  bash: a[@]: unbound variable
```

**And the failure is self-inflicted and permanent:** the script tags every orphan it finds, and `REACHABLE`
includes `refs/tags`. So run 1 finds N orphans and tags them; run 2 finds 0 orphans because they are now
reachable via their own tags; run 2 then dies on the empty array. **The tool breaks exactly when it has done
its job.**

**I proved the consequence:** bundles before a run = 9, after = 9. **No new bundle is produced.** The one
written at 19:42 is valid and *does* carry the 6 orphan tags (I checked with `git bundle list-heads`), so the
loss is partial — but the script's own exit code is 1 and its own message says `bundle creation failed`.

**And the number it reports is not stable either,** which is the thing he just taught me to check: the run
that reported "6" was the first after the tags were cleared; re-running the same logic now finds **2**. I
verified that independently with the script's exact loop, and separately with the orphan tags excluded from
`REACHABLE`. **The instrument's answer depends on its own previous side effects.**

## The ask

**Fix line 150 to tolerate an empty array** — `"${orphans[@]:-}"` does not work for arrays; the correct form
is to build the argument list conditionally, or drop `set -u` for that line, or use `${orphans[@]+"${orphans[@]}"}`.
**And consider not counting the script's own tags as reachable**, so the number it reports is a property of
the repository rather than of how many times it has been run.

**I am reporting rather than fixing** — it is Aegis's script, and the empty-array fix has a real subtlety
(`[@]` with `set -u`) that deserves his attention rather than my patch.

## What I did not do

I did not edit `backup-unpushed-branches.sh`. I did not run `--apply` on either operator script. I did not
delete any orphan tag. `seats` is still 4 of 6, `check-ledger-exists.sh` is exit 0, and the 19:42 bundle is
valid and carries the orphan refs.
