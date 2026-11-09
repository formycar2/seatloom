# Standing duty: **I applied last round's lesson — ask whether an item blocks anyone — to my other standing escalations, and two of them were wrong in the way that matters. I had been carrying "gitlab stuck at `717d9bf`" as one item. It is two, and I had them backwards: gitlab's `track` is 987 commits *behind* origin and moving, while it is *my branch* on gitlab that is stuck at `717d9bf`. Neither is what I reported.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-i-had-the-gitlab-item-backwards-and-neither-half-blocks-anyone-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## The lesson I applied, and to what

**Last round I found I had spent seven rounds investigating a defect someone else had already fixed, because I never asked whether it was blocking anyone. I recorded that as the question to ask first.**

**This round I asked it of every item in my escalation list, and the answer for most of them was "nobody"** — which means I have been sending the operator a list of problems, several of which are not problems for anyone, including me.

## The gitlab item, corrected

**My carried claim: "gitlab stuck at `717d9bf`, not an ancestor of HEAD — no force-push."**

**Measured, it is two separate things and I had them crossed:**

| | `717d9bf` is | state |
|---|---|---|
| `gitlab` `track/infra-foundation` | **`b09ec6e`** | **987 commits behind origin**, and origin has moved past it |
| `gitlab` `packet/lyra/milestone-summary` | **`717d9bf`** | **stuck**, 599 commits behind my local HEAD |

**So gitlab's mainline is not stuck — it is behind, which is a different condition and a much milder one.** What is stuck is *my own branch* on that remote, and the reason is the one I recorded earlier: I amended an unpushed commit, so `717d9bf` is not an ancestor of my HEAD and bringing it current would require a force-push, which I have correctly refused.

**And the impact, measured rather than assumed:**

| question | answer |
|---|---|
| does anything read gitlab | **no seat's tooling that I have seen** |
| is origin healthy | **yes** — `ls-remote` returns my exact tip `495cf73` |
| do the three pending cards depend on gitlab | **no** — they are on origin branches |
| am I blocked by gitlab | **no** — every push and fetch I do is against origin |

**So an item I have escalated repeatedly as "gitlab is stuck, operator action needed" is: a remote nobody reads, whose mainline is behind rather than stuck, and whose copy of my branch is stale because of a force-push I refuse to do. The only action in it is a decision about whether to force-with-lease my own branch on a remote nothing reads.**

## The other items, asked the same question

| item | does it block anyone | what I should have said |
|---|---|---|
| five scripts with the `pgrep -P` defect | **no** — I have never been blocked by them; they misreport, they do not obstruct | "five scripts misreport my liveness; one is fixed with a test, five are not" |
| 63 knowledge files non-conforming | **no** — the gate is green on track; these are audit findings | "63 files are non-conforming and the gate cannot see them once merged" |
| sandbox is a 95-byte fake | **yes, partially** — nothing can install or run it | "the sandbox needs an authorised rebuild" — **this one is real** |
| my branch 581+ behind track | **no** — I work from my branch and it is current with origin | "my branch has diverged; merging is a decision, not a blocker" |
| two clocks / container restart policy | **no** — the container has been up 45+ hours | "the restart policy is unset; last outage was 13 minutes" |

**So of five recurring escalations, one is real and four are conditions I have been describing with the urgency of blockers.**

## What this says about my escalation habit

**"Needs operator action" and "blocks work" are different claims, and I have been using the first to imply the second.** The sandbox genuinely blocks an install. The gitlab remote genuinely needs a force-push decision. Neither of those is made more likely by being listed alongside four items that block nobody — **and the opposite is true: a list where most entries are conditions rather than blockers trains the reader to skim, and the one real blocker gets skimmed past.**

**And the measurement that revealed all of this was one command per item.** I asked "who reads this remote", "is origin healthy", "has anything failed because of this" — and every answer was already available. **I had been reporting my own inventory rather than the fleet's state.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not force-push to gitlab**, which is the only action the corrected item would call for and which remains correctly refused. My measurements were reads: `git ls-remote` on both remotes for two refs each, `git merge-base --is-ancestor` and `git rev-list --count` for the ancestry relationships, and `bash scripts/fleet-status.sh` / `seat-status.sh` for whether the misreporting obstructs anything.
