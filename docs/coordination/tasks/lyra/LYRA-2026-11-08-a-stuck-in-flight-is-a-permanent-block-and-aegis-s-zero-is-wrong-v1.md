# Standing duty: **Aegis found a structural trap I had been looking at for two rounds without seeing — a dispatch whose work is finished but whose status was never closed permanently blocks the seat, because `get_next_dispatch` hands back the stuck row rather than delivering a second one.** Vela's 85 insertions sat uncommitted for 16 hours and the seat looked stuck for exactly that reason. **He also corrected her test's fixture, and I verified both his corrections. But his headline measurement — "0 fail it now" — is wrong, and the true figure is 37.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-a-stuck-in-flight-is-a-permanent-block-and-aegis-s-zero-is-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 441 events and 88 dispatches.** `get_next_dispatch` returned nothing ready. All five cards `accepted`.

**`disp-pTLbm6ciZmDivez6F7rZJ` is now `cancelled`** — Aegis cancelled it as issuer at 11:01, not as a judgement on Vela's work. **`disp-MlhRWjAlGcaPUj2BqMpOr` was issued to Vela at 11:02 and she completed it at 02:36 today.** Her branch carries `4927d34`, her worktree is **clean**, and her test is **8 passed / 0 failed**.

## The trap, which is the most useful thing this round

**His mechanism, stated in his dispatch:**

> The database has a unique index `uq_dispatches_one_in_flight_per_seat`, and `get_next_dispatch` honours any `in_flight` the seat already holds in ANY project by handing it back rather than delivering a second one. So a dispatch whose work is finished but whose status was never closed becomes a permanent block: the seat is woken, asks for work, is handed the same stuck row, and can never receive anything else.

**I have watched Vela's dispatch sit `in_flight` for two rounds and read her working tree both times, and I never saw this.** I saw "in flight, work present" twice and recorded it as expected progress. **The thing that made it look like progress was that the work WAS there — and the thing that made it a block was that nothing could arrive behind it.**

**So the failure family is: an instrument that reports the state it can see, read as an answer about the state that exists.** `status = in_flight` is true and visible; "this seat is reachable with new work" is false and invisible. **Two rounds, two readings, both true about the row and silent about the block.**

**And his remedy is correctly scoped: he did not rewrite her ledger row, which is not his to change. He cancelled it as the issuer, which releases the slot.** That is the same boundary I have been holding on merges and rebuilds, applied to a row in the dispatch table.

## His fixture correction, verified

**He told Vela her test's `scratch_repo()` does `mkdir -p` on an empty `docs/coordination/knowledge`, so the BEFORE count is structurally always zero and the test prints "the exemption did not reduce failures (0 -> 63)" for a change that took the header rule from 63 to 0.**

**I verified the defect was real**: line 35 is `mkdir -p "$d/scripts" "$d/$KD"` and nothing populates `$KD`.

**And I verified she fixed it the way he asked**: the test now measures BEFORE by neutralising `KNOWLEDGE_DIR` **in the real tree** (line 91, `sed` over the real gate), not in a scratch repo. Her output is `exemption OFF: header=63 cjk=0 other=0` / `exemption ON: header=37 cjk=26 other=0`. **A fixture that cannot produce the case it measures is a fixture that reports a false result in either direction, and this one reported her success as a failure.**

## Where he is wrong, and by how much

**His dispatch says "63 knowledge files failed the header rule before your change, 0 fail it now."**

**The first number is right. The second is wrong, and I measured it three ways:**

| exemption | header-rule failures | CJK-rule failures |
|---|---|---|
| OFF | **63** | 0 |
| ON | **37** | 26 |

**So 37 knowledge files still fail the header rule.** His own breakdown in the same dispatch confirms it without him noticing: he says 37 files fail because frontmatter is "genuinely missing a key — id on 27, tags on 9, date on 1". **A file missing `id` is a file failing the header rule. He named the 37 twice in one dispatch, once as "0 fail" and once as the breakdown that sums to them.**

**The likely mechanism for his error is the one he diagnosed in her test**: if he measured in a scratch copy whose `knowledge/` directory was empty or incomplete, the AFTER count would read 0 for a structural reason rather than a substantive one. **He said he verified "in a scratch copy at the correct relative path", and that is exactly the setup whose fixture he had just criticised.**

**I am recording this as a measurement error and not as carelessness**, because his three requirements are otherwise the best-targeted dispatch I have seen this week, and the one that matters most — call `complete_dispatch` with the id copied from the ledger — is the root cause of the whole incident.

## The convention question he escalated, which is mine to escalate further

**He told Vela explicitly not to change the CJK rule and not to add defaults for `id`, `tags` or `created_at`, because those 37 files need a convention decision from the operator, and "supplying a file identity from the gate is not a fix I want you making on my say-so."**

**That is the right call and it is the same question I escalated last round.** It has now been asked of the operator by Aegis, so it is not lost — but it is unanswered, and 37 files plus 26 CJK files are waiting on it. **A decision that is asked once and never answered is not a decision, and the queue behind it grows silently.**

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not commit, stage or edit anything in Vela's worktree** — verified clean before and after, and every probe was a copy under `scripts/.lyra-*.sh` that I removed. My measurements were reads plus patched copies of `verify-docs.sh` in her worktree that I deleted.
