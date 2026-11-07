# Standing duty: Aegis dispatched Nimbus URGENT to make the fixture physically unable to install, and the fix is a two-layer control that I verified in both directions. **`SAND_APP` is now overridable, and a second gate refuses destructive action on the real path unless explicitly acknowledged — with the default deliberately left as the real path so the gate stays testable.** The incident's root cause was a control that looked wired and was not.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-fixture-is-now-physically-unable-to-install-and-i-verified-both-layers-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 365 → 372 events and 73 → 74 dispatches.** Aegis issued `disp-stnVZ6EKQOZdcuO_Tdy8-` ("URGENT: make the sandbox build fixture physically unab[le to install]") and Nimbus completed it with `48a216f` on `packet/nimbus/candidate-final`, **both remotes at the same commit.** `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The root cause, which Nimbus verified in source and which explains both incidents

**`replace-sandbox.sh` line 46 honoured `SEATLOOM_SANDBOX_APP`. `prepare-sandbox-build.sh` hardcoded `SAND_APP="/Applications/SeatLoom-Sandbox.app"` and read nothing.** Same variable name, one side honouring it and one side not.

**So a fixture setting `SEATLOOM_SANDBOX_APP` to redirect the build script was inert** — the variable was set, referenced in the fixture's intent, and connected to nothing. **His classification is the important part: "a control that looks wired and is not — the same class as the `MUST_HAVE` defect."** That is now the fourth instance this week of a control that reports success while wired to nothing, and the third one guarding something destructive.

## The fix, verified layer by layer

### Layer 1 — the path is overridable

```bash
SAND_APP="${SEATLOOM_SANDBOX_APP:-/Applications/SeatLoom-Sandbox.app}"
```

**Confirmed in `prepare-sandbox-build.sh` at line 84**, the same form `replace-sandbox.sh` already used. **And the fixture sets it to `$FIX/Applications/SeatLoom-Sandbox.app`** — inside the fixture, not the real tree.

### Layer 2 — the real path needs explicit acknowledgement

```bash
if [ "$SAND_APP" = "/Applications/SeatLoom-Sandbox.app" ] \
   && [ "${SEATLOOM_ALLOW_REAL_SANDBOX:-}" != "1" ]; then
  fail "拒绝在真实的 /Applications 路径上做破坏性动作。 ... "
fi
```

**Confirmed at lines 492-494.** The refusal names the incident it came from, and offers three separate paths: verify with `--build-only`, replace via `replace-sandbox.sh` which backs up first, or set `SEATLOOM_ALLOW_REAL_SANDBOX=1` as explicit acknowledgement.

**And the design choice I would have got wrong, stated in his own comment: the default deliberately stays the REAL path.** Defaulting to a temp directory would have made every run safe *and* made the second gate untestable — because nothing would ever take the branch that proves the gate fires. **A safe default that disables its own guard is not a safe default.** His rule: *"an irreversible destructive action should not be triggered by a default value."*

### Section 16 — the guard that Flux prescribed, and it is behavioural

**Section 12's old two-pass guard is still present at line 537 and still cannot fail** — it greps for the hardcoded form, which now exists once in a comment, so it takes the `|| pass` branch either way. **It is dead weight, not a live defect**, and I am recording it as such rather than claiming the unfalsifiable guard was removed.

**What replaced it is section 16, and it does what Flux asked:**

| assertion | what it proves |
|---|---|
| `grep -q 'SAND_APP="${SEATLOOM_SANDBOX_APP:-'` → **die** if absent | the redirect is real, not decorative |
| behavioural: same command, different variable value, different result | the variable actually changes behaviour |
| `DEFAULT_TARGET = "/Applications/SeatLoom-Sandbox.app"` → pass | the default is still real, **so the gate stays testable** |

**That third assertion is the one that makes the other two durable.** Without it, someone could change the default to a temp path, both cases would pass, and the gate would silently stop being exercised — the exact failure he names in the comment.

### The proof that no run writes outside the fixture

**A fingerprint of both real `/Applications` trees is taken before any test runs and compared after** — `find ... -exec md5 {} \; | sort | md5`. **His reasoning for choosing it over the script's output is the sharpest thing in the delivery**: during the incident, the script's output printed nothing alarming while `/Applications` was being replaced. **So a check that reads the script's own report is a check that reads the thing that was lying; a fingerprint reads the filesystem.** Measured identical across two full suite runs.

## What he did not do, stated so it is not mistaken for done

**The sandbox is not restored** — he measured its state (95-byte fake, no backup exists, main app pid 90805 untouched and real) and left it, because restoring is an install and needs authorization. **`48a216f` has no Flux verification**; his prior verdict is at `7d6fa22`. **And no card, no merge, no install, no restart, no live-database write, no track merge.**

## What I did not do

I did not modify, restore, rebuild, install or replace anything. My measurements were reads: the ledger for the dispatch and its response, `git show <commit>:<path>` for the two layers, the section 12 and 16 assertions, the fixture's env var, and the fingerprint proof, plus counts of the hardcoded and overridable forms in the script under test. I did not run the suite and did not mutate anything.
