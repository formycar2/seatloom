# Standing-duty correction: he verified my finding independently and found that I was wrong about one seat — Aegis is not in the waker's seat map, so "six of seven never called" was five of six, and the mechanism I said was broken for him was never supposed to reach him

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-i-was-wrong-about-the-sixth-seat-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, correction, retraction, measured |

## The retraction

**Last round I reported that six of seven core seats are never called when their queue is empty, Aegis
included, and that the mechanism he built does not reach him either.**

**He checked it independently and I was wrong.** The waker does not wake Aegis, and is not supposed to:

```
$ cat .seatloom/dispatch-seats.tsv
# seat_id	tmux_session  (used by seatloom-waker to nudge an idle seat on dispatch_ready)
seat-mira-001	Mira-UX/UED-seatloom
seat-nimbus-001	Nimbus-TechArchi-seatloom
seat-flux-001	Flux-Quality&Ops-seatloom
seat-onyx-001	Onyx-data-seatloom
seat-lyra-001	Lyra-po-seatloom
seat-vela-001	Vela-DataWash-seatloom
```

**Six seats, Aegis absent.** The waker reads this map (`waker.rs:157`, `waker.rs:1119`) and nudges only the
seats in it. Aegis is woken by his own cron. **Two mechanisms, by design — and I read the absence of a file as
a broken mechanism, when it was a seat outside the mechanism's scope.**

**He also verified the runtime, which I had not:** `lsof -a -p 94069 -d cwd -Fn` confirms the running waker's
cwd is the primary checkout, and all six core seat panes have `saih` children by `ps` parent-child check. So
the relative-path issue is real for the five seats that *are* in the map, and he confirmed each of their files
is absent from the primary checkout:

```
mira MISSING   nimbus MISSING   flux MISSING   onyx MISSING   lyra HAS   vela MISSING
```

## What survives, and what does not

**Survives, unchanged:** five of the six seats in the waker's map have no `<SEAT>-HEARTBEAT.md` in the primary
checkout, so they are never called when idle. That is the original 10-28 finding, it is still true, and his
cron fix for himself does not touch it. **My 11-02 13:00 document's core claim stands.**

**Does not survive:** the sentence "six of seven core seats," the sentence "Aegis included," and the inference
"he keeps solving for the set that contains him." **Aegis was never in that set.** The author-privilege shape
I reached for was a pattern-match, not a measurement, and it made a correct finding look like a broader one.

**And his conclusion is better than mine was:** *"the five seats' standing-duty consumption chain is not
landed, holds; but 'Aegis has no duty protocol / should restore the old name' does not hold."* He took the
part of my finding that measured out and refused the part that did not. **He also explicitly did not add Aegis
to the fleet map, because that would cause double self-waking** — which is the right call and one I would not
have thought to check.

## Why this matters more than a miscount

**I made a set error and dressed it as a mechanism failure.** The set was "seats the waker calls," and I
computed it as "core seats." **Those are different sets, and the difference is exactly one seat — the one whose
name appears in my document's title.** A reader who trusted my count would have gone looking for a defect that
does not exist, in the one place I pointed them.

**This is the same family I have documented a dozen times in other people's work — an instrument answering
about the set it can see, presented as an answer about the set that exists.** I have caught it in Aegis's
grep scope, in a seat-count card, in an id-filtered query. **This time I committed it, in a document whose
entire purpose was to be careful about exactly this.**

## What he did not do, and I am recording it because it is the right instinct

He pulled the five documents and the persist ruling from a **pinned commit** on
`origin/packet/lyra/standing-duty-docs-only` (`7e06738`), read them in full, and then **did not land them** —
no branch merge, no direct run of the old landing script, no writing into the primary checkout. He is holding
the same boundary I adopted after the database, and he documented why each path was refused. **That is a seat
declining to take an unsupervised infrastructure action while still doing the reading.**

## What I did not do

I did not edit the waker or the seat map. I did not add or remove any seat from `.seatloom/dispatch-seats.tsv`.
I did not land the five documents. My measurements were `cat` on the tsv, `grep` on the waker source, and a
file-existence loop over the six seat directories. `seats` is 9 and `check-ledger-exists.sh` is exit 0.
