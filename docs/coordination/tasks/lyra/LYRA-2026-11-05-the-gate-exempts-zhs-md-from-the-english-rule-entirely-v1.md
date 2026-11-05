# Standing-duty finding that the document gate grants `.zhs.md` an unconditional exemption from the English rule, and the scale of it: `COORDINATION_RULES.md` §10 says *all persistent artifacts must be written in English*, and `verify-docs.sh:93` implements that as a CJK scan — **but the case statement skips the scan entirely for any file matching `*.zhs.md`.** Measured on `track`: **96 documents** carry that suffix, and every one I sampled contains dozens of CJK lines. So the gate that enforces §10 cannot see 96 persistent artifacts, and the exemption is total rather than graduated.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-gate-exempts-zhs-md-from-the-english-rule-entirely-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, gate, rules |

## Why I was looking, which matters for how much to trust this

I went to update my tree's `verify-docs.sh` to the `track` version, because last round I measured that mine was the
pre-10-31 one. **Reading the diff instead of applying it blind is what surfaced this** — and applying it blind would
have been the faster, more confident-looking move.

**The diff is +51/−1.** The 51 are the guard against running outside a repository. The **−1 is the line I want to
talk about**: my tree's version required task filenames to match `ROLE-YYYY-MM-DD-<topic>-vN.md` strictly, and
`track`'s adds `(\.zhs)?` — an allowance for the Chinese-suffixed form.

## The rule, and what the gate does about it

**`COORDINATION_RULES.md` §10, verbatim:**

> All persistent artifacts (docs, task packets, reviews, acceptance files) must be written in English.

**`verify-docs.sh`, the CJK check:**

```bash
case "$base" in
  *.zhs.md) : ;;                       # <-- the whole check is skipped
  *) if python3 -c "...re.search('[<CJK range>]', ...)" "$f"; then
       echo "$f: contains CJK text; new docs must be English (§10)"
     fi ;;
esac
```

**So a file named `*.zhs.md` is not scanned for CJK at all.** Not scanned leniently — not scanned.

## The scale, measured on `track`

| | measured |
|---|---|
| documents on `track` matching `*.zhs.md` | **96** |
| CJK lines in the first three sampled | 58, 67, 79 |

**So 96 persistent artifacts are invisible to the gate that enforces §10, and they are not marginal** — they carry
tens of CJK lines each.

**I am not claiming those 96 are violations of the operator's intent.** They may be a deliberate carve-out: terminal
output should be Chinese for fast readability, and someone may have decided that certain coordination notes follow
the same rule. **But that decision is not in §10, and the gate does not encode a decision — it encodes an absence.**
A reader of §10 concludes one thing; a reader of the gate concludes another; and the gate is what runs.

## Why this is the week's failure family and not a nitpick

**This is an instrument answering about the set it can see, presented as an answer about the set that exists.** The
gate reports "all documents conform to §10" while being structurally unable to examine 96 of them. I have recorded
that family against other seats' instruments repeatedly this week — an empty grep, a green check against an empty
subject, a cached remote ref. **This one is in the gate I run on my own work, and I have been citing its exit 0 as
evidence that my documents conform.**

**They do conform — I checked my filenames against the strict form and they pass, and the CJK scan caught real
violations in my own drafts three times this week.** So the gate was live for me. **But it was live for me because my
files do not carry the suffix that buys the exemption, and I had not known that was the reason.**

## What I am doing, and the boundary

**I am not editing the gate, and I am not renaming anything.** Two reasons:

1. **The `.zhs.md` allowance was added deliberately and mutation-tested** — its comment says the tolerance was
   surfaced by a mutation that survived the first time. Someone reasoned about this. I am not going to reverse a
   reasoned, tested decision in the same hour I noticed it.
2. **Whether §10 should have a `.zhs.md` carve-out is the operator's to rule on**, because §10 is his policy and the
   gate is the fleet's instrument. A seat unilaterally deciding which artifacts are exempt from the language policy
   is exactly the silent scope expansion my role forbids.

**What I am doing is recording the divergence with its number**, so it stops being a thing everyone assumes is
settled: §10 says all persistent artifacts are English; the gate cannot see 96 of them; nobody has ruled on whether
that is intended.

**And I am updating my tree's gate after all**, because the +51 guard is a strict improvement and my version's
stricter filename rule is one my own files already satisfy. **But I am recording that I am adopting the weaker
filename rule knowingly, not discovering it later.**

## What I did not do

I did not edit any script or product code. I did not rename any `.zhs.md` document, merge, push, accept, cancel, or
re-open any card, restart any application, or write to the live database. My measurements were reads: the `+51/−1`
diff between my tree's and `track`'s gate, the case statement at `verify-docs.sh:92-97`, §10 in full, the
`git ls-tree` count of `.zhs.md` documents on `track`, the CJK line counts in three of them, and my own filenames
against both regexes. The ledger check is exit 0, `seats` is 10, `canonical_events` is 38.
