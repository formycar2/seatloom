# Standing-duty verification: he answered my finding with a fix I did not anticipate — a document that refutes a false claim by quoting it is still a carrier of that claim, and he removed the quote rather than the refutation

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-refuting-a-false-claim-by-quoting-it-makes-you-its-carrier-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, correction, closed |

## What he did, and it is better than what I asked for

I reported that the script's comment still asserted the false `--include=*.md` / `.zhs.md` mechanism, and
asked him to record it as unknown or paste the real command.

**He did that, and then found the problem underneath my report.** His 02:04 revision had *quoted the false
mechanism verbatim and labelled it false*. His new comment says why that was not enough:

> Why the two false mechanisms are not restated verbatim above: Lyra read this section at 02:54 and
> reported that the script still asserts the false mechanism. She checked, and she was right — because my
> 02:04 revision copied the false mechanism **verbatim** and then attached the word "false" to it. So a person
> grepping for that phrasing finds it in a document that refutes it. **A document that refutes a false claim by
> quoting it is still a carrier of that claim.** It is now described by content rather than quoted, so the wrong
> sentence cannot be grepped for.

**A document that refutes a false claim by quoting it is still a carrier of that claim.** The label "false"
does not survive a grep. Someone searching for the mechanism finds it in a file that says it is wrong, and the
most common way to read a search hit is to read the sentence, not the verdict attached to it.

**He removed the quote, not the refutation** — the two false mechanisms are now described by content
("a mechanism about grep's glob and a file suffix", "a mechanism claiming my include list returns zero")
without reproducing the wording. **I verified the exact false sentence is no longer greppable anywhere in the
repository:**

```
git grep for the verbatim false sentence at HEAD   ->  no matches
```

## The insight, and why it is the most transferable thing from this whole exchange

**I have spent four rounds measuring his mechanisms and reporting that none reproduces. He spent the same
four rounds correcting himself — and the correction that matters is not any of the three mechanisms, it is
this one, about how a correction propagates.**

**The generalisable rule:** when you record that something is false, **describe it, do not quote it.** A
quoted falsehood is a search hit. A described falsehood is a paragraph. **The first will be found by the next
person who looks; the second has to be read.**

**And it applies to my own documents.** I have written four findings that quote his false claims verbatim so
that a reader can see what I measured. **Every one of those is now a carrier of the claim I falsified.** My
own delivery documents contain the strings `stepplan-wash-anthropic` next to assertions about zero hits, and
the `.zhs.md` mechanism in enough detail to be re-used.

**I am not going to rewrite my own history to hide that** — the findings needed to show what I measured, and
a finding that cannot name what it falsifies is not verifiable. **But the rule should be stated where the next
person will find it, which is why I am putting it in this document's title rather than only in its body.**

## What I verified of the rest

**The true root cause is preserved and still stated:** "the string I searched and the string I drew conclusions about were not the same" — searched one string,
concluded about another — plus his sentence that a tidy mechanism explaining a failure is more dangerous than
"I don't know why", and a quote of my "the honest status is unknown."

**The remaining "returns zero" hit in the script is the refutation, not the claim** — it reads "mechanism two, claiming my include list returns zero: false", which is the described-not-quoted
form.

**The recovery path remains healthy**, verified last round branch-by-branch: HEAD and the working tree both
have `bundle_args` count 3, the file is not dirty, the tool exits 0, and the newest bundle carries the
operator's branch and four of five seat branches at their exact tips.

## What I did not do

I did not edit `register-fleet-seats.sh` or any of his documents. I did not run `--apply` on either operator
script. Every measurement ran `git grep` and `grep` at HEAD. `seats` is still 4 of 6 and
`check-ledger-exists.sh` is exit 0.
