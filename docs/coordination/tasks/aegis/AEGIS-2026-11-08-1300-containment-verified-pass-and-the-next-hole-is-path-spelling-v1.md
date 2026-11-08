| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1300-containment-verified-pass-and-the-next-hole-is-path-spelling |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, sandbox, verification, docs-gate |

# Containment verified PASS with nothing destroyed, and the next hole is that the guard compares strings while the filesystem does not

## The verdict

`ev-H6JDfr293lSV2kabLi8WI` — **pass** on the containment fix at `33909f1`. All four of my
requirements met, and this time nothing was damaged:

> /Applications WAS NOT WRITTEN — both real applications are byte-identical before and after
> every measurement, and I did not run any mutation that could reach the real path.

The verifier did something I did not ask for and should have: it polled the combined fingerprint
of both real applications **every three seconds for the whole run** — 161 samples, one unique
value. That is a per-mutation measurement rather than an end-of-run one, which is exactly the
property the last four attempts claimed and did not have.

It also confirmed the call-site change is complete: the real path appears in code at exactly two
assignment sites (line 93 for `REAL_SANDBOX_PATH`, line 514 for `PROTECTED_PATH`) and everywhere
else inside comments. No third literal for a partial redirect to leave behind.

## The next hole, and it is the same class

The verifier found what the redirect space could not show. **The guard compares strings. The
filesystem does not.**

The guard at line 540 is two bash `[ = ]` tests. macOS's default filesystem is case-insensitive,
and path spelling is not canonical. The verifier built a stand-in directory and drove the script
with the target set to differently-spelled paths to that same directory: a trailing slash, a dot
component, a doubled slash, a bare dot, and a `Contents/..` component. **All six were judged
not-equal by the gate, all six reached install, and all six destroyed the target.** Each was then
verified to resolve to the same directory by reading a sentinel file through it. All of it inside
`/tmp`.

I reproduced the underlying fact myself, also inside `/tmp`: the same directory with and without a
trailing slash is not equal as strings and is the same directory on disk.

Ranked honestly, using the verifier's own words: it requires a caller to pass a differently-spelled
real path, no existing caller does, and the fixture does not, so it is **not reachable by an
accidental default**. It is a hole in the guard's model, not an active path to damage. It is still
worth closing, because it is the same class that has now cost the real sandbox five times.

Every previous fix corrected **which strings** were compared. None corrected **that strings are
what is being compared**.

Dispatched `disp-uIOs7Wu0iP8gyR5lBVtw6` at priority 98 with the one-line shape of the fix: resolve
both sides before comparing, and add the assertion whose absence let this survive five fixes.

## The docs finding, one step further

Last hour I classified the 328. This hour I found the fact that changes the recommendation: **27
of the 90 knowledge files already carry both the frontmatter and the complete eight-key doc header,
and all 27 pass the gate today.** The compliant shape already exists inside the same directory.

So the knowledge convention was never missing a template. It was never written down that **both**
are required. A contributor today has two documented instructions that contradict each other and no
signal which is enforced.

Dispatched `disp-pTLbm6ciZmDivez6F7rZJ` at priority 97 with the measured numbers, and with the
direction that matters: a knowledge file with **neither** form must still be refused, and one with
a header but broken frontmatter must still be refused. A gate change that cannot fail is worse than
no change.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- **Push confirmed**: remote and local both at `82f203f`. The intermittent network resolved on the
  third attempt of the verify loop.
- Both pending cards clickable.
- Sandbox: 95-byte fake, fingerprint `0d00e85f3682fb8bdf52b9a2aceecdfb`, no backup. Still needs
  your authorisation. Nothing I ran this hour wrote to it.
- Main application: pid 90805, untouched.

## Needs you

- **The sandbox rebuild** is still the only thing between six verified deliverables and you. It has
  been a fake for three days.
- **The 63 knowledge files** — the fix is dispatched; after it lands the gate stops reporting them.
- **Which remote is the source of truth.** gitlab is 935+ commits behind and I have not pushed there.
